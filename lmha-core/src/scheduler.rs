use chrono::{DateTime, Utc, Local, Timelike, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::Rng;

use crate::{DeviceState, SchedulingType};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerAction {
    Nothing,
    SwitchOn(Uuid),
    SwitchOff(Uuid),
    UpdateScheduling(Uuid, SchedulingType),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceContext {
    pub id: Uuid,
    pub current_state: DeviceState,
    pub last_state_change: Option<DateTime<Utc>>,
    pub is_enabled: bool,
    pub expected_load: i32,
    pub scheduling_type: SchedulingType,
    pub device_runtime: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct StateEvent {
    pub timestamp: DateTime<Utc>,
    pub state: DeviceState,
}

#[derive(Debug)]
pub struct SchedulerInput<R: Rng> {
    pub pv_production: i32,
    pub house_consumption: i32,
    pub devices: Vec<DeviceContext>,
    pub history: std::collections::HashMap<Uuid, Vec<StateEvent>>,
    pub now: DateTime<Utc>,
    pub day_deadline: NaiveTime,
    pub debounce_duration_secs: i64,
    pub rng: R,
}

fn get_start_of_day(now: DateTime<Utc>, day_deadline: NaiveTime) -> DateTime<Utc> {
    let now_local = now.with_timezone(&Local);
    let mut start_local = now_local.with_hour(day_deadline.hour()).unwrap()
        .with_minute(day_deadline.minute()).unwrap()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap();
    
    if now_local.time() < day_deadline {
        start_local = start_local - chrono::Duration::days(1);
    }
    start_local.with_timezone(&Utc)
}

fn calculate_runtime_mins(history: &[StateEvent], start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    let mut total_mins = 0;
    let mut last_on_time: Option<DateTime<Utc>> = None;

    // Sort history by timestamp just in case
    let mut sorted_history = history.to_vec();
    sorted_history.sort_by_key(|e| e.timestamp);

    for event in sorted_history {
        if event.timestamp > end { break; }
        
        match event.state {
            DeviceState::On => {
                if last_on_time.is_none() {
                    last_on_time = Some(event.timestamp.max(start));
                }
            }
            DeviceState::Off | DeviceState::Unknown => {
                if let Some(on_time) = last_on_time {
                    let off_time = event.timestamp.min(end);
                    if off_time > on_time {
                        total_mins += (off_time - on_time).num_minutes();
                    }
                    last_on_time = None;
                }
            }
        }
    }

    // If still on at the end of the window
    if let Some(on_time) = last_on_time {
        if end > on_time {
            total_mins += (end - on_time).num_minutes();
        }
    }

    total_mins
}

#[derive(Debug)]
struct DailyRunInfo {
    has_started: bool,
    is_currently_running: bool,
    current_run_duration_mins: i64,
    total_runtime_today_mins: i64,
}

fn get_daily_run_info(history: &[StateEvent], start_of_day: DateTime<Utc>, now: DateTime<Utc>) -> DailyRunInfo {
    let mut sorted_history = history.to_vec();
    sorted_history.sort_by_key(|e| e.timestamp);
    
    // Filter history to only include events from this day
    let today_history: Vec<_> = sorted_history.into_iter()
        .filter(|e| e.timestamp >= start_of_day && e.timestamp <= now)
        .collect();

    let has_started = today_history.iter().any(|e| e.state == DeviceState::On);
    
    let last_event = today_history.last();
    let is_currently_running = last_event.map(|e| e.state == DeviceState::On).unwrap_or(false);
    
    let current_run_duration_mins = if is_currently_running {
        let last_on_start = today_history.iter()
            .rev()
            .take_while(|e| e.state == DeviceState::On)
            .last()
            .map(|e| e.timestamp)
            .unwrap_or(now); // Should not happen if is_currently_running is true
        (now - last_on_start).num_minutes()
    } else {
        0
    };

    let total_runtime_today_mins = calculate_runtime_mins(&today_history, start_of_day, now);

    DailyRunInfo {
        has_started,
        is_currently_running,
        current_run_duration_mins,
        total_runtime_today_mins,
    }
}

pub fn decide_action<R: Rng>(input: SchedulerInput<R>) -> SchedulerAction {
    // 1. Check for Forced state expirations
    for device in &input.devices {
        match &device.scheduling_type {
            SchedulingType::ForceOn { until } | SchedulingType::ForceOff { until } => {
                if input.now >= *until {
                    return SchedulerAction::UpdateScheduling(device.id, SchedulingType::Boiler);
                }
            }
            _ => {}
        }
    }

    // 2. Handle absolute overrides (Skip debounce for forced states)
    for device in &input.devices {
        if !device.is_enabled {
            continue;
        }

        match device.scheduling_type {
            SchedulingType::ForceOn { .. } => {
                if device.current_state != DeviceState::On {
                    return SchedulerAction::SwitchOn(device.id);
                }
            }
            SchedulingType::ForceOff { .. } => {
                if device.current_state != DeviceState::Off {
                    return SchedulerAction::SwitchOff(device.id);
                }
            }
            _ => {}
        }
    }

    // 3. New Simple Boiler Logic
    let start_of_day = get_start_of_day(input.now, input.day_deadline);
    
    // Catch-up window is defined as 4 hours before the NEXT deadline.
    // The next deadline is start_of_day + 24 hours.
    let next_deadline = start_of_day + chrono::Duration::days(1);
    let catchup_start = next_deadline - chrono::Duration::hours(4);
    let is_catchup_window = input.now >= catchup_start && input.now < next_deadline;

    let mut eligible_for_pv = Vec::new();

    for device in &input.devices {
        if !device.is_enabled || device.scheduling_type != SchedulingType::Boiler {
            continue;
        }

        let history = input.history.get(&device.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let run_info = get_daily_run_info(history, start_of_day, input.now);

        // a. If currently running, check if it should stay on
        if run_info.is_currently_running {
            if run_info.current_run_duration_mins < device.device_runtime as i64 {
                // Keep it on until it reaches runtime
                if device.current_state != DeviceState::On {
                    return SchedulerAction::SwitchOn(device.id);
                }
                continue; // Stay on
            } else {
                // Runtime reached, turn it off
                if device.current_state != DeviceState::Off {
                    return SchedulerAction::SwitchOff(device.id);
                }
                continue;
            }
        }

        // b. If already ran today, leave it off
        if run_info.has_started {
            if device.current_state != DeviceState::Off {
                return SchedulerAction::SwitchOff(device.id);
            }
            continue;
        }

        // c. Catch-up window check
        if is_catchup_window {
            // Force on for its full runtime
            if device.current_state != DeviceState::On {
                return SchedulerAction::SwitchOn(device.id);
            }
            continue;
        }

        // d. PV Activation check (only if NOT in catchup window)
        // Note: PV window is from start_of_day until catchup_start
        if input.now >= start_of_day && input.now < catchup_start {
            let net_balance = input.pv_production - input.house_consumption;
            let on_threshold = (0.7 * device.expected_load as f64) as i32;
            
            if net_balance > on_threshold {
                eligible_for_pv.push(device.id);
            }
        }
    }

    // e. Randomly select ONE device for PV activation
    if !eligible_for_pv.is_empty() {
        let mut rng = input.rng;
        let idx = rng.gen_range(0..eligible_for_pv.len());
        return SchedulerAction::SwitchOn(eligible_for_pv[idx]);
    }

    SchedulerAction::Nothing
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Timelike, TimeZone};
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashMap;

    #[test]
    fn test_pv_activation_and_lock_on() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
        let day_deadline = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        // 10:00 AM
        let base_time = Utc::now().with_hour(10).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        
        let mut history = HashMap::new();
        history.insert(device_id, Vec::new());

        let devices = vec![DeviceContext {
            id: device_id,
            current_state: DeviceState::Off,
            last_state_change: None,
            is_enabled: true,
            expected_load: 5000,
            scheduling_type: SchedulingType::Boiler,
            device_runtime: 180, // 3h
        }];

        // 1. PV Surplus > 70% -> SwitchOn
        let input_on = SchedulerInput {
            pv_production: 5000,
            house_consumption: 1000,
            devices: devices.clone(),
            history: history.clone(),
            now: base_time,
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_on), SchedulerAction::SwitchOn(device_id));

        // 2. Device is ON, but only for 10 mins. PV drops -> Should stay ON
        let mut history_on = history.clone();
        history_on.get_mut(&device_id).unwrap().push(StateEvent { timestamp: base_time, state: DeviceState::On });
        
        let mut devices_on = devices.clone();
        devices_on[0].current_state = DeviceState::On;

        let input_stay_on = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices_on.clone(),
            history: history_on.clone(),
            now: base_time + Duration::minutes(10),
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_stay_on), SchedulerAction::Nothing);

        // 3. Device is ON for 181 mins -> Should SwitchOff
        let input_off = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices_on.clone(),
            history: history_on.clone(),
            now: base_time + Duration::minutes(181),
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_off), SchedulerAction::SwitchOff(device_id));
    }

    #[test]
    fn test_catchup_window() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
        let day_deadline = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        
        // Local 1:01 AM is within 4h before 5:00 AM
        let now_local = Local::now().with_hour(1).unwrap().with_minute(1).unwrap().with_second(0).unwrap();
        let now_utc = now_local.with_timezone(&Utc);
        
        let mut history = HashMap::new();
        history.insert(device_id, Vec::new());

        let devices = vec![DeviceContext {
            id: device_id,
            current_state: DeviceState::Off,
            last_state_change: None,
            is_enabled: true,
            expected_load: 5000,
            scheduling_type: SchedulingType::Boiler,
            device_runtime: 180,
        }];

        let input = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices.clone(),
            history: history.clone(),
            now: now_utc,
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input), SchedulerAction::SwitchOn(device_id));
    }

    #[test]
    fn test_vacation_transition_at_deadline_local() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
        let day_deadline = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        
        // Vacation ends at 5:00 AM local time
        let vacation_until_local = Local::now().with_hour(5).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        let vacation_until_utc = vacation_until_local.with_timezone(&Utc);
        
        // Current time is 5:01 AM local time
        let now_local = vacation_until_local + Duration::minutes(1);
        let now_utc = now_local.with_timezone(&Utc);
        
        let devices = vec![DeviceContext {
            id: device_id,
            current_state: DeviceState::Off,
            last_state_change: None,
            is_enabled: true,
            expected_load: 5000,
            scheduling_type: SchedulingType::ForceOff { until: vacation_until_utc },
            device_runtime: 180,
        }];

        let mut history = HashMap::new();
        history.insert(device_id, Vec::new());

        // First invocation: Should transition to Boiler mode
        let input1 = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices.clone(),
            history: history.clone(),
            now: now_utc,
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        
        let action1 = decide_action(input1);
        assert_eq!(action1, SchedulerAction::UpdateScheduling(device_id, SchedulingType::Boiler));

        // Second invocation: Device is now in Boiler mode
        let mut devices_boiler = devices.clone();
        devices_boiler[0].scheduling_type = SchedulingType::Boiler;
        
        let input2 = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices_boiler,
            history: history.clone(),
            now: now_utc,
            day_deadline,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };

        // Should NOT trigger catch-up because we just started a new day at 5:00 AM local
        let action2 = decide_action(input2);
        assert_eq!(action2, SchedulerAction::Nothing);
    }
}

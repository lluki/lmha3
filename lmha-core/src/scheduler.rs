use chrono::{DateTime, Utc};
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
    pub debounce_duration_secs: i64,
    pub rng: R,
}

const CHARGE_WINDOW_START_HOUR: u32 = 1; // 1am
const DAY_START_HOUR: u32 = 5; // 5am

fn get_start_of_day(now: DateTime<Utc>) -> DateTime<Utc> {
    use chrono::Timelike;
    let mut start = now.with_hour(DAY_START_HOUR).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
    if now.hour() < DAY_START_HOUR {
        start = start - chrono::Duration::days(1);
    }
    start
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
    let start_of_day = get_start_of_day(input.now);
    use chrono::Timelike;
    let is_catchup_window = input.now.hour() >= CHARGE_WINDOW_START_HOUR && input.now.hour() < DAY_START_HOUR;

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
                // Runtime reached, turn it off (unless it's catchup window and others are running? No, each device is independent)
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

        // d. PV Activation check (only if between 5am and 1am)
        let is_pv_window = input.now.hour() >= DAY_START_HOUR || input.now.hour() < CHARGE_WINDOW_START_HOUR;
        if is_pv_window {
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
    use chrono::{Duration, Timelike};
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashMap;

    #[test]
    fn test_pv_activation_and_lock_on() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
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
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_off), SchedulerAction::SwitchOff(device_id));
    }

    #[test]
    fn test_catchup_window() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
        // 1:00 AM
        let base_time = Utc::now().with_hour(1).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        
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

        // No PV, but in catchup window -> SwitchOn
        let input = SchedulerInput {
            pv_production: 0,
            house_consumption: 1000,
            devices: devices.clone(),
            history: history.clone(),
            now: base_time,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input), SchedulerAction::SwitchOn(device_id));
    }

    #[test]
    fn test_random_selection() {
        let d1 = Uuid::new_v4();
        let d2 = Uuid::new_v4();
        let base_time = Utc::now().with_hour(10).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        
        let mut history = HashMap::new();
        history.insert(d1, Vec::new());
        history.insert(d2, Vec::new());

        let devices = vec![
            DeviceContext {
                id: d1,
                current_state: DeviceState::Off,
                last_state_change: None,
                is_enabled: true,
                expected_load: 1000,
                scheduling_type: SchedulingType::Boiler,
                device_runtime: 60,
            },
            DeviceContext {
                id: d2,
                current_state: DeviceState::Off,
                last_state_change: None,
                is_enabled: true,
                expected_load: 1000,
                scheduling_type: SchedulingType::Boiler,
                device_runtime: 60,
            }
        ];

        // Plenty of PV for both, but we pick one at random (based on RNG seed)
        let input = SchedulerInput {
            pv_production: 10000,
            house_consumption: 0,
            devices: devices.clone(),
            history: history.clone(),
            now: base_time,
            debounce_duration_secs: 0,
            rng: StdRng::seed_from_u64(123),
        };
        
        let action = decide_action(input);
        match action {
            SchedulerAction::SwitchOn(id) => assert!(id == d1 || id == d2),
            _ => panic!("Expected SwitchOn"),
        }
    }

    #[test]
    fn test_overrides_precedence() {
        let rng = StdRng::seed_from_u64(42);
        let device_id = Uuid::new_v4();
        let base_time = Utc::now().with_hour(10).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        let until = base_time + Duration::hours(1);
        
        let mut history = HashMap::new();
        history.insert(device_id, Vec::new());

        let devices = vec![DeviceContext {
            id: device_id,
            current_state: DeviceState::On,
            last_state_change: None,
            is_enabled: true,
            expected_load: 5000,
            scheduling_type: SchedulingType::ForceOff { until },
            device_runtime: 180,
        }];

        // Even with huge PV, ForceOff should result in SwitchOff
        let input = SchedulerInput {
            pv_production: 20000,
            house_consumption: 0,
            devices: devices.clone(),
            history: history.clone(),
            now: base_time,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input), SchedulerAction::SwitchOff(device_id));
    }
}

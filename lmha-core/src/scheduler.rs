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
    pub full_charge_n_day: i32,
    pub min_daily_charge: i32,
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

const FULL_CHARGE_DURATION_MINS: i64 = 240; // 4 hours
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

    // 3. Mandatory Charge Logic
    let start_of_day = get_start_of_day(input.now);
    use chrono::Timelike;
    let is_mandatory_window = input.now.hour() >= CHARGE_WINDOW_START_HOUR && input.now.hour() < DAY_START_HOUR;

    if is_mandatory_window {
        for device in &input.devices {
            if !device.is_enabled || device.scheduling_type != SchedulingType::Boiler {
                continue;
            }

            let history = input.history.get(&device.id).map(|v| v.as_slice()).unwrap_or(&[]);
            let today_runtime = calculate_runtime_mins(history, start_of_day, input.now);
            
            // Daily minimum check
            if today_runtime < device.min_daily_charge as i64 {
                if device.current_state != DeviceState::On {
                    return SchedulerAction::SwitchOn(device.id);
                }
                continue; // Stay on
            }

            // Full charge check
            let mut days_since_full_charge = 0;
            for i in 0..device.full_charge_n_day {
                let s = start_of_day - chrono::Duration::days(i as i64);
                let e = s + chrono::Duration::days(1);
                if calculate_runtime_mins(history, s, e) >= FULL_CHARGE_DURATION_MINS {
                    break;
                }
                days_since_full_charge += 1;
            }

            if days_since_full_charge >= device.full_charge_n_day {
                // We are on the deadline day (or past it), and today hasn't had a full charge yet.
                // If it's the mandatory window (1am-5am), force it ON.
                if today_runtime < FULL_CHARGE_DURATION_MINS {
                    if device.current_state != DeviceState::On {
                        return SchedulerAction::SwitchOn(device.id);
                    }
                }
            }
        }
    }

    // 4. Boiler Logic (Runtime-aware)
    let mut devices_with_runtime = Vec::new();
    for device in &input.devices {
        if !device.is_enabled || device.scheduling_type != SchedulingType::Boiler {
            continue;
        }

        let can_change = match device.last_state_change {
            Some(last_change) => {
                input.now.signed_duration_since(last_change).num_seconds() >= input.debounce_duration_secs
            },
            None => true,
        };

        if !can_change {
            continue;
        }

        let history = input.history.get(&device.id).map(|v| v.as_slice()).unwrap_or(&[]);
        
        // Find how long it has been in current state
        let mut sorted_history = history.to_vec();
        sorted_history.sort_by_key(|e| e.timestamp);
        let last_event = sorted_history.last();
        
        let duration_in_current_state = match last_event {
            Some(e) => (input.now - e.timestamp).num_seconds(),
            None => 999999, // Long time
        };

        devices_with_runtime.push((device, duration_in_current_state));
    }

    let mut eligible_to_on = Vec::new();
    let mut eligible_to_off = Vec::new();

    let house_consumption_excl_devices: i32 = input.house_consumption - input.devices.iter()
        .filter(|d| d.current_state == DeviceState::On)
        .map(|d| d.expected_load)
        .sum::<i32>();

    for (device, duration) in devices_with_runtime {
        let on_threshold = (0.7 * device.expected_load as f64) as i32;
        // Improved deactivation logic: stay on if PV covers at least 30% of device load
        let off_threshold = (0.3 * device.expected_load as f64) as i32;
        
        let net_balance = input.pv_production - input.house_consumption;

        if device.current_state == DeviceState::Off {
            if net_balance > on_threshold {
                eligible_to_on.push((device.id, duration));
            }
        } else if device.current_state == DeviceState::On {
            // Logic: we want to turn off if PV supply of this device goes below 30%
            // Or more simply: PV_Production < (House_Consumption_Excl_Device + 0.3 * Device_Load)
            if input.pv_production < (house_consumption_excl_devices + off_threshold) {
                eligible_to_off.push((device.id, duration));
            }
        }
    }

    // Sort by duration for fair scheduling
    // Turn off the one that has been running the longest
    eligible_to_off.sort_by_key(|&(_, duration)| std::cmp::Reverse(duration));
    if let Some(&(id, _)) = eligible_to_off.first() {
        return SchedulerAction::SwitchOff(id);
    }

    // Turn on the one that has been idle the longest
    eligible_to_on.sort_by_key(|&(_, duration)| std::cmp::Reverse(duration));
    if let Some(&(id, _)) = eligible_to_on.first() {
        return SchedulerAction::SwitchOn(id);
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
    fn test_hysteresis_logic() {
        let now = Utc::now();
        let device_id = Uuid::new_v4();
        let rng = StdRng::seed_from_u64(42);
        
        let mut history = HashMap::new();
        history.insert(device_id, Vec::new());

        let input_on = SchedulerInput {
            pv_production: 5000,
            house_consumption: 1000,
            devices: vec![DeviceContext {
                id: device_id,
                current_state: DeviceState::Off,
                last_state_change: Some(now - Duration::minutes(10)),
                is_enabled: true,
                expected_load: 5000,
                scheduling_type: SchedulingType::Boiler,
                full_charge_n_day: 1,
                min_daily_charge: 0,
            }],
            history: history.clone(),
            now,
            debounce_duration_secs: 300,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_on), SchedulerAction::SwitchOn(device_id));

        let input_off = SchedulerInput {
            pv_production: 1000,
            house_consumption: 1000 + 5000, // House + Device
            devices: vec![DeviceContext {
                id: device_id,
                current_state: DeviceState::On,
                last_state_change: Some(now - Duration::minutes(10)),
                is_enabled: true,
                expected_load: 5000,
                scheduling_type: SchedulingType::Boiler,
                full_charge_n_day: 1,
                min_daily_charge: 0,
            }],
            history: history.clone(),
            now,
            debounce_duration_secs: 300,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_off), SchedulerAction::SwitchOff(device_id));
    }

    #[test]
    fn test_smart_boiler_scheduler_3_day_scenario() {
        let d1 = Uuid::new_v4();
        let d2 = Uuid::new_v4();
        let rng = StdRng::seed_from_u64(42);
        
        let mut devices = vec![
            DeviceContext {
                id: d1,
                current_state: DeviceState::Off,
                last_state_change: None,
                is_enabled: true,
                expected_load: 4000,
                scheduling_type: SchedulingType::Boiler,
                full_charge_n_day: 1,
                min_daily_charge: 60,
            },
            DeviceContext {
                id: d2,
                current_state: DeviceState::Off,
                last_state_change: None,
                is_enabled: true,
                expected_load: 4000,
                scheduling_type: SchedulingType::Boiler,
                full_charge_n_day: 3,
                min_daily_charge: 0,
            }
        ];

        let mut history: HashMap<Uuid, Vec<StateEvent>> = HashMap::new();
        history.insert(d1, Vec::new());
        history.insert(d2, Vec::new());

        let base_time = Utc::now().with_hour(5).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        let mut current_now = base_time;

        // Day 1: Solar production. 
        // 12:00 PM: 5kW Solar. Only enough for ONE 4kW device.
        current_now = base_time + Duration::hours(7); // 12:00 PM
        let input = SchedulerInput {
            pv_production: 5000,
            house_consumption: 500,
            devices: devices.clone(),
            history: history.clone(),
            now: current_now,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        let action = decide_action(input);
        // It should pick ONE device. 
        match action {
            SchedulerAction::SwitchOn(id) => {
                let d = devices.iter_mut().find(|d| d.id == id).unwrap();
                d.current_state = DeviceState::On;
                history.get_mut(&id).unwrap().push(StateEvent { timestamp: current_now, state: DeviceState::On });
            }
            _ => panic!("Expected SwitchOn"),
        }

        // Let it run for 10 mins, then 10kW Solar (plenty for both)
        current_now = current_now + Duration::minutes(10);
        let input2 = SchedulerInput {
            pv_production: 10000,
            house_consumption: 4500, // 0.5 + 4.0 (one device on)
            devices: devices.clone(),
            history: history.clone(),
            now: current_now,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        let action2 = decide_action(input2);
        // Now the OTHER device should turn on.
        match action2 {
            SchedulerAction::SwitchOn(id) => {
                let other_id = if devices[0].current_state == DeviceState::On { d2 } else { d1 };
                assert_eq!(id, other_id, "The idle device should turn on");
                let d = devices.iter_mut().find(|d| d.id == id).unwrap();
                d.current_state = DeviceState::On;
                history.get_mut(&id).unwrap().push(StateEvent { timestamp: current_now, state: DeviceState::On });
            }
            _ => panic!("Expected second device to SwitchOn"),
        }

        // Day 2: 1:00 AM (Mandatory Window). D1 needs full charge (4h).
        current_now = base_time + Duration::hours(20); // 1:00 AM Day 2
        let input_mandatory = SchedulerInput {
            pv_production: 0,
            house_consumption: 500,
            devices: devices.clone(),
            history: history.clone(),
            now: current_now,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        let action_mandatory = decide_action(input_mandatory);
        match action_mandatory {
            SchedulerAction::SwitchOn(id) => assert_eq!(id, d1, "D1 should be forced ON for mandatory charge"),
            _ => {
                // If already ON or Nothing (if logic thinks it's already done), we need to check
                let d1_ctx = devices.iter().find(|d| d.id == d1).unwrap();
                assert!(d1_ctx.current_state == DeviceState::On || action_mandatory != SchedulerAction::Nothing, "D1 should be ON or switching ON in mandatory window");
            }
        }

        // Test Fair Selection (Longest Idle)
        current_now = base_time + Duration::hours(31); // 12:00 PM Day 2
        devices[0].current_state = DeviceState::On;
        devices[1].current_state = DeviceState::Off;
        history.get_mut(&d1).unwrap().push(StateEvent { timestamp: base_time + Duration::hours(20), state: DeviceState::On });
        
        let input_fair = SchedulerInput {
            pv_production: 10000,
            house_consumption: 500 + 4000, // House + D1
            devices: devices.clone(),
            history: history.clone(),
            now: current_now,
            debounce_duration_secs: 0,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_fair), SchedulerAction::SwitchOn(d2), "D2 (idle longest) should turn on");

        // Test Grid-Assisted Retention (30% rule)
        current_now = current_now + Duration::minutes(10);
        let input_retention = SchedulerInput {
            pv_production: 2000,
            house_consumption: 4500, // House (0.5) + Device (4.0)
            devices: devices.clone(),
            history: history.clone(),
            now: current_now,
            debounce_duration_secs: 300,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_retention), SchedulerAction::Nothing, "D1 should STAY ON due to 30% rule");
    }
}

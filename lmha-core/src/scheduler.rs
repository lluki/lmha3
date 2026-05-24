use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::{seq::SliceRandom, Rng};

use crate::DeviceState;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerAction {
    Nothing,
    SwitchOn(Uuid),
    SwitchOff(Uuid),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceContext {
    pub id: Uuid,
    pub current_state: DeviceState,
    pub last_state_change: Option<DateTime<Utc>>,
    pub is_enabled: bool,
    pub expected_load: f64,
}

#[derive(Debug)]
pub struct SchedulerInput<R: Rng> {
    pub pv_production: f64,
    pub house_consumption: f64,
    pub devices: Vec<DeviceContext>,
    pub now: DateTime<Utc>,
    pub debounce_duration_secs: i64,
    pub rng: R,
}

pub fn decide_action<R: Rng>(mut input: SchedulerInput<R>) -> SchedulerAction {
    let net_balance = input.pv_production - input.house_consumption;
    
    let mut eligible_to_on = Vec::new();
    let mut eligible_to_off = Vec::new();

    for device in input.devices {
        if !device.is_enabled {
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

        let on_threshold = 0.7 * device.expected_load;
        let off_threshold = 0.3 * device.expected_load;

        if device.current_state == DeviceState::Off && net_balance > on_threshold {
            eligible_to_on.push(device.id);
        } else if device.current_state == DeviceState::On && net_balance < off_threshold {
            eligible_to_off.push(device.id);
        }
    }

    if let Some(&id) = eligible_to_on.choose(&mut input.rng) {
        return SchedulerAction::SwitchOn(id);
    }
    
    if let Some(&id) = eligible_to_off.choose(&mut input.rng) {
        return SchedulerAction::SwitchOff(id);
    }

    SchedulerAction::Nothing
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_hysteresis_logic() {
        let now = Utc::now();
        let device_id = Uuid::new_v4();
        let rng = StdRng::seed_from_u64(42);
        
        let input_on = SchedulerInput {
            pv_production: 5000.0,
            house_consumption: 1000.0,
            devices: vec![DeviceContext {
                id: device_id,
                current_state: DeviceState::Off,
                last_state_change: Some(now - Duration::minutes(10)),
                is_enabled: true,
                expected_load: 5000.0,
            }],
            now,
            debounce_duration_secs: 300,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_on), SchedulerAction::SwitchOn(device_id));

        let input_off = SchedulerInput {
            pv_production: 1000.0,
            house_consumption: 1000.0,
            devices: vec![DeviceContext {
                id: device_id,
                current_state: DeviceState::On,
                last_state_change: Some(now - Duration::minutes(10)),
                is_enabled: true,
                expected_load: 5000.0,
            }],
            now,
            debounce_duration_secs: 300,
            rng: rng.clone(),
        };
        assert_eq!(decide_action(input_off), SchedulerAction::SwitchOff(device_id));
    }

    #[test]
    fn test_multiple_eligible_devices_random() {
        let now = Utc::now();
        let device_1 = Uuid::new_v4();
        let device_2 = Uuid::new_v4();
        let rng = StdRng::seed_from_u64(42);
        
        let input = SchedulerInput {
            pv_production: 10000.0,
            house_consumption: 0.0,
            devices: vec![
                DeviceContext {
                    id: device_1,
                    current_state: DeviceState::Off,
                    last_state_change: None,
                    is_enabled: true,
                    expected_load: 5000.0,
                },
                DeviceContext {
                    id: device_2,
                    current_state: DeviceState::Off,
                    last_state_change: None,
                    is_enabled: true,
                    expected_load: 5000.0,
                }
            ],
            now,
            debounce_duration_secs: 300,
            rng,
        };

        let action = decide_action(input);
        match action {
            SchedulerAction::SwitchOn(id) => assert!(id == device_1 || id == device_2),
            _ => panic!("Expected SwitchOn, got {:?}", action),
        }
    }
}

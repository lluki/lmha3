## ADDED Requirements

### Requirement: Device Connectivity Status
The Admin UI SHALL display the connectivity status of each device based on the 20s offline threshold.

#### Scenario: Display offline status in list
- **WHEN** a device's `last_request_time` - `last_feedback_time` > 20s
- **THEN** the device summary card SHALL display an "Offline" badge or indicator

#### Scenario: Display state sync progress
- **WHEN** a device's `desired_state` differs from its `current_state` (observed)
- **THEN** the UI SHALL show a "Syncing..." or "Pending" status for that device

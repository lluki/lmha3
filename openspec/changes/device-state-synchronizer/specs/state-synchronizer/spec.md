## ADDED Requirements

### Requirement: State Synchronization Logic
The system SHALL maintain a state synchronizer responsible for ensuring the observed state of a device matches the desired state. 

#### Scenario: Immediate sync on desired state change
- **WHEN** the desired state of a device is updated in the database
- **THEN** the synchronizer SHALL immediately (within 3s) attempt to send the corresponding MQTT command to the device

#### Scenario: Sync on device coming online
- **WHEN** a device announces its presence (e.g., via a heartbeat or state message)
- **THEN** the synchronizer SHALL check if the observed state matches the desired state and send a command if they differ

#### Scenario: Sync on server startup
- **WHEN** the server starts up
- **THEN** the synchronizer SHALL iterate through all enabled devices and attempt to sync their states if they differ from the desired state

### Requirement: Active Heartbeat Polling
The system SHALL proactively poll devices to verify connectivity and update telemetry.

#### Scenario: Periodic device status poll
- **WHEN** a device has not sent an unsolicited message for 5 minutes
- **THEN** the system SHALL send a `Shelly.GetStatus` request to the device to verify connectivity.

### Requirement: Offline Detection
The system SHALL monitor the delay between a requested state change and the received feedback, as well as general device activity.

#### Scenario: Device marked as offline due to failed request
- **WHEN** the difference between `last_request_time` and `last_feedback_time` for a device exceeds 20 seconds
- **THEN** the device status SHALL be displayed as "Offline"

#### Scenario: Device marked as offline due to inactivity (Idle)
- **WHEN** the server has not received any message (status, online, heartbeat) from a device for more than 5 minutes
- **THEN** the device status SHALL be displayed as "Offline"

### Requirement: Initial State Alignment
The system SHALL ensure the desired state is correctly initialized to prevent unintended toggles.

#### Scenario: Initialize desired state on device creation
- **WHEN** a new device is created or discovered
- **THEN** the system SHALL initialize its `desired_state` to match its current `observed_state`.

#### Scenario: Align desired state on server startup
- **WHEN** the server starts up
- **THEN** the system SHALL set `desired_state = current_state` for all devices before starting the synchronizer loop, unless a persistent desired state is already known and pending.

### Requirement: Last Seen and Request Tracking
The system SHALL update tracking timestamps to accurately reflect device activity.

#### Scenario: Update last seen on confirmed sync
- **WHEN** a device successfully confirms a state change matching the desired state
- **THEN** the system SHALL update the `last_seen` or `last_feedback_time` timestamp for that device

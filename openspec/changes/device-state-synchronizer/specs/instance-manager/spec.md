## ADDED Requirements

### Requirement: Instance Identity and Priority
Each LMHA instance SHALL have a unique identity and an assigned priority level.

- **Priority Levels**: 
  - `PROD`: High priority (e.g., 100)
  - `DEV`: Low priority (e.g., 10)
- **Identity**: A unique string (e.g., "prod-server-1", "dev-laptop-lukas").

#### Scenario: Instance priority is configurable
- **WHEN** the server starts
- **THEN** it MUST load its identity and priority from environment variables (`INSTANCE_ID`, `INSTANCE_PRIORITY`).

### Requirement: Conflict Detection via MQTT
The system SHALL use MQTT heartbeats to detect other running instances and prevent control conflicts.

#### Scenario: Heartbeat broadcast
- **WHEN** an instance is running
- **THEN** it SHALL publish a heartbeat to `lmha3/instances/<id>` every 10 seconds containing its priority.

#### Scenario: Detecting higher priority instance
- **WHEN** an instance receives a heartbeat from a different ID on `lmha3/instances/+`
- **AND** the received priority is HIGHER than its own priority
- **THEN** it SHALL log a "BIG FAT WARNING" to the console and UI.
- **AND** it SHALL enter "PASSIVE" mode, disabling automatic state synchronization and scheduling to prevent fighting.

#### Scenario: Resuming control
- **WHEN** an instance is in "PASSIVE" mode
- **AND** it has not seen a higher-priority heartbeat for more than 30 seconds
- **THEN** it SHALL exit "PASSIVE" mode and resume normal operation.

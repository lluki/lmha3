## ADDED Requirements

### Requirement: Extended Device State Tracking
The `devices` table SHALL be extended to support robust state synchronization and offline detection.

- `desired_state`: Enum (ON, OFF) - The state the system intends for the device.
- `last_request_time`: Timestamp - When the last state change command was sent.
- `last_feedback_time`: Timestamp - When the last confirmed state message was received from the device.

#### Scenario: Schema supports sync tracking
- **WHEN** the `devices` table is inspected
- **THEN** it MUST contain `desired_state`, `last_request_time`, and `last_feedback_time` columns.

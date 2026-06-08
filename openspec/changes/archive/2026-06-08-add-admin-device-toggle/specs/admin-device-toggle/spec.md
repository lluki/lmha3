## ADDED Requirements

### Requirement: Admin Device Manual Toggle
The Admin UI SHALL provide a button to immediately toggle the state of a device (ON to OFF, or OFF to ON) without requiring a change to the scheduling configuration.

#### Scenario: Admin toggles device from modal
- **WHEN** an administrator clicks the "Toggle" button in the device detail modal
- **THEN** the system SHALL send a toggle command to the device
- **AND** the UI SHALL reflect the pending state change (Syncing status)

#### Scenario: Admin toggles device from summary card
- **WHEN** an administrator clicks a toggle icon/button directly on a device summary card
- **THEN** the system SHALL send a toggle command to the device
- **AND** the UI SHALL reflect the pending state change

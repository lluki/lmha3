## ADDED Requirements

### Requirement: Simple Boiler Configuration UI
The device configuration form (both in creation and edit modes) SHALL include an input for the `device_runtime` (in minutes) for devices using the `BOILER` scheduling type.

#### Scenario: Admin sets runtime in UI
- **WHEN** the admin edits a device and enters "180" into the Runtime field
- **THEN** the system saves 180 minutes as the `device_runtime` for that device

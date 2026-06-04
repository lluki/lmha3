## ADDED Requirements

### Requirement: Device Last Heartbeat Display
The Admin UI SHALL display the last heartbeat timestamp for each device in the detail overview modal.

#### Scenario: View heartbeat in overview
- **WHEN** the administrator opens the device overview modal
- **THEN** they see a "Last Heartbeat" field displaying the last communication time

## MODIFIED Requirements

### Requirement: Admin Entity Edit Mode
The admin detail view SHALL provide an explicit "Edit" mode to allow modifications to entity configurations.

#### Scenario: Admin toggles edit mode
- **WHEN** the administrator clicks the "Edit" button in a detail view
- **THEN** all editable fields become active inputs, and a "Save" button (labeled exactly "Save") and "Cancel" button are displayed

#### Scenario: Admin saves changes
- **WHEN** the administrator clicks "Save" after making changes in Edit mode
- **THEN** the system updates the entity via the API and returns to the read-only detail view with the updated information

### Requirement: Simple Boiler Configuration UI
The device configuration form (both in creation and edit modes) and the overview modal SHALL display the `device_runtime` (in minutes). The UI SHALL conditionally show the override expiration field ("Until") based on the selected mode.

#### Scenario: Admin sets runtime in UI
- **WHEN** the admin edits a device and enters "180" into the Runtime field
- **THEN** the system saves 180 minutes as the `device_runtime` for that device

#### Scenario: Visibility of Runtime and Until fields
- **WHEN** the admin views or edits a device
- **THEN** the "Runtime" field is always visible regardless of the selected mode
- **AND** the "Until" field is ONLY visible if the mode is "Force On" or "Force Off"

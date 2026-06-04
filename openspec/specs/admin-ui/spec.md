# Spec: Admin UI

## Overview
Administrative interface for managing system entities (Houses, Tenants, and Devices).
## Requirements
### Requirement: Structured Admin Entity Management
The admin interface SHALL organize Houses, Tenants, and Devices into distinct sections using a summary-detail interaction pattern to minimize clutter and accidental edits.

#### Scenario: Admin views summary list
- **WHEN** the administrator navigates to the Admin panel
- **THEN** they see a list of compact summary cards for Houses, Tenants, and Devices, displaying only essential identifying information

#### Scenario: Admin views entity details
- **WHEN** the administrator clicks on a summary card
- **THEN** a detail view (modal or slide-over) opens showing all configuration fields for that entity in a read-only state

### Requirement: Admin Entity Edit Mode
The admin detail view SHALL provide an explicit "Edit" mode to allow modifications to entity configurations.

#### Scenario: Admin toggles edit mode
- **WHEN** the administrator clicks the "Edit" button in a detail view
- **THEN** all editable fields become active inputs, and a "Save" button (labeled exactly "Save") and "Cancel" button are displayed

#### Scenario: Admin saves changes
- **WHEN** the administrator clicks "Save" after making changes in Edit mode
- **THEN** the system updates the entity via the API and returns to the read-only detail view with the updated information

### Requirement: Unified Entity Creation
The admin panel SHALL provide a prominent, unified creation button for adding new Houses, Tenants, or Devices.

#### Scenario: Admin opens creation dialog
- **WHEN** the administrator clicks the "+" button on the admin overview
- **THEN** they are prompted to choose the type of entity to create, followed by a dialog containing the necessary creation fields

### Requirement: Device Connectivity Status
The Admin UI SHALL display the connectivity status of each device based on the 20s offline threshold.

#### Scenario: Display offline status in list
- **WHEN** a device's `last_request_time` - `last_feedback_time` > 20s
- **THEN** the device summary card SHALL display an "Offline" badge or indicator

#### Scenario: Display state sync progress
- **WHEN** a device's `desired_state` differs from its `current_state` (observed)
- **THEN** the UI SHALL show a "Syncing..." or "Pending" status for that device

### Requirement: Device Last Heartbeat Display
The Admin UI SHALL display the last heartbeat timestamp for each device in the detail overview modal.

#### Scenario: View heartbeat in overview
- **WHEN** the administrator opens the device overview modal
- **THEN** they see a "Last Heartbeat" field displaying the last communication time

### Requirement: Simple Boiler Configuration UI
The device configuration form (both in creation and edit modes) and the overview modal SHALL display the `device_runtime` (in minutes). The UI SHALL conditionally show the override expiration field ("Until") based on the selected mode.

#### Scenario: Admin sets runtime in UI
- **WHEN** the admin edits a device and enters "180" into the Runtime field
- **THEN** the system saves 180 minutes as the `device_runtime` for that device

#### Scenario: Visibility of Runtime and Until fields
- **WHEN** the admin views or edits a device
- **THEN** the "Runtime" field is always visible regardless of the selected mode
- **AND** the "Until" field is ONLY visible if the mode is "Force On" or "Force Off"


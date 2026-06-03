## ADDED Requirements

### Requirement: Shelly ID Discovery
The system SHALL scan MQTT logs for unregistered Shelly device IDs and provide them as suggestions in the Admin panel when creating new devices.

#### Scenario: Admin sees suggested Shelly IDs
- **WHEN** an admin opens the "Create Device" form
- **THEN** the system displays a dropdown or list of recently seen MQTT topics/IDs that are not yet registered in the database

### Requirement: Improved Boiler Runtime Tracking
The dashboard SHALL display the total duration a boiler-mode device has been ON during the current 24-hour period, starting from 5:00 AM.

#### Scenario: View daily runtime
- **WHEN** a user views the device overview at 10:00 AM
- **THEN** the system shows the total ON time accumulated since 5:00 AM that day

## MODIFIED Requirements

### Requirement: Decision Engine (Background Thread)
The decision engine SHALL run as a background process that iterates through all configured houses. For each house, it MUST:
- Retrieve current PV production and house consumption from its specific Home Assistant instance.
- Incorporate house-scoped historical device state data.
- MUST accept an explicit "now" timestamp for deterministic behavior and testing.

#### Scenario: Multi-house scheduling cycle
- **WHEN** the scheduler triggers a new cycle
- **THEN** it sequentially processes "House A" (using its credentials) and "House B" (using its credentials) without cross-contamination of state

### Requirement: Data Integration
The system SHALL integrate with per-house Home Assistant REST APIs using credentials stored in the `houses` database table.
- Poll interval: 5 minutes per house.

#### Scenario: Fetching per-house telemetry
- **WHEN** the system polls for House A production
- **THEN** it uses the specific IP and token associated with House A in the database

### Requirement: Manual Override
Tenants can manually toggle devices via the Web UI.
- A manual toggle triggers an immediate MQTT command and sets a "Manual Override" state that lasts for a configurable duration (default 1 hour).
- **Reload Trigger**: A manual toggle MUST trigger an immediate refresh of the UI state for the affected house.

#### Scenario: Manual toggle refresh
- **WHEN** a tenant clicks the toggle button for a device
- **THEN** the MQTT command is sent AND the web UI immediately reloads the house dashboard to reflect the pending/new state

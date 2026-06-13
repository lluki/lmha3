## MODIFIED Requirements

### Requirement: Fixed Daily Runtime Algorithm (BOILER mode)
- The system SHALL ensure each device in `BOILER` mode runs for a configurable `device_runtime` (default 180 minutes) exactly once during each 24-hour cycle (House Deadline to House Deadline).
- **Guaranteed Duration**: Once a device has been activated (either via PV or catch-up), it SHALL remain ON until it has completed its `device_runtime` or the cycle ends.
- **Single Run**: Once a device has completed its required runtime for the current cycle, it SHALL remain OFF for the remainder of that cycle (unless manually overridden).

#### Scenario: PV-based activation
- **WHEN** it is between the House Deadline and 4 hours prior to the next House Deadline, and PV surplus (PV Production - House Consumption) > 70% of a device's expected load, AND the device has not yet run during the current cycle
- **THEN** the system SHALL select one such device at random and turn it ON.

#### Scenario: Catch-up window
- **WHEN** it is 4 hours prior to the House Deadline and a device has not yet started its run during the current cycle
- **THEN** the system SHALL turn the device ON for its full `device_runtime`.

### Requirement: Manual Override & Forced States
- Tenants can manually toggle devices via the Web UI.
- A manual toggle triggers an immediate MQTT command and sets a "Manual Override" state (`FORCE_ON` or `FORCE_OFF`) with a `scheduling_until` timestamp (default 1 hour).
- **Vacation Absence**: Tenants SHALL be able to set a "Vacation Absence" for devices. Saving a vacation date MUST set the device scheduling state to `FORCE_OFF` until the House Deadline local time of the day *prior* to the selected return date. After this time, the scheduler SHALL resume normal `BOILER` mode control.
- **Precedence**: Forced states take absolute precedence over the automated `BOILER` logic.
- **Reload Trigger**: A manual toggle or setting a vacation MUST trigger an immediate refresh of the UI state for the affected house/device.

#### Scenario: Manual toggle refresh
- **WHEN** a tenant clicks the toggle button for a device
- **THEN** the MQTT command is sent AND the web UI immediately reloads the house dashboard to reflect the pending/new state

#### Scenario: Setting vacation mode
- **WHEN** a tenant sets a vacation return date to 2026-06-10 and the house deadline is "06:00"
- **THEN** the device is set to `FORCE_OFF` with a `scheduling_until` timestamp of 2026-06-09 06:00:00

#### Scenario: Vacation mode expiration
- **WHEN** the current time surpasses the `scheduling_until` timestamp (2026-06-09 06:01:00)
- **THEN** the device automatically transitions from `FORCE_OFF` back to `BOILER` scheduling mode

### Requirement: Runtime Tracking
- The dashboard SHALL display the total duration a boiler-mode device has been ON during the current 24-hour period (House Deadline to House Deadline).

#### Scenario: View daily runtime
- **WHEN** a user views the device overview at 10:00 AM and the house deadline is "05:00"
- **THEN** the system shows the total ON time accumulated since 05:00 AM that day.

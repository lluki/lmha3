## MODIFIED Requirements

### Requirement: 3. Manual Override:

- Tenants SHALL be able to set a "Vacation Absence" for devices via the Web UI.
- Saving a vacation date MUST set the device scheduling state to `FORCE_OFF` until 5:00 AM local time of the day *prior* to the selected return date. After this time, the scheduler SHALL resume normal `BOILER` mode control.
- **Reload Trigger**: Setting a vacation MUST trigger an immediate refresh of the UI state for the affected device.

#### Scenario: Setting vacation mode
- **WHEN** a tenant sets a vacation return date to 2026-06-10
- **THEN** the device is set to `FORCE_OFF` with a `scheduling_until` timestamp of 2026-06-09 05:00:00

#### Scenario: Vacation mode expiration
- **WHEN** the current time surpasses the `scheduling_until` timestamp (2026-06-09 05:01:00)
- **THEN** the device automatically transitions from `FORCE_OFF` back to `BOILER` scheduling mode

#### Scenario: Manual toggle refresh
- **WHEN** a tenant clicks the toggle button for a device
- **THEN** the MQTT command is sent AND the web UI immediately reloads the house dashboard to reflect the pending/new state

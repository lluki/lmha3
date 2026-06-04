# Spec: Load Management

## Overview
The logic for matching load consumption with solar production and handling manual tenant overrides.

## Requirements

1. **Decision Engine (Background Thread):**
   - Runs in a dedicated background thread within the server process.
   - The decision engine SHALL run as a background process that iterates through all configured houses. For each house, it MUST:
     - Retrieve current PV production and house consumption from its specific Home Assistant instance using the house-specific host, token, and entity IDs.
     - Incorporate house-scoped historical device state data.
     - MUST accept an explicit "now" timestamp for deterministic behavior and testing.
   - **Scenario: Multi-house scheduling cycle**
     - **WHEN** the scheduler triggers a new cycle
     - **THEN** it sequentially processes "House A" (using its credentials and entity IDs) and "House B" (using its credentials and entity IDs) without cross-contamination of state

2. **Fixed Daily Runtime Algorithm (BOILER mode):**
   - The system SHALL ensure each device in `BOILER` mode runs for a configurable `device_runtime` (default 180 minutes) exactly once during each 24-hour cycle (5:00 AM to 5:00 AM).
   - **Guaranteed Duration**: Once a device has been activated (either via PV or catch-up), it SHALL remain ON until it has completed its `device_runtime` or the cycle ends.
   - **Single Run**: Once a device has completed its required runtime for the current cycle, it SHALL remain OFF for the remainder of that cycle (unless manually overridden).

   #### Scenario: PV-based activation
   - **WHEN** it is between 5:00 AM and 1:00 AM, and PV surplus (PV Production - House Consumption) > 70% of a device's expected load, AND the device has not yet run during the current cycle
   - **THEN** the system SHALL select one such device at random and turn it ON.

   #### Scenario: Catch-up window
   - **WHEN** it is 1:00 AM and a device has not yet started its run during the current cycle
   - **THEN** the system SHALL turn the device ON for its full `device_runtime`.

3. **Manual Override & Forced States:**
   - Tenants can manually toggle devices via the Web UI.
   - A manual toggle triggers an immediate MQTT command and sets a "Manual Override" state (`FORCE_ON` or `FORCE_OFF`) with a `scheduling_until` timestamp (default 1 hour).
   - **Precedence**: Forced states take absolute precedence over the automated `BOILER` logic.
   - **Reload Trigger**: A manual toggle MUST trigger an immediate refresh of the UI state for the affected house.
   - **Scenario: Manual toggle refresh**
     - **WHEN** a tenant clicks the toggle button for a device
     - **THEN** the MQTT command is sent AND the web UI immediately reloads the house dashboard to reflect the pending/new state

4. **Data Integration:**
   - The system SHALL integrate with per-house Home Assistant REST APIs using credentials stored in the `houses` database table.
   - Poll interval: 5 minutes per house.
   - **Scenario: Fetching per-house telemetry**
     - **WHEN** the system polls for House A production
     - **THEN** it uses the specific IP and token associated with House A in the database

5. **Hysteresis & Safety:**
   - **Debounce:** Minimum 5 minutes between state changes for any device to prevent rapid cycling (except for Manual Overrides).
   - **Activation Threshold (OFF -> ON):** A device in `BOILER` mode SHALL only turn ON if `PV_Production - House_Consumption > 0.7 * Device_Expected_Load`.
   - **Safety Margin:** The 70% threshold acts as a buffer to avoid toggling on minor fluctuations.

6. **MQTT Integration:**
   - ON: Publish `Switch.Set(on: true)` to `[mqtt_topic]/rpc`.
   - OFF: Publish `Switch.Set(on: false)` to `[mqtt_topic]/rpc`.
   - Monitor `[mqtt_topic]/status/switch:0` for state confirmation.

7. **Shelly ID Discovery:**
   The system SHALL scan MQTT logs for unregistered Shelly device IDs and provide them as suggestions in the Admin panel when creating new devices.

   #### Scenario: Admin sees suggested Shelly IDs
   - **WHEN** an administrator opens the "Create Entity" dialog and selects "Device"
   - **THEN** the system displays a list of recently discovered MQTT topics/IDs that are not yet registered, which can be selected to pre-fill the creation form

8. **Runtime Tracking:**
   - The dashboard SHALL display the total duration a boiler-mode device has been ON during the current 24-hour period (5:00 AM to 5:00 AM).
   - **Scenario: View daily runtime**
     - **WHEN** a user views the device overview at 10:00 AM
     - **THEN** the system shows the total ON time accumulated since 5:00 AM that day.

## Logic Priority

1. **Manual / Forced States:**
   - `FORCE_ON` / `FORCE_OFF` take absolute precedence until `scheduling_until` is reached.
   - `NONE` disables all automated scheduling for the device.

2. **Auto-Transition:**
   - When `now() >= scheduling_until`, the device automatically transitions from `FORCE_*` back to `BOILER`.

3. **Fixed Daily Runtime (BOILER):**
   - If currently running and `current_run_duration < device_runtime`, stay ON.
   - If already ran today, stay OFF.
   - If in Catch-up window (1:00 AM - 5:00 AM), turn ON.
   - If PV Surplus > 70% Expected Load, turn ON (random selection among eligible).
   - Otherwise, stay OFF.

4. **Hardware safety limits (Debounce):**
   - Minimum 5 minutes between state changes.

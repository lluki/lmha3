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
   - **Scenario: History-aware polling**
     - **WHEN** the scheduler is invoked
     - **THEN** it retrieves current PV/Consumption and the necessary history of ON/OFF events for all Boiler-mode devices for the active house cycle
2. **Mandatory Charging:**
   - The system SHALL guarantee minimum charge levels and periodic full charges (defined as 4h of runtime within a 5am-5am window).
   - **Full Charge Deadline**: If `full_charge_n_day` days have passed without a 4h charge, the system SHALL force the device ON starting at 1am (4h before the 5am deadline) on the final day.
   - **Daily Minimum**: The system SHALL ensure `min_daily_charge` is met within every 5am-5am cycle, forcing the device ON if the deadline approaches and the quota is not met.
   - **Scenario: Full charge trigger**
     - **WHEN** it is 1am and a device requiring a full charge every 1 day has not reached 4h of runtime since 5am the previous day
     - **THEN** the device is forced ON regardless of PV production
3. **Manual Override:**
   - Tenants can manually toggle devices via the Web UI.
   - A manual toggle triggers an immediate MQTT command and sets a "Manual Override" state that lasts for a configurable duration (default 1 hour) before the scheduler resumes control.
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
   - **Debounce:** Minimum 5 minutes between state changes for any device to prevent rapid cycling.
   - **Activation Threshold (OFF -> ON):** A device in `BOILER` mode SHALL only turn ON if `PV_Production - House_Consumption > 0.7 * Device_Expected_Load`.
   - **Retention Threshold (ON -> ON):** A device in `BOILER` mode SHALL remain ON if `PV_Production > (House_Consumption_Excl_Devices + 0.3 * Device_Expected_Load)`.
   - **Margin:** Configurable buffer (e.g., 200W) to avoid toggling on minor fluctuations.
6. **MQTT Integration:**
   - ON: Publish `Switch.Set(on: true)` to `[mqtt_topic]/rpc`.
   - OFF: Publish `Switch.Set(on: false)` to `[mqtt_topic]/rpc`.
   - Monitor `[mqtt_topic]/status/switch:0` for state confirmation.
7. **Shelly ID Discovery:**
   The system SHALL scan MQTT logs for unregistered Shelly device IDs and provide them as suggestions in the Admin panel when creating new devices. The interaction SHALL be integrated into the unified entity creation dialog.

   #### Scenario: Admin sees suggested Shelly IDs
   - **WHEN** an administrator opens the "Create Entity" dialog and selects "Device"
   - **THEN** the system displays a list of recently discovered MQTT topics/IDs that are not yet registered, which can be selected to pre-fill the creation form

8. **Improved Boiler Runtime Tracking:**
   - The dashboard SHALL display the total duration a boiler-mode device has been ON during the current 24-hour period, starting from 5:00 AM.
   - **Scenario: View daily runtime**
     - **WHEN** a user views the device overview at 10:00 AM
     - **THEN** the system shows the total ON time accumulated since 5:00 AM that day

### Requirement: Fixed Daily Runtime Algorithm
The system SHALL ensure each device in `BOILER` mode runs for a configurable `device_runtime` (default 3 hours) exactly once during each 24-hour cycle (5:00 AM to 5:00 AM).

#### Scenario: PV-based activation
- **WHEN** it is between 5:00 AM and 1:00 AM, and PV surplus (PV Production - House Consumption) > 70% of a device's expected load, AND the device has not yet run during the current cycle
- **THEN** the system SHALL select one such device at random and turn it ON

#### Scenario: Guaranteed duration
- **WHEN** a device has been activated via the PV-based activation or catch-up window
- **THEN** the system SHALL keep it ON for the full `device_runtime` regardless of subsequent PV production changes

#### Scenario: Catch-up window
- **WHEN** it is 1:00 AM and a device has not yet run during the current cycle
- **THEN** the system SHALL turn the device ON for its full `device_runtime`

## Home Assistant details

Home Assistant credentials (IP/Host and Access Token) are managed per-house in the `houses` table. The two sensors of interest for each house are:
- `sensor.pv_production`
- `sensor.house_consumption`

## Logic Priority
1. **Manual / Forced States:**
   - `FORCE_ON` / `FORCE_OFF` take absolute precedence until `scheduling_until` is reached.
   - `NONE` disables all automated scheduling for the device.
2. **Auto-Transition:**
   - When `now() > scheduling_until`, the device automatically transitions from `FORCE_*` back to `BOILER`. The scheduler returns an `UpdateScheduling` action to persist this transition.
3. **Hardware safety limits (Debounce):** Minimum 5 minutes between state changes.
4. **Mandatory Charging:**
   - Forces device ON if `full_charge_n_day` or `min_daily_charge` deadlines are approaching.
   - Mandatory window is 1:00 AM to 5:00 AM.
5. **Production/Demand matching (BOILER):**
   - **Fair Distribution**: Picks device OFF longest to turn ON; device ON longest to turn OFF.
   - **Hysteresis**:
     - Turn ON if Net PV > 70% of Expected Load.
     - Turn OFF if Total PV < (House Consumption Excl. Devices + 30% of Expected Load).
   - **Incremental Activation**: Supports activating multiple devices across sequential cycles (one action per cycle).
   - **Scenario: Longest idle activation**
     - **WHEN** two devices are eligible for BOILER mode activation
     - **THEN** the device that has been OFF for the longest duration is activated first
   - **Scenario: Grid-assisted retention**
     - **WHEN** a 4kW device is ON, PV is 3kW, and other house consumption is 1.5kW (Total consumption 5.5kW)
     - **THEN** 1.5kW (other) + 0.3 * 4kW = 2.7kW. Since 3kW > 2.7kW, the device remains ON

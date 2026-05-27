# Spec: Load Management

## Overview
The logic for matching load consumption with solar production and handling manual tenant overrides.

## Requirements
1. **Decision Engine (Background Thread):**
   - Runs in a dedicated background thread within the server process.
   - Periodically polls **Home Assistant (localhost)** for current PV production and house consumption.
   - Incorporates historical device state data to fulfill long-term scheduling constraints.
   - MUST accept an explicit "now" timestamp for deterministic behavior and testing.
   - **Scenario: History-aware polling**
     - **WHEN** the scheduler is invoked
     - **THEN** it retrieves current PV/Consumption and the necessary history of ON/OFF events for all Boiler-mode devices
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
3. **Data Integration:**
   - Use Home Assistant REST API for telemetry.
   - Poll interval: 5 minutes.
4. **Hysteresis & Safety:**
   - **Debounce:** Minimum 5 minutes between state changes for any device to prevent rapid cycling.
   - **Margin:** Configurable buffer (e.g., 200W) to avoid toggling on minor fluctuations.
3. **MQTT Integration:**
   - ON: Publish `on` to `[mqtt_topic]/rpc` (Shelly Switch.Set command).
   - OFF: Publish `off` to `[mqtt_topic]/rpc`.
   - Monitor `[mqtt_topic]/status/switch:0` for state confirmation.

## Home Assistant details

Use the token found in secrets/ha-token.md . The two sensors of interest are:
HA runs on port 8123 . Use the IP 192.168.178.31 , it will work in prod as well as in development.

## Logic Priority
1. **Manual / Forced States:**
   - `FORCE_ON` / `FORCE_OFF` take absolute precedence until `scheduling_until` is reached.
   - `NONE` disables all automated scheduling for the device.
2. **Auto-Transition:**
   - When `now() > scheduling_until`, the device automatically transitions from `FORCE_*` back to `BOILER`.
3. **Hardware safety limits (Debounce):** Minimum 5 minutes between state changes.
4. **Mandatory Charging:**
   - Forces device ON if `full_charge_n_day` or `min_daily_charge` deadlines are approaching.
5. **Production/Demand matching (BOILER):**
   - **Fair Distribution**: Picks device OFF longest to turn ON; device ON longest to turn OFF.
   - **Improved Hysteresis**: Stay ON if `PV_Production > (House_Consumption_Excl_Device + 0.3 * Device_Load)`.
   - **Incremental Activation**: Supports activating multiple devices across sequential cycles.
   - **Scenario: Longest idle activation**
     - **WHEN** two devices are eligible for BOILER mode activation
     - **THEN** the device that has been OFF for the longest duration is activated first
   - **Scenario: Grid-assisted retention**
     - **WHEN** a 4kW device is ON, PV is 3kW, and other house consumption is 1.5kW
     - **THEN** the device remains ON

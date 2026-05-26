# Spec: Load Management

## Overview
The logic for matching load consumption with solar production and handling manual tenant overrides.

## Requirements
1. **Decision Engine (Background Thread):**
   - Runs in a dedicated background thread within the server process.
   - Periodically polls **Home Assistant (localhost)** for current PV production and house consumption.
   - Logic: `If PV_Production > (House_Consumption + Device_Load + Margin) -> ON`.
2. **Manual Override:**
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
4. **Production/Demand matching (BOILER):**
   - Logic: `If PV_Production > (House_Consumption + Device_Load + Margin) -> ON`.
   - Uses the device's `expected_load` for calculation.

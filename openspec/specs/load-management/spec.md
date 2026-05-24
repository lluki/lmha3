# Spec: Load Management

## Overview
The logic for matching load consumption with solar production.

## Requirements
1. **Decision Engine:**
   - The scheduler periodically polls **Home Assistant (running on localhost)** for current PV production and house consumption.
   - If `PV_Production > (House_Consumption + Device_Load + Margin)`, turn device ON.
   - If `PV_Production < (House_Consumption + Margin)`, turn device OFF.
2. **Data Integration (Home Assistant):**
   - Use the Home Assistant REST API.
   - Authentication via Long-Lived Access Token.
   - Poll interval: Every 5 minutes (aligned with Telemetry).
3. **Hysteresis & Safety:**
   - **Debounce:** Minimum 5 minutes between state changes for any device to prevent rapid cycling.
   - **Margin:** Configurable buffer (e.g., 200W) to avoid toggling on minor fluctuations.
3. **MQTT Integration:**
   - ON: Publish `on` to `[mqtt_topic]/rpc` (Shelly Switch.Set command).
   - OFF: Publish `off` to `[mqtt_topic]/rpc`.
   - Monitor `[mqtt_topic]/status/switch:0` for state confirmation.

## Logic Priority
1. Manual override (if implemented) takes precedence.
2. Hardware safety limits (Debounce).
3. Production/Demand matching.

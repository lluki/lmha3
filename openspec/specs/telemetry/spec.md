# Spec: Telemetry

## Overview
Persistence of environmental and system data.

## Requirements
1. **Data Sources & Frequency:**
   - **PV Production & House Consumption:** Polled from **Home Assistant (localhost REST API)** every **5 minutes**.
   - **Device States:** Persisted immediately upon state change (via MQTT events).
2. **Persistence Strategy:**
   - Store all readings in the `telemetry` table.
   - Every load toggle event MUST include a `metadata` JSON entry explaining the trigger.
3. **Retention:**
   - All telemetry data (power and events) is stored **indefinitely**. No aggregation or deletion is performed.


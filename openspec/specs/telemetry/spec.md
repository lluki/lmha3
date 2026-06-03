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

4. **Optimized Telemetry History API:**
   The system SHALL provide a telemetry history API that supports server-side filtering and truncation. When "All Telemetry" is disabled in the UI, the server MUST return a representative subset of state changes that does not truncate prematurely.

   #### Scenario: Fetching filtered history
   - **WHEN** the UI requests telemetry history with the `events_only` filter enabled
   - **THEN** the server performs another round-trip if necessary to ensure the requested page size is filled with relevant state events before truncating


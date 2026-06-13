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
   - **UTC Enforcement**: All timestamps persisted in the database MUST be stored in UTC format.
   - **API Standards**: All JSON API responses containing timestamps MUST use ISO8601 format with the UTC timezone specifier 'Z' (e.g., "2026-06-13T03:07:59Z").

   #### Scenario: Storing telemetry in UTC
   - **WHEN** a telemetry reading is captured at 5:00 PM local time
   - **THEN** it is stored in the database as the equivalent UTC timestamp

   #### Scenario: API timestamp serialization
   - **WHEN** a client requests telemetry history
   - **THEN** the server returns timestamps as ISO8601 strings ending in 'Z'

3. **Retention:**
   - All telemetry data (power and events) is stored **indefinitely**. No aggregation or deletion is performed.

4. **Optimized Telemetry History API:**
   The system SHALL provide a telemetry history API that supports server-side filtering and truncation. When "All Telemetry" is disabled in the UI, the server MUST return a representative subset of state changes that does not truncate prematurely.

   #### Scenario: Fetching filtered history
   - **WHEN** the UI requests telemetry history with the `events_only` filter enabled
   - **THEN** the server performs another round-trip if necessary to ensure the requested page size is filled with relevant state events before truncating

5. **UI Timezone Translation:**
   - The system SHALL translate all UTC timestamps received from the API into the user's local timezone for display in the Web UI.
   - Browser-provided timezone settings SHALL be used for this translation.

   #### Scenario: Localized display
   - **WHEN** the UI receives "2026-06-13T03:00:00Z" and the user is in UTC+2
   - **THEN** the UI displays "05:00:00" to the user


## Why

Admin users need a quick way to verify that the system is functioning correctly, especially the core integrations with Home Assistant (PV metrics) and MQTT (device communication). This reduces the time needed to diagnose connectivity or data fetching issues.

## What Changes

- Add a "Run Healthcheck" button to the Admin Panel.
- Implement a health check routine that:
    - Verifies PV power source connectivity for all houses.
    - Checks MQTT server status.
    - Pings all configured devices via MQTT to ensure they are responsive.
- Add a visual feedback system (spinner during execution, success/failure icons).
- Return descriptive text for each health check step.

## Capabilities

### New Capabilities
- `system-health`: Defines the requirements and metrics for checking the overall system health, including PV connectivity, MQTT status, and device responsiveness.

### Modified Capabilities
- `admin-ui`: Requirements for the health check interface, including the trigger button, execution state (spinner), and result visualization.

## Impact

- **Frontend**: Admin panel UI additions.
- **Backend**: New health check endpoints and MQTT/PV fetch logic.
- **Dependencies**: Uses existing MQTT and PV (Home Assistant) integrations.

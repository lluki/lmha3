## ADDED Requirements

### Requirement: Admin Health Check Interface
The admin panel SHALL include a "Run Healthcheck" interface that allows administrators to trigger and view the status of system-wide health checks.

#### Scenario: Triggering health check
- **WHEN** the administrator clicks the "Run Healthcheck" button
- **THEN** a spinner is displayed to indicate the check is in progress, and the button is disabled

#### Scenario: Displaying health check results
- **WHEN** the health checks (PV, MQTT, Devices) are completed
- **THEN** the spinner is removed, and each check item displays a short descriptive text of what was done, along with a success (friendly uptick icon) or failure (big red X) indicator

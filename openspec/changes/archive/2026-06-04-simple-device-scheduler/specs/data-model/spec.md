## MODIFIED Requirements

### Requirement: Advanced Boiler Configuration
The system SHALL support configuration for devices in Boiler mode:
- **device_runtime**: The total duration (in minutes) the device must run every day (5 AM to 5 AM window). Default is 180 minutes (3 hours).

#### Scenario: Admin configures boiler runtime
- **WHEN** an admin sets `device_runtime` to 120 for a device
- **THEN** the system persists this value and uses it to calculate scheduling and catch-up windows
- **AND** the configuration is scoped to the device within its respective house

#### Scenario: User modifies own device
- **WHEN** a tenant updates the `expected_load` of a device they own
- **THEN** the system persists the value and applies it to future scheduling decisions
- **AND** the system prevents them from modifying `name`, `mqtt_topic`, or `tenant_id` (Admin only)

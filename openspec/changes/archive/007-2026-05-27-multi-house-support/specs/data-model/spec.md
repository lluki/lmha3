## ADDED Requirements

### Requirement: Multi-House Schema Mapping
The data model SHALL support a hierarchical relationship where Houses contain both Tenants and Devices. Each House MUST store its own integration credentials for external services (Home Assistant).

#### Scenario: Database supports house association
- **WHEN** the database schema is queried
- **THEN** it confirms `tenants` and `devices` tables have a mandatory `house_id` foreign key referencing the `houses` table

## MODIFIED Requirements

### Requirement: Advanced Boiler Configuration
The system SHALL support advanced configuration for devices in Boiler mode:
- **full_charge_n_day**: Number of days (1-8) within which a "full charge" (4h contiguous or aggregate) must occur.
- **min_daily_charge**: Minimum number of minutes/hours the device must run every day (5am to 5am window).

#### Scenario: Admin configures boiler
- **WHEN** an admin sets `full_charge_n_day` to 3 for a device
- **THEN** the system persists this value and uses it to calculate mandatory charge deadlines
- **AND** the configuration is scoped to the device within its respective house

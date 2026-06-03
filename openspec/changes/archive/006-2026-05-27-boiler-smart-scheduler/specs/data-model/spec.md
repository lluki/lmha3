## ADDED Requirements

### Requirement: Advanced Boiler Configuration
The system SHALL support advanced configuration for devices in Boiler mode:
- **full_charge_n_day**: Number of days (1-8) within which a "full charge" (4h contiguous or aggregate) must occur.
- **min_daily_charge**: Minimum number of minutes/hours the device must run every day (5am to 5am window).

#### Scenario: Admin configures boiler
- **WHEN** an admin sets `full_charge_n_day` to 3 for a device
- **THEN** the system persists this value and uses it to calculate mandatory charge deadlines

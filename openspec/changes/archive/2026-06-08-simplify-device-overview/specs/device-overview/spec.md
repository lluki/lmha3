## ADDED Requirements

### Requirement: Simplified Device Overview Dashboard
The system SHALL provide a simplified, read-only device overview dashboard for tenants, displaying only essential facts without granular manual controls.

#### Scenario: Viewing a healthy device
- **WHEN** a tenant views the device overview for a boiler that communicated 2 minutes ago
- **THEN** the system displays the device name, "Health: Yes", its current on/off status, today's runtime in hours/minutes, and the current mode ("Normal" or "Vacation mode until <date>").

#### Scenario: Viewing an unhealthy device
- **WHEN** a tenant views the device overview for a boiler that last communicated 2 hours ago
- **THEN** the system displays "Health: No, <timestamp>" and the other essential facts.

#### Scenario: Omitting removed data
- **WHEN** a tenant views the device overview
- **THEN** the system SHALL NOT display the device owner or granular manual toggle buttons.

### Requirement: Vacation Absence Modal
The system SHALL provide a mobile-friendly action to declare a vacation absence directly from the device overview.

#### Scenario: Opening the vacation modal
- **WHEN** a tenant clicks "Set Vacation Absence"
- **THEN** a modal dialog opens containing a date picker pre-initialized to today, an explanatory note ("The boiler will start 24h before..."), and "Save" / "Cancel" buttons.

#### Scenario: Saving a vacation absence
- **WHEN** a tenant selects a date and clicks "Save"
- **THEN** the modal closes, the vacation is submitted to the backend, and the device's mode updates to display "Vacation mode until <selected date>".

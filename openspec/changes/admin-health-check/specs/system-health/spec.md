## ADDED Requirements

### Requirement: PV Power Source Health Check
The system SHALL verify the connectivity and data-fetching status of the PV power source (Home Assistant integration) for all configured houses.

#### Scenario: PV source is reachable
- **WHEN** the health check is triggered
- **THEN** the system fetches the current PV power metric for each house and marks the check as success if all fetches succeed

#### Scenario: PV source is unreachable
- **WHEN** the health check is triggered and one or more houses fail to return a PV metric
- **THEN** the check is marked as failure for those houses with a descriptive error message

### Requirement: MQTT Server Health Check
The system SHALL verify that the MQTT server is up and reachable by the backend.

#### Scenario: MQTT server is up
- **WHEN** the health check is triggered
- **THEN** the system verifies its connection to the MQTT broker and marks the check as success

#### Scenario: MQTT server is down
- **WHEN** the health check is triggered and the connection to the MQTT broker is lost or cannot be established
- **THEN** the check is marked as failure with a descriptive error message

### Requirement: Device Responsiveness Check
The system SHALL verify that all configured devices are responsive via MQTT.

#### Scenario: All devices respond
- **WHEN** the health check is triggered
- **THEN** the system sends a ping/status request to all devices and receives a response within 12 seconds, marking the check as success

#### Scenario: Some devices are unresponsive
- **WHEN** the health check is triggered and one or more devices do not respond within 12 seconds
- **THEN** those specific devices are marked as unresponsive in the health check results

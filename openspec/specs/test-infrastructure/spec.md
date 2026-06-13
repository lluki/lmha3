# Capability: Test Infrastructure

## Purpose
The Test Infrastructure capability provides a robust and isolated environment for executing tests that require external dependencies, such as an MQTT broker, by managing their lifecycle and configuration automatically.

## Requirements

### Requirement: Automated Mosquitto Broker Management
The test harness SHALL automatically start a local `mosquitto` broker process when initializing tests that require MQTT connectivity.

#### Scenario: Successful broker startup
- **WHEN** the test harness is initialized
- **THEN** a `mosquitto` process is started and becomes ready for connections

### Requirement: Dynamic Port Selection
The test harness SHALL select a dynamic, available port for the `mosquitto` broker to avoid conflicts with other services or concurrent test runs.

#### Scenario: Unique port assignment
- **WHEN** multiple test harnesses are initialized concurrently
- **THEN** each harness starts a `mosquitto` broker on a unique port

### Requirement: Resource Cleanup
The test harness SHALL ensure the `mosquitto` process is terminated when the harness is dropped or when the test process exits.

#### Scenario: Process termination on drop
- **WHEN** the test harness object is dropped
- **THEN** the associated `mosquitto` process is terminated

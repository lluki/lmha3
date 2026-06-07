## ADDED Requirements

### Requirement: Grafana Telemetry Visualization
The system SHALL provide a Grafana instance configured to visualize system telemetry data, including PV production, house consumption, and device states.

#### Scenario: Accessing Grafana
- **WHEN** an administrator navigates to `https://your-domain.com/grafana`
- **THEN** they are presented with the Grafana login page or dashboard
- **AND** the connection is secured via TLS

### Requirement: Automated Data Source Configuration
The Grafana instance SHALL be automatically configured with the local PostgreSQL database as a data source using a Unix socket for communication.

#### Scenario: Data source availability
- **WHEN** Grafana starts
- **THEN** it has a pre-configured data source named "PostgreSQL" pointing to the `lmha3` database
- **AND** the data source connection is functional

### Requirement: Pivot Views for Telemetry
The PostgreSQL database SHALL include helper views that pivot telemetry data to make it easier to query in Grafana. These views MUST support filtering by `house_id`.

#### Scenario: Querying house metrics
- **WHEN** a user queries the `view_telemetry_house_metrics` for a specific `house_id`
- **THEN** they receive a time-series result with `pv_production` and `house_consumption` as separate columns.

#### Scenario: Querying device states
- **WHEN** a user queries the `view_telemetry_device_states` for a specific `house_id`
- **THEN** they receive a time-series result with a column for each device's state (ON/OFF mapped to 1/0).

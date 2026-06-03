## ADDED Requirements

### Requirement: Optimized Telemetry History API
The system SHALL provide a telemetry history API that supports server-side filtering and truncation. When "All Telemetry" is disabled in the UI, the server MUST return a representative subset of state changes that does not truncate prematurely.

#### Scenario: Fetching filtered history
- **WHEN** the UI requests telemetry history with the `events_only` filter enabled
- **THEN** the server performs another round-trip if necessary to ensure the requested page size is filled with relevant state events before truncating

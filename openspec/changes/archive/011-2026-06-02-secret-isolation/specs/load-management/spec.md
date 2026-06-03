## MODIFIED Requirements

### Requirement: Decision Engine (Background Thread)
Runs in a dedicated background thread within the server process. The decision engine SHALL run as a background process that iterates through all configured houses. For each house, it MUST:
- Retrieve current PV production and house consumption from its specific Home Assistant instance using the house-specific host, token, and entity IDs.
- Incorporate house-scoped historical device state data.
- MUST accept an explicit "now" timestamp for deterministic behavior and testing.

#### Scenario: Multi-house scheduling cycle
- **WHEN** the scheduler triggers a new cycle
- **THEN** it sequentially processes "House A" (using its credentials and entity IDs) and "House B" (using its credentials and entity IDs) without cross-contamination of state

#### Scenario: History-aware polling
- **WHEN** the scheduler is invoked
- **THEN** it retrieves current PV/Consumption and the necessary history of ON/OFF events for all Boiler-mode devices for the active house cycle

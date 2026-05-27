# Spec: House Management

## Overview
Management of multiple physical properties (Houses) and their associated credentials and configurations.

## Requirements

### Requirement: House Entity Management
The system SHALL support the management of multiple houses (physical properties). Each house record MUST include a unique name, the Home Assistant host address, and a long-lived access token.

#### Scenario: Admin creates a new house
- **WHEN** an administrator provides a name, HA host, and token
- **THEN** a new house is created and becomes available for tenant association

### Requirement: Admin House Selection
The system SHALL provide administrators with a global house selector that filters all dashboard and management data to the chosen house.

#### Scenario: Admin selects a house
- **WHEN** an administrator selects "House B" from the global dropdown
- **THEN** the UI reloads and displays devices, telemetry, and metrics only for "House B"

### Requirement: UI House Indicators
The system SHALL clearly display the name of the currently active house in the Overview and Admin panel headers.

#### Scenario: House name visibility
- **WHEN** any user views the Overview dashboard
- **THEN** the active house name is displayed prominently as a header above the PV production data

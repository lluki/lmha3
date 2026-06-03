## MODIFIED Requirements

### Requirement: House Entity Management
The system SHALL support the management of multiple houses (physical properties). Each house record MUST include a unique name, the Home Assistant host address, and a long-lived access token. The administration of these entities SHALL follow the summary-detail interaction pattern.

#### Scenario: Admin creates a new house
- **WHEN** an administrator opens the creation dialog and provides a name, HA host, and token
- **THEN** a new house is created and becomes available for tenant association

#### Scenario: Admin updates a house
- **WHEN** an administrator opens a house detail view, enters edit mode, modifies fields, and saves
- **THEN** the house configuration is updated and the view returns to read-only mode

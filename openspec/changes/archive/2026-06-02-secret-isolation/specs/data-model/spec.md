## MODIFIED Requirements

### Requirement: Multi-House Schema Mapping
The data model SHALL support a hierarchical relationship where Houses contain both Tenants and Devices. Each House MUST store its own integration credentials and configuration for external services (Home Assistant).

#### Scenario: Database supports house association
- **WHEN** the database schema is queried
- **THEN** it confirms `tenants` and `devices` tables have a mandatory `house_id` foreign key referencing the `houses` table
- **AND** the `houses` table contains `ha_url`, `ha_token`, `ha_pv_entity_id`, and `ha_consumption_entity_id` columns

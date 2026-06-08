## MODIFIED Requirements

### Requirement: Structured Admin Entity Management
The admin interface SHALL organize Houses, Tenants, and Devices into distinct sections using a summary-detail interaction pattern to minimize clutter and accidental edits. For devices, the detail view SHALL include a tabbed interface to separate configuration from extended features like script management.

#### Scenario: Admin views device details with scripts
- **WHEN** the administrator clicks on a device summary card
- **THEN** a detail view opens with multiple tabs
- **AND** a "Scripts" tab is visible for Shelly Gen2+ devices
- **AND** clicking the "Scripts" tab displays a list of installed scripts with their status (running/stopped) and controls to toggle them

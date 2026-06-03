## MODIFIED Requirements

### Requirement: User Management CRUD
The system SHALL allow administrators to create, read, update, and delete tenant accounts. Deletion MUST be restricted if the tenant has active devices assigned to them. The administration of these entities SHALL follow the summary-detail interaction pattern.

#### Scenario: Admin deletes tenant with devices
- **WHEN** an admin attempts to delete a tenant who owns one or more devices (via the detail view)
- **THEN** the system rejects the deletion with an error message indicating active device ownership

#### Scenario: Admin deletes tenant without devices
- **WHEN** an admin attempts to delete a tenant who has no assigned devices
- **THEN** the system successfully removes the tenant account

#### Scenario: Admin updates tenant
- **WHEN** an admin opens a tenant detail view, enters edit mode, modifies fields (username, house, admin status, or password), and saves
- **THEN** the tenant account is updated and the view returns to read-only mode

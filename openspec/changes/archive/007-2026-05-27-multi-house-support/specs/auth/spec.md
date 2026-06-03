## ADDED Requirements

### Requirement: User Management CRUD
The system SHALL allow administrators to create, read, update, and delete tenant accounts. Deletion MUST be restricted if the tenant has active devices assigned to them.

#### Scenario: Admin deletes tenant with devices
- **WHEN** an admin attempts to delete a tenant who owns one or more devices
- **THEN** the system rejects the deletion with an error message indicating active device ownership

#### Scenario: Admin deletes tenant without devices
- **WHEN** an admin attempts to delete a tenant who has no assigned devices
- **THEN** the system successfully removes the tenant account

## MODIFIED Requirements

### Requirement: Authorization Levels
The system SHALL enforce the following access control levels:
- **House Read Scoping:** A logged-in tenant can view the status of all devices, PV production, house consumption, and telemetry history ONLY for their assigned house.
- **Owner/Admin Write (Toggles):** A tenant can toggle (ON/OFF) devices they own within their house. Administrators can toggle ANY device in any house.
- **Admin Global Access:** Administrators can view and manage all houses, including switching their active view between houses.
- **Admin Only (System Management):** Only users with the `is_admin` flag can create/delete houses, devices, manage tenants, or perform other system-level configuration changes.

#### Scenario: Tenant attempts to view another house
- **WHEN** a tenant from "House A" attempts to access data for "House B"
- **THEN** the system denies access or returns an empty/filtered result set matching "House A"

#### Scenario: Admin toggles device in any house
- **WHEN** an administrator selects a device in "House B" and toggles it
- **THEN** the system executes the command regardless of the admin's personal house association

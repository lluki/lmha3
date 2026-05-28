# Spec: Authentication

## Overview
Secure access control for the `lmha3` web interface.

## Requirements
1. **User Management CRUD:**
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

2. **Authorization Levels:**
   The system SHALL enforce the following access control levels:
   - **House Read Scoping:** A logged-in tenant can view the status of all devices, PV production, house consumption, and telemetry history ONLY for their assigned house.
   - **Owner/Admin Write (Toggles):** A tenant can toggle (ON/OFF) devices they own within their house. Administrators can toggle ANY device in any house.
   - **Admin Global Access:** Administrators can view and manage all houses, including switching their active view between houses.
   - **Admin Only (System Management):** Only users with the `is_admin` flag can create/delete houses, devices, manage tenants, or perform other system-level configuration changes.
   - **User Self-Service:** Any logged-in tenant can change their own password.

   #### Scenario: Tenant attempts to view another house
   - **WHEN** a tenant from "House A" attempts to access data for "House B"
   - **THEN** the system denies access or returns an empty/filtered result set matching "House A"

   #### Scenario: Admin toggles device in any house
   - **WHEN** an administrator selects a device in "House B" and toggles it
   - **THEN** the system executes the command regardless of the admin's personal house association

3. **Session Management:**
   - Secure cookie-based or JWT sessions.
   - Session duration: 24 hours.
3. **Password Security:**
   - Hashes stored using Argon2id.
   - Minimum password length: 12 characters.
4. **Endpoint Protection:**
   - All `/api/*` and UI routes (except `/login`) require a valid session.
   - Unauthorized access results in a redirect to login or 401 Unauthorized.

## User Flow
1. User visits `solar.lluki.me`.
2. Nginx forwards request to Rust backend.
3. Backend checks for session cookie.
4. If missing, redirect to `/login`.
5. Upon successful login, session is created and user redirected to Dashboard.

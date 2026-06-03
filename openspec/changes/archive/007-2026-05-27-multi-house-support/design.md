## Context

The current `lmha3` architecture is designed for a single PV installation. User authentication exists, but all users share a global view of all devices and telemetry. External integration credentials (Home Assistant) are hardcoded or read from local secret files, preventing multi-property management.

## Goals / Non-Goals

**Goals:**
- Transition the data model to support a 1:N relationship between Houses and Tenants/Devices.
- Decentralize Home Assistant configuration by storing it in the database per House.
- Implement strictly scoped data visibility based on user roles and house association.
- Improve administrative efficiency with User CRUD and device discovery.
- Enhance dashboard metrics with cumulative 24h runtime.

**Non-Goals:**
- Implementing fine-grained permissions (staying with Admin/Tenant roles).
- Support for multiple PV systems per single house.
- Automated migration of Home Assistant configurations (manual entry required once).

## Decisions

### 1. Database Schema Evolution
- **New `houses` table**: `id (UUID)`, `name (TEXT)`, `ha_host (TEXT)`, `ha_token (TEXT)`.
- **Update `tenants` table**: Add `house_id (UUID, NOT NULL)`.
- **Update `devices` table**: Add `house_id (UUID, NOT NULL)`.
- **Migration Strategy**: Create a "Default House" during migration and associate all existing tenants/devices with it.

### 2. House-Aware Authorization
- **Session State**: The user session will now include `house_id`.
- **Admin Capability**: Admins will have a `current_view_house_id` in their session, which can be updated via a new `/api/admin/select-house` endpoint.
- **Middleware**: API endpoints will use the session's `house_id` to filter database queries and ensure tenants cannot access data outside their scope.

### 3. Per-House Scheduling
- **Logic Change**: The scheduler background thread will fetch all records from the `houses` table.
- **Sequential Execution**: It will iterate through each house, instantiate a temporary HA client using that house's credentials, and execute the existing load management logic for devices associated with that `house_id`.
- **Rationale**: Given the expectation of "a handful of houses," sequential execution is simplest and avoids complex concurrency management while ensuring state isolation.

### 4. Telemetry History Optimization
- **Server-Side Filtering**: The `/api/telemetry` endpoint will accept a `filter_events_only` boolean.
- **Smart Truncation**: If the filter is active, the server will query the `telemetry` table specifically for `source = 'DEVICE_STATE'`. It will continue querying until it fulfills the requested limit or exhausts the window, preventing the UI from showing empty lists when events are sparse.

### 5. Shelly Discovery
- **Log Parsing**: A new internal utility will scan the `logs/` directory for MQTT topic patterns (e.g., `shellies/shellypro1pm-<id>/...`).
- **Discovery API**: `/api/admin/discover-devices` will return a list of unique IDs found in logs that do not currently exist in the `devices` table.

### 6. 24h Boiler Metrics
- **Calculation**: Sum the intervals between `ON` and `OFF` states (or `now()` if currently `ON`) in the `telemetry` table where `timestamp >= today at 05:00:00`.

## Risks / Trade-offs

- **[Risk] Home Assistant Connectivity** → If one house's HA instance is down, sequential scheduling might hang or delay other houses. *Mitigation*: Implement strict timeouts (e.g., 5s) for all HA REST calls.
- **[Risk] Token Security** → Storing HA tokens in plaintext in the DB is a security downgrade from `secrets/`. *Mitigation*: Design the schema to support encrypted columns in the future; for now, restrict access to the `houses` table to the backend service and DB owner.
- **[Trade-off] Dashboard Reloads** → Forcing a full page reload on house switch is simpler than implementing a full SPA state reset. *Mitigation*: Ensure fast initial load times for the dashboard.

## 1. Database & Schema Migration

- [x] 1.1 Create migration `008_multi_house_support.sql` to add `houses` table.
- [x] 1.2 Update `tenants` and `devices` tables with `house_id` and foreign key constraints.
- [x] 1.3 Seed a "Default House" and associate all existing data with it.
- [x] 1.4 Update Rust database models in `lmha-core/src/db.rs` to include `House` struct and new fields.

## 2. House-Scoped Authentication & API

- [x] 2.1 Update user session struct to include `house_id` (and `current_view_house_id` for admins).
- [x] 2.2 Implement `POST /api/admin/select-house` for administrators to switch their active view.
- [x] 2.3 Refactor database query functions in `lmha-core/src/db.rs` to accept `house_id` for filtering.
- [x] 2.4 Update existing API endpoints in `server/src/main.rs` to enforce house-scoping using session data.

## 3. Per-House Scheduler Refactor

- [x] 3.1 Update scheduler logic in `lmha-core/src/scheduler.rs` to fetch all houses from the DB.
- [x] 3.2 Refactor the main loop to iterate through each house and execute load management using house-specific HA credentials.
- [x] 3.3 Implement 5s timeouts for Home Assistant REST requests to prevent blocking the scheduler.
- [x] 3.4 Update `lmha-core/src/ha.rs` to support dynamic HA host/token injection.

## 4. Telemetry & Metrics Improvements

- [x] 4.1 Update `GET /api/telemetry` to support server-side filtering and truncation for events.
- [x] 4.2 Implement `calc_boiler_runtime_24h` in `db.rs` to compute runtime since 5:00 AM.
- [x] 4.3 Update the dashboard data API to include the new 24h runtime metric.

## 5. Administrative UI Enhancements

- [x] 5.1 Implement User CRUD API (Create, Read, Update, Delete) with safety check for active devices.
- [x] 5.2 Implement Shelly ID discovery API by scanning `logs/mqtt.log`.
- [x] 5.3 Update Admin UI to include the House selector (for admins) and the new User Management section.
- [x] 5.4 Update Device Creation form to suggest IDs found via discovery.
- [x] 5.5 Remove redundant system logs from the Admin panel.

## 6. Dashboard & UI Polish

- [x] 6.1 Update `index.html` and `app.js` to display the active house name in the header.
- [x] 6.2 Ensure UI reloads on house switch or manual device toggle.
- [x] 6.3 Update the boiler status card to show "24h Runtime" instead of "Last On".
- [x] 6.4 Fix the history table behavior when "Show All Telemetry" is unchecked to use the new optimized API.

## 7. Verification

- [x] 7.1 Verify multi-house scheduling with a simulated second house.
- [x] 7.2 Verify tenant isolation (House A tenant cannot see House B data).
- [x] 7.3 Manually verify User CRUD and Shelly discovery in the Admin UI.

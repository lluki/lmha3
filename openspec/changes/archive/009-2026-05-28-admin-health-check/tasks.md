## 1. Backend Implementation

- [x] 1.1 Add `+/rpc-response` to MQTT subscriptions in `server/src/main.rs` to ensure full coverage of incoming messages.
- [x] 1.2 Implement the health check logic in a new route `(GET) (/api/admin/healthcheck)` in `server/src/main.rs`.
- [x] 1.3 Implement the PV check logic: iterate through all houses and use `lmha_core::ha::fetch_ha_state` to verify connectivity.
- [x] 1.4 Implement the MQTT check logic: verify the MQTT client in `AppState` is initialized and functional.
- [x] 1.5 Implement the Device check logic: record current time, broadcast status requests to all devices, sleep for 12 seconds, and verify `last_heartbeat` updates in the database.
- [x] 1.6 Ensure the endpoint is protected by the `is_admin` check.

## 2. Frontend Implementation

- [x] 2.1 Add health check state management (triggering, loading spinner, results storage) to the admin panel in `server/public/app.js`.
- [x] 2.2 Add the "Run Healthcheck" button to the Admin UI.
- [x] 2.3 Implement the results visualization: display short descriptive text for each check with success (uptick) or failure (red X) icons.
- [x] 2.4 Ensure the UI handles the 10s wait gracefully with a spinner and disabled button.

## 3. Verification

- [x] 3.1 Verify that only admin users can trigger the health check.
- [x] 3.2 Verify that the PV check correctly identifies houses with unreachable Home Assistant instances.
- [x] 3.3 Verify that the MQTT check correctly identifies broker connectivity issues.
- [x] 3.4 Verify that the Device check correctly identifies responsive vs. unresponsive devices after the 10s timeout.

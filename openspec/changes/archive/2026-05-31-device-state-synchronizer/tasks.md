## 1. Database & Data Model

- [x] 1.1 Create migration `010_add_device_sync_fields.sql` to add `desired_state`, `last_request_time`, and `last_feedback_time` to the `devices` table.
- [x] 1.2 Update `Device` struct in `lmha-core/src/lib.rs` to include the new fields.
- [x] 1.3 Update `Db` methods in `lmha-core/src/db.rs` to persist and load the new fields.
- [x] 1.4 Add `get_out_of_sync_devices` method to `Db` to retrieve devices where `desired_state != current_state`.

## 2. Server-Side Synchronizer

- [x] 2.1 Implement event-driven sync in `run_main_loop`: when a device sends any message, check if `desired_state != current_state` and sync if needed.
- [x] 2.2 Update `run_main_loop` in `server/src/main.rs` to update `last_feedback_time` on all relevant MQTT messages (rpc-response, status, online).
- [x] 2.3 Update API `/api/devices/{id}/toggle` to set `desired_state` and `last_request_time` in addition to sending the immediate MQTT command.
- [x] 2.4 Update `run_scheduler_loop` to update `desired_state` and `last_request_time` and send the MQTT command immediately.
- [x] 2.5 Implement initial state alignment on server startup: `desired_state = current_state`.
- [x] 2.6 Implement active background polling loop (every 5 minutes): send `Shelly.GetStatus` to devices with `last_feedback_time` > 5m.
- [x] 2.7 Add `instance_id` and `instance_priority` to `Config` in `lmha-core/src/config.rs`.
- [x] 2.8 Implement `InstanceHeartbeat` logic in `server/src/main.rs`: publish heartbeat and monitor other instances.
- [x] 2.9 Implement "Passive Mode" check in scheduler and synchronizer to skip MQTT publishes if a higher-priority instance exists.

## 3. Frontend & UI

- [x] 3.1 Update `server/public/app.js` to display "Offline" status if request fails (>20s) OR device is idle (>5m).
- [x] 3.2 Add a visual indicator for "Syncing..." when `desired_state != current_state`.
- [x] 3.3 Add a "BIG FAT WARNING" banner in the UI if the instance is in Passive mode due to a conflict.
- [x] 3.4 Update `server/public/style.css` to include styles for the Offline badge, Syncing indicator, and Conflict warning.

## 4. Verification & Testing

- [x] 4.1 Create a test case in `server/tests/device_sync_tests.rs` to verify that an out-of-sync device eventually receives a command.
- [x] 4.2 Verify that a device is correctly marked as "Offline" in the UI when it fails to respond within 20s.
- [x] 4.3 Verify that server restart triggers a state reconciliation for all devices.

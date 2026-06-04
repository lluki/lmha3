## 1. Database & Data Model

- [x] 1.1 Create migration `012_simple_boiler_runtime.sql` to add `device_runtime` and remove `full_charge_n_day`, `min_daily_charge`
- [x] 1.2 Update `Device` struct in `lmha-core/src/lib.rs`
- [x] 1.3 Update `DeviceContext` struct in `lmha-core/src/scheduler.rs`
- [x] 1.4 Update `Db` struct and methods in `lmha-core/src/db.rs` (list_devices, update_device_config, etc.)

## 2. Scheduler Implementation

- [x] 2.1 Refactor `calculate_runtime_mins` in `scheduler.rs` to support contiguous run tracking
- [x] 2.2 Implement `has_run_today` helper in `scheduler.rs` using the 5 AM - 5 AM window
- [x] 2.3 Rewrite `decide_action` to implement the new algorithm (PV activation, random selection, 1 AM catch-up, lock ON for duration)
- [x] 2.4 Update `run_scheduler_loop` in `server/src/main.rs` to pass correct device context

## 3. API & Backend

- [x] 3.1 Update `DevicePatch` struct in `server/src/main.rs`
- [x] 3.2 Update `/api/devices/{id}` PATCH handler in `server/src/main.rs`
- [x] 3.3 Update `/api/devices` GET handler to return new fields
- [x] 3.4 Ensure `calc_boiler_runtime_24h` uses the 5 AM start time correctly

## 4. Frontend & UI

- [x] 4.1 Update device configuration form in `server/public/app.js` to include `device_runtime`
- [x] 4.2 Remove `full_charge_n_day` and `min_daily_charge` from the UI
- [x] 4.3 Update device detail view and overview cards to show `device_runtime`

## 5. Verification

- [x] 5.1 Update unit tests in `lmha-core/src/scheduler.rs`
- [x] 5.2 Update integration tests in `server/tests/device_sync_tests.rs`
- [x] 5.3 Verify catch-up logic (1 AM) using a simulated clock
- [x] 5.4 Verify PV random activation logic
- [x] 5.5 Verify that Force ON/OFF and Manual Overrides still take precedence
- [x] 5.6 Run all project tests and verify no regressions

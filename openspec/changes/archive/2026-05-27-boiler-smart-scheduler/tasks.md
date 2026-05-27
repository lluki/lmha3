## 1. Database & Models

- [x] 1.1 Create migration `007_boiler_advanced_config.sql` to add `full_charge_n_day` and `min_daily_charge` to `devices`.
- [x] 1.2 Update `Device` struct in `lmha-core/src/lib.rs` with new fields and default values.
- [x] 1.3 Update `db.rs` to handle persistence of new configuration parameters.

## 2. Core Scheduler Implementation

- [x] 2.1 Update `SchedulerInput` and `DeviceContext` in `scheduler.rs` to include historical event telemetry.
- [x] 2.2 Implement 5am-5am window logic for calculating daily runtimes.
- [x] 2.3 Implement mandatory charge detection logic (full charge and daily min).
- [x] 2.4 Refactor `decide_action` to use runtime-based sorting (longest idle / longest running).
- [x] 2.5 Update the deactivation threshold logic to account for the 30% PV load requirement.
- [x] 2.6 Integrate history-based mandatory triggers into the main `decide_action` flow.

## 3. Server & API

- [x] 3.1 Update the telemetry fetching logic to provide the last 8 days of state changes to the scheduler.
- [x] 3.2 Update device configuration endpoints to validate `full_charge_n_day` (max 8).
- [x] 3.3 Add `FULL_CHARGE_DURATION` constant (4h) and `CHARGE_WINDOW_START` (1am).

## 4. Web UI

- [x] 4.1 Update device configuration UI in `app.js` to show advanced boiler settings.
- [x] 4.2 Add validation for the new inputs in the admin interface.

## 5. Verification & Tests

- [x] 5.1 Implement a 3-day randomized history test case as described in `prop.md`.
- [x] 5.2 Verify mandatory charge triggers at 1am when quotas aren't met.
- [x] 5.3 Verify fair distribution (longest idle device turns on first).
- [x] 5.4 Verify grid-assisted retention (30% rule).
- [x] 5.5 Verify incremental activation of multiple devices.

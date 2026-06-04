## Why

The current "Boiler" scheduling algorithm, which manages mandatory charging, minimum daily charges, and periodic full charges, is overly complex to configure and understand. This change replaces it with a simpler, more predictable algorithm that guarantees each device runs for a fixed duration once per day, preferably using PV surplus but falling back to a 1 AM catch-up window.

## What Changes

- **ALGORITHM REPLACEMENT**: The existing `BOILER` scheduling logic is replaced with a "Fixed Daily Runtime" algorithm.
- **PRESERVATION**: Support for "Force ON", "Force OFF", and "Manual Override" (1-hour window) is preserved and remains higher priority than the automated scheduler.
- **NEW LOGIC**:
    - The scheduling day is defined as 5:00 AM to 5:00 AM.
    - **PV Activation**: If PV surplus (PV Production - House Consumption) exceeds 70% of a device's power rating, the system selects one device at random from the set of devices that have not yet run today.
    - **Guaranteed Runtime**: Once activated, a device remains ON for its full "device runtime" (default 3 hours), regardless of PV production changes.
    - **Catch-up Window**: At 1:00 AM, any device that has not yet run during the current 5 AM-5 AM cycle is forced ON for its full runtime.
- **CONFIG MODIFICATIONS**:
    - Add `device_runtime` (duration in hours/minutes) to the device configuration.
    - **REMOVAL**: Remove `min_daily_charge` and `full_charge_n_day` from the configuration and UI.
- **BREAKING**: Database schema changes to remove old boiler-specific fields and add the new runtime field. Existing scheduling logic will be completely replaced.

## Capabilities

### Modified Capabilities
- `load-management`: Replace the complex `BOILER` mode logic and mandatory charging requirements with the new fixed runtime algorithm.
- `admin-ui`: Update the device management UI to reflect the new configuration fields and remove deprecated ones.
- `data-model`: Update the device entity schema to support the new `device_runtime` field and remove `min_daily_charge` and `full_charge_n_day`.

## Impact

- **Core Scheduler**: `lmha-core/src/scheduler.rs` requires a complete rewrite of the boiler scheduling branch.
- **Database**: Migrations needed to update the `devices` table schema.
- **Backend API**: Update DTOs and database queries in the server and admin crates.
- **Frontend**: Update UI forms in the dashboard/admin panel.
- **Tests**: Major updates to `server/tests/device_sync_tests.rs` and other scheduling-related tests to verify the new invariants (each device runs exactly once for the specified duration every 5 AM-5 AM cycle).

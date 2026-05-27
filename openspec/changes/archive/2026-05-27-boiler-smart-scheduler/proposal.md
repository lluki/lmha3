## Why

The current boiler scheduling mode only considers real-time excess production. It lacks the ability to guarantee long-term appliance health (e.g., preventing Legionella with full charges) and daily minimum energy quotas, nor does it fairly distribute runtime across multiple devices.

## What Changes

- **Guaranteed Charging Cycles**: Support for `full_charge_n_day` (guarantee 4h charge every N days) and `min_daily_charge` (daily minimum).
- **Fair Scheduling Strategy**: Switch from random device selection to "longest running" for deactivation and "longest idle" for activation.
- **Improved Load Accounting**: Refine the deactivation threshold to account for fixed loads and partial grid fetching to keep devices running during minor production dips.
- **History-Aware Scheduling**: The scheduler will now incorporate historical telemetry (on/off events) to make decisions about mandatory charges.
- **Admin UI Updates**: Add configuration for `full_charge_n_day` (max 8) and `min_daily_charge`.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `data-model`: Add `full_charge_n_day` and `min_daily_charge` to Device entity.
- `load-management`: Implement mandatory charge logic, cycle detection (5am-5am), and runtime-based prioritization.

## Impact

- **lmha-core**: Significant refactoring of the `scheduler` to handle history and stateful constraints (while remaining function-stateless). Update database models and telemetry fetching.
- **server**: API updates for device configuration and admin UI enhancements.
- **migrations**: New migration for device configuration fields.

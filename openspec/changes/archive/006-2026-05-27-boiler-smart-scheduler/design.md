## Context

The `lmha3` system manages devices primarily based on real-time solar production. The current "Boiler" mode uses a random selection strategy and lacks memory of past events, making it impossible to guarantee hygiene-related cycles (full charges) or daily minimums. This design extends the scheduler to be history-aware while maintaining its functional purity (stateless core).

## Goals / Non-Goals

**Goals:**
- Implement a 5am-5am "Scheduling Day".
- Support `full_charge_n_day` and `min_daily_charge` constraints.
- Implement runtime-based prioritization (Fairness).
- Refine load accounting to allow partial grid usage (30% PV threshold).

**Non-Goals:**
- Persistent state in the scheduler (it must derive everything from input history).
- Forecasting weather/production.

## Decisions

### 1. Scheduler Input Extension
The `SchedulerInput` struct will be updated to include:
- `history`: A list of `(timestamp, device_id, state)` events covering the last 8 days (max `full_charge_n_day`).
- `now`: Explicit `DateTime<Utc>` for determinism.

*Rationale:* By passing history into the core function, the scheduler remains a pure, testable function while having the context needed for long-term constraints.

### 2. Time Window & Deadlines
- **Full Charge**: Defined as a cumulative 4 hours within one 5am-5am window.
- **Cycle Period**: 5am to 5am next day.
- **Mandatory Trigger**: On the deadline day (determined by `full_charge_n_day`), if the 4h goal hasn't been met, the scheduler will force the device ON from 1am to 5am.

### 3. Fair Selection Strategy
- **Activation Pool**: If production > threshold, eligible devices are sorted by `duration_since_last_on` (descending).
- **Deactivation Pool**: If production < threshold, active devices are sorted by `current_on_duration` (descending).

*Rationale:* This prevents "device starving" where one device stays off while others cycle.

### 4. Grid-Assisted Hysteresis
Instead of checking `PV > (Total_Consumption + Margin)`, we use:
`PV > (House_Consumption_Excl_Device + 0.3 * Device_Expected_Load)`.
This handles the case where a device is ON but potentially cycling its own internal thermostat (load fluctuation).

### 5. Admin Constraints
- `full_charge_n_day`: Range 1-8.
- `FULL_CHARGE_DURATION`: Constant 4h (Hardcoded in core).

## Risks / Trade-offs

- **[Risk] High History Volume** → **Mitigation**: Filter history events to only include the specific devices in Boiler mode and only the last 8 days.
- **[Risk] Thermostat Ambiguity** → **Mitigation**: The design assumes that if a device is "ON" (MQTT state), it counts towards the quota, even if its internal thermostat has temporarily cut the actual load. This aligns with the requirement to deal with 2kW house consumption while 2x4kW devices are marked ON.
- **[Risk] Clock Skew** → **Mitigation**: Use UTC for all logic, with 5am local time mapped to the appropriate UTC offset.

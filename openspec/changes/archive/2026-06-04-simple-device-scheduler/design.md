## Context

The current scheduling logic for `BOILER` mode is complex, involving mandatory daily minimums and multi-day "full charge" deadlines. This complexity makes the system hard to maintain and difficult for users to configure. The goal is to replace this with a simpler "Single Daily Runtime" algorithm.

## Goals / Non-Goals

**Goals:**
- Replace complex boiler scheduling with a simple daily runtime guarantee (default 3 hours).
- Implement a 5:00 AM to 5:00 AM scheduling cycle.
- Prioritize PV surplus activation (70% threshold) with random selection among eligible devices.
- Implement a 1:00 AM catch-up window for devices that haven't run by then.
- Simplify the database schema and UI by removing deprecated fields.
- **PRESERVE**: Maintain existing "Force ON", "Force OFF", and manual override (1-hour window) functionality as higher priority than the scheduler.

**Non-Goals:**
- Supporting multiple runs per day for a single device.
- Complex fair-sharing between multiple houses (stays house-scoped).
- Changing manual override behavior (remains higher priority).

## Decisions

### 1. Data Model Simplification
We will remove `full_charge_n_day` and `min_daily_charge` from the `devices` table and add `device_runtime` (INTEGER, minutes).
- **Rationale**: A single parameter is easier to understand and covers the core requirement of "guaranteed daily runtime".
- **Migration**: Existing devices will be migrated to a default `device_runtime` of 180 minutes (3 hours).

### 2. Scheduler Logic Overhaul
The `decide_action` function in `lmha_core::scheduler.rs` will be refactored to:
- Identify if a device "has run" in the current 5 AM - 5 AM window. A device is considered to have run if it has completed a contiguous session of at least `device_runtime` minutes since 5 AM.
- If a device is currently in the middle of its runtime (activated < `device_runtime` minutes ago), the scheduler SHALL keep it ON.
- **Random Selection**: When multiple devices are eligible for PV-based activation, the system will use a provided RNG to select one at random.
- **Catch-up**: At 1:00 AM, the logic transitions from "wait for PV" to "force ON" for all devices that haven't run.

### 3. "Run Tracking" via Telemetry
The system already uses `db.calc_boiler_runtime_24h` and `db.get_device_history`. We will update `calculate_runtime_mins` or create a new `has_completed_runtime_today` helper that specifically looks for a completed session or enough accumulated time in the current cycle.
- **Decision**: To match the user's "stays on for 3h" requirement, we will track the *last activation time* within the current cycle. If a device was turned ON by the scheduler and hasn't reached its runtime yet, it is "locked" in the ON state.

### 4. Admin UI Consolidation
The device configuration form will be updated to remove the two old fields and add a single "Daily Runtime (minutes)" field.

## Risks / Trade-offs

- **[Risk] Multiple devices starting at once at 1 AM** → [Mitigation] This might cause a surge in grid demand. However, given the project scale (household), this is acceptable. If needed, a small stagger could be added later.
- **[Risk] Manual override interrupting a run** → [Mitigation] Manual overrides (`FORCE_ON`/`FORCE_OFF`) will continue to take precedence. If a user turns a device OFF manually during its 3h window, it will stay OFF until the override expires. The scheduler will then re-evaluate if it still needs to complete its runtime (e.g., in the 1 AM window).
- **[Risk] History gaps** → [Mitigation] If telemetry is missing, the scheduler might re-run a device. This is a "fail-safe" towards ensuring the run happens.

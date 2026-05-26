# Design: 009-Device Management UI

## Data Model Updates

### Devices Table
Add the following columns:
- `expected_load`: `INTEGER` (Watts)
- `scheduling_type`: `TEXT` or `ENUM` (`none`, `force-off`, `force-on`, `boiler`)
- `scheduling_until`: `TIMESTAMP` (Optional)

## Backend Implementation

### Rust Sum Type
In `lmha-core` or `server/src/main.rs`:
```rust
enum SchedulingType {
    None,
    ForceOff { until: DateTime<Utc> },
    ForceOn { until: DateTime<Utc> },
    Boiler,
}
```
*Note: The database representation will likely be a string for `type` and a separate nullable timestamp column (null only for `None` and `Boiler`).*

### Scheduling Algorithm logic
1. If `None`: Do nothing.
2. If `ForceOff`:
   - If `now() < until`: Ensure device is OFF.
   - If `now() >= until`: Switch mode to `Boiler` in DB and proceed with Boiler logic.
3. If `ForceOn`:
   - If `now() < until`: Ensure device is ON.
   - If `now() >= until`: Switch mode to `Boiler` in DB and proceed with Boiler logic.
4. If `Boiler`: Execute existing production-dependent logic.

### Logging
- Log a `tracing::info!` message when a device enters a `force-*` state.
- Log a `tracing::info!` message when a device leaves a `force-*` state (transitions to `boiler`).

## API Endpoints
- `GET /api/devices`: Returns list of devices with their scheduling config.
- `PATCH /api/devices/{id}`: Updates `expected_load`, `scheduling_type`, and `scheduling_until`.

## Frontend UI
- **Device List:** Display name, `expected_load`, and current `scheduling_type`.
- **Edit Controls:**
  - Input for `expected_load` (always visible).
  - Dropdown for `scheduling_type`.
  - Date/Time picker for `scheduling_until` (only visible when `force-on` or `force-off` is selected).
- **Validation:** Ensure `scheduling_until` is in the future when setting a force state.

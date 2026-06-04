## Why

The current Admin UI obscures the "Runtime" setting when a device is in an override mode (Force On/Off), even though the device returns to "Boiler" mode once the override expires. Furthermore, the "Until" field is shown when it's not applicable, and the lack of a "Last Heartbeat" in the overview modal hinders quick connectivity checks.

## What Changes

- **Device Edit Modal**: 
    - Always display the "Runtime" field, regardless of the selected mode.
    - Only display the "Until" (override expiration) field if the mode is set to "Force On" or "Force Off".
    - Rename the "Save All" button to "Save".
- **Device Overview Modal**:
    - Always display the "Runtime" value.
    - Add a "Last Heartbeat" field to show when the device last communicated with the system.
    - Only display the "Until" value if the device is currently in an override state.

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `admin-ui`: Refine device modal field visibility logic, add heartbeat display to overview, and update action button labeling.

## Impact

- **Frontend**: Modifications to `server/public/app.js` and potentially `server/public/index.html` (if template-based) to update modal rendering logic and button text.
- **Data Model**: No changes needed; `last_heartbeat` (likely `last_feedback_time` or similar) and `device_runtime` are already available.

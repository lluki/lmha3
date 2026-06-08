## Why

Regular users recently had manual toggle controls removed from their overview to simplify the interface and prevent accidental overrides. However, administrators still need a quick way to manually control devices for testing, maintenance, or emergency overrides without having to change the scheduling mode to FORCE_ON/OFF.

## What Changes

- **Admin UI**: Add a "Toggle" button to the device details modal in the Admin panel.
- **Admin UI**: Optionally add a quick toggle button to the device cards in the Admin grid.
- **Cleanup**: Remove dead code in `app.js` left over from previous UI simplifications (`handleSchedulingChangeOverview`, `updateDeviceConfigOverview`).
- **Backend**: Reuse the existing `POST /api/devices/{id}/toggle` endpoint.

## Capabilities

### New Capabilities
- `admin-device-toggle`: Provides administrators with a direct manual toggle button in the admin device management interface.

### Modified Capabilities
- `admin-ui`: Update the admin device management screens to include the new toggle control.

## Impact

- **UI**: New button in the admin device modal and/or cards.
- **API**: No changes (existing endpoint is used).
- **Dependencies**: None.

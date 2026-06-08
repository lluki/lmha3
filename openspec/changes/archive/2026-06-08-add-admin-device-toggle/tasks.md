## 1. UI Implementation (Summary Card)

- [x] 1.1 Update `renderDeviceCard` in `server/public/app.js` to add a small "Toggle" button/icon in the header or body.
- [x] 1.2 Implement `event.stopPropagation()` on the card toggle button to prevent opening the detail modal.
- [x] 1.3 Ensure the card toggle button calls `window.toggleDevice(d.id, 'admin')`.

## 2. UI Implementation (Detail Modal)

- [x] 2.1 Update `renderDeviceDetails` in `server/public/app.js` to include a "Toggle Device" button in the read-only view of the "Settings" tab.
- [x] 2.2 Place the "Toggle Device" button alongside the "Edit Config" and "Delete Device" buttons.
- [x] 2.3 Ensure the modal toggle button calls `window.toggleDevice(d.id, 'admin')`.

## 3. Cleanup & Optimization

- [x] 3.1 Remove dead function `window.handleSchedulingChangeOverview` in `server/public/app.js`.
- [x] 3.2 Remove dead function `window.updateDeviceConfigOverview` in `server/public/app.js`.
- [x] 3.3 Ensure `window.toggleDevice` is correctly integrated and no longer "dead".

## 4. Verification

- [x] 4.1 Verify that clicking the toggle on a device card successfully sends the MQTT command and refreshes the admin view.
- [x] 4.2 Verify that clicking the toggle in the device detail modal successfully sends the MQTT command and refreshes the admin view.
- [x] 4.3 Ensure the "Syncing..." status appears correctly during the transition.
- [x] 4.4 Confirm that no regressions were introduced by removing the dead functions.

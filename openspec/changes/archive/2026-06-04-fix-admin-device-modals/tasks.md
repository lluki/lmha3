## 1. Modify Device Edit Modal

- [x] 1.1 Update `renderDeviceDetails` in `server/public/app.js` to remove the conditional `display:none` style from the `modal-boiler-config` div.
- [x] 1.2 Update the submit button text from "Save All" to "Save" in the device edit form template within `renderDeviceDetails`.
- [x] 1.3 Refactor `handleSchedulingChangeInModal` to remove the logic that hides/shows `modal-boiler-config`.

## 2. Modify Device Overview Modal

- [x] 2.1 Update the overview section of `renderDeviceDetails` to always display the "Daily Runtime" row.
- [x] 2.2 Add a new "Last Heartbeat" row to the overview section of `renderDeviceDetails` using the formatted `lastFeedback` variable.
- [x] 2.3 Ensure the "Mode Until" row in the overview is correctly displayed only when an override is active.

## 3. Verification

- [x] 3.1 Manually verify that the Runtime field is visible in the Edit modal for all scheduling modes.
- [x] 3.2 Manually verify that the Until field in the Edit modal only appears for "Force ON" and "Force OFF" modes.
- [x] 3.3 Manually verify that "Daily Runtime" and "Last Heartbeat" are visible in the Read-only Overview modal.
- [x] 3.4 Verify that clicking "Save" successfully persists changes.

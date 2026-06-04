## Why

The current device overview page in the Web UI is overly complex and displays too many modifiable options that users should no longer interact with directly. This change simplifies the user experience by providing a clean, read-only view of essential device facts and replacing granular controls with a single, mobile-friendly "Set Vacation Absence" action.

## What Changes

- **Remove granular device settings**: The overview page will no longer allow direct modifications of individual settings.
- **Remove device owner**: The owner of the device will no longer be displayed.
- **Simplify device information**: Display only key facts:
  - Device Name
  - Health Status: Yes or (No, <last seen>)
  - On/Off Status: Yes, No, or Unknown
  - Today's Runtime: Equal to the current 24h runtime
  - Mode: "Normal" for Boiler, or "Vacation mode until <day>" for Force Off.
- **Add Vacation Mode action**: Introduce a prominent, mobile-friendly "Set Vacation Absence" button.
- **Vacation Modal**: A dialog with a simple date picker (day only, pre-initialized to today), an explanatory note about the boiler starting 24h early, and Save/Cancel buttons.
- **Scheduling logic**: Saving the vacation sets the device state to `FORCE OFF` until 5am of the day *before* the selected return date.
- **BREAKING**: Removed direct access to all other modifiable device options from the UI. Anything not explicitly mentioned will be removed from the overview.

## Capabilities

### New Capabilities

- `device-overview`: Replaces the current complex overview with a simplified, read-only dashboard and a single "Set Vacation Absence" flow.

### Modified Capabilities

- `load-management`: Updates the scheduling behavior and API to support setting a device to `FORCE OFF` until a specific date and time (5am of the previous day), rather than just simple manual toggles.

## Impact

- **Web UI (`server/public/app.js`, `index.html`)**: Complete redesign of the "Overview" tab for devices.
- **API/Backend**: A new or modified endpoint to handle the "Set Vacation Absence" request, computing the end time as 5am of the day prior to the selected date.
- **Data Model**: The device scheduling state needs to support a specific end timestamp for the `FORCE OFF` state, which may already be partially supported but needs to be mapped to the new vacation logic.

## 1. Backend Implementation

- [x] 1.1 Add logic to calculate `scheduling_until` timestamp for 5:00 AM of the day before the chosen vacation date.
- [x] 1.2 Implement the `POST /api/devices/{id}/vacation` endpoint in `server/src/main.rs` that accepts a return date.
- [x] 1.3 Ensure the endpoint updates the device's scheduling state to `FORCE_OFF` with the calculated `scheduling_until` timestamp and saves to the database.

## 2. Frontend Implementation - HTML & Modal

- [x] 2.1 Remove existing granular manual toggle controls and owner information from the device overview template in `server/public/app.js` or `index.html`.
- [x] 2.2 Add the simplified display fields (Health, Status, Runtime, Mode) and the "Set Vacation Absence" button to the UI layout.
- [x] 2.3 Create the `<dialog>` modal in `server/public/index.html` for vacation scheduling with a date picker (`<input type="date">`) and informational text.

## 3. Frontend Implementation - JavaScript Logic

- [x] 3.1 Implement open/close logic for the vacation modal in `server/public/app.js`.
- [x] 3.2 Initialize the modal's date picker to the current date when opened.
- [x] 3.3 Implement the "Save" action to send the selected date to `POST /api/devices/{id}/vacation`.
- [x] 3.4 Reload the device dashboard immediately after a successful save.

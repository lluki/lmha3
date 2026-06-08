## 1. Backend: Async-to-Sync RPC Bridge

- [x] 1.1 Add `SyncSender` registry for RPC responses in `AppState`.
- [x] 1.2 Implement RPC ID generation (atomic counter).
- [x] 1.3 Update MQTT event loop to capture responses from `+/rpc-response/rpc` and dispatch them to the registry.
- [x] 1.4 Implement a helper function `send_rpc_sync` that sends a message and waits for the response with a timeout.

## 2. Backend: API Endpoints

- [x] 2.1 Implement `GET /api/admin/devices/{id}/scripts` endpoint.
- [x] 2.2 Implement `POST /api/admin/devices/{id}/scripts/{script_id}/start` endpoint.
- [x] 2.3 Implement `POST /api/admin/devices/{id}/scripts/{script_id}/stop` endpoint.
- [x] 2.4 Add authorization checks (admin or owner) to the new endpoints.

## 3. Frontend: Device Detail Tabs

- [x] 3.1 Refactor the device detail modal in `app.js` to support tabbed navigation.
- [x] 3.2 Add \"Settings\" and \"Scripts\" tabs to the device detail view.
- [x] 3.3 Implement the \"Scripts\" tab UI: list scripts, show status, and provide start/stop buttons.
- [x] 3.4 Connect the \"Scripts\" tab to the new backend API endpoints.


## 4. Verification & Testing

- [x] 4.1 Verify listing scripts in the Admin UI.
- [x] 4.2 Verify starting and stopping a script from the Admin UI.
- [x] 4.3 Test timeout handling by simulating a non-responsive device.
- [x] 4.4 Verify that non-admin/non-owners cannot access the script endpoints.

## 5. Script Content Editor

- [x] 5.1 Implement `GET /api/admin/devices/{id}/scripts/{script_id}/code` endpoint (Script.GetCode).
- [x] 5.2 Implement `PUT /api/admin/devices/{id}/scripts/{script_id}/code` endpoint (Script.PutCode).
- [x] 5.3 Add an \"Edit\" button to the script list table in `app.js`.
- [x] 5.4 Create a script editor view/modal that fetches and displays script code in a textarea.
- [x] 5.5 Implement the \"Save\" functionality to update the script code via the API.
- [x] 5.6 Add verification tests for getting and putting script code.

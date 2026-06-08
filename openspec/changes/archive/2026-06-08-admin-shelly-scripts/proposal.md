## Why

Currently, Shelly Pro/Gen2+ scripts must be managed via the device's local web interface or complex manual MQTT calls. Integrating script management into the `lmha3` admin panel allows administrators and device owners to easily list, enable, and disable scripts without leaving the central management dashboard.

## What Changes

- **Admin UI**: Add a "Scripts" tab to the Shelly device detail view.
- **Backend API**: Add endpoints to list, start, and stop scripts on a specific device.
- **MQTT Integration**: Implement an async-to-sync RPC mechanism to interact with Shelly scripts over MQTT and receive immediate responses for the API.
- **Access Control**: Ensure only admin users and device owners can access script management features.

## Capabilities

### New Capabilities
- `shelly-script-management`: Capability to list, enable (start), and disable (stop) scripts on Shelly Gen2+ devices via MQTT RPC.

### Modified Capabilities
- `shelly-protocol`: Update to include the async-to-sync RPC pattern for script methods.
- `admin-ui`: Extend device detail view with a scripts management tab.

## Impact

- `server/src/main.rs`: New API endpoints and async-to-sync MQTT RPC implementation.
- `server/public/app.js`: New frontend logic for the scripts tab.
- `server/public/index.html`: UI updates for the new tab.
- `lmha-core/src/db.rs`: Potential updates for device owner verification if not already fully integrated for this scope.

## Context

The `lmha3` system manages Shelly Gen2+ devices primarily via MQTT. While most operations are asynchronous (send command, wait for state update via `NotifyStatus`), script management requires a synchronous request-response cycle for a better user experience in the admin panel.

## Goals / Non-Goals

**Goals:**
- Implement a generic async-to-sync MQTT RPC bridge in the Rust backend.
- Expose script management (list, start, stop) via new API endpoints.
- Update the Admin UI to support a tabbed interface for device details, including a "Scripts" tab.

**Non-Goals:**
- Editing script code via the Admin UI (PutCode).
- Creating or deleting scripts.
- Managing script schedules.

## Decisions

### 1. Async-to-Sync Bridge using Channels and a Global Registry
To bridge the asynchronous nature of MQTT (`rumqttc`) with synchronous API handlers (`rouille`), we will:
- Use a `HashMap<u32, SyncSender<serde_json::Value>>` protected by a `Mutex` to register pending RPC requests.
- When an API request comes in, generate a unique ID, create a channel, and register it in the map.
- Send the MQTT message with the unique ID and `src` set to `<topic>/rpc-response`.
- The MQTT event loop will check incoming messages on `+/rpc-response/rpc`. If the `id` matches an entry in the map, it sends the payload through the channel and removes the entry.
- The API thread waits on the channel with a 5-second timeout.

**Alternatives Considered:**
- *Polling the database*: Too slow and doesn't capture the immediate RPC result (e.g., error messages from the device).
- *WebSockets*: Overkill for this specific simple request-response requirement.

### 2. Admin UI Tabbed Interface
We will refactor the device detail modal in `app.js` to use a tabbed navigation. This allows us to keep the configuration clean while adding "Scripts" and potentially other future features (like "Debug Logs" or "Network Info").

### 3. API Endpoints
- `GET /api/admin/devices/{id}/scripts`: Lists scripts.
- `POST /api/admin/devices/{id}/scripts/{script_id}/start`: Starts a script.
- `POST /api/admin/devices/{id}/scripts/{script_id}/stop`: Stops a script.

## Risks / Trade-offs

- **[Risk]** MQTT message loss or device offline. → **Mitigation**: 5-second timeout on the synchronous wait with a clear error message to the UI.
- **[Risk]** Race condition in ID generation. → **Mitigation**: Use an atomic counter for RPC IDs.
- **[Risk]** Memory leak if responses never arrive. → **Mitigation**: Clean up the registry on timeout or use a TTL-based approach if the registry grows large (though unlikely given the low volume of admin operations).

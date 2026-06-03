## Context

Admin users currently lack a high-level view of system health. When data is missing, they must check logs or individual device pages. A unified health check provides immediate feedback on the state of the system's external integrations (Home Assistant and MQTT).

## Goals / Non-Goals

**Goals:**
- Provide a single endpoint to trigger a system-wide health check.
- Verify PV metric fetching from Home Assistant for all houses.
- Verify MQTT broker connectivity.
- Verify responsiveness of all configured devices.
- Provide a user-friendly UI with progress indication and clear results.

**Non-Goals:**
- Fixing connectivity issues automatically.
- Extensive stress testing of the MQTT broker.
- Historic health check data persistence.

## Decisions

### 1. Health Check Routine Implementation
The health check will be implemented as a new synchronous endpoint `/api/admin/healthcheck`.

- **PV Check**: The backend will iterate through all houses and use the existing `lmha_core::ha::fetch_ha_state` function. Failure to fetch a metric for any house marks this check as failed.
- **MQTT Check**: The backend will verify that the MQTT client is initialized and active. A small test publish/subscribe could be performed, but given the existing `run_main_loop`, checking the client's internal state or a successful recent loop iteration is sufficient.
- **Device Check**: 
    1. Capture the current timestamp (`start_time`).
    2. Broadcast a "Get Status" RPC to all devices via MQTT.
    3. The server will `thread::sleep` for 12 seconds.
    4. The server will then query the database for devices whose `last_heartbeat` is newer than `start_time - 5s` (to account for clock jitter/delays).
    5. Devices not meeting this criterion are marked as unresponsive.

### 2. Frontend Integration
The "Run Healthcheck" button will be added to the Admin Panel.
- It will trigger the GET request.
- A local state will track `inProgress`.
- While `inProgress` is true, a spinner replaces the results and the button is disabled.
- The 10s delay is handled server-side, so the frontend just waits for the HTTP response.

### 3. Error Reporting
Each check will return:
- `status`: "ok" or "error"
- `message`: A short descriptive string (e.g., "Fetched PV for 3/3 houses", "MQTT Broker unreachable").
- `details`: (Optional) list of specific failed houses or devices.

## Risks / Trade-offs

- **[Risk] Synchronous Block** → The health check endpoint blocks a thread for 10s. 
  - *Mitigation*: Since it's an admin-only, infrequently used feature, this is acceptable for the current `rouille` architecture.
- **[Risk] MQTT Race Condition** → A device might respond just after the 10s timeout.
  - *Mitigation*: 10s is a generous timeout for local network MQTT communication.
- **[Risk] Shelly Gen1 vs Gen2** → Different topics and payloads for status requests.
  - *Mitigation*: The backend logic will branch based on the device's known topic pattern or metadata (Shellies Gen1 usually start with `shellies/`).

## Context

The current implementation of device control in `lmha3` is "fire and forget". When the scheduler or a user toggles a device, an MQTT message is sent immediately. If the device is offline or the message is lost, the server's view of the device state becomes inconsistent with reality until the next successful status update. Furthermore, there is no explicit "Offline" status, making it difficult for users to know if their devices are actually responsive.

## Goals / Non-Goals

**Goals:**
- Guarantee eventual consistency between intended (desired) and actual (observed) device states.
- Provide real-time visibility into device connectivity ("Online" vs "Offline").
- Immediate (within 3s) execution of state changes when a device is online.
- Automatic recovery when a device or the server comes back online.

**Non-Goals:**
- Multi-region or highly distributed MQTT brokers (assuming a single local/private broker).
- Complex retry backoff strategies (simple periodic sync is sufficient).

## Decisions

### 1. Data Model Extension
We will add `desired_state`, `last_request_time`, and `last_feedback_time` to the `devices` table.
- **Rationale**: Decoupling desired state from current (observed) state allows the system to track intent. Timestamps enable precise offline detection.
- **Alternatives**: Using a separate `pending_actions` table. Rejected for simplicity; the `devices` table already tracks state.

### 2. Synchronizer Placement & Logic
The State Synchronizer will be integrated into the existing `server` architecture, moving away from high-frequency database polling.

- **Event-Driven Sync**:
  - **On Action**: When the API or Scheduler updates `desired_state`, they immediately publish the MQTT command.
  - **On Reconnect**: When the MQTT loop receives an `online` or `status` message from a device, it triggers a check: if `desired_state != current_state`, it immediately sends a sync command.
- **Background Active Poll (Heartbeat)**:
  - A background task runs every 5 minutes.
  - It identifies devices whose `last_feedback_time` is older than 5 minutes.
  - It sends a `Shelly.GetStatus` request to these devices.
  - If a device still fails to respond within 20s of this active poll (or any state change request), it is flagged as "Offline".

### 3. Immediate Sync Trigger
The system prioritizes immediate execution for low latency (<3s).
- **Mechanism**: The API and Scheduler perform the initial "sync" by updating `desired_state` and publishing MQTT simultaneously. The "Synchronizer" logic acts as a safety net (reconnect-based) and a validator.

### 4. Offline Thresholds
A device is considered "Offline" if either of these conditions are met:
- **Failed Request**: `last_request_time > last_feedback_time + 20 seconds`.
- **Inactivity (Idle)**: `now > last_feedback_time + 5 minutes`.
- **Rationale**: 20s handles immediate request failures; 5m handles passive connectivity loss for idle devices.

### 5. Instance Conflict Management
To prevent multiple servers from fighting over the same devices, we introduce a priority-based "Passive" mode.

- **Detection**: Instances subscribe to `lmha3/instances/+`. Heartbeats are JSON payloads: `{"priority": 100, "timestamp": "..."}`.
- **Passive Mode**: If a higher priority instance is detected, the current instance will:
  - Skip all MQTT publishes in the `run_state_synchronizer_loop` and `run_scheduler_loop`.
  - Display a warning in the UI (via a new `/api/status` endpoint or extended `/api/me`).
- **Rationale**: Ensures production stability while allowing developers to run full stacks locally against the same broker without causing device toggling "oscillations."

## Risks / Trade-offs

- **[Risk] Database Contention** → [Mitigation] The synchronizer loop uses a 2s interval and only queries for out-of-sync devices to minimize DB load.
- **[Risk] MQTT Message Storm** → [Mitigation] Synchronizer only sends commands if `last_request_time` was more than 5s ago for a specific device, preventing rapid-fire retries for truly offline devices.

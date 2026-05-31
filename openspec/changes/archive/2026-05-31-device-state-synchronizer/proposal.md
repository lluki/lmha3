## Why

The current system triggers device state changes immediately but fails to ensure delivery or handle offline devices. If a device is offline when a command is sent, the state transition is lost, leading to inconsistency between the server's intended state and the device's actual state. This change introduces a robust state synchronization mechanism to ensure reliability and provides clear visibility into device connectivity.

## What Changes

- **Desired vs. Observed State**: Separate tracking of what state the server wants a device to be in vs. the last confirmed state reported by the device.
- **State Synchronizer**: A new component responsible for reconciling desired and observed states, with immediate triggers for changes and background syncing for offline-to-online transitions.
- **Offline Detection**: Implementation of a 20s timeout between requested state changes and confirmed feedback to identify and display "Offline" devices.
- **Server Startup Sync**: Automatic synchronization of all device states when the server starts.
- **Last Seen Tracking**: Update "last seen" and "last request" timestamps to support health monitoring.
- **Instance Conflict Detection**: Prevent multiple instances from "fighting" by using MQTT heartbeats with priority levels (e.g., Prod vs Dev).

## Capabilities

### New Capabilities
- `state-synchronizer`: Logic for reconciling desired and observed states, handling re-attempts, and managing the 20s offline timeout.
- `instance-manager`: Handles instance identity, priority, and heartbeat-based conflict detection.

### Modified Capabilities
- `data-model`: Update `devices` table to support `desired_state`, `last_request_time`, and `last_feedback_time`.
- `admin-ui`: Update device list and management panel to show "Offline" status and differentiate between desired/observed states.

## Impact

- **Database**: Schema migration for the `devices` table.
- **MQTT**: State update logic will now flow through the synchronizer.
- **Core**: New background task or thread for state synchronization.
- **Frontend**: UI updates to display connectivity status and sync progress.

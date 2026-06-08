# Spec: Shelly Gen2+ Protocol (MQTT RPC)

## Overview
This document describes the subset of the Shelly Gen2+ RPC protocol used by `lmha3` to control and monitor devices over MQTT.

## MQTT Topic Structure

| Direction | Topic Pattern | Description |
|-----------|---------------|-------------|
| Outbound  | `<topic>/rpc` | RPC Requests sent by the server to the device. |
| Inbound   | `<src>/rpc`   | RPC Responses sent by the device back to the specified `src` topic. |
| Inbound   | `<topic>/events/rpc` | Unsolicited status notifications published by the device. |
| Inbound   | `<topic>/online` | Presence message (`true`/`false`) published on connection change. |

## RPC Commands

### 1. Toggle Switch (`Switch.Set`)
Used to turn the relay ON or OFF.

**Request Payload:**
```json
{
  "id": 1,
  "src": "<device-topic>/rpc-response",
  "method": "Switch.Set",
  "params": {
    "id": 0,
    "on": true
  }
}
```

**Response Payload (`<device-topic>/rpc-response/rpc`):**
```json
{
  "id": 1,
  "src": "shellypro1-f8b3b7fa08bc",
  "dst": "<device-topic>/rpc-response",
  "result": {
    "was_on": false
  }
}
```
*Note: The response confirms the command was received and processed, but `NotifyStatus` or an explicit poll is required to confirm the final state if `Switch.Set` did not trigger a change.*

### 2. Get Device Status (`Shelly.GetStatus`)
Used for periodic polling and health checks.

**Request Payload:**
```json
{
  "id": 100,
  "src": "<device-topic>/rpc-response",
  "method": "Shelly.GetStatus"
}
```

**Response Payload (`<device-topic>/rpc-response/rpc`):**
```json
{
  "id": 100,
  "src": "shellypro1-f8b3b7fa08bc",
  "dst": "<device-topic>/rpc-response",
  "result": {
    "switch:0": {
      "id": 0,
      "source": "MQTT",
      "output": true,
      "apower": 12.5,
      "voltage": 230.1,
      "current": 0.05,
      "aenergy": { "total": 1234.5, "by_minute": [0.0, 0.0, 0.0] },
      "temperature": { "tC": 28.5, "tF": 83.3 }
    },
    "sys": { "uptime": 12345, "time": "12:34", "unixtime": 1780261954 },
    "wifi": { "sta_ip": "192.168.1.10", "rssi": -65 }
  }
}
```

## Status Notifications (`events/rpc`)
The device automatically publishes status updates when a component changes.

**Payload Example:**
```json
{
  "method": "NotifyStatus",
  "params": {
    "switch:0": {
      "id": 0,
      "output": true,
      "apower": 1500.0
    }
  }
}
```

## Requirements & Best Practices

### Requirement: Standardized Feedback
Always set `src` to `<device-topic>/rpc-response` to ensure the response is captured by the global `+/rpc-response/#` subscription. For synchronous RPC calls, the system SHALL use a unique ID and monitor the response topic to bridge the asynchronous MQTT message back to a synchronous API response.

#### Scenario: Async-to-Sync Response Bridge
- **WHEN** an RPC command is issued via the API
- **THEN** the system generates a unique `id` and sets `src` to `<device-topic>/rpc-response`
- **AND** it synchronously waits for a message on `<device-topic>/rpc-response/rpc` matching the generated `id`
- **AND** it returns the `result` or `error` from that message to the API caller

### Requirement: Double-Click Sync
Follow every `Switch.Set` with a `Shelly.GetStatus` poll to ensure the internal database accurately reflects the state, even if the device was already in the requested state.

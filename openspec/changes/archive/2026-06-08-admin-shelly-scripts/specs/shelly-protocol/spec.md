## MODIFIED Requirements

### Requirement: Standardized Feedback
Always set `src` to `<device-topic>/rpc-response` to ensure the response is captured by the global `+/rpc-response/#` subscription. For synchronous RPC calls, the system SHALL use a unique ID and monitor the response topic to bridge the asynchronous MQTT message back to a synchronous API response.

#### Scenario: Async-to-Sync Response Bridge
- **WHEN** an RPC command is issued via the API
- **THEN** the system generates a unique `id` and sets `src` to `<device-topic>/rpc-response`
- **AND** it synchronously waits for a message on `<device-topic>/rpc-response/rpc` matching the generated `id`
- **AND** it returns the `result` or `error` from that message to the API caller

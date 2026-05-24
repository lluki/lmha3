# Proposal: 008-Logging Improvement

## Problem
Currently, the application uses standard `println!` and `eprintln!` for logging. This output is captured by the systemd journal, but it is not easily accessible via the web interface. Furthermore, `println!` lacks structured logging benefits like levels (INFO, WARN, ERROR) and metadata.

## Proposed Solution
Implement a structured logging system using the `tracing` ecosystem that simultaneously logs to the systemd journal (via stdout) and an in-memory buffer accessible via an API.

### 1. Backend: Structured Logging
- Integrate `tracing` and `tracing-subscriber`.
- Implement a custom `tracing::Layer` that captures log events into a thread-safe circular buffer (e.g., `VecDeque`).
- Replace existing `println!` and `eprintln!` calls with `tracing` macros (`info!`, `warn!`, `error!`, etc.).
- Expose the log buffer via a new authenticated API endpoint: `GET /api/logs`.

### 2. Frontend: Log Viewer
- Add a new "Logs" section to the web dashboard.
- Periodically fetch logs from `/api/logs` and display them with appropriate styling based on severity level.
- Support basic filtering or auto-scrolling to the latest logs.

## Success Criteria
- [ ] Logs are visible in the systemd journal (`journalctl -u lmha3`).
- [ ] Logs are visible in the Web UI for authenticated users.
- [ ] Log levels (INFO, ERROR, etc.) are correctly distinguished.

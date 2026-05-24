# Design: 008-Logging Improvement

## Backend Implementation Details

### Dependencies
Add to `server/Cargo.toml`:
- `tracing = "0.1"`
- `tracing-subscriber = { version = "0.3", features = ["json"] }`
- `serde = { version = "1.0", features = ["derive"] }` (Already present)

### Log Buffer Structure
```rust
#[derive(Serialize, Clone, Debug)]
struct LogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    level: String,
    message: String,
    target: String,
}

struct LogBuffer {
    entries: std::collections::VecDeque<LogEntry>,
    max_size: usize,
}
```

### Tracing Layer
A custom `Layer` implementation that converts `tracing::Event` into `LogEntry` and pushes it to a globally accessible or `AppState`-held `LogBuffer`.

### API Endpoint
`GET /api/logs`
- **Auth:** Requires a valid session.
- **Response:** `JSON` array of `LogEntry`.

## Frontend Implementation Details

### UI Components
- A new "Logs" tab in the top navigation menu.
- A new container `<section id="logs-section" style="display:none;">`.
- An additional log display box included in the `/admin` view.
- A table or list view for logs.
- Severity colors:
    - ERROR: Red
    - WARN: Yellow/Orange
    - INFO: Default/Green
    - DEBUG: Gray

### Polling Mechanism
The frontend will fetch logs every 5-10 seconds when the logs section or admin panel is active.

## NixOS Integration
The `tracing-subscriber::fmt` layer will be configured to log to `stdout`. Since the service is managed by systemd, these will automatically flow into the journal. No special `tracing-journald` dependency is strictly required unless we want native journal fields (like `PRIORITY`), but standard `fmt` is usually cleaner for mixed environments.

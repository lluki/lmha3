# Tasks: 008-Logging Improvement

- [x] 1. Backend: Dependency Setup
    - [x] 1.1 Add `tracing` and `tracing-subscriber` to `server/Cargo.toml`.
- [x] 2. Backend: Log Infrastructure
    - [x] 2.1 Implement `LogEntry` and `LogBuffer`.
    - [x] 2.2 Implement custom `tracing::Layer` for the buffer.
    - [x] 2.3 Initialize `tracing` in `main.rs`.
- [x] 3. Backend: API implementation
    - [x] 3.1 Implement `GET /api/logs` handler in `server/src/main.rs`.
- [x] 4. Backend: Refactoring
    - [x] 4.1 Replace `println!` with `tracing::info!`.
    - [x] 4.2 Replace `eprintln!` with `tracing::error!`.
- [x] 5. Frontend: Log Viewer
    - [x] 5.1 Add "Logs" tab to the top navigation.
    - [x] 5.2 Create log viewer UI in `index.html` (for the tab) and add a log box in the `/admin` template.
    - [x] 5.3 Implement log fetching and rendering logic in `app.js`.
- [x] 6. Verification
    - [x] 6.1 Verify logs are printed to console.
    - [x] 6.2 Verify logs are visible in the Web UI.

# Tasks: 008-Logging Improvement

- [ ] 1. Backend: Dependency Setup
    - [ ] 1.1 Add `tracing` and `tracing-subscriber` to `server/Cargo.toml`.
- [ ] 2. Backend: Log Infrastructure
    - [ ] 2.1 Implement `LogEntry` and `LogBuffer`.
    - [ ] 2.2 Implement custom `tracing::Layer` for the buffer.
    - [ ] 2.3 Initialize `tracing` in `main.rs`.
- [ ] 3. Backend: API implementation
    - [ ] 3.1 Implement `GET /api/logs` handler in `server/src/main.rs`.
- [ ] 4. Backend: Refactoring
    - [ ] 4.1 Replace `println!` with `tracing::info!`.
    - [ ] 4.2 Replace `eprintln!` with `tracing::error!`.
- [ ] 5. Frontend: Log Viewer
    - [ ] 5.1 Add "Logs" tab to the top navigation.
    - [ ] 5.2 Create log viewer UI in `index.html` (for the tab) and add a log box in the `/admin` template.
    - [ ] 5.3 Implement log fetching and rendering logic in `app.js`.
- [ ] 6. Verification
    - [ ] 6.1 Verify logs are printed to console.
    - [ ] 6.2 Verify logs are visible in the Web UI.

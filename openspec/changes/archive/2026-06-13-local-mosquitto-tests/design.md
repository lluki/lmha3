## Context

The current integration tests in `server/tests/` depend on a pre-existing Mosquitto broker running on `localhost:1883`. This setup is fragile, prevents parallel test execution, and complicates CI environments. We need a way to spawn a dedicated Mosquitto instance for each `TestHarness` instance.

## Goals / Non-Goals

**Goals:**
- Automatically start `mosquitto` on a random available port when `TestHarness::new()` is called.
- Ensure the `mosquitto` process is cleaned up (killed) when `TestHarness` is dropped.
- Update `TestHarness` and tests to use the dynamic port.
- Support Mosquitto 2.x by providing a minimal configuration that allows anonymous connections.

**Non-Goals:**
- Support for complex MQTT configurations (SSL, Authentication) in tests.
- Installation of Mosquitto (assumed to be in `$PATH`).

## Decisions

### 1. Port Discovery
Use `std::net::TcpListener::bind("127.0.0.1:0")` to find an available port, then immediately close it and use its port number for Mosquitto.

### 2. Mosquitto Configuration
Since Mosquitto 2.0+ refuses anonymous connections by default, we will generate a temporary configuration file:
```
listener <port>
allow_anonymous true
```
We'll use `std::env::temp_dir()` to store this file.

### 3. RAII for Process Management
Add `mosquitto_child: Child` and `mosquitto_config_path: PathBuf` fields to `TestHarness`.
The `Drop` implementation will:
1. Kill the Mosquitto process.
2. Delete the temporary configuration file.

### 4. Config Handling in TestHarness
Update `TestHarness::new` to no longer rely on `LMHA_MQTT_HOST`/`PORT` environment variables, instead using the newly spawned broker's details.

## Risks / Trade-offs

- **[Risk]** Mosquitto might not be in the PATH. → **Mitigation**: The test will panic with a clear error message if `mosquitto` fails to spawn.
- **[Risk]** Port race condition between closing the listener and starting Mosquitto. → **Mitigation**: Minimal risk in local environments; retry logic could be added if it proves flaky.
- **[Risk]** Orphaned processes if the test runner crashes (SIGKILL). → **Mitigation**: Standard limitation of child processes in Rust; could use `shared_child` or similar crates if it becomes a major issue, but `std::process::Child` is sufficient for now.

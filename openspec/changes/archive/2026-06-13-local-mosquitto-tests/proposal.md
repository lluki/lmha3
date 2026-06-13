## Why

Currently, the test suite depends on an externally managed or pre-running Mosquitto broker, defaulting to `localhost:1883`. This makes the test environment non-hermetic, as concurrent test runs or different local configurations can lead to flakiness and interference. Spawning a dedicated, local Mosquitto instance for each test run ensures a clean, isolated environment and simplifies CI setup.

## What Changes

- **Test Harness Update**: Modify `server/tests/common/mod.rs` to automatically spawn a `mosquitto` process during test initialization.
- **Dynamic Port Allocation**: The spawned Mosquitto will use a dynamically assigned port to avoid conflicts with other services or parallel test runs.
- **Lifecycle Management**: The test harness will be responsible for killing the Mosquitto process when tests complete or the harness is dropped.
- **Environment Configuration**: Update the test configuration to use the dynamic host/port instead of relying on external environment variables.
- **Dependency Requirement**: Ensure `mosquitto` is available in the `$PATH` for the test suite to run.

## Capabilities

### New Capabilities
- `test-infrastructure`: Infrastructure for hermetic testing with local service mocks (Mosquitto).

### Modified Capabilities
- (None)

## Impact

- **Hermeticity**: Tests will be more reliable and independent of the host's MQTT state.
- **Parallelism**: Multiple test runs can occur on the same machine without port conflicts.
- **Environment**: CI and developer environments must have `mosquitto` installed and in `$PATH`.

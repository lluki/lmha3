## 1. Preparation

- [x] 1.1 Verify `mosquitto` is available in the `$PATH`.

## 2. Test Harness Modification

- [x] 2.1 Update `TestHarness` struct in `server/tests/common/mod.rs` to include `mosquitto_child: std::process::Child` and `mosquitto_config_path: std::path::PathBuf`.
- [x] 2.2 Implement dynamic port discovery logic in `TestHarness::new` using `std::net::TcpListener`.
- [x] 2.3 Implement temporary `mosquitto.conf` generation to allow anonymous connections.
- [x] 2.4 Implement `mosquitto` process spawning in `TestHarness::new` using the generated config and discovered port.
- [x] 2.5 Update `Config` object creation in `TestHarness::new` to use the dynamic port and handle optional/null MQTT credentials.
- [x] 2.6 Update `Drop` implementation for `TestHarness` to kill the `mosquitto` process and delete the temporary config file.
- [x] 2.7 Update existing tests to handle optional MQTT credentials (remove `unwrap()`).

## 3. Verification

- [x] 3.1 Run `mqtt_tests.rs` to verify the local broker is correctly utilized.
- [x] 3.2 Run the full test suite (`cargo test --test '*'`) to ensure all tests pass in the new hermetic environment.

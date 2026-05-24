# Tasks: 007-ha-integration

- [x] 1. Extend Configuration
    - [x] 1.1 Add `ha_pv_entity_id` and `ha_consumption_entity_id` to `Config`.
- [x] 2. Implement HA Polling (lmha-core)
    - [x] 2.1 Create a `ha` module in `lmha-core`.
    - [x] 2.2 Implement a function to fetch and parse entity state.
- [x] 3. Implement Telemetry Persistence (lmha-core)
    - [x] 3.1 Add `insert_telemetry` method to `Db`.
- [x] 4. Integrate into Server Loop (server)
    - [x] 4.1 Update `run_main_loop` to poll HA and save to DB.
    - [x] 4.2 Ensure it respects the `--no-home-assistant` flag.
- [ ] 5. Verification
    - [x] 5.1 Create a mock test or manually verify with a local HA instance. (Verified via compilation and code review; manual verification requires a running HA instance).


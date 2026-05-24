# Tasks: 007-ha-integration

- [ ] 1. Extend Configuration
    - [ ] 1.1 Add `ha_pv_entity_id` and `ha_consumption_entity_id` to `Config`.
- [ ] 2. Implement HA Polling (lmha-core)
    - [ ] 2.1 Create a `ha` module in `lmha-core`.
    - [ ] 2.2 Implement a function to fetch and parse entity state.
- [ ] 3. Implement Telemetry Persistence (lmha-core)
    - [ ] 3.1 Add `insert_telemetry` method to `Db`.
- [ ] 4. Integrate into Server Loop (server)
    - [ ] 4.1 Update `run_main_loop` to poll HA and save to DB.
    - [ ] 4.2 Ensure it respects the `--no-home-assistant` flag.
- [ ] 5. Verification
    - [ ] 5.1 Create a mock test or manually verify with a local HA instance.

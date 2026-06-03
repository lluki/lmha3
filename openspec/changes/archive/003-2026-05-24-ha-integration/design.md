# Design: 007-ha-integration

## Architecture Overview
The background main loop will now include logic to poll Home Assistant using the `ureq` crate.

## Components

### 1. HA Client (lmha-core)
- A helper function/module to fetch state from `/api/states/<entity_id>`.
- Authentication using the Long-Lived Access Token.

### 2. Telemetry Persistence (lmha-core)
- Add a method to `Db` to insert telemetry records.

### 3. Polling Logic (server)
- The main loop will call the HA client every 5 minutes.
- It will parse the JSON response to extract the `state` value (converted to `f64`).

## Configuration
- Entity IDs will be configurable via environment variables:
  - `HA_PV_ENTITY_ID`
  - `HA_CONSUMPTION_ENTITY_ID`

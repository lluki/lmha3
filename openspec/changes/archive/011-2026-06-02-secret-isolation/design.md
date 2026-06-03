## Context

The current configuration loading logic mixes global service secrets (Database URL, MQTT credentials) with house-specific integration details (Home Assistant host/token). Furthermore, these secrets are currently leaking into the git history via `dev.sh`, `secrets/` files, and hardcoded fallbacks in `main.rs`.

## Goals / Non-Goals

**Goals:**
- Move all service-level secrets (`DATABASE_URL`, `MQTT_USER`, `MQTT_PASSWORD`) to environment variables or an ignored `.env` file.
- Move all house-specific configuration (`HA_URL`, `HA_TOKEN`, `HA_ENTITY_IDs`) to the `houses` database table.
- Eliminate all hardcoded secrets from scripts and source code.
- Update NixOS modules to align with the new storage pattern.

**Non-Goals:**
- Implementing secret encryption at rest (will be handled as a separate future improvement).
- Migrating historical telemetry data between houses (assumes current data is correctly mapped to "Default House").

## Decisions

### 1. Unified Config Loading with `dotenvy`
- **Decision**: Use `dotenvy` in `lmha-core/src/config.rs` to load `.env` files automatically in development.
- **Rationale**: standard practice in Rust/Rust-adjacent ecosystems for local dev environment management.
- **Alternatives**: Manually parsing a config file; using `clap` for everything (inconvenient for many secrets).

### 2. Database-backed House Integration
- **Decision**: Extend the `houses` table with `ha_url`, `ha_token`, `ha_pv_entity_id`, and `ha_consumption_entity_id`.
- **Rationale**: Essential for multi-house scalability where each property has its own HA instance. The database is the natural place for property-specific configuration.

### 3. Removal of `secrets/` Directory
- **Decision**: Delete the `secrets/` directory and remove the logic in `main.rs` that reads from it.
- **Rationale**: The directory was being tracked in Git, defeating its purpose. `.env` (ignored) is a safer alternative for local dev.

### 4. NixOS Module Environment Mapping
- **Decision**: Update `nix/module.nix` to only pass global secrets via environment. House-specific config will be managed via the Admin UI (persisted to DB).
- **Rationale**: Simplifies the Nix configuration and delegates property-specific settings to the application's own management plane.

## Risks / Trade-offs

- **[Risk] Bootstrapping** → The first "Default House" needs its HA credentials to be functional. *Mitigation*: Migration `011` will seed the default house using existing environment variables if present, or leave them empty for the admin to fill via UI.
- **[Trade-off] Plaintext DB Secrets** → HA tokens will be in plaintext in the DB. *Mitigation*: Ensure the DB is only accessible to the `lmha3` user and consider column encryption in a future iteration.

## Migration Plan

1. **Schema Update**: Create migration `011_house_config_extension.sql`.
2. **Core Refactor**: Update `lmha-core` structs (`Config`, `House`) and DB methods.
3. **Server Refactor**: Update `main.rs` to stop reading `secrets/` and pass the new `House` config to the scheduler.
4. **Dev Environment**: Update `dev.sh` to remove hardcoded values and add `.env` to `.gitignore`.
5. **Nix Update**: Adjust `nix/module.nix` options.

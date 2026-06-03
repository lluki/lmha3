## Why

Currently, several sensitive secrets (Home Assistant tokens, MQTT credentials, Database URLs) are either tracked in Git or hardcoded in scripts like `dev.sh`. This poses a significant security risk and makes the project difficult to share or deploy securely. Moving these to appropriate isolated storage (Database for house-specific config, `.env` for global service credentials) is essential for security and multi-house scalability.

## What Changes

- **Secret Removal**: Remove all hardcoded secrets from `dev.sh`, `server/src/main.rs`, and delete tracked secret files in `secrets/`.
- **Database Schema Migration**: Update the `houses` table to include Home Assistant configuration fields (`ha_url`, `ha_pv_entity_id`, `ha_consumption_entity_id`) and migrate existing data. **BREAKING**
- **Configuration Loading**: Refactor `lmha-core` and `server` to load global secrets (DB URL, MQTT credentials) from an ignored `.env` file and house-specific config from the database.
- **Nix/NixOS Updates**: Update Nix expressions and NixOS modules to align with the new secret loading strategy, ensuring compatibility with production secret management (e.g., sops-nix).
- **Environment Isolation**: Ensure `.env` is added to `.gitignore` and enforced.

## Capabilities

### New Capabilities
- `secret-management`: Provides a unified interface and pattern for handling sensitive configuration across environment variables and database storage.

### Modified Capabilities
- `data-model`: Update `houses` table schema to store full Home Assistant integration details.
- `house-management`: Update management logic to handle the new configuration fields.
- `load-management`: Update the scheduler and HA client to fetch configuration from the database instead of global environment variables.

## Impact

- **Database**: Migration to `houses` table.
- **Backend**: `Config` struct and loading logic in `lmha-core/src/config.rs`.
- **Deployment**: NixOS module options and environment variable requirements will change.
- **Dev UX**: `dev.sh` will now require a `.env` file to be present (or passed via environment).

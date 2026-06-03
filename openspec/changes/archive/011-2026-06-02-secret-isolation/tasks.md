## 1. Database & Schema

- [x] 1.1 Create migration `011_house_config_extension.sql` to add `ha_url`, `ha_token`, `ha_pv_entity_id`, `ha_consumption_entity_id` to `houses`.
- [x] 1.2 Update `INSERT` seed in migration `008` (if re-runnable) or add to `011` to migrate current environment variables into the 'Default House' row.

## 2. Core Refactor (lmha-core)

- [x] 2.1 Update `House` struct in `lib.rs` to include new HA config fields.
- [x] 2.2 Update `Db::list_houses` and `Db::get_house` in `db.rs` to select new columns.
- [x] 2.3 Update `Config` struct and `Config::from_env` in `config.rs` to load from `.env` using `dotenvy` and remove HA-specific global fields.
- [x] 2.4 Add `dotenvy` to `lmha-core/Cargo.toml`.

## 3. Server Refactor (server)

- [x] 3.1 Remove `secrets/` reading logic from `main.rs`.
- [x] 3.2 Update `run_scheduler_loop` to use house-specific host, token, and entity IDs from the `House` object.
- [x] 3.3 Update `run_health_check_loop` to be house-aware if necessary (check current implementation).
- [x] 3.4 Update Admin API for house creation/update to handle new fields.

## 4. Environment & Dev Scripts

- [x] 4.1 Update `.gitignore` to include `.env`.
- [x] 4.2 Create `.env.example` template with required service keys.
- [x] 4.3 Refactor `dev.sh` to remove hardcoded secrets and check for `.env` availability.
- [x] 4.4 Delete tracked `secrets/` directory from git (`git rm -r secrets`).

## 5. Nix & NixOS

- [x] 5.1 Update `nix/module.nix` options to remove house-specific HA settings (now in DB).
- [x] 5.2 Ensure Nix module correctly passes `DATABASE_URL` and `MQTT` credentials to the systemd service.

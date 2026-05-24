# lmha3: Load Management Hagenholz

Load management for the Hagenholz neighborhood to match physical tenant loads (Shelly 1 Pro) with solar production.

## Setup

1. **DB**: `createdb lmha3`
2. **Schema**: Apply `migrations/*.sql`
3. **Config**: Create `.env` (see `lmha-core/src/config.rs` for keys)
4. **User**: `cargo run -p lmha-admin`

## Commands

- **Build**: `cargo build`
- **Test**: `cargo test` (Runs integrated harness with temp DBs)
- **Run API**: `cargo run -p api`
- **Run Scheduler**: `cargo run -p scheduler`

## Auth Model
- **Hashing**: Argon2id
- **Sessions**: PostgreSQL-backed cookies (24h)
- **Access**: Global Read, Owner Write (for control)

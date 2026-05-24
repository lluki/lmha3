# lmha3: Load Management Hagenholz

Synchronous Rust-based load management for the Hagenholz neighborhood. Matches physical tenant loads (Shelly 1 Pro) with solar production.

## Setup

1. **DB**: `createdb lmha3`
2. **Schema**: Apply `migrations/*.sql`
3. **Config**: Create `.env` (see `lmha-core/src/config.rs` for keys)
4. **User**: `cargo run -p lmha-admin`

## Commands

- **Build**: `cargo build`
- **Test**: `cargo test` (Integrated harness with temp DBs)
- **Run Server**: `cargo run -p server`

## Architecture
- **Backend**: Rust (Rouille) + PostgreSQL
- **Frontend**: Vanilla JS Single-Page App (`server/public`)
- **API**: JSON-based auth and control endpoints
- **MQTT**: Bidirectional status and RPC control for Shelly hardware
- **Auth**: Argon2id + PostgreSQL-backed cookies (24h)
- **Access**: Global Read, Owner Write

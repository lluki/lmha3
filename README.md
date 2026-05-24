# lmha3: Load Management Hagenholz

A high-performance, synchronous Rust-based load management solution for the Hagenholz neighborhood. It synchronizes physical tenant loads (Shelly 1 Pro switches) with local solar production to maximize energy self-consumption.

## Architecture

The project is structured as a Rust workspace with four main components:

- **`lmha-core`**: Shared library containing the data model, database logic, and configuration management.
- **`api`**: A synchronous web server (using `rouille`) providing the dashboard and authentication system.
- **`scheduler`**: A background process that polls Home Assistant for energy data and controls Shelly devices via MQTT.
- **`lmha-admin`**: A CLI tool for secure user/tenant management.

## Tech Stack

- **Language**: Rust (Stable 1.95+)
- **Database**: PostgreSQL (Raw SQL, no ORM)
- **Web Framework**: Rouille (Synchronous)
- **Messaging**: MQTT (Shelly 1 Pro integration)
- **Energy Source**: Home Assistant REST API (Polled on localhost)
- **Deployment**: Standard Linux / Systemd (NixOS friendly)

## Getting Started

### Prerequisites

- Rust (installed via `rustup`)
- PostgreSQL (running locally)
- Home Assistant (running on localhost)

### Setup Database

1. Create the database:
   ```bash
   createdb lmha3
   ```
2. Apply migrations:
   ```bash
   psql lmha3 -f migrations/001_initial_schema.sql
   psql lmha3 -f migrations/002_add_sessions.sql
   ```

### Configuration

Create a `.env` file in the project root:
```env
DATABASE_URL=postgres://your_user@localhost/lmha3
HA_URL=http://localhost:8123
HA_TOKEN=your_home_assistant_long_lived_token
MQTT_HOST=solar.lluki.me
MQTT_PORT=1884
```

### User Management

To create a new tenant/user:
```bash
cargo run -p lmha-admin
```
The tool will prompt you for a username and password (hashed with Argon2id).

## Authentication & Security

- **Hashing**: All passwords are hashed using **Argon2id**.
- **Sessions**: Cookie-based session management. Sessions are persisted in PostgreSQL and expire after 24 hours.
- **Permissions**:
    - **Global Read**: Any logged-in tenant can view the status of all devices and global energy production.
    - **Owner Write**: Only the owner of a device can toggle its state or change its configuration.

## Testing

Run the authentication integration tests:
```bash
DATABASE_URL="postgres://your_user@localhost/lmha3" HA_TOKEN=dummy cargo test -p api --test auth_tests
```

## OpenSpec Implementation

This project follows the **OpenSpec** methodology for Spec-Driven Development.
- All technical requirements are in `openspec/specs/`.
- All architectural decisions are in `openspec/project.md`.
- Active work is tracked in `openspec/changes/`.

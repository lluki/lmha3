# Project: lmha3 (Load Management Hagenholz)

## Context
A load management solution for the Hagenholz neighborhood. It synchronizes physical tenant loads (Shelly 1 Pro switches) with local solar production to maximize self-consumption.

## Core Capabilities
- **Demand/Supply Matching:** Monitors PV production and house consumption to toggle loads via MQTT.
- **Tenant Management:** Secure web interface with per-tenant logins to monitor status and view logs.
- **Basic Scheduling:** Simple logic to match demand with production (sophisticated constraints deferred post-MVP).
- **Telemetry:** Logs PV production, total house consumption, and all load toggle events (with reasoning).

## Tech Stack
- **Language:** Rust (Unified server process with internal scheduler thread).
- **Database:** PostgreSQL (Core state, configurations, and historical telemetry).
- **Messaging:** MQTT (Broker at `solar.lluki.me:1884`, Shelly 1 Pro hardware).
- **Infrastructure:** NixOS (Single Systemd service, no containerization, deployed on `lisa`).
- **Web Entry:** Nginx reverse proxy (App handles authentication; Nginx handles routing).

## Deployment & Operations
- **Production Server:** The application is deployed on the server named `lisa`.
- **Operating System:** NixOS.
- **Access:** SSH access to `lisa` is available via `ssh lisa`.
- **NixOS Configuration:** The configuration for `lisa` is located at `/etc/nixos/nixos-config/configuration.nix`.
- **Deployment Process:** Managed via the `project-versioning` skill. Deployment involves updating the tag in the NixOS configuration on `lisa` and running `sudo nixos-rebuild switch`.
- **Production Stats:** When production statistics or live data are required for analysis or debugging, they must be sourced from the `lisa` instance.

## Global Rules & Constraints
1. **Authentication:** No public endpoints. Every action and view requires a valid tenant session.
2. **Unified Process:** The application runs as a single binary. The load management scheduler runs in a dedicated background thread.
3. **Control Mode:** Supports a `--no-scheduler` flag for API-only operation (useful for testing/maintenance).
4. **Hardware Safety:** Implement safeguards for physical switches (e.g., debounce toggles, minimum state duration).
4. **Data Integrity:** Telemetry (PV/Consumption) must be persisted to PostgreSQL for auditing and UI charts.
5. **No SSL in-app:** The application assumes it runs behind a proxy that manages the external network layer.

## Technical Specs
- **MQTT Port:** 1884
- **Host:** `solar.lluki.me`
- **User Interface:** Accessible via `https://solar.lluki.me` (behind Nginx).

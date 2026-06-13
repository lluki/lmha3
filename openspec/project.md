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
- **Messaging:** MQTT (Broker at `localhost:1883`, Shelly 1 Pro hardware).
- **Infrastructure:** NixOS (Single Systemd service, no containerization).
- **Web Entry:** Nginx reverse proxy (App handles authentication; Nginx handles routing).

## Deployment & Operations
- **Production Server:** The application is typically deployed on a dedicated NixOS server.
- **Operating System:** NixOS.
- **NixOS Configuration:** The configuration is managed via Nix modules.
- **Deployment Process:** Managed via standard NixOS tools (`nixos-rebuild switch`).
- **Production Stats:** Production statistics or live data should be sourced from the active production instance.

## Global Rules & Constraints
1. **Authentication:** No public endpoints. Every action and view requires a valid tenant session.
2. **Unified Process:** The application runs as a single binary. The load management scheduler runs in a dedicated background thread.
3. **Control Mode:** Supports a `--no-scheduler` flag for API-only operation (useful for testing/maintenance).
4. **Hardware Safety:** Implement safeguards for physical switches (e.g., debounce toggles, minimum state duration).
5. **Time Handling:** All internal representations, database storage, and API data exchange MUST use UTC (ISO8601 with 'Z' suffix). The Web UI SHALL handle localization by translating UTC timestamps to the user's browser-configured local timezone for display.
6. **Data Integrity:** Telemetry (PV/Consumption) must be persisted to PostgreSQL for auditing and UI charts.
7. **No SSL in-app:** The application assumes it runs behind a proxy that manages the external network layer.

## Technical Specs
- **MQTT Port:** 1883
- **Host:** `localhost`
- **User Interface:** Accessible via `https://lmha3.example.com` (behind Nginx).

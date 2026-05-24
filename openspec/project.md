# Project: lmha3 (Load Management Hagenholz)

## Context
A load management solution for the Hagenholz neighborhood. It synchronizes physical tenant loads (Shelly 1 Pro switches) with local solar production to maximize self-consumption.

## Core Capabilities
- **Demand/Supply Matching:** Monitors PV production and house consumption to toggle loads via MQTT.
- **Tenant Management:** Secure web interface with per-tenant logins to monitor status and view logs.
- **Basic Scheduling:** Simple logic to match demand with production (sophisticated constraints deferred post-MVP).
- **Telemetry:** Logs PV production, total house consumption, and all load toggle events (with reasoning).

## Tech Stack
- **Language:** Rust (for both Backend API and Scheduler process).
- **Database:** PostgreSQL (Core state, configurations, and historical telemetry).
- **Messaging:** MQTT (Broker at `solar.lluki.me:1884`, Shelly 1 Pro hardware).
- **Infrastructure:** Linux (Systemd services, no containerization).
- **Web Entry:** Nginx reverse proxy (App handles authentication; Nginx handles routing).

## Global Rules & Constraints
1. **Authentication:** No public endpoints. Every action and view requires a valid tenant session.
2. **Standard Linux Patterns:** Configuration and deployment should follow standard Linux service patterns (Systemd).
3. **Hardware Safety:** Implement safeguards for physical switches (e.g., debounce toggles, minimum state duration).
4. **Data Integrity:** Telemetry (PV/Consumption) must be persisted to PostgreSQL for auditing and UI charts.
5. **No SSL in-app:** The application assumes it runs behind a proxy that manages the external network layer.

## Technical Specs
- **MQTT Port:** 1884
- **Host:** `solar.lluki.me`
- **User Interface:** Accessible via `https://solar.lluki.me` (behind Nginx).

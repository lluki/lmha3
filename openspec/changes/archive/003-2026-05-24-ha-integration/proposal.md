# Proposal: 007-ha-integration

## Intent
Integrate the system with Home Assistant to fetch real-time solar production and house consumption data.

## Objectives
- Implement a synchronous HTTP client to poll Home Assistant's REST API.
- Fetch PV production and house consumption values every 5 minutes.
- Store these values in the `telemetry` table in PostgreSQL.

## Success Criteria
- The server logs PV and consumption data periodically.
- Data is correctly persisted in the database.
- The system handles HA connection errors gracefully.

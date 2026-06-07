## Why

The current system provides limited visualization of historical telemetry data (PV production, house consumption, and device states). Enabling Grafana will provide a powerful tool for analyzing energy patterns, verifying scheduler decisions, and monitoring system health. Integrating it directly into the existing infrastructure ensures a seamless experience for administrators.

## What Changes

- **Infrastructure**: Enable Grafana service on the `prod` NixOS machine.
- **Data Integration**: Configure Grafana to use the local PostgreSQL database (via Unix socket) as a data source.
- **Networking**: Update Nginx configuration to proxy `/grafana` to the Grafana service.
- **Database**: Create helper views in PostgreSQL to simplify common Grafana queries (e.g., pivoted telemetry data).
- **Security**: Set default Grafana administrator credentials to `admin/[PASSWORD]`.

## Capabilities

### New Capabilities
- `telemetry-visualization`: Visualization and dashboarding of system telemetry data using Grafana.

### Modified Capabilities
- `admin-ui`: The administrative interface is extended to include Grafana as a linked/embedded visualization tool.

## Impact

- **NixOS Configuration**: Changes to `/etc/nixos/nixos-config/configuration.nix` (or a new module) on `prod`.
- **Database**: New views in the `lmha3` database.
- **Performance**: Minimal impact from Grafana service and queries on the existing database.

## Context

The `lmha3` project collects telemetry data in a PostgreSQL database on the `prod` NixOS machine. Currently, there is no high-level visualization tool for this data. The user wants to enable Grafana on `prod`, connect it to the `lmha3` database, and make it accessible via `https://your-domain.com/grafana`.

## Goals / Non-Goals

**Goals:**
- Enable Grafana as a systemd service on `prod` using NixOS modules.
- Securely connect Grafana to PostgreSQL via Unix sockets.
- Expose Grafana through the existing Nginx reverse proxy at `/grafana`.
- Simplify data querying with PostgreSQL views.
- Configure default administrator credentials.

**Non-Goals:**
- Creating complex dashboards (only basic infrastructure and views).
- Setting up complex alerting (out of scope for initial setup).

## Decisions

### 1. NixOS Grafana Configuration
We will use the native NixOS `services.grafana` module.
- **Rationale**: Provides declarative management and easy integration with the rest of the system.
- **Details**: 
    - `server.root_url` will be set to `https://your-domain.com/grafana/`.
    - `server.serve_from_sub_path` will be enabled.
    - `security.admin_user` and `admin_password` will be set to `admin/[PASSWORD]`.

### 2. Data Source Provisioning
Grafana will be configured using declarative provisioning.
- **Rationale**: Ensures the data source is always available and correctly configured without manual intervention.
- **Details**: Connection via `/run/postgresql` using the `lmha3` database.

### 3. Nginx Proxying
The `your-domain.com` virtual host on `prod` will be updated to include a location block for `/grafana`.
- **Rationale**: Centralizes external access through the existing domain.

### 4. PostgreSQL Helper Views
A new migration will be added to create views that pivot the `telemetry` table.
- **Rationale**: The current `telemetry` table is narrow (long-format). A pivoted view (wide-format) is often easier to use in Grafana for comparing multiple metrics over time.

## Risks / Trade-offs

- **[Risk]** Grafana performance impact on the small machine. → **Mitigation**: Grafana is generally lightweight; we will monitor resource usage.
- **[Risk]** Password in Nix configuration. → **Mitigation**: The user explicitly asked for `admin/[PASSWORD]`. In a production environment, we would use a secret file.

## Migration Plan

1. Update `prod` NixOS configuration.
2. Apply the configuration (`nixos-rebuild switch`).
3. Add a new SQL migration to `lmha3` to create the views.
4. Verify accessibility and data connectivity.

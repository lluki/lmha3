## 1. Database Schema Enhancements

- [x] 1.1 Create a new migration `013_grafana_views.sql` to add helper views
- [x] 1.2 Implement `view_telemetry_house_metrics` for PV and consumption
- [x] 1.3 Implement `view_telemetry_device_states` with state-to-numeric mapping
- [x] 1.4 Apply migrations to the local development database

## 2. NixOS Configuration (prod)

- [x] 2.1 Enable `services.grafana` on `prod`
- [x] 2.2 Configure Grafana to use Unix socket for PostgreSQL (`/run/postgresql`)
- [x] 2.3 Set `root_url` and `serve_from_sub_path` for `/grafana`
- [x] 2.4 Set admin credentials: `admin/[PASSWORD]`
- [x] 2.5 Declaratively provision the `lmha3` PostgreSQL data source
- [x] 2.6 Update Nginx `your-domain.com` to proxy `/grafana` to localhost:3000

## 3. UI Integration

- [x] 3.1 Add a link to Grafana in the Admin UI navigation or dashboard

## 4. Verification

- [x] 4.1 Verify Grafana is accessible at `https://your-domain.com/grafana`
- [x] 4.2 Verify the "PostgreSQL" data source is working
- [x] 4.3 Test querying the new views from the Grafana Explore view

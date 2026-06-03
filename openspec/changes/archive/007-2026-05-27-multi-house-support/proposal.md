## Why

The current system assumes a single household with a single PV installation and global visibility for all users. To support scaling to multiple distinct properties, the system must transition to a multi-house architecture where PV configuration is decentralized and user access is scoped to their specific house.

## What Changes

- **Multi-House Architecture**: Introduce a `Houses` entity to encapsulate PV installation details (Home Assistant address and tokens).
- **Scoped Visibility**: Tenants will only see and control devices/telemetry for their assigned house. Admins maintain global visibility via a house selector.
- **Dynamic Configuration**: Move Home Assistant connection settings from hardcoded files/specs to the database, manageable via the Admin UI.
- **Admin UI Enhancements**:
    - House selection dropdown for administrators.
    - Comprehensive User CRUD (Create, Read, Update, Delete) with safety checks (prevent deletion of users with active devices).
    - Removal of redundant system logs (redundant with the main Logs view).
    - Shelly ID discovery: Suggest unregistered Shelly IDs found in MQTT logs during device creation.
- **Telemetry Optimization**: Implement server-side filtering and truncation for the telemetry history to fix "too much truncation" in the UI when "Show All Telemetry" is unchecked.
- **Boiler Metrics**: Update the dashboard to show cumulative runtime in the last 24h (from 5am) instead of just the last-on period.
- **UI/UX Polish**:
    - Explicit house headers in Overview and Admin panels.
    - Automatic UI refresh/reload on house change or manual toggle.

## Capabilities

### New Capabilities
- `house-management`: Management of physical properties, including their specific Home Assistant integration settings (API token, host).

### Modified Capabilities
- `data-model`: Update schema to include Houses and associate Tenants/Devices with Houses.
- `auth`: Implement house-scoped authorization for tenants and global house selection for admins.
- `load-management`: Update the scheduler to iterate through houses and use house-specific Home Assistant credentials.
- `telemetry`: Enhance the telemetry API to support server-side filtering/truncation for history events.

## Impact

- **Database**: Migration required to add `houses` table and update `tenants` and `devices` tables with foreign keys.
- **API**: `/api/telemetry` and other endpoints must become house-aware.
- **UI**: Significant updates to the Dashboard and Admin panels to support house selection and scoped views.
- **Scheduler**: Logic needs to fetch per-house configuration from the DB instead of using global defaults.
- **Breaking**: Existing configuration in `secrets/ha-token.md` and hardcoded HA IPs will be deprecated in favor of DB-stored values. **BREAKING**

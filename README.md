# lmha3: Lasten Management Hagenholz

Load management for multi-tenant houses sharing a photovoltaics installation. Matches physical tenant loads (Shelly 1 Pro) with solar production. 

Early work, use at your own risk.

## Architecture

```mermaid
flowchart TB
 subgraph subGraph1["lmha3 Server"]
        Web["Web API / Frontend"]
        Sched["Scheduler Loop"]
  end
    Shelly["Shelly 1 Pro Devices"] <-- Status / RPC --> MQTT["MQTT Broker"]
    MQTT <-- Pub/Sub --> subGraph1
    HA["Home Assistant"] -- PV & Consumption Data --> subGraph1
    subGraph1 <--> DB[("PostgreSQL")]
    subGraph1 <-- HTTP/JSON --> User(("User Browser"))
```
### Components
- **Server**: 
    - **Web API**: Serves the vanilla JS Single-Page App and provides JSON endpoints for authentication and control. Both for administrators and tenants
    - **Scheduler Loop**: Background thread that matches solar production (obtained from Home Assistant) with tenant loads.
- **lmha-admin**: CLI tool for initial user creation and system management.
- **PostgreSQL**: Stores persistent state, historical telemetry, and user sessions.
- **MQTT Broker**: Handles bidirectional communication with physical Shelly hardware.
- **Home Assistant**: Source for real-time solar production and total house consumption.


## Configuration

The application is configured via environment variables.

| Variable | Description | Default |
|----------|-------------|---------|
| `LMHA_DATABASE_URL` | PostgreSQL connection string (e.g., `postgresql://...`) | (Required) |
| `LMHA_MQTT_HOST` | Hostname of the MQTT broker | `localhost` |
| `LMHA_MQTT_PORT` | Port of the MQTT broker | `1883` |
| `LMHA_MQTT_USER` | Username for MQTT authentication | (Optional) |
| `LMHA_MQTT_PASSWORD`| Password for MQTT authentication | (Optional) |
| `LMHA_INSTANCE_ID` | Unique identifier for this instance | `lmha3-<random>` |
| `LMHA_INSTANCE_PRIORITY`| Instance priority (higher number = higher priority) | `10` |

Remaining configuration (devices/PV URLs and API tokens) can be done via web interface and is stored in the database.

## Deployment

1. Install NixOS.
2. Drop the snippet below in your NixOs configuration and run `nixos-rebuild switch`.
3. On first start, the server will initialize the database and create a user `admin` with password `admin`. You should then immediately change the password in the webinterface.
4. Create the rest of the configuration in the Web UI with the admin user.

```nix
{ config, pkgs, ... }:
let
    lmha3-src = builtins.fetchGit {
        url = "https://github.com/example/lmha3.git";
        ref = "refs/tags/v0.0.20";
    };
in
{
  imports = [
    (import "${lmha3-src}/nix/module.nix")
  ];

  services.lmha3 = {
    enable = true;
    databaseUrl = "postgresql:///lmha3?host=/run/postgresql";
    mqtt = {
      host = "localhost";
      port = 1883;
    };
  };

  # Dependency: PostgreSQL
  services.postgresql = {
    enable = true;
    ensureDatabases = [ "lmha3" ];
    ensureUsers = [
      {
        name = "lmha3";
        ensureDBOwnership = true;
      }
    ];
  };

  # Dependency: Mosquitto (MQTT Broker)
  services.mosquitto = {
    enable = true;
    listeners = [
      {
        acl = [ "pattern readwrite #" ];
        address = "localhost";
        port = 1883;
        settings.allow_anonymous = true;
      }
    ];
  };
}
```

### Optional: Grafana Visualization

To enable Grafana dashboards for telemetry visualization, add the following to your configuration. This will automatically provision a PostgreSQL data source and use the helper views created by the migrations.

```nix
  services.grafana = {
    enable = true;
    settings.server = {
      domain = "your-domain.com";
      root_url = "https://your-domain.com/grafana/";
      serve_from_sub_path = true;
    };
    provision = {
      enable = true;
      datasources.settings.datasources = [
        {
          name = "PostgreSQL";
          type = "postgres";
          url = "/run/postgresql"; 
          database = "lmha3";
          user = "lmha3"; 
          jsonData = {
            database = "lmha3";
            postgresVersion = 1500; # Adjust to your PostgreSQL version
            sslmode = "disable";
          };
        }
      ];
    };
  };

  # Optional: Proxy /grafana via Nginx
  services.nginx.virtualHosts."your-domain.com".locations."/grafana" = {
    proxyPass = "http://127.0.0.1:3000";
    proxyWebsockets = true;
  };
```

**Database Note:** Since Grafana runs as the `grafana` user, you must ensure it can connect to the `lmha3` database. If using Unix socket connection as shown above, you may need to adjust your PostgreSQL authentication settings to allow the `grafana` user to log in as `lmha3`. An easy (but less secure) way is to use `trust` for local connections:

```nix
  services.postgresql.authentication = lib.mkForce ''
    local   all             all                                     trust
    host    all             all             127.0.0.1/32            trust
    host    all             all             ::1/128                 trust
  '';
```

### Optional: Single Sign-On (SSO) for Grafana

If you want users logged into `lmha3` to be automatically logged into Grafana, you can use the **Auth Proxy** feature. `lmha3` provides a verification endpoint at `/api/auth/verify` for this purpose.

1.  **Update Grafana Settings:**
    Enable the auth proxy in your `services.grafana.settings`:
    ```nix
    services.grafana.settings."auth.proxy" = {
      enabled = true;
      header_name = "X-WEBAUTH-USER";
      header_property = "username";
      auto_sign_up = true;
      # Role mapping
      enable_login_token = true;
      header_role_name = "X-WEBAUTH-ROLE";
      role_attribute_path = "Viewer"; # Default if no header
    };
    ```

2.  **Update Nginx Configuration:**
    Configure Nginx to verify the session with `lmha3` and pass both user and role headers:
    ```nix
    services.nginx.virtualHosts."your-domain.com" = {
      locations."/grafana" = {
        proxyPass = "http://127.0.0.1:3000";
        proxyWebsockets = true;
        extraConfig = ''
          auth_request /api/auth/verify;
          auth_request_set $user $upstream_http_x_auth_user;
          auth_request_set $role $upstream_http_x_auth_role;
          proxy_set_header X-WEBAUTH-USER $user;
          proxy_set_header X-WEBAUTH-ROLE $role;
        '';
      };
      # Ensure the verify endpoint is accessible for internal auth requests
      locations."/api/auth/verify" = {
        proxyPass = "http://127.0.0.1:8765"; # Port of lmha3
        extraConfig = ''
          proxy_pass_request_body off;
          proxy_set_header Content-Length "";
          proxy_set_header X-Original-URI $request_uri;
        '';
      };
    };
    ```

## Development

`nix develop` followed by `cargo test` or `cargo build` or ``cargo run -p server`

There is a convenience script `./dev.sh`

## Scheduler Data Model

The scheduler manages devices using four distinct modes: **Manual**, **Force ON**, **Force OFF**, and **Boiler**. In **Manual** mode, the scheduler ignores the device, allowing for external or manual control. **Force ON** and **Force OFF** provide temporary overrides that hold the device in a specific state until a set expiration time is reached. The **Boiler** mode is the primary automation state, where the scheduler dynamically toggles the device to match available solar production (PV surplus) while ensuring daily runtime targets are met. By prioritizing devices in Boiler mode against a calculated "load budget," the system maximizes self-consumption. Further details on the decision logic and priority scoring are available in the [Load Management Specification](openspec/specs/load-management/spec.md).

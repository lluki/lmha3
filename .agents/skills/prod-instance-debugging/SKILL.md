---
name: prod-instance-debugging
description: Workflow for debugging the production instance of LMHA3. Use when asked to investigate issues on prod, check logs, or explore the production database.
---

# Production Instance Debugging

This skill provides a standard workflow for accessing and debugging the production environment.

## Environment Variables
The following environment variables are used for access:
- `LMHA_DEBUG_PROD_HOST`: The SSH host for the production instance.
- `LMHA_DEBUG_PROD_SHELLY_IP`: The IP address of the test Shelly device.

## SSH Access
To access the production host, use the following command:
```bash
ssh $LMHA_DEBUG_PROD_HOST
```
Note: Ensure your SSH key is authorized for passwordless access.

## Log Inspection
The main service logs can be accessed via `journalctl`. Since there is a high volume of debug messages, it is recommended to use filtering or search:

- View logs for the service:
  ```bash
  journalctl -u lmha3
  ```
- View only recent logs:
  ```bash
  journalctl -u lmha3 -n 100 -f
  ```
- Filter by keyword:
  ```bash
  journalctl -u lmha3 | grep "ERROR"
  ```

## System Configuration
To learn about the system setup and DB parameters, check the NixOS configuration:
```bash
cat /etc/nixos/configuration.nix
```
Look for `services.postgresql` or database-related environment variables in the service definition.

## Database Exploration
To connect to the database, use `psql` with the parameters found in the configuration:
```bash
psql -U <user> -d <database>
```

## Test Shelly Access
If you need to interact with the test Shelly device in the production environment:
- IP Address: `$LMHA_DEBUG_PROD_SHELLY_IP`
- Use `curl` or other HTTP tools to access its local API.

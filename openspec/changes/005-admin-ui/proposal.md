# Proposal: 005-admin-ui

## Intent
Create a web-based administrative interface to monitor and manage the system locally.

## Objectives
- Implement an `/admin` dashboard.
- Display all registered tenants and their devices.
- Provide a `dev.sh` script to automate local environment setup.

## Success Criteria
- Navigating to `/admin` as a logged-in user shows a list of all tenants and devices.
- `sh dev.sh` successfully starts the API with a ready-to-use database.

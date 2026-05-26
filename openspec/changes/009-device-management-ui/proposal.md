# Proposal: 009-Device Management UI

## Problem Statement
The current system allows adding devices manually to the database, but there is no user-friendly way to manage them. Furthermore, the scheduling logic is fixed to a production-dependent "boiler" mode, providing no flexibility for manual overrides or temporary forced states.

## Proposed Solution
Implement a Device Management UI within the Admin dashboard that allows:
1. Viewing all devices associated with the tenant.
2. Configuring `expected_load` (in Watts) for each device.
3. Managing scheduling types: `None`, `Force-Off`, `Force-On`, and `Boiler`.
4. Setting a transition timestamp for "Force-*" states to automatically revert to "Boiler" mode.

## Goals
- Provide a clear interface for device configuration.
- Introduce flexible scheduling modes.
- Ensure the scheduling algorithm respects these new modes.
- Maintain auditability through logging of state transitions.

## Scope
- Database schema updates for `expected_load` and scheduling fields.
- Backend API for device listing and updates.
- Frontend UI components for device management.
- Update `lmha-core` and `server` scheduling logic.

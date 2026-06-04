## Context

The Admin UI allows managing devices through a modal that has two states: Read-only Overview and Edit Mode. Currently, certain fields like "Runtime" are only visible when the device is explicitly in "Boiler" mode. However, "Runtime" is a persistent configuration that remains relevant even when a device is temporarily overridden (Force ON/OFF).

## Goals / Non-Goals

**Goals:**
- Ensure "Runtime" configuration is always visible and editable in the Admin UI.
- Improve the "Until" field visibility logic to be strictly based on override modes.
- Provide "Last Heartbeat" information in the overview for better diagnostics.
- Standardize the "Save" button label.

**Non-Goals:**
- Changes to the backend API or data schema.
- Changes to the User (Tenant) UI.

## Decisions

- **Unconditional Runtime Display**:
    - In `server/public/app.js`, remove CSS `display: none` logic for the runtime field container in both the edit form and the read-only overview.
    - Update `handleSchedulingChangeInModal` to no longer toggle visibility of the boiler config container based on the mode.
  
- **Conditional "Until" Display**:
    - Maintain (and refine if necessary) the logic that shows the "Until" (override expiration) field only when the selected mode is `FORCE_ON` or `FORCE_OFF`.
  
- **Last Heartbeat in Overview**:
    - Add a new `detail-row` to the device overview modal displaying `last_feedback_time` formatted as a local string.
  
- **Button Labeling**:
    - Change "Save All" to "Save" in the device edit form template.

## Risks / Trade-offs

- **[Risk] UI Clutter** → The "Runtime" field will now take up space even for devices in "Manual" mode. 
    - *Mitigation*: The field is small (number input) and standardizing its presence makes the UI more predictable.
- **[Risk] Confusion over "Until"** → Users might wonder why "Until" disappears.
    - *Mitigation*: "Until" only has functional meaning for temporary overrides, so hiding it when in "Boiler" or "Manual" mode reduces cognitive load.

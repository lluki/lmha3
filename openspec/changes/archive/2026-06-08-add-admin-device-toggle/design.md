## Context

Manual toggles were recently removed from the user-facing overview to simplify the interface. However, administrators still require this functionality for troubleshooting and immediate control. The backend logic for toggling is already implemented and exposed via `/api/devices/{id}/toggle`.

## Goals / Non-Goals

**Goals:**
- Restore manual toggle functionality for administrators.
- Provide both a "quick toggle" from the admin list view and a toggle within the device detail modal.
- Ensure the UI refreshes and shows "Syncing..." status immediately after a toggle.

**Non-Goals:**
- Changing the scheduling logic or the core toggle implementation.
- Restoring manual toggles to the non-admin overview page.

## Decisions

### 1. Placement of Toggle Buttons
- **Detail Modal**: A "Toggle" button will be added to the device detail modal (Settings tab) when not in edit mode. This is the safest place for a toggle.
- **Summary Card**: A small toggle icon or button will be added to the top-right of the admin device summary cards. This allows for quick testing of multiple devices without opening modals.

### 2. Implementation Strategy
- Use the existing `window.toggleDevice(id, context)` function in `server/public/app.js`.
- Pass `'admin'` as the context to ensure `renderAdmin()` is called after a successful toggle, which refreshes the admin view.
- For the card-level toggle, use `event.stopPropagation()` to prevent the modal from opening when the toggle is clicked.

## Risks / Trade-offs

- **[Risk] Accidental Toggles** → The card-level toggle button should be clearly distinct from the card background and use standard "action" styling to signal it's an interactive element.
- **[Trade-off] UI Clutter** → Adding more buttons to the summary cards increases visual noise. However, for an administrative interface, functionality and speed are prioritized over minimalist aesthetics.

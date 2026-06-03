## Why

The current admin UI is cluttered with tables containing many inline inputs, making it difficult to scan and prone to accidental edits. Transitioning to a view-detail-edit pattern will improve clarity, usability, and data integrity by separating browsing from modification.

## What Changes

- **Summary Cards**: Replace table rows with compact summary cards for Houses, Tenants, and Devices.
- **Detail View**: Implement a detail view (modal or slide-over) that opens upon clicking an entry.
- **Edit Mode**: Add an "Edit" button in the detail view to toggle between read-only and editable fields.
- **Creation Dialog**: Replace inline creation forms with a unified "+" button that opens a creation dialog.
- **Improved UX**: Clearer separation between different management sections and better feedback during updates.

## Capabilities

### New Capabilities
- `admin-ui`: Defines the structured interaction patterns for administrative management of system entities (Houses, Tenants, Devices).

### Modified Capabilities
- `house-management`: Update requirements to reflect the new interaction pattern for house configuration.

## Impact

- `server/public/app.js`: Major refactor of `renderAdmin` and related UI logic.
- `server/public/index.html`: Support for modals and new navigation elements.
- `server/public/style.css`: Styling for cards, modals, and interactive states.
- `server/src/main.rs`: No API changes expected, but UI logic will change significantly.

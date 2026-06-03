## Context

The administrative interface currently relies on dense tables with inline inputs for managing system entities (Houses, Tenants, Devices). While functional, this layout is visually cluttered and lacks clear separation between observation and modification. The application uses Pico CSS v2 for its design system and is primarily driven by a single-page application pattern in `app.js`.

## Goals / Non-Goals

**Goals:**
- Implement a summary-detail-edit pattern for all administrative entities.
- Use card-based layouts for high-level overview.
- Utilize native HTML5 `<dialog>` elements for modals.
- Create a unified "Creation" entry point for all entities.
- Improve visual hierarchy and reduce cognitive load in the Admin panel.

**Non-Goals:**
- Backend API refactoring (unless strictly necessary).
- Redesign of the main Dashboard/Overview tab.
- Introduction of heavy frontend frameworks (React, Vue, etc.).

## Decisions

- **UI Pattern**: Summary Cards + Modal Details. Summary cards will show identifying info (e.g., Name, House). Clicking a card opens a `<dialog>` with full details.
- **Modal Strategy**: Use HTML5 `<dialog>` with Pico CSS. This avoids external dependencies and leverages native browser behavior for focus management and backdrop.
- **Unified Creation**: A single floating or prominent "+" button will open a choice dialog (House/Tenant/Device) to trigger the respective creation form in a modal.
- **View/Edit Toggle**: Inside the detail modal, fields are read-only by default. An "Edit" button replaces labels with inputs and shows "Save/Cancel" buttons.
- **Componentization**: Refactor `renderAdmin` into logical sub-renderers (e.g., `renderHouseGrid`, `renderEntityCard`) to improve maintainability of the frontend code.

## Risks / Trade-offs

- [Risk] Increased click depth for simple edits. → [Mitigation] Ensure the "Edit" transition is seamless and use keyboard shortcuts (Escape to close, Enter to save) where possible.
- [Risk] Managing modal state in a vanilla JS app. → [Mitigation] Use a simple `openModal(content)` helper to handle lifecycle and DOM injection.
- [Risk] CSS styling collisions. → [Mitigation] Scope new styles in `style.css` under an `.admin-refactor` or similar class if necessary, though Pico's semantic approach should minimize this.

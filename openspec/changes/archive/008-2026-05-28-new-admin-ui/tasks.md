## 1. Foundation & Styles

- [x] 1.1 Add `<dialog id="admin-modal">` and a floating unified "+" button to `server/public/index.html`.
- [x] 1.2 Add CSS for `.admin-grid`, `.summary-card`, and modal styling to `server/public/style.css`.
- [x] 1.3 Implement `showModal(title, content)` and `closeModal()` helpers in `server/public/app.js`.

## 2. House Management Refactor

- [x] 2.1 Create `renderHouseCard(house)` function to generate summary card HTML.
- [x] 2.2 Create `renderHouseDetails(house, isEdit)` function to generate modal content.
- [x] 2.3 Refactor `renderAdmin` to display Houses in a card-based grid instead of a table.
- [x] 2.4 Update `updateHouse` and `deleteHouse` handlers to work with the new modal-based flow.

## 3. Tenant Management Refactor

- [x] 3.1 Create `renderTenantCard(tenant, houseName)` function for summary view.
- [x] 3.2 Create `renderTenantDetails(tenant, houses, isEdit)` function for modal view.
- [x] 3.3 Update `renderAdmin` to display Tenants in a card-based grid.
- [x] 3.4 Update `updateTenant` and `deleteTenant` handlers for the modal flow.

## 4. Device Management Refactor

- [x] 4.1 Create `renderDeviceCard(device, tenantName)` function for summary view.
- [x] 4.2 Create `renderDeviceDetails(device, tenants, isEdit)` function for modal view.
- [x] 4.3 Update `renderAdmin` to display Devices in a card-based grid.
- [x] 4.4 Update `updateDeviceConfigAdmin` and `deleteDevice` handlers for the modal flow.

## 5. Unified Creation Flow

- [x] 5.1 Implement `openCreationDialog()` to show options for creating House, Tenant, or Device.
- [x] 5.2 Implement `renderCreateHouseForm()`, `renderCreateTenantForm()`, and `renderCreateDeviceForm()` for the modal.
- [x] 5.3 Ensure Shelly discovery suggestions are integrated into the `renderCreateDeviceForm`.

## 6. Verification & Final Cleanup

- [x] 6.1 Verify full CRUD lifecycle for Houses in the new UI.
- [x] 6.2 Verify full CRUD lifecycle for Tenants in the new UI.
- [x] 6.3 Verify full CRUD lifecycle for Devices in the new UI.
- [x] 6.4 Remove all legacy table-based rendering code and inline forms from `app.js`.

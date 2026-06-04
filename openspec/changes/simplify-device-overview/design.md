## Context

The current Web UI overview page provides too much granularity for regular users, exposing underlying state controls (like manual toggles, forcing on/off, etc.) that can lead to confusion and misconfiguration. To streamline the user experience, the device overview must become a read-only dashboard showing key facts (health, current state, 24h runtime). The only modifiable action should be a new "Vacation Mode" which allows users to declare an absence. The system must then translate this return date into a `FORCE_OFF` scheduling state that ends at 5:00 AM on the day *before* the chosen date, ensuring a full heat-up cycle occurs before the user returns.

## Goals / Non-Goals

**Goals:**
- Provide a simplified, mobile-friendly device overview dashboard.
- Display only: Device Name, Health Status (Yes, No with last seen), On/Off Status, Today's Runtime, and current Mode.
- Introduce a "Set Vacation Absence" modal with a date picker.
- Automatically calculate the `FORCE_OFF` expiration timestamp (5:00 AM of the day prior to the selected return date).
- Remove all other manual toggle controls and owner information from the overview UI.

**Non-Goals:**
- Changing the underlying MQTT or Shelly integration.
- Modifying the Admin UI (admins may still need granular controls elsewhere, though the current scope only mentions the "overview page of the Web UI under my devices").
- Altering the core Boiler or scheduling algorithm (other than utilizing the existing `FORCE_OFF` and `scheduling_until` mechanism).

## Decisions

- **UI Framework / Components**: We will continue using Pico.css and plain HTML/JS for the frontend. The modal will be a standard HTML `<dialog>` element, triggered by the "Set Vacation Absence" button.
- **Date Picker**: We will use a standard `<input type="date">`. It is natively supported on mobile devices, providing a highly accessible and touch-friendly experience without external libraries.
- **Backend API for Vacation Mode**: We will introduce a new or modified REST endpoint (e.g., `POST /api/devices/{id}/vacation`) that accepts a target return date (`YYYY-MM-DD`). The backend will calculate the `scheduling_until` timestamp (Return Date minus 1 day, at 05:00:00 local time) and update the device's scheduling state to `FORCE_OFF`.
- **Runtime Calculation**: The requirement asks for "How long was the device on today? (this is equal to the current 24h runtime)". We will reuse the existing API endpoint that provides the accumulated runtime since 5:00 AM for the dashboard.
- **Health Status**: Computed from the existing `last_seen` timestamp. If recently seen (e.g., within 5-10 minutes), display "Yes". Otherwise, display "No, <last_seen time>".

## Risks / Trade-offs

- **[Risk] Timezone Issues**: The calculation of "5am of the previous day" relies on correct timezone handling.
  - **Mitigation**: The backend will use the server's local timezone (or a configured specific timezone) to perform date math, ensuring the 5:00 AM target aligns with the physical location of the boiler.
- **[Risk] Admin vs User Access**: If the UI is completely stripped of manual toggles, admins might lose a quick way to override devices if they use the same UI.
  - **Mitigation**: The Admin tab should remain separate and can retain advanced controls, or admins can use direct DB/API access if needed. This change strictly targets the "My Devices" overview.

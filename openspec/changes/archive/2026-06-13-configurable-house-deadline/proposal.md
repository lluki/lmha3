## Why

The current scheduler implementation uses UTC hours for its "catch-up" window (1 AM - 5 AM) and day-start (5 AM) logic. This causes unexpected behavior in timezones other than UTC (e.g., triggering a night charge at 5 AM local time because it's still 3 AM UTC). 

## What Changes

- Add a configurable "deadline" (end of day) time to the `houses` table.
- Refactor the scheduler to use this house-specific deadline to calculate day windows and catch-up periods.
- Standardize all internal and API data exchange to use UTC (ISO8601 with 'Z').
- Update the frontend to correctly translate UTC timestamps to the user's local timezone.

## Capabilities

### New Capabilities
- `house-deadline-config`: Allows users to configure the specific time (e.g., "05:00") when their energy day ends, which determines the catch-up window.

### Modified Capabilities
- `load-management`: Update the scheduling logic to use house-specific deadline offsets instead of hardcoded UTC hours.
- `telemetry`: Ensure all telemetry timestamps are consistently handled as UTC across the stack.

## Impact

- **Database**: `houses` table needs a new column for the deadline time.
- **Backend (lmha-core)**: Scheduler logic and telemetry insertion/retrieval.
- **API (server)**: All timestamp serialization must include the 'Z' suffix.
- **Frontend (UI)**: JS timezone translation for all displayed timestamps.

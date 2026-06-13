## Context

The current `lmha3` system implements time-of-day logic (such as the 1 AM - 5 AM "catch-up" window and the 5 AM start-of-day) using hardcoded UTC hours. This leads to incorrect scheduling behavior for houses located in different timezones (e.g., Switzerland at UTC+2). Furthermore, there is inconsistency in how timestamps are serialized and displayed across the stack.

## Goals / Non-Goals

**Goals:**
- Make the "Day Deadline" (energy day boundary) configurable per house.
- Decouple the catch-up window logic from specific wall-clock hours, making it relative to the deadline (e.g., "4 hours before deadline").
- Standardize all internal state and API communication on UTC (ISO8601 with 'Z' suffix).
- Enable automatic local timezone display in the Web UI using browser settings.

**Non-Goals:**
- Implementing full timezone database support (we will stick to "hours before deadline" logic which is simpler and covers the user's needs).
- Support for complex seasonal shifts beyond what the OS `Local` timezone or browser already handles.

## Decisions

### 1. Database Schema Update
We will add a `day_deadline` column to the `houses` table.
- **Type**: `TIME WITHOUT TIME ZONE` (or `TEXT` if simpler for Rust integration, but `TIME` is more idiomatic).
- **Default**: `'05:00:00'`.
- **Rationale**: This allows the user to specify exactly when their "energy day" ends in their local time.

### 2. Scheduler Logic (lmha-core)
The `SchedulerInput` will be extended to include the `day_deadline` (as a `chrono::NaiveTime`).
- **`get_start_of_day`**: This function will be refactored to take the `day_deadline`. It will use the house's local time (derived from `now.with_timezone(&Local)`) to find the most recent occurrence of that deadline time.
- **Catch-up Window**: Instead of `hour >= 1 && hour < 5`, we will use `now >= (deadline - 4 hours) && now < deadline`.

### 3. API Serialization (server)
We will audit all `chrono::DateTime` usages in the `server` and `lmha-core` crates.
- **Enforcement**: Use `DateTime<Utc>` for all public-facing structs.
- **Serialization**: Ensure `serde` serializes these as ISO8601 strings with the `Z` suffix. (Chrono's default `serde` implementation for `DateTime<Utc>` does this correctly).

### 4. Frontend Localization (UI)
The `app.js` will be updated to handle timestamp display.
- **Strategy**: Pass the raw ISO8601 string from the API to `new Date(timestampString)`.
- **Display**: Use `.toLocaleString()` or `.toLocaleTimeString()` to show the value in the user's browser-configured timezone.
- **Logic**: For "Runtime since [Deadline]", the UI will need to know the house's deadline time to calculate the correct window for display.

## Risks / Trade-offs

- **[Risk]** Server `Local` timezone mismatch with House location → **[Mitigation]** While not perfect, using the server's local time for "Day Start" calculation is a significant improvement over hardcoded UTC. Long-term, adding a specific `timezone` string to the house would be the next step.
- **[Risk]** Data migration for existing houses → **[Mitigation]** The SQL migration will provide a default '05:00:00' for all existing records.
- **[Risk]** Ambiguity during DST jumps (the "missing hour" or "double hour") → **[Mitigation]** Using `chrono`'s `Local` offset handling for the "Start of Day" calculation handles these transitions gracefully in most cases.

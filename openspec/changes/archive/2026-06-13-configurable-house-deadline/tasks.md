## 1. Database & Schema

- [x] 1.1 Create migration `014_add_house_deadline.sql` to add `day_deadline` column to `houses` table
- [x] 1.2 Update `House` struct in `lmha-core/src/lib.rs` to include `day_deadline` (as `chrono::NaiveTime`)
- [x] 1.3 Update `Db` methods in `lmha-core/src/db.rs` to select/insert/update the new field

## 2. Scheduler Refactor

- [x] 2.1 Update `get_start_of_day` in `lmha-core/src/scheduler.rs` to take `day_deadline` as an argument
- [x] 2.2 Refactor `decide_action` in `lmha-core/src/scheduler.rs` to use a relative "4-hour before deadline" catch-up window
- [x] 2.3 Update `run_scheduler_loop` in `server/src/main.rs` to pass the house-specific deadline to the scheduler logic
- [x] 2.4 Verify scheduler changes with unit tests, including DST transition edge cases

## 3. Backend API & UTC Enforcement

- [x] 3.1 Audit `server/src/main.rs` and `lmha-core/src/db.rs` to ensure all `DateTime` fields are `DateTime<Utc>`
- [x] 3.2 Update House creation (`POST /api/houses`) and update (`PATCH /api/houses/{id}`) endpoints to accept `day_deadline`
- [x] 3.3 Ensure all JSON telemetry responses include the 'Z' suffix in timestamps

## 4. Frontend & Localization

- [x] 4.1 Update the "House Management" UI to allow editing the "Day Deadline"
- [x] 4.2 Create a utility function in `app.js` for localized timestamp formatting
- [x] 4.3 Replace all hardcoded "5:00 AM" references in the UI with the dynamic house deadline
- [x] 4.4 Refactor UI timestamp rendering (Logs, History, Device Runtime) to use browser local time

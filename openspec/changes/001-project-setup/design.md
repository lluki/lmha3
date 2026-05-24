# Design: 001-project-setup (Synchronous/Simple)

## Architecture Overview
A multi-binary Rust project favoring synchronous execution and direct SQL for simplicity.

### Components
1. **Shared Library (`lmha3-core`):** 
   - Shared data types (Structs).
   - Database interaction logic using the `postgres` crate (Direct SQL).
   - MQTT client logic (using `rumqttc` in sync mode).
2. **API (`lmha3-api`):** 
   - Synchronous web server using `rouille` (built on `tiny-http`).
   - Handles authentication and dashboard rendering.
3. **Scheduler (`lmha3-scheduler`):** 
   - Main loop that sleeps and polls.
   - Polling Home Assistant via `ureq` (Synchronous HTTP client).

### Data Access
- **Library:** `postgres` crate.
- **Approach:** Hand-written SQL queries. No ORM.
- **Migrations:** Simple SQL files executed at startup or managed via a basic custom script.

### Rationale
- **No Async:** Avoids `tokio`, `Future` traits, and `await` complexity. 
- **Direct SQL:** Transparent performance and easier debugging of queries.
- **Portability:** Minimizes dependencies and compile times.

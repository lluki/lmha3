# Tasks: 002-authentication

- [x] 1. Database Schema Update
    - [x] 1.1 Create migration for `sessions` table.
- [x] 2. Core Authentication Logic (lmha-core)
    - [x] 2.1 Implement `verify_password` using Argon2.
    - [x] 2.2 Implement `hash_password` for initial user creation.
    - [x] 2.3 Add database functions to find tenants and manage sessions.
- [x] 3. API Implementation (api)
    - [x] 3.1 Implement `/login` POST handler.
    - [x] 3.2 Implement `/logout` handler.
    - [x] 3.3 Create authentication middleware/guard for protected routes.
    - [x] 3.4 Create a basic "Dashboard" protected route showing tenant info.
- [x] 4. Verification
    - [x] 4.1 Create a test script or manual verification steps for login/auth flow.

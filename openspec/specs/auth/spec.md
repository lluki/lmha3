# Spec: Authentication

## Overview
Secure access control for the `lmha3` web interface.

## Requirements
1. **Authorization Levels:**
   - **Global Read (Unconditional):** Any logged-in tenant can view the status of all devices, PV production, house consumption, and all telemetry history events.
   - **Owner/Admin Write (Toggles):** A tenant can toggle (ON/OFF) devices they own. Administrators can toggle ANY device in the system.
   - **Admin Only (System Management):** Only users with the `is_admin` flag can create/delete devices, manage tenants, or perform other system-level configuration changes.
2. **Session Management:**
   - Secure cookie-based or JWT sessions.
   - Session duration: 24 hours.
3. **Password Security:**
   - Hashes stored using Argon2id.
   - Minimum password length: 12 characters.
4. **Endpoint Protection:**
   - All `/api/*` and UI routes (except `/login`) require a valid session.
   - Unauthorized access results in a redirect to login or 401 Unauthorized.

## User Flow
1. User visits `solar.lluki.me`.
2. Nginx forwards request to Rust backend.
3. Backend checks for session cookie.
4. If missing, redirect to `/login`.
5. Upon successful login, session is created and user redirected to Dashboard.

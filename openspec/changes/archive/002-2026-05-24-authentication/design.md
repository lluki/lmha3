# Design: 002-authentication

## Overview
A synchronous, cookie-based authentication system using `rouille` and `argon2`.

## Components

### 1. User Persistence
- Logic in `lmha-core` to fetch a `Tenant` by username and verify password hashes.

### 2. Password Hashing
- Use the `argon2` crate for secure hashing and verification.

### 3. Session Management
- Use `rouille::input::cookies` for session tracking.
- For the MVP, a simple session token stored in a `sessions` table in PostgreSQL or a simplified in-memory map (with PostgreSQL preferred for persistence across restarts).

### 4. Middleware/Guard
- A wrapper function in the API to check for a valid session before allowing access to protected routes.

## Schema Update
- Add a `sessions` table to PostgreSQL:
  ```sql
  CREATE TABLE sessions (
      id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
      tenant_id UUID NOT NULL REFERENCES tenants(id),
      expires_at TIMESTAMPTZ NOT NULL,
      created_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```

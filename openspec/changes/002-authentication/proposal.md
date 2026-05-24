# Proposal: 002-authentication

## Intent
Implement a secure, tenant-based authentication system for the LMHA3 web interface.

## Objectives
- Implement password hashing using Argon2id.
- Create login and logout endpoints.
- Implement session management (cookie-based).
- Enforce the "Global Read" and "Owner Write" authorization model.

## Success Criteria
- Users can login with valid credentials.
- Logged-in users can view all data.
- Only owners can perform actions on their devices (verified via mock endpoints).
- Unauthorized users are redirected to the login page.

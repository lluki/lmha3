# Spec: Secret Management

## Overview
Management of global and house-specific secrets, ensuring isolation and security.

## Requirements

### Requirement: Environment Secret Isolation
The system SHALL load global credentials and connection strings from environment variables, which MUST be isolated from version control.

#### Scenario: Service loads secrets from environment
- **WHEN** the service starts
- **THEN** it MUST attempt to load `DATABASE_URL`, `MQTT_USER`, and `MQTT_PASSWORD` from the process environment
- **AND** it SHALL support loading these from a `.env` file in development

### Requirement: House Secret Isolation
The system SHALL store house-specific integration secrets and configuration in the database.

#### Scenario: Service retrieves house secrets
- **WHEN** the scheduler or telemetry fetcher processes a specific house
- **THEN** it MUST retrieve the `ha_url`, `ha_token`, and entity IDs from the `houses` table
- **AND** it MUST NOT use global environment variable fallbacks for these fields

### Requirement: Git Hygiene for Secrets
The system SHALL ensure no secrets are committed to the repository.

#### Scenario: Protected repository
- **WHEN** a developer attempts to commit
- **THEN** the `.gitignore` file MUST prevent `.env` and `secrets/` files from being tracked
- **AND** no hardcoded default credentials SHALL exist in script files (e.g., `dev.sh`)

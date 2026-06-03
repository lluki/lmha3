# Spec: Instance Manager

## Overview
Management of LMHA instance identity, priority, and conflict resolution.

## Requirements

### Requirement: Instance Identity and Priority
Each LMHA instance SHALL have a unique identity and an assigned priority level.

- **Priority Levels**: 
  - `PROD`: High priority (e.g., 100)
  - `DEV`: Low priority (e.g., 10)
- **Identity**: A unique string (e.g., "prod-server-1", "dev-laptop-user").

#### Scenario: Instance priority is configurable
- **WHEN** the server starts
- **THEN** it MUST load its identity and priority from environment variables (`LMHA_INSTANCE_ID`, `LMHA_INSTANCE_PRIORITY`).

### Requirement: Conflict Detection via MQTT
The system SHALL use MQTT heartbeats to detect other running instances and prevent control conflicts.

#### Scenario: Heartbeat announcement
- **WHEN** an instance starts
- **THEN** it SHALL publish a heartbeat to `lmha3/instances/<id>` with `retain=true` containing its priority and `status: "online"`.
- **AND** it SHALL configure a Last Will and Testament (LWT) to publish `status: "offline"` with `retain=true` to the same topic.

#### Scenario: Detecting higher priority instance
- **WHEN** an instance receives a heartbeat from a different ID on `lmha3/instances/+`
- **THEN** it SHALL track the priority of all known instances.
- **IF** any known instance has a priority HIGHER than its own priority
- **THEN** it SHALL enter "PASSIVE" mode, disabling automatic state synchronization and scheduling.

#### Scenario: Resuming control
- **WHEN** an instance is in "PASSIVE" mode
- **AND** no higher-priority instances are currently detected as "online"
- **THEN** it SHALL exit "PASSIVE" mode and resume normal operation.

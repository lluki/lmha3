# Spec: Data Model (MVP)

## Overview
This spec defines the PostgreSQL schema and relationships for the `lmha3` MVP.

## Schema Entities

### 0. Houses
Represents a physical property.
- `id`: UUID (Primary Key)
- `name`: String (Unique)
- `ha_host`: String
- `ha_token`: String
- `created_at`: Timestamp

### 1. Tenants
Represents a household or user account.
- `id`: UUID (Primary Key)
- `house_id`: UUID (Foreign Key to Houses)
- `username`: String (Unique)
- `password_hash`: String
- `created_at`: Timestamp

### 2. Devices
Physical Shelly 1 Pro switches associated with a tenant.
- `id`: UUID (Primary Key)
- `house_id`: UUID (Foreign Key to Houses)
- `tenant_id`: UUID (Foreign Key to Tenants)
- `mqtt_topic`: String (Base topic for Shelly)
- `name`: String
- `is_enabled`: Boolean
- `current_state`: Enum (ON, OFF, UNKNOWN)
- `expected_load`: Integer (Watts)
- `scheduling_type`: Enum (NONE, FORCE_ON, FORCE_OFF, BOILER)
- `scheduling_until`: Timestamp (Optional, for FORCE_* transitions)
- `full_charge_n_day`: Integer (Optional, for BOILER mode)
- `min_daily_charge`: Integer (Minutes, for BOILER mode)

### 3. Telemetry (Time-Series)
Historical data for system and devices.
- `timestamp`: Timestamp (Primary Key part)
- `house_id`: UUID (Foreign Key to Houses)
- `source`: Enum (PV_PRODUCTION, HOUSE_CONSUMPTION, DEVICE_STATE)
- `device_id`: UUID (Optional)
- `value`: Double
- `metadata`: JSONB (Context/Reasoning)

## Relationships
- **House (1) -> Tenant (N)**: A house can have multiple tenants.
- **House (1) -> Device (N)**: A house contains multiple devices.
- **Tenant (1) -> Device (N)**: A tenant can own multiple devices.
- **Device (1) -> Telemetry (N)**: Historical state changes.

## Requirements

### Requirement: Multi-House Schema Mapping
The data model SHALL support a hierarchical relationship where Houses contain both Tenants and Devices. Each House MUST store its own integration credentials for external services (Home Assistant).

#### Scenario: Database supports house association
- **WHEN** the database schema is queried
- **THEN** it confirms `tenants` and `devices` tables have a mandatory `house_id` foreign key referencing the `houses` table

### Requirement: Advanced Boiler Configuration
The system SHALL support advanced configuration for devices in Boiler mode:
- **full_charge_n_day**: Number of days (1-8) within which a "full charge" (4h contiguous or aggregate) must occur.
- **min_daily_charge**: Minimum number of minutes/hours the device must run every day (5am to 5am window).

#### Scenario: Admin configures boiler
- **WHEN** an admin sets `full_charge_n_day` to 3 for a device
- **THEN** the system persists this value and uses it to calculate mandatory charge deadlines
- **AND** the configuration is scoped to the device within its respective house

#### Scenario: User modifies own device
- **WHEN** a tenant updates the `expected_load` of a device they own
- **THEN** the system persists the value and applies it to future scheduling decisions
- **AND** the system prevents them from modifying `name`, `mqtt_topic`, or `tenant_id` (Admin only)

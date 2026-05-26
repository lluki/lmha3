# Spec: Data Model (MVP)

## Overview
This spec defines the PostgreSQL schema and relationships for the `lmha3` MVP.

## Schema Entities

### 1. Tenants
Represents a household or user account.
- `id`: UUID (Primary Key)
- `username`: String (Unique)
- `password_hash`: String
- `created_at`: Timestamp

### 2. Devices
Physical Shelly 1 Pro switches associated with a tenant.
- `id`: UUID (Primary Key)
- `tenant_id`: UUID (Foreign Key to Tenants)
- `mqtt_topic`: String (Base topic for Shelly)
- `name`: String
- `is_enabled`: Boolean
- `current_state`: Enum (ON, OFF, UNKNOWN)
- `expected_load`: Integer (Watts)
- `scheduling_type`: Enum (NONE, FORCE_ON, FORCE_OFF, BOILER)
- `scheduling_until`: Timestamp (Optional, for FORCE_* transitions)

### 3. Telemetry (Time-Series)
Historical data for system and devices.
- `timestamp`: Timestamp (Primary Key part)
- `source`: Enum (PV_PRODUCTION, HOUSE_CONSUMPTION, DEVICE_STATE)
- `device_id`: UUID (Optional)
- `value`: Double
- `metadata`: JSONB (Context/Reasoning)

## Relationships
- **Tenant (1) -> Device (N)**: A tenant can own multiple devices.
- **Device (1) -> Telemetry (N)**: Historical state changes.

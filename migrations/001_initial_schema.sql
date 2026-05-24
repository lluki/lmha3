CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TYPE device_state AS ENUM ('ON', 'OFF', 'UNKNOWN');

CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    mqtt_topic TEXT NOT NULL,
    name TEXT NOT NULL,
    is_enabled BOOLEAN DEFAULT TRUE,
    current_state device_state DEFAULT 'UNKNOWN'
);

CREATE TYPE telemetry_source AS ENUM ('PV_PRODUCTION', 'HOUSE_CONSUMPTION', 'DEVICE_STATE');

CREATE TABLE telemetry (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source telemetry_source NOT NULL,
    device_id UUID REFERENCES devices(id),
    value DOUBLE PRECISION NOT NULL,
    metadata JSONB
);

CREATE INDEX idx_telemetry_timestamp ON telemetry(timestamp DESC);

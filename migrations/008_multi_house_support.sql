-- 008_multi_house_support.sql
-- Create houses table and associate tenants/devices with houses

CREATE TABLE IF NOT EXISTS houses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE,
    ha_host TEXT NOT NULL,
    ha_token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed a default house for existing data
INSERT INTO houses (name, ha_host, ha_token)
VALUES ('Default House', '192.168.178.31', 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJkNDg3MjUzYmQ0MDg0ODU2OTY2YTU1ODA3NjhlMTFiMCIsImlhdCI6MTc3OTYyMTQ5MywiZXhwIjoyMDk0OTgxNDkzfQ.W0-9noqqRGQwILHmtmcVv9_8Ql83fF_7QZQrOrheGvY')
ON CONFLICT (name) DO NOTHING;

-- Add house_id to tenants
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS house_id UUID REFERENCES houses(id);

-- Update existing tenants to point to the default house
UPDATE tenants SET house_id = (SELECT id FROM houses WHERE name = 'Default House') WHERE house_id IS NULL;

-- Make house_id NOT NULL for tenants
ALTER TABLE tenants ALTER COLUMN house_id SET NOT NULL;

-- Add house_id to devices
ALTER TABLE devices ADD COLUMN IF NOT EXISTS house_id UUID REFERENCES houses(id);

-- Update existing devices to point to the default house
UPDATE devices SET house_id = (SELECT id FROM houses WHERE name = 'Default House') WHERE house_id IS NULL;

-- Make house_id NOT NULL for devices
ALTER TABLE devices ALTER COLUMN house_id SET NOT NULL;

-- Add current_view_house_id to sessions for admins
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS current_view_house_id UUID REFERENCES houses(id);

-- Add house_id to telemetry
ALTER TABLE telemetry ADD COLUMN IF NOT EXISTS house_id UUID REFERENCES houses(id);

-- Update existing telemetry to point to the default house
UPDATE telemetry SET house_id = (SELECT id FROM houses WHERE name = 'Default House') WHERE house_id IS NULL;

-- Make house_id NOT NULL for telemetry
ALTER TABLE telemetry ALTER COLUMN house_id SET NOT NULL;

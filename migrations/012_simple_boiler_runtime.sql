-- Simplify boiler configuration by using a single daily runtime parameter
ALTER TABLE devices ADD COLUMN IF NOT EXISTS device_runtime INTEGER NOT NULL DEFAULT 180;
ALTER TABLE devices DROP COLUMN IF EXISTS full_charge_n_day;
ALTER TABLE devices DROP COLUMN IF EXISTS min_daily_charge;

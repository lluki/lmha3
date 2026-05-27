-- Add advanced boiler configuration parameters
ALTER TABLE devices ADD COLUMN IF NOT EXISTS full_charge_n_day INTEGER NOT NULL DEFAULT 1;
ALTER TABLE devices ADD COLUMN IF NOT EXISTS min_daily_charge INTEGER NOT NULL DEFAULT 0;

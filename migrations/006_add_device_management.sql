-- Add columns for device management and flexible scheduling
ALTER TABLE devices ADD COLUMN IF NOT EXISTS scheduling_type TEXT NOT NULL DEFAULT 'boiler';
ALTER TABLE devices ADD COLUMN IF NOT EXISTS scheduling_until TIMESTAMP WITH TIME ZONE;

-- Ensure expected_load is an integer (Watts)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name='devices' AND column_name='expected_load' AND data_type='double precision'
    ) THEN
        ALTER TABLE devices ALTER COLUMN expected_load TYPE INTEGER USING expected_load::INTEGER;
    END IF;
END $$;

ALTER TABLE devices ALTER COLUMN expected_load SET DEFAULT 2000;

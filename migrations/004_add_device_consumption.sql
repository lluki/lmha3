DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type t JOIN pg_enum e ON t.oid = e.enumtypid WHERE t.typname = 'telemetry_source' AND e.enumlabel = 'DEVICE_CONSUMPTION') THEN
        ALTER TYPE telemetry_source ADD VALUE 'DEVICE_CONSUMPTION';
    END IF;
END
$$;

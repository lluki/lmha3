ALTER TABLE devices ADD COLUMN IF NOT EXISTS desired_state device_state DEFAULT 'UNKNOWN';
ALTER TABLE devices ADD COLUMN IF NOT EXISTS last_request_time TIMESTAMPTZ;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='devices' AND column_name='last_heartbeat') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='devices' AND column_name='last_feedback_time') THEN
            ALTER TABLE devices RENAME COLUMN last_heartbeat TO last_feedback_time;
        ELSE
            -- Both exist, just drop the old one to avoid collision
            ALTER TABLE devices DROP COLUMN last_heartbeat;
        END IF;
    END IF;
END $$;

-- Initialize desired_state to current_state for existing devices
UPDATE devices SET desired_state = current_state WHERE desired_state = 'UNKNOWN';

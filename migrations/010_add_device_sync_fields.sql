ALTER TABLE devices ADD COLUMN IF NOT EXISTS desired_state device_state DEFAULT 'UNKNOWN';
ALTER TABLE devices ADD COLUMN IF NOT EXISTS last_request_time TIMESTAMPTZ;
ALTER TABLE devices RENAME COLUMN last_heartbeat TO last_feedback_time;

-- Initialize desired_state to current_state for existing devices
UPDATE devices SET desired_state = current_state WHERE desired_state = 'UNKNOWN';

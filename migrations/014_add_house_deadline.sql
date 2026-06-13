-- 014_add_house_deadline.sql
-- Add configurable day deadline to houses

ALTER TABLE houses ADD COLUMN IF NOT EXISTS day_deadline TIME WITHOUT TIME ZONE DEFAULT '05:00:00';
UPDATE houses SET day_deadline = '05:00:00' WHERE day_deadline IS NULL;
ALTER TABLE houses ALTER COLUMN day_deadline SET NOT NULL;

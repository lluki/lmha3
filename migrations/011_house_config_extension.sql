-- 011_house_config_extension.sql
-- Add more configuration fields to houses table and rename ha_host to ha_url

ALTER TABLE houses RENAME COLUMN ha_host TO ha_url;

ALTER TABLE houses ADD COLUMN IF NOT EXISTS ha_pv_entity_id TEXT;
ALTER TABLE houses ADD COLUMN IF NOT EXISTS ha_consumption_entity_id TEXT;

-- Update existing data with some defaults if they were null, 
-- though we expect them to be set via UI later.
UPDATE houses SET 
    ha_url = CASE WHEN ha_url NOT LIKE 'http%' THEN 'http://' || ha_url ELSE ha_url END,
    ha_pv_entity_id = COALESCE(ha_pv_entity_id, 'sensor.panel_production_power'),
    ha_consumption_entity_id = COALESCE(ha_consumption_entity_id, 'sensor.house_load_power')
WHERE ha_pv_entity_id IS NULL OR ha_consumption_entity_id IS NULL;

-- Make them NOT NULL after seeding
ALTER TABLE houses ALTER COLUMN ha_pv_entity_id SET NOT NULL;
ALTER TABLE houses ALTER COLUMN ha_consumption_entity_id SET NOT NULL;

-- Helper views for Grafana visualization

-- View for PV production and House consumption per house
-- Uses Last Observation Carried Forward (LOCF) logic to align asynchronous data points
CREATE OR REPLACE VIEW view_telemetry_house_metrics AS
WITH raw_data AS (
    SELECT
        t.timestamp,
        t.house_id,
        h.name as house_name,
        CASE WHEN t.source = 'PV_PRODUCTION' THEN t.value END as pv,
        CASE WHEN t.source = 'HOUSE_CONSUMPTION' THEN t.value END as house
    FROM telemetry t
    JOIN houses h ON t.house_id = h.id
    WHERE t.source IN ('PV_PRODUCTION', 'HOUSE_CONSUMPTION')
),
grouped_data AS (
    SELECT
        *,
        COUNT(pv) OVER (PARTITION BY house_id ORDER BY timestamp) as pv_grp,
        COUNT(house) OVER (PARTITION BY house_id ORDER BY timestamp) as house_grp
    FROM raw_data
)
SELECT
    timestamp,
    house_id,
    house_name,
    FIRST_VALUE(pv) OVER (PARTITION BY house_id, pv_grp ORDER BY timestamp) as pv_production,
    FIRST_VALUE(house) OVER (PARTITION BY house_id, house_grp ORDER BY timestamp) as house_consumption
FROM grouped_data;

-- View for device states per house (mapped to numeric: ON=1, OFF=0)
CREATE OR REPLACE VIEW view_telemetry_device_states AS
SELECT
    t.timestamp,
    t.house_id,
    h.name as house_name,
    d.name as device_name,
    u.username as owner_name,
    CASE 
        WHEN t.value > 0 THEN 1 -- Assuming ON is represented by positive value
        ELSE 0 
    END as state_numeric
FROM telemetry t
JOIN devices d ON t.device_id = d.id
JOIN houses h ON t.house_id = h.id
JOIN tenants u ON d.tenant_id = u.id
WHERE t.source = 'DEVICE_STATE';

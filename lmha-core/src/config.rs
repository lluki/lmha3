use std::env;
use dotenvy::dotenv;
use uuid::Uuid;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_user: Option<String>,
    pub mqtt_password: Option<String>,
    pub ha_url: String,
    pub ha_token: String,
    pub ha_pv_entity_id: Option<String>,
    pub ha_consumption_entity_id: Option<String>,
    pub instance_id: String,
    pub instance_priority: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        let instance_id = env::var("INSTANCE_ID").unwrap_or_else(|_| {
            let id = Uuid::new_v4().to_string();
            format!("lmha3-{}", &id[..8])
        });
        let instance_priority = env::var("INSTANCE_PRIORITY")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            mqtt_host: env::var("MQTT_HOST").unwrap_or_else(|_| "solar.lluki.me".to_string()),
            mqtt_port: env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1884".to_string())
                .parse()
                .expect("MQTT_PORT must be a number"),
            mqtt_user: env::var("MQTT_USER").ok(),
            mqtt_password: env::var("MQTT_PASSWORD").ok(),
            ha_url: env::var("HA_URL").unwrap_or_else(|_| "http://192.168.178.31:8123".to_string()),
            ha_token: env::var("HA_TOKEN").expect("HA_TOKEN must be set"),
            ha_pv_entity_id: env::var("HA_PV_ENTITY_ID").ok().or_else(|| Some("sensor.panel_production_power".to_string())),
            ha_consumption_entity_id: env::var("HA_CONSUMPTION_ENTITY_ID").ok().or_else(|| Some("sensor.house_load_power".to_string())),
            instance_id,
            instance_priority,
        }
    }
}

use std::env;
use dotenvy::dotenv;

pub struct Config {
    pub database_url: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub ha_url: String,
    pub ha_token: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            mqtt_host: env::var("MQTT_HOST").unwrap_or_else(|_| "solar.lluki.me".to_string()),
            mqtt_port: env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1884".to_string())
                .parse()
                .expect("MQTT_PORT must be a number"),
            ha_url: env::var("HA_URL").unwrap_or_else(|_| "http://localhost:8123".to_string()),
            ha_token: env::var("HA_TOKEN").expect("HA_TOKEN must be set"),
        }
    }
}

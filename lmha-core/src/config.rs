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
    pub instance_id: String,
    pub instance_priority: u32,
    pub instance_topic_prefix: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        let instance_id = env::var("LMHA_INSTANCE_ID").unwrap_or_else(|_| {
            let id = Uuid::new_v4().to_string();
            format!("lmha3-{}", &id[..8])
        });
        let instance_priority = env::var("LMHA_INSTANCE_PRIORITY")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let instance_topic_prefix = env::var("LMHA_INSTANCE_TOPIC_PREFIX")
            .unwrap_or_else(|_| "lmha3/instances/".to_string());

        Self {
            database_url: env::var("LMHA_DATABASE_URL").expect("LMHA_DATABASE_URL must be set"),
            mqtt_host: env::var("LMHA_MQTT_HOST").unwrap_or_else(|_| "localhost".to_string()),
            mqtt_port: env::var("LMHA_MQTT_PORT")
                .unwrap_or_else(|_| "1883".to_string())
                .parse()
                .expect("LMHA_MQTT_PORT must be a number"),
            mqtt_user: env::var("LMHA_MQTT_USER").ok(),
            mqtt_password: env::var("LMHA_MQTT_PASSWORD").ok(),
            instance_id,
            instance_priority,
            instance_topic_prefix,
        }
    }
}

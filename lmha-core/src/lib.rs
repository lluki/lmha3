pub mod config;
pub mod db;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|p| p.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceState {
    On,
    Off,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub mqtt_topic: String,
    pub name: String,
    pub is_enabled: bool,
    pub current_state: DeviceState,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TelemetrySource {
    PvProduction,
    HouseConsumption,
    DeviceState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub timestamp: DateTime<Utc>,
    pub source: TelemetrySource,
    pub device_id: Option<Uuid>,
    pub value: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

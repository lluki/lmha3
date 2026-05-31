pub mod config;
pub mod db;
pub mod ha;
pub mod scheduler;

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
pub struct House {
    pub id: Uuid,
    pub name: String,
    pub ha_host: String,
    pub ha_token: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub house_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantPublic {
    pub id: Uuid,
    pub house_id: Uuid,
    pub username: String,
    pub is_admin: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum SchedulingType {
    None,
    ForceOff { until: DateTime<Utc> },
    ForceOn { until: DateTime<Utc> },
    Boiler,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub house_id: Uuid,
    pub mqtt_topic: String,
    pub name: String,
    pub is_enabled: bool,
    pub expected_load: i32,
    pub scheduling_type: SchedulingType,
    pub current_state: DeviceState,
    pub desired_state: DeviceState,
    pub last_request_time: Option<DateTime<Utc>>,
    pub last_feedback_time: Option<DateTime<Utc>>,
    pub full_charge_n_day: i32,
    pub min_daily_charge: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TelemetrySource {
    PvProduction,
    HouseConsumption,
    DeviceState,
    DeviceConsumption,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub timestamp: DateTime<Utc>,
    pub source: TelemetrySource,
    pub device_id: Option<Uuid>,
    pub value: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub current_view_house_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub session_id: Uuid,
    pub tenant_id: Uuid,
    pub house_id: Uuid,
    pub username: String,
    pub is_admin: bool,
}

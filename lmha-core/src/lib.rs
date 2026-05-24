pub mod config;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
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

use postgres::{Client, NoTls};
use crate::config::Config;
use crate::{Tenant, Session, Device};
use uuid::Uuid;
use chrono::{Utc, Duration};

pub struct Db {
    client: Client,
}

impl Db {
    pub fn connect(config: &Config) -> Result<Self, postgres::Error> {
        let client = Client::connect(&config.database_url, NoTls)?;
        Ok(Self { client })
    }

    pub fn get_tenant_by_username(&mut self, username: &str) -> Option<Tenant> {
        let row = self.client.query_opt(
            "SELECT id, username, password_hash, created_at FROM tenants WHERE username = $1",
            &[&username],
        ).ok()??;

        Some(Tenant {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            created_at: row.get(3),
        })
    }

    pub fn create_session(&mut self, tenant_id: Uuid) -> Result<Uuid, postgres::Error> {
        let expires_at = Utc::now() + Duration::days(1);
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO sessions (id, tenant_id, expires_at) VALUES ($1, $2, $3)",
            &[&id, &tenant_id, &expires_at],
        )?;
        Ok(id)
    }

    pub fn get_session(&mut self, session_id: Uuid) -> Option<Session> {
        let row = self.client.query_opt(
            "SELECT id, tenant_id, expires_at, created_at FROM sessions WHERE id = $1 AND expires_at > NOW()",
            &[&session_id],
        ).ok()??;

        Some(Session {
            id: row.get(0),
            tenant_id: row.get(1),
            expires_at: row.get(2),
            created_at: row.get(3),
        })
    }

    pub fn get_user_info(&mut self, session_id: Uuid) -> Option<crate::UserInfo> {
        let row = self.client.query_opt(
            "SELECT s.id, s.tenant_id, t.username FROM sessions s JOIN tenants t ON s.tenant_id = t.id WHERE s.id = $1 AND s.expires_at > NOW()",
            &[&session_id],
        ).ok()??;

        let username: String = row.get(2);
        Some(crate::UserInfo {
            session_id: row.get(0),
            tenant_id: row.get(1),
            is_admin: username == "admin",
            username,
        })
    }

    pub fn create_tenant(&mut self, username: &str, password_hash: &str) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO tenants (id, username, password_hash) VALUES ($1, $2, $3)",
            &[&id, &username, &password_hash],
        )?;
        Ok(id)
    }

    pub fn create_device(&mut self, tenant_id: Uuid, mqtt_topic: &str, name: &str) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO devices (id, tenant_id, mqtt_topic, name) VALUES ($1, $2, $3, $4)",
            &[&id, &tenant_id, &mqtt_topic, &name],
        )?;
        Ok(id)
    }

    pub fn list_tenants(&mut self) -> Result<Vec<Tenant>, postgres::Error> {
        let rows = self.client.query("SELECT id, username, password_hash, created_at FROM tenants", &[])?;
        Ok(rows.into_iter().map(|row| Tenant {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            created_at: row.get(3),
        }).collect())
    }

    pub fn list_devices(&mut self) -> Result<Vec<Device>, postgres::Error> {
        let rows = self.client.query("SELECT id, tenant_id, mqtt_topic, name, is_enabled, expected_load, current_state::TEXT, last_heartbeat FROM devices", &[])?;
        Ok(rows.into_iter().map(|row| Device {
            id: row.get(0),
            tenant_id: row.get(1),
            mqtt_topic: row.get(2),
            name: row.get(3),
            is_enabled: row.get(4),
            expected_load: row.get(5),
            current_state: match row.get::<_, &str>(6) {
                "ON" => crate::DeviceState::On,
                "OFF" => crate::DeviceState::Off,
                _ => crate::DeviceState::Unknown,
            },
            last_heartbeat: row.get(7),
        }).collect())
    }


    pub fn update_device_heartbeat(&mut self, mqtt_topic: &str) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET last_heartbeat = NOW() WHERE mqtt_topic = $1",
            &[&mqtt_topic],
        )?;
        Ok(())
    }

    pub fn update_device_state(&mut self, mqtt_topic: &str, state: crate::DeviceState) -> Result<(), postgres::Error> {
        let state_str = match state {
            crate::DeviceState::On => "ON",
            crate::DeviceState::Off => "OFF",
            crate::DeviceState::Unknown => "UNKNOWN",
        };
        self.client.execute(
            "UPDATE devices SET current_state = $1::text::device_state WHERE mqtt_topic = $2",
            &[&state_str, &mqtt_topic],
        )?;
        Ok(())
    }

    pub fn insert_telemetry(&mut self, source: crate::TelemetrySource, device_id: Option<Uuid>, value: f64, metadata: Option<serde_json::Value>) -> Result<(), postgres::Error> {
        let source_str = match source {
            crate::TelemetrySource::PvProduction => "PV_PRODUCTION",
            crate::TelemetrySource::HouseConsumption => "HOUSE_CONSUMPTION",
            crate::TelemetrySource::DeviceState => "DEVICE_STATE",
            crate::TelemetrySource::DeviceConsumption => "DEVICE_CONSUMPTION",
        };
        self.client.execute(
            "INSERT INTO telemetry (source, device_id, value, metadata) VALUES ($1::text::telemetry_source, $2, $3, $4)",
            &[&source_str, &device_id, &value, &metadata],
        )?;
        Ok(())
    }

    pub fn get_latest_metrics(&mut self) -> Result<(f64, f64), postgres::Error> {
        let rows = self.client.query(
            "SELECT 
                (SELECT value FROM telemetry WHERE source = 'PV_PRODUCTION'::telemetry_source ORDER BY timestamp DESC LIMIT 1) as pv,
                (SELECT value FROM telemetry WHERE source = 'HOUSE_CONSUMPTION'::telemetry_source ORDER BY timestamp DESC LIMIT 1) as cons",
            &[],
        )?;
        let row = &rows[0];
        Ok((row.get::<_, Option<f64>>(0).unwrap_or(0.0), row.get::<_, Option<f64>>(1).unwrap_or(0.0)))
    }

    pub fn list_telemetry(&mut self, tenant_id: Option<Uuid>, limit: i64) -> Result<Vec<crate::Telemetry>, postgres::Error> {
        let rows = if let Some(tid) = tenant_id {
            self.client.query(
                "SELECT timestamp, source::TEXT, device_id, value, metadata FROM telemetry 
                 WHERE device_id IS NULL OR device_id IN (SELECT id FROM devices WHERE tenant_id = $1)
                 ORDER BY timestamp DESC LIMIT $2",
                &[&tid, &limit],
            )?
        } else {
            self.client.query(
                "SELECT timestamp, source::TEXT, device_id, value, metadata FROM telemetry 
                 ORDER BY timestamp DESC LIMIT $1",
                &[&limit],
            )?
        };

        Ok(rows.into_iter().map(|row| crate::Telemetry {
            timestamp: row.get(0),
            source: match row.get::<_, &str>(1) {
                "PV_PRODUCTION" => crate::TelemetrySource::PvProduction,
                "HOUSE_CONSUMPTION" => crate::TelemetrySource::HouseConsumption,
                "DEVICE_STATE" => crate::TelemetrySource::DeviceState,
                "DEVICE_CONSUMPTION" => crate::TelemetrySource::DeviceConsumption,
                _ => unreachable!(),
            },
            device_id: row.get(2),
            value: row.get(3),
            metadata: row.get(4),
        }).collect())
    }

    pub fn delete_session(&mut self, session_id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute("DELETE FROM sessions WHERE id = $1", &[&session_id])?;
        Ok(())
    }
}

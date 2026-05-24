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

    pub fn create_tenant(&mut self, username: &str, password_hash: &str) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO tenants (id, username, password_hash) VALUES ($1, $2, $3)",
            &[&id, &username, &password_hash],
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
        let rows = self.client.query("SELECT id, tenant_id, mqtt_topic, name, is_enabled, current_state::TEXT, last_heartbeat FROM devices", &[])?;
        Ok(rows.into_iter().map(|row| Device {
            id: row.get(0),
            tenant_id: row.get(1),
            mqtt_topic: row.get(2),
            name: row.get(3),
            is_enabled: row.get(4),
            current_state: match row.get::<_, &str>(5) {
                "ON" => crate::DeviceState::On,
                "OFF" => crate::DeviceState::Off,
                _ => crate::DeviceState::Unknown,
            },
            last_heartbeat: row.get(6),
        }).collect())
    }


    pub fn update_device_heartbeat(&mut self, mqtt_topic: &str) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET last_heartbeat = NOW() WHERE mqtt_topic = $1",
            &[&mqtt_topic],
        )?;
        Ok(())
    }

    pub fn delete_session(&mut self, session_id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute("DELETE FROM sessions WHERE id = $1", &[&session_id])?;
        Ok(())
    }
}

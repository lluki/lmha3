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

    pub fn run_migrations(&mut self, migrations_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client.batch_execute("CREATE TABLE IF NOT EXISTS _migrations (name TEXT PRIMARY KEY, applied_at TIMESTAMPTZ DEFAULT NOW())")?;

        let mut migration_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(migrations_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    migration_files.push(path);
                }
            }
        } else {
            return Err(format!("Could not read migrations directory: {}", migrations_dir).into());
        }
        migration_files.sort();

        for path in migration_files {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let already_applied: bool = self.client.query_one("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = $1)", &[&name])?.get(0);
            
            if !already_applied {
                let content = std::fs::read_to_string(&path)?;
                self.client.batch_execute(&content)?;
                self.client.execute("INSERT INTO _migrations (name) VALUES ($1)", &[&name])?;
            }
        }
        Ok(())
    }

    pub fn ensure_seeded(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let house_count: i64 = self.client.query_one("SELECT COUNT(*) FROM houses", &[])?.get(0);
        
        let house_id = if house_count == 0 {
            println!("Seeding default house...");
            self.create_house(
                "Default House",
                "http://localhost:8123",
                "placeholder_token",
                "sensor.pv_power",
                "sensor.total_consumption"
            )?
        } else {
            self.client.query_one("SELECT id FROM houses LIMIT 1", &[])?.get(0)
        };

        let tenant_count: i64 = self.client.query_one("SELECT COUNT(*) FROM tenants", &[])?.get(0);
        if tenant_count == 0 {
            println!("Seeding default admin user...");
            let password_hash = crate::hash_password("admin")?;
            self.create_tenant("admin", &password_hash, house_id, true)?;
        }

        Ok(())
    }

    pub fn get_tenant_by_username(&mut self, username: &str) -> Option<Tenant> {
        let row = self.client.query_opt(
            "SELECT id, username, password_hash, created_at, house_id, is_admin FROM tenants WHERE username = $1",
            &[&username],
        ).ok()??;

        Some(Tenant {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            created_at: row.get(3),
            house_id: row.get(4),
            is_admin: row.get(5),
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
            "SELECT id, tenant_id, expires_at, created_at, current_view_house_id FROM sessions WHERE id = $1 AND expires_at > NOW()",
            &[&session_id],
        ).ok()??;

        Some(Session {
            id: row.get(0),
            tenant_id: row.get(1),
            expires_at: row.get(2),
            created_at: row.get(3),
            current_view_house_id: row.get(4),
        })
    }

    pub fn get_user_info(&mut self, session_id: Uuid) -> Option<crate::UserInfo> {
        let row = self.client.query_opt(
            "SELECT s.id, s.tenant_id, t.username, t.house_id, s.current_view_house_id, t.is_admin FROM sessions s JOIN tenants t ON s.tenant_id = t.id WHERE s.id = $1 AND s.expires_at > NOW()",
            &[&session_id],
        ).ok()??;

        let is_admin: bool = row.get(5);
        let house_id: Uuid = row.get(3);
        let current_view_house_id: Option<Uuid> = row.get(4);

        Some(crate::UserInfo {
            session_id: row.get(0),
            tenant_id: row.get(1),
            house_id: if is_admin { current_view_house_id.unwrap_or(house_id) } else { house_id },
            is_admin,
            username: row.get(2),
        })
    }

    pub fn create_tenant(&mut self, username: &str, password_hash: &str, house_id: Uuid, is_admin: bool) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO tenants (id, username, password_hash, house_id, is_admin) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &username, &password_hash, &house_id, &is_admin],
        )?;
        Ok(id)
    }

    pub fn create_device(&mut self, tenant_id: Uuid, mqtt_topic: &str, name: &str, house_id: Uuid) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO devices (id, tenant_id, mqtt_topic, name, house_id) VALUES ($1, $2, $3, $4, $5)",
            &[&id, &tenant_id, &mqtt_topic, &name, &house_id],
        )?;
        Ok(id)
    }

    pub fn list_tenants(&mut self) -> Result<Vec<Tenant>, postgres::Error> {
        let rows = self.client.query("SELECT id, username, password_hash, created_at, house_id, is_admin FROM tenants", &[])?;
        Ok(rows.into_iter().map(|row| Tenant {
            id: row.get(0),
            username: row.get(1),
            password_hash: row.get(2),
            created_at: row.get(3),
            house_id: row.get(4),
            is_admin: row.get(5),
        }).collect())
    }

    pub fn list_devices(&mut self, house_id: Option<Uuid>) -> Result<Vec<Device>, postgres::Error> {
            let (query, params): (&str, &[&(dyn postgres::types::ToSql + Sync)]) = if let Some(hid) = &house_id {
                ("SELECT id, tenant_id, mqtt_topic, name, is_enabled, expected_load, current_state::TEXT, desired_state::TEXT, last_request_time, last_feedback_time, scheduling_type, scheduling_until, device_runtime, house_id FROM devices WHERE house_id = $1", &[hid])
            } else {
                ("SELECT id, tenant_id, mqtt_topic, name, is_enabled, expected_load, current_state::TEXT, desired_state::TEXT, last_request_time, last_feedback_time, scheduling_type, scheduling_until, device_runtime, house_id FROM devices", &[])
            };

            let rows = self.client.query(query, params)?;
            Ok(rows.into_iter().map(|row| {
                let s_type: String = row.get(10);
                let s_until: Option<chrono::DateTime<chrono::Utc>> = row.get(11);

                let scheduling_type = match s_type.as_str() {                "none" => crate::SchedulingType::None,
                "force-on" => crate::SchedulingType::ForceOn { until: s_until.unwrap_or_else(Utc::now) },
                "force-off" => crate::SchedulingType::ForceOff { until: s_until.unwrap_or_else(Utc::now) },
                _ => crate::SchedulingType::Boiler,
            };

            Device {
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
                desired_state: match row.get::<_, &str>(7) {
                    "ON" => crate::DeviceState::On,
                    "OFF" => crate::DeviceState::Off,
                    _ => crate::DeviceState::Unknown,
                },
                last_request_time: row.get(8),
                last_feedback_time: row.get(9),
                scheduling_type,
                device_runtime: row.get(12),
                house_id: row.get(13),
            }
        }).collect())
    }

    pub fn list_houses(&mut self) -> Result<Vec<crate::House>, postgres::Error> {
        let rows = self.client.query("SELECT id, name, ha_url, ha_token, ha_pv_entity_id, ha_consumption_entity_id, created_at, day_deadline FROM houses ORDER BY name ASC", &[])?;
        Ok(rows.into_iter().map(|row| crate::House {
            id: row.get(0),
            name: row.get(1),
            ha_url: row.get(2),
            ha_token: row.get(3),
            ha_pv_entity_id: row.get(4),
            ha_consumption_entity_id: row.get(5),
            created_at: row.get(6),
            day_deadline: row.get(7),
        }).collect())
    }

    pub fn get_house(&mut self, id: Uuid) -> Result<Option<crate::House>, postgres::Error> {
        let row = self.client.query_opt("SELECT id, name, ha_url, ha_token, ha_pv_entity_id, ha_consumption_entity_id, created_at, day_deadline FROM houses WHERE id = $1", &[&id])?;
        Ok(row.map(|row| crate::House {
            id: row.get(0),
            name: row.get(1),
            ha_url: row.get(2),
            ha_token: row.get(3),
            ha_pv_entity_id: row.get(4),
            ha_consumption_entity_id: row.get(5),
            created_at: row.get(6),
            day_deadline: row.get(7),
        }))
    }

    pub fn create_house(&mut self, name: &str, ha_url: &str, ha_token: &str, ha_pv_entity_id: &str, ha_consumption_entity_id: &str) -> Result<Uuid, postgres::Error> {
        let id = Uuid::new_v4();
        self.client.execute(
            "INSERT INTO houses (id, name, ha_url, ha_token, ha_pv_entity_id, ha_consumption_entity_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&id, &name, &ha_url, &ha_token, &ha_pv_entity_id, &ha_consumption_entity_id],
        )?;
        Ok(id)
    }

    pub fn delete_house(&mut self, id: Uuid) -> Result<(), String> {
        // Safety checks: check if any tenants or devices are associated with this house
        let tenants_count: i64 = self.client.query_one("SELECT COUNT(*) FROM tenants WHERE house_id = $1", &[&id]).map_err(|e| e.to_string())?.get(0);
        if tenants_count > 0 {
            return Err("Cannot delete house with associated tenants".to_string());
        }
        let devices_count: i64 = self.client.query_one("SELECT COUNT(*) FROM devices WHERE house_id = $1", &[&id]).map_err(|e| e.to_string())?.get(0);
        if devices_count > 0 {
            return Err("Cannot delete house with associated devices".to_string());
        }

        // Delete telemetry first (or cascade)
        self.client.execute("DELETE FROM telemetry WHERE house_id = $1", &[&id]).map_err(|e| e.to_string())?;
        self.client.execute("DELETE FROM sessions WHERE current_view_house_id = $1", &[&id]).map_err(|e| e.to_string())?;
        self.client.execute("DELETE FROM houses WHERE id = $1", &[&id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_session_view_house(&mut self, session_id: Uuid, house_id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute("UPDATE sessions SET current_view_house_id = $1 WHERE id = $2", &[&house_id, &session_id])?;
        Ok(())
    }


    pub fn update_device_feedback(&mut self, mqtt_topic: &str) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET last_feedback_time = NOW() WHERE mqtt_topic = $1",
            &[&mqtt_topic],
        )?;
        Ok(())
    }

    pub fn update_device_desired_state(&mut self, id: Uuid, state: crate::DeviceState) -> Result<(), postgres::Error> {
        let state_str = match state {
            crate::DeviceState::On => "ON",
            crate::DeviceState::Off => "OFF",
            crate::DeviceState::Unknown => "UNKNOWN",
        };
        self.client.execute(
            "UPDATE devices SET desired_state = $1::text::device_state, last_request_time = NOW() WHERE id = $2",
            &[&state_str, &id],
        )?;
        Ok(())
    }

    pub fn update_device_request_time(&mut self, id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET last_request_time = NOW() WHERE id = $1",
            &[&id],
        )?;
        Ok(())
    }

    pub fn get_out_of_sync_devices(&mut self) -> Result<Vec<Device>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, tenant_id, mqtt_topic, name, is_enabled, expected_load, current_state::TEXT, desired_state::TEXT, last_request_time, last_feedback_time, scheduling_type, scheduling_until, device_runtime, house_id 
             FROM devices 
             WHERE desired_state != current_state AND is_enabled = TRUE",
            &[],
        )?;
        
        Ok(rows.into_iter().map(|row| {
            let s_type: String = row.get(10);
            let s_until: Option<chrono::DateTime<chrono::Utc>> = row.get(11);

            let scheduling_type = match s_type.as_str() {
                "none" => crate::SchedulingType::None,
                "force-on" => crate::SchedulingType::ForceOn { until: s_until.unwrap_or_else(Utc::now) },
                "force-off" => crate::SchedulingType::ForceOff { until: s_until.unwrap_or_else(Utc::now) },
                _ => crate::SchedulingType::Boiler,
            };

            Device {
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
                desired_state: match row.get::<_, &str>(7) {
                    "ON" => crate::DeviceState::On,
                    "OFF" => crate::DeviceState::Off,
                    _ => crate::DeviceState::Unknown,
                },
                last_request_time: row.get(8),
                last_feedback_time: row.get(9),
                scheduling_type,
                device_runtime: row.get(12),
                house_id: row.get(13),
            }
        }).collect())
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

    pub fn update_device_scheduling(&mut self, id: Uuid, scheduling_type: crate::SchedulingType) -> Result<(), postgres::Error> {
        let (type_str, until) = match scheduling_type {
            crate::SchedulingType::None => ("none", None),
            crate::SchedulingType::ForceOn { until } => ("force-on", Some(until)),
            crate::SchedulingType::ForceOff { until } => ("force-off", Some(until)),
            crate::SchedulingType::Boiler => ("boiler", None),
        };
        self.client.execute(
            "UPDATE devices SET scheduling_type = $1, scheduling_until = $2 WHERE id = $3",
            &[&type_str, &until, &id],
        )?;
        Ok(())
    }

    pub fn update_device_config(&mut self, id: Uuid, expected_load: i32, device_runtime: i32) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET expected_load = $1, device_runtime = $2 WHERE id = $3",
            &[&expected_load, &device_runtime, &id],
        )?;
        Ok(())
    }

    pub fn insert_telemetry(&mut self, source: crate::TelemetrySource, device_id: Option<Uuid>, value: i32, metadata: Option<serde_json::Value>, house_id: Uuid) -> Result<(), postgres::Error> {
        let source_str = match source {
            crate::TelemetrySource::PvProduction => "PV_PRODUCTION",
            crate::TelemetrySource::HouseConsumption => "HOUSE_CONSUMPTION",
            crate::TelemetrySource::DeviceState => "DEVICE_STATE",
            crate::TelemetrySource::DeviceConsumption => "DEVICE_CONSUMPTION",
        };
        self.client.execute(
            "INSERT INTO telemetry (source, device_id, value, metadata, house_id) VALUES ($1::text::telemetry_source, $2, $3, $4, $5)",
            &[&source_str, &device_id, &(value as f64), &metadata, &house_id],
        )?;
        Ok(())
    }

    pub fn get_latest_metrics(&mut self, house_id: Uuid) -> Result<(i32, i32), postgres::Error> {
        let rows = self.client.query(
            "SELECT 
                (SELECT value FROM telemetry WHERE source = 'PV_PRODUCTION'::telemetry_source AND house_id = $1 ORDER BY timestamp DESC LIMIT 1) as pv,
                (SELECT value FROM telemetry WHERE source = 'HOUSE_CONSUMPTION'::telemetry_source AND house_id = $1 ORDER BY timestamp DESC LIMIT 1) as cons",
            &[&house_id],
        )?;
        let row = &rows[0];
        Ok((
            row.get::<_, Option<f64>>(0).unwrap_or(0.0) as i32,
            row.get::<_, Option<f64>>(1).unwrap_or(0.0) as i32
        ))
    }

    pub fn list_telemetry(&mut self, house_id: Uuid, limit: i64, events_only: bool) -> Result<Vec<crate::Telemetry>, postgres::Error> {
        let query = if events_only {
            "SELECT timestamp, source::TEXT, device_id, value, metadata FROM telemetry 
             WHERE house_id = $1 AND source = 'DEVICE_STATE'::telemetry_source
             ORDER BY timestamp DESC LIMIT $2"
        } else {
            "SELECT timestamp, source::TEXT, device_id, value, metadata FROM telemetry 
             WHERE house_id = $1
             ORDER BY timestamp DESC LIMIT $2"
        };

        let rows = self.client.query(query, &[&house_id, &limit])?;

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
            value: row.get::<_, f64>(3) as i32,
            metadata: row.get(4),
        }).collect())
    }

    pub fn get_device_history(&mut self, device_id: Uuid, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::scheduler::StateEvent>, postgres::Error> {
        let rows = self.client.query(
            "SELECT timestamp, value FROM telemetry 
             WHERE source = 'DEVICE_STATE'::telemetry_source AND device_id = $1 AND timestamp >= $2
             ORDER BY timestamp ASC",
            &[&device_id, &since],
        )?;

        Ok(rows.into_iter().map(|row| crate::scheduler::StateEvent {
            timestamp: row.get(0),
            state: if row.get::<_, f64>(1) > 0.5 { crate::DeviceState::On } else { crate::DeviceState::Off },
        }).collect())
    }

    pub fn calc_boiler_runtime_24h(&mut self, device_id: Uuid, day_deadline: chrono::NaiveTime) -> Result<i32, postgres::Error> {
        use chrono::Timelike;
        let now = Utc::now();
        // We need to calculate start of day in local time then convert to UTC
        // But here we are in Db which doesn't know about Local...
        // Actually, we can use the logic from scheduler.rs if we move it to a helper.
        // For now, let's just do a simple version or wait until we have the helper.
        
        // Let's use the local time on the server for now as discussed in design.md
        let now_local = now.with_timezone(&chrono::Local);
        let mut start_of_day_local = now_local.with_hour(day_deadline.hour()).unwrap()
            .with_minute(day_deadline.minute()).unwrap()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();
        
        if now_local.time() < day_deadline {
            start_of_day_local -= Duration::days(1);
        }
        let start_of_day = start_of_day_local.with_timezone(&Utc);

        let history = self.get_device_history(device_id, start_of_day)?;
        let mut total_minutes = 0;
        let mut last_on_time: Option<chrono::DateTime<Utc>> = None;

        for event in history {
            match (event.state, last_on_time) {
                (crate::DeviceState::On, None) => {
                    last_on_time = Some(event.timestamp);
                }
                (crate::DeviceState::Off, Some(on_time)) => {
                    let duration = event.timestamp - on_time;
                    total_minutes += duration.num_minutes() as i32;
                    last_on_time = None;
                }
                _ => {}
            }
        }

        if let Some(on_time) = last_on_time {
            let duration = now - on_time;
            total_minutes += duration.num_minutes() as i32;
        }

        Ok(total_minutes)
    }

    pub fn delete_tenant(&mut self, id: Uuid) -> Result<(), String> {
        let tenant_row = self.client.query_one("SELECT username FROM tenants WHERE id = $1", &[&id]).map_err(|e| e.to_string())?;
        let username: String = tenant_row.get(0);
        if username == "admin" {
            return Err("Cannot delete the system administrator account".to_string());
        }

        let rows = self.client.query("SELECT COUNT(*) FROM devices WHERE tenant_id = $1", &[&id]).map_err(|e| e.to_string())?;
        let count: i64 = rows[0].get(0);
        if count > 0 {
            return Err("Cannot delete tenant with active devices".to_string());
        }

        self.client.execute("DELETE FROM sessions WHERE tenant_id = $1", &[&id]).map_err(|e| e.to_string())?;
        self.client.execute("DELETE FROM tenants WHERE id = $1", &[&id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_house(&mut self, id: Uuid, name: &str, ha_url: &str, ha_token: &str, ha_pv_entity_id: &str, ha_consumption_entity_id: &str, day_deadline: chrono::NaiveTime) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE houses SET name = $1, ha_url = $2, ha_token = $3, ha_pv_entity_id = $4, ha_consumption_entity_id = $5, day_deadline = $6 WHERE id = $7",
            &[&name, &ha_url, &ha_token, &ha_pv_entity_id, &ha_consumption_entity_id, &day_deadline, &id],
        )?;
        Ok(())
    }

    pub fn update_tenant_admin(&mut self, id: Uuid, username: &str, house_id: Uuid, is_admin: bool) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE tenants SET username = $1, house_id = $2, is_admin = $3 WHERE id = $4",
            &[&username, &house_id, &is_admin, &id],
        )?;
        Ok(())
    }

    pub fn update_tenant_password_admin(&mut self, id: Uuid, password_hash: &str) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE tenants SET password_hash = $1 WHERE id = $2",
            &[&password_hash, &id],
        )?;
        Ok(())
    }

    pub fn delete_device(&mut self, id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute("DELETE FROM telemetry WHERE device_id = $1", &[&id])?;
        self.client.execute("DELETE FROM devices WHERE id = $1", &[&id])?;
        Ok(())
    }

    pub fn update_device_config_admin(&mut self, id: Uuid, name: &str, mqtt_topic: &str, tenant_id: Uuid, expected_load: i32, device_runtime: i32) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE devices SET name = $1, mqtt_topic = $2, tenant_id = $3, expected_load = $4, device_runtime = $5 WHERE id = $6",
            &[&name, &mqtt_topic, &tenant_id, &expected_load, &device_runtime, &id],
        )?;
        Ok(())
    }

    pub fn delete_session(&mut self, session_id: Uuid) -> Result<(), postgres::Error> {
        self.client.execute("DELETE FROM sessions WHERE id = $1", &[&session_id])?;
        Ok(())
    }
}

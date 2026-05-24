use lmha_core::config::Config;
use lmha_core::hash_password;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use ureq;
use uuid::Uuid;
use postgres::{Client, NoTls};

pub struct TestHarness {
    pub db_name: String,
    pub api_child: Child,
    pub config: Config,
    pub port: u16,
}

impl TestHarness {
    pub fn new(port: u16, no_scheduler: bool) -> Self {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        let migrations = [
            include_str!("../../../migrations/001_initial_schema.sql"),
            include_str!("../../../migrations/002_add_sessions.sql"),
            include_str!("../../../migrations/003_add_device_heartbeat.sql"),
        ];
        
        let db_url = format!("host=/var/run/postgresql dbname={} user=lukas", db_name);
        let mut db_client = Client::connect(&db_url, NoTls).unwrap();
        for migration in migrations {
            db_client.batch_execute(migration).unwrap();
        }

        let config = Config {
            database_url: db_url.clone(),
            mqtt_host: "solar.lluki.me".to_string(),
            mqtt_port: 1884,
            mqtt_user: Some("admin".to_string()),
            mqtt_password: Some("freebird".to_string()),
            ha_url: "http://localhost:8123".to_string(),
            ha_token: "test_token".to_string(),
        };

        let mut cmd = Command::new("/home/lukas/dev/lmha3/target/debug/server");
        cmd.arg("--port").arg(port.to_string());
        if no_scheduler {
            cmd.arg("--no-scheduler");
        }
        cmd.env("DATABASE_URL", &config.database_url)
           .env("HA_TOKEN", &config.ha_token)
           .env("MQTT_USER", config.mqtt_user.as_ref().unwrap())
           .env("MQTT_PASSWORD", config.mqtt_password.as_ref().unwrap());

        let api_child = cmd.spawn().expect("Failed to start Server");

        let agent = ureq::AgentBuilder::new().build();
        let mut success = false;
        for _ in 0..100 {
            if let Ok(resp) = agent.get(&format!("http://localhost:{}/login", port)).call() {
                if resp.status() == 200 {
                    success = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        if !success { panic!("Server failed to start on port {}", port); }

        Self { db_name, api_child, config, port }
    }

    pub fn create_user(&self, username: &str, password: &str) -> Uuid {
        let mut client = Client::connect(&self.config.database_url, NoTls).unwrap();
        let hashed = hash_password(password).unwrap();
        let id = Uuid::new_v4();
        client.execute("INSERT INTO tenants (id, username, password_hash) VALUES ($1, $2, $3)", &[&id, &username, &hashed]).unwrap();
        id
    }

    pub fn create_device(&self, tenant_id: Uuid, name: &str, topic: &str) -> Uuid {
        let mut client = Client::connect(&self.config.database_url, NoTls).unwrap();
        let id = Uuid::new_v4();
        client.execute("INSERT INTO devices (id, tenant_id, name, mqtt_topic) VALUES ($1, $2, $3, $4)", &[&id, &tenant_id, &name, &topic]).unwrap();
        id
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = self.api_child.kill();
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("DROP DATABASE IF EXISTS {}", self.db_name), &[]).ok();
    }
}

use lmha_core::config::Config;
use lmha_core::hash_password;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use ureq;
use uuid::Uuid;
use postgres::{Client, NoTls};

#[allow(dead_code)]
pub struct TestHarness {
    pub db_name: String,
    pub api_child: Child,
    pub config: Config,
    pub port: u16,
}

impl TestHarness {
    pub fn new(port: u16, no_scheduler: bool) -> Self {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        
        // 1. Create temporary DB
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=user", NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        // 2. Run migrations
        let db_url = format!("host=/var/run/postgresql dbname={} user=user", db_name);
        let migrations_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
        
        let config = Config {
            database_url: db_url.clone(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_user: Some("admin".to_string()),
            mqtt_password: Some("test_password".to_string()),
            instance_id: format!("test-{}", Uuid::new_v4().simple()),
            instance_priority: 10,
        };

        let mut db = lmha_core::db::Db::connect(&config).unwrap();
        db.run_migrations(migrations_path.to_str().unwrap()).unwrap();

        // 3. Start Server
        let current_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let nix_bin = current_dir.join("result/bin/server");
        let debug_bin = current_dir.join("target/debug/server");
        
        let binary_path = if nix_bin.exists() {
            nix_bin.canonicalize().unwrap().to_str().unwrap().to_string()
        } else if debug_bin.exists() {
            debug_bin.canonicalize().unwrap().to_str().unwrap().to_string()
        } else {
            "server".to_string()
        };

        println!("TestHarness: Using binary at {}", binary_path);
        let mut cmd = Command::new(binary_path);
        cmd.arg("--port").arg(port.to_string());
        if no_scheduler {
            cmd.arg("--no-scheduler");
        }
        cmd.env("LMHA_DATABASE_URL", &config.database_url)
           .env("LMHA_MQTT_USER", config.mqtt_user.as_ref().unwrap())
           .env("LMHA_MQTT_PASSWORD", config.mqtt_password.as_ref().unwrap())
           .env("LMHA_MIGRATIONS_DIR", migrations_path);
        let api_child = cmd.spawn().expect("Failed to start Server");

        // 4. Wait for server to be ready
        let agent = ureq::AgentBuilder::new().build();
        let mut success = false;
        for _ in 0..100 {
            if let Ok(resp) = agent.get(&format!("http://localhost:{}", port)).call() {
                if resp.status() == 200 {
                    success = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        if !success { panic!("Server failed to start on port {}", port); }

        // Give the server a moment to connect to MQTT and subscribe
        thread::sleep(Duration::from_secs(1));

        Self { db_name, api_child, config, port }
    }

    pub fn create_user(&self, username: &str, password: &str) -> Uuid {
        println!("TestHarness: Creating user {} in DB {}", username, self.db_name);
        let mut client = Client::connect(&self.config.database_url, NoTls).unwrap();
        let house_id: Uuid = client.query_one("SELECT id FROM houses LIMIT 1", &[]).unwrap().get(0);
        let hashed = hash_password(password).unwrap();
        let id = Uuid::new_v4();
        let is_admin = username == "admin";
        client.execute("INSERT INTO tenants (id, username, password_hash, house_id, is_admin) VALUES ($1, $2, $3, $4, $5)", &[&id, &username, &hashed, &house_id, &is_admin]).unwrap();
        id
    }

    #[allow(dead_code)]
    pub fn create_device(&self, tenant_id: Uuid, name: &str, topic: &str) -> Uuid {
        let mut client = Client::connect(&self.config.database_url, NoTls).unwrap();
        let house_id: Uuid = client.query_one("SELECT house_id FROM tenants WHERE id = $1", &[&tenant_id]).unwrap().get(0);
        let id = Uuid::new_v4();
        client.execute("INSERT INTO devices (id, tenant_id, name, mqtt_topic, house_id) VALUES ($1, $2, $3, $4, $5)", &[&id, &tenant_id, &name, &topic, &house_id]).unwrap();
        id
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = self.api_child.kill();
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=user", NoTls).unwrap();
        client.execute(&format!("DROP DATABASE IF EXISTS {}", self.db_name), &[]).ok();
    }
}

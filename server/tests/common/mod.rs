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
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        // 2. Run migrations
        let db_url = format!("host=/var/run/postgresql dbname={} user=lukas", db_name);
        let mut db_client = Client::connect(&db_url, NoTls).unwrap();

        let mut migration_files = Vec::new();
        let migrations_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
        if let Ok(entries) = std::fs::read_dir(&migrations_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    migration_files.push(path);
                }
            }
        } else {
            panic!("Could not read migrations directory: {:?}", migrations_path);
        }
        migration_files.sort();

        for path in migration_files {
            let content = std::fs::read_to_string(&path).unwrap();
            db_client.batch_execute(&content).unwrap();
        }

        let config = Config {
            database_url: db_url.clone(),
            mqtt_host: "solar.lluki.me".to_string(),
            mqtt_port: 1884,
            mqtt_user: Some("admin".to_string()),
            mqtt_password: Some("freebird".to_string()),
            instance_id: format!("test-{}", Uuid::new_v4().simple()),
            instance_priority: 10,
        };

        // 3. Start Server
        let current_dir = std::path::PathBuf::from("/home/lukas/dev/lmha3");
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
        cmd.env("DATABASE_URL", &config.database_url)
           .env("MQTT_USER", config.mqtt_user.as_ref().unwrap())
           .env("MQTT_PASSWORD", config.mqtt_password.as_ref().unwrap())
           .env("LMHA3_MIGRATIONS_DIR", migrations_path);

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
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("DROP DATABASE IF EXISTS {}", self.db_name), &[]).ok();
    }
}

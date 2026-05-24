use lmha_core::config::Config;
use lmha_core::db::Db;
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
}

impl TestHarness {
    pub fn new() -> Self {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        
        // 1. Create temporary DB
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        // 2. Run migrations
        let migrations = [
            include_str!("../../migrations/001_initial_schema.sql"),
            include_str!("../../migrations/002_add_sessions.sql"),
        ];
        
        let mut db_client = Client::connect(&format!("host=/var/run/postgresql dbname={} user=lukas", db_name), NoTls).unwrap();
        for migration in migrations {
            db_client.batch_execute(migration).unwrap();
        }

        // 3. Prepare config
        let config = Config {
            database_url: format!("host=/var/run/postgresql dbname={} user=lukas", db_name),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1884,
            ha_url: "http://localhost:8123".to_string(),
            ha_token: "test_token".to_string(),
        };

        // 4. Start API (ensure it's built first)
        let api_child = Command::new("/home/lukas/dev/lmha3/target/debug/api")
            .env("DATABASE_URL", &config.database_url)
            .env("HA_TOKEN", &config.ha_token)
            .spawn()
            .expect("Failed to start API");

        thread::sleep(Duration::from_millis(500)); // Should be fast since it's already built

        Self { db_name, api_child, config }
    }

    pub fn create_user(&self, username: &str, password: &str) -> Uuid {
        let mut db = Db::connect(&self.config).unwrap();
        let hashed = hash_password(password).unwrap();
        db.create_tenant(username, &hashed).unwrap()
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = self.api_child.kill();
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("DROP DATABASE IF EXISTS {}", self.db_name), &[]).ok();
    }
}

#[test]
fn test_full_auth_cycle() {
    let harness = TestHarness::new();
    let tenant_id = harness.create_user("alice", "password123");
    let agent = ureq::AgentBuilder::new().build();

    // Login
    let login_resp = agent.post("http://localhost:8000/login")
        .send_form(&[("username", "alice"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Access Dashboard
    let dash_resp = agent.get("http://localhost:8000/")
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert!(dash_resp.into_string().unwrap().contains(&tenant_id.to_string()));

    // Logout
    let logout_resp = agent.post("http://localhost:8000/logout")
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert!(logout_resp.get_url().contains("/login"));
}

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
    pub fn new() -> Self {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        let port = 8003; 
        
        let mut client = Client::connect("host=/var/run/postgresql dbname=postgres user=lukas", NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        let migrations = [
            include_str!("../../migrations/001_initial_schema.sql"),
            include_str!("../../migrations/002_add_sessions.sql"),
        ];
        
        let db_url = format!("host=/var/run/postgresql dbname={} user=lukas", db_name);
        let mut db_client = Client::connect(&db_url, NoTls).unwrap();
        for migration in migrations {
            db_client.batch_execute(migration).unwrap();
        }

        let config = Config {
            database_url: db_url.clone(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1884,
            ha_url: "http://localhost:8123".to_string(),
            ha_token: "test_token".to_string(),
        };

        let hashed = hash_password("password123").unwrap();
        db_client.execute("INSERT INTO tenants (username, password_hash) VALUES ('alice', $1)", &[&hashed]).unwrap();

        let api_child = Command::new("/home/lukas/dev/lmha3/target/debug/server")
            .arg("--no-scheduler")
            .arg("--port")
            .arg(port.to_string())
            .env("DATABASE_URL", &config.database_url)
            .env("HA_TOKEN", &config.ha_token)
            .spawn()
            .expect("Failed to start Server");

        let agent = ureq::AgentBuilder::new().build();
        let mut success = false;
        for _ in 0..50 {
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
    let agent = ureq::AgentBuilder::new()
        .redirects(0) // Don't follow redirects to capture the cookie
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // Login
    let login_resp = agent.post(&format!("{}/login", base_url))
        .send_form(&[("username", "alice"), ("password", "password123")]);
    
    let resp = match login_resp {
        Ok(r) => r,
        Err(ureq::Error::Status(303, r)) => r,
        Err(e) => panic!("Login failed: {:?}", e),
    };

    let cookie = resp.header("Set-Cookie").expect("No session cookie set").to_string();

    // Access Dashboard
    let dash_resp = agent.get(&base_url)
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert!(dash_resp.into_string().unwrap().contains("Admin Dashboard"));

    // Logout
    let logout_resp = agent.post(&format!("{}/logout", base_url))
        .set("Cookie", &cookie)
        .call();
    
    let resp = match logout_resp {
        Ok(r) => r,
        Err(ureq::Error::Status(303, r)) => r,
        Err(e) => panic!("Logout failed: {:?}", e),
    };
    assert!(resp.header("Location").unwrap().contains("/login"));
}

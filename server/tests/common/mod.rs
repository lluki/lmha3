use lmha_core::config::Config;
use lmha_core::hash_password;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
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
    pub mosquitto_child: Child,
    pub mosquitto_config_path: PathBuf,
    pub config: Config,
    pub port: u16,
}

impl TestHarness {
    pub fn new(port: u16, no_scheduler: bool) -> Self {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        
        let base_db_url = std::env::var("LMHA_DATABASE_URL").unwrap_or_else(|_| "host=/var/run/postgresql dbname=postgres user=user".to_string());
        
        // 1. Create temporary DB using base URL but connecting to 'postgres' first to create the new one
        let base_params = base_db_url.split_whitespace().collect::<Vec<_>>();
        let mut create_params = Vec::new();
        for param in base_params {
            if param.starts_with("dbname=") {
                create_params.push("dbname=postgres");
            } else {
                create_params.push(param);
            }
        }
        let create_url = create_params.join(" ");

        let mut client = Client::connect(&create_url, NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        // 2. Run migrations using the new DB name
        let mut test_params = Vec::new();
        for param in create_params {
            if param.starts_with("dbname=") {
                test_params.push(format!("dbname={}", db_name));
            } else {
                test_params.push(param.to_string());
            }
        }
        let db_url = test_params.join(" ");
        let migrations_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
        
        // Start Local Mosquitto
        let mqtt_port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let mosquitto_config_path = std::env::temp_dir().join(format!("mosquitto_{}.conf", Uuid::new_v4().simple()));
        let mut config_file = std::fs::File::create(&mosquitto_config_path).unwrap();
        writeln!(config_file, "listener {}", mqtt_port).unwrap();
        writeln!(config_file, "allow_anonymous true").unwrap();
        
        let mosquitto_child = Command::new("mosquitto")
            .arg("-c")
            .arg(&mosquitto_config_path)
            .spawn()
            .expect("Failed to start mosquitto");
        
        // Wait for mosquitto to start
        thread::sleep(Duration::from_millis(500));

        let config = Config {
            database_url: db_url.clone(),
            mqtt_host: "127.0.0.1".to_string(),
            mqtt_port,
            mqtt_user: None,
            mqtt_password: None,
            instance_id: format!("test-{}", Uuid::new_v4().simple()),
            instance_priority: 10,
            instance_topic_prefix: format!("tests/{}/instances/", Uuid::new_v4().simple()),
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
           .env("LMHA_MIGRATIONS_DIR", migrations_path)
           .env("LMHA_INSTANCE_TOPIC_PREFIX", &config.instance_topic_prefix)
           .env("LMHA_MQTT_HOST", &config.mqtt_host)
           .env("LMHA_MQTT_PORT", config.mqtt_port.to_string());
        
        if let Some(user) = &config.mqtt_user {
            cmd.env("LMHA_MQTT_USER", user);
        }
        if let Some(pass) = &config.mqtt_password {
            cmd.env("LMHA_MQTT_PASSWORD", pass);
        }

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

        Self { db_name, api_child, mosquitto_child, mosquitto_config_path, config, port }
    }

    pub fn create_user(&self, username: &str, password: &str) -> Uuid {
        println!("TestHarness: Creating user {} in DB {}", username, self.db_name);
        let mut client = Client::connect(&self.config.database_url, NoTls).unwrap();
        let house_id: Uuid = client.query_one("SELECT id FROM houses LIMIT 1", &[]).unwrap().get(0);
        let hashed = hash_password(password).unwrap();
        let id = Uuid::new_v4();
        let is_admin = username == "admin";
        
        // Use ON CONFLICT to handle cases where the admin user might already exist due to seeding
        let row = client.query_opt(
            "INSERT INTO tenants (id, username, password_hash, house_id, is_admin) 
             VALUES ($1, $2, $3, $4, $5) 
             ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash, is_admin = EXCLUDED.is_admin
             RETURNING id", 
            &[&id, &username, &hashed, &house_id, &is_admin]
        ).unwrap();
        
        row.map(|r| r.get(0)).unwrap_or(id)
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
        let _ = self.mosquitto_child.kill();
        let _ = std::fs::remove_file(&self.mosquitto_config_path);

        let base_db_url = std::env::var("LMHA_DATABASE_URL").unwrap_or_else(|_| "host=/var/run/postgresql dbname=postgres user=user".to_string());
        let base_params = base_db_url.split_whitespace().collect::<Vec<_>>();
        let mut create_params = Vec::new();
        for param in base_params {
            if param.starts_with("dbname=") {
                create_params.push("dbname=postgres");
            } else {
                create_params.push(param);
            }
        }
        let create_url = create_params.join(" ");

        let mut client = Client::connect(&create_url, NoTls).unwrap();
        client.execute(&format!("DROP DATABASE IF EXISTS {}", self.db_name), &[]).ok();
    }
}

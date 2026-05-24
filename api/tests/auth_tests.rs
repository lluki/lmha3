use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::hash_password;
use std::process::Command;
use std::thread;
use std::time::Duration;
use ureq;

#[test]
fn test_login_flow() {
    let config = Config::from_env();
    let mut db = Db::connect(&config).expect("Failed to connect to DB");

    // 1. Setup test user
    let username = format!("testuser_{}", uuid::Uuid::new_v4());
    let password = "testpassword123";
    let hashed = hash_password(password).unwrap();
    let tenant_id = db.create_tenant(&username, &hashed).unwrap();

    // 2. Start API server
    let mut api_process = Command::new("/home/lukas/dev/lmha3/target/debug/api")
        .env("DATABASE_URL", &config.database_url)
        .env("HA_TOKEN", &config.ha_token)
        .spawn()
        .expect("Failed to start API server");

    // Wait for server to start
    thread::sleep(Duration::from_secs(2));

    let agent = ureq::AgentBuilder::new().build();

    // 3. Try to access dashboard without login (should redirect to /login)
    let resp = agent.get("http://localhost:8000/").call().unwrap();
    assert!(resp.get_url().contains("/login"));

    // 4. Perform login
    let login_resp = agent.post("http://localhost:8000/login")
        .send_form(&[
            ("username", &username),
            ("password", password),
        ])
        .unwrap();

    assert_eq!(login_resp.status(), 200);
    let cookie = login_resp.header("Set-Cookie").expect("No session cookie set");

    // 5. Access dashboard with session
    let dash_resp = agent.get("http://localhost:8000/")
        .set("Cookie", cookie)
        .call()
        .unwrap();
    assert_eq!(dash_resp.status(), 200);
    assert!(dash_resp.into_string().unwrap().contains(&tenant_id.to_string()));

    // 6. Logout
    let logout_resp = agent.post("http://localhost:8000/logout")
        .set("Cookie", cookie)
        .call()
        .unwrap();
    assert!(logout_resp.get_url().contains("/login"));

    // Cleanup
    let _ = api_process.kill();
}

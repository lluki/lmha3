mod common;
use common::TestHarness;
use ureq;
use std::time::Duration;

#[test]
fn test_log_access() {
    let harness = TestHarness::new(8001, true);
    let base_url = format!("http://localhost:{}", harness.port);
    let agent = ureq::AgentBuilder::new().build();

    // 1. Unauthenticated access to logs should fail
    let resp = agent.get(&format!("{}/api/logs", base_url)).call();
    assert_eq!(resp.is_err(), true); // Expect error (401 handled as error by ureq)
    if let Err(ureq::Error::Status(code, _)) = resp {
        assert_eq!(code, 401);
    }

    // 2. Authenticated access to logs should succeed
    let user_id = harness.create_user("alice", "password123");
    
    // Login
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!("username=alice&password=password123"))
        .expect("Login failed");
    
    let cookie = login_resp.header("Set-Cookie").expect("No session cookie set").to_string();
    let session_id = cookie.split(';').next().unwrap().split('=').nth(1).unwrap();

    // Access logs
    let logs_resp = agent.get(&format!("{}/api/logs", base_url))
        .set("Cookie", &format!("session_id={}", session_id))
        .call()
        .expect("Logs fetch failed");
    
    assert_eq!(logs_resp.status(), 200);
    
    // Verify response is a JSON array
    let logs: Vec<serde_json::Value> = logs_resp.into_json().expect("Failed to parse logs JSON");
    assert!(logs.len() > 0); // Server starts and should have logged its own start
}

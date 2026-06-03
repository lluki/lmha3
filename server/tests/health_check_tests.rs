mod common;
use common::TestHarness;
use ureq;
use serde_json::Value;

#[test]
fn test_healthcheck_admin_only() {
    let harness = TestHarness::new(8007, true);
    let _admin_id = harness.create_user("admin", "password123");
    let _user_id = harness.create_user("alice", "password123");
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // 1. Unauthenticated
    let resp = agent.get(&format!("{}/api/admin/healthcheck", base_url)).call();
    assert_eq!(resp.unwrap_err().into_response().unwrap().status(), 401);

    // 2. Authenticate as regular user
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "alice"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    let resp = agent.get(&format!("{}/api/admin/healthcheck", base_url))
        .set("Cookie", &cookie)
        .call();
    assert_eq!(resp.unwrap_err().into_response().unwrap().status(), 403);

    // 3. Authenticate as admin
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "admin"), ("password", "password123")])
        .unwrap();
    let admin_cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Healthcheck takes 10s due to device check
    let resp = agent.get(&format!("{}/api/admin/healthcheck", base_url))
        .set("Cookie", &admin_cookie)
        .call()
        .unwrap();
    
    assert_eq!(resp.status(), 200);
    let json: Value = resp.into_json().unwrap();
    
    assert!(json.get("pv").is_some());
    assert!(json.get("mqtt").is_some());
    assert!(json.get("devices").is_some());
}

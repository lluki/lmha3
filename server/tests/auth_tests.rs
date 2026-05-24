mod common;
use common::TestHarness;
use ureq;

#[test]
fn test_full_auth_cycle() {
    let harness = TestHarness::new(8006, true);
    let _tenant_id = harness.create_user("alice", "password123");
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // 1. Check /api/me without session
    let me_resp = agent.get(&format!("{}/api/me", base_url)).call();
    assert_eq!(me_resp.unwrap_err().into_response().unwrap().status(), 401);

    // 2. Login
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "alice"), ("password", "password123")]);
    
    let resp = match login_resp {
        Ok(r) => r,
        Err(e) => panic!("Login failed: {:?}", e),
    };

    assert_eq!(resp.status(), 200);
    let cookie = resp.header("Set-Cookie").expect("No session cookie set").to_string();
    let json: serde_json::Value = resp.into_json::<serde_json::Value>().unwrap();
    assert_eq!(json["status"], "ok");

    // 3. Access /api/me with session
    let me_resp_auth = agent.get(&format!("{}/api/me", base_url))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert_eq!(me_resp_auth.status(), 200);
    let me_json: serde_json::Value = me_resp_auth.into_json::<serde_json::Value>().unwrap();
    assert!(me_json.get("id").is_some());

    // 4. Logout
    let logout_resp = agent.post(&format!("{}/api/logout", base_url))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert_eq!(logout_resp.status(), 200);

    // 5. Verify session is cleared
    let me_resp_final = agent.get(&format!("{}/api/me", base_url))
        .set("Cookie", &cookie)
        .call();
    assert_eq!(me_resp_final.unwrap_err().into_response().unwrap().status(), 401);
}

mod common;
use common::TestHarness;
use ureq;

#[test]
fn test_full_auth_cycle() {
    let harness = TestHarness::new(8006, true);
    let tenant_id = harness.create_user("alice", "password123");
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
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

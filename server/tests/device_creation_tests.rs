mod common;
use common::TestHarness;
use ureq;
use uuid::Uuid;
use postgres::{Client, NoTls};

#[test]
fn test_admin_device_creation() {
    let harness = TestHarness::new(8007, true);
    let _admin_id = harness.create_user("admin", "admin123");
    let alice_id = harness.create_user("alice", "alice123");
    
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // 1. Login as admin
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "admin"), ("password", "admin123")])
        .expect("Admin login failed");
    
    let cookie = login_resp.header("Set-Cookie").expect("No session cookie set").to_string();

    // 2. Create a device for alice as admin
    let device_name = "Alice Light";
    let device_topic = "shelly-alice-123";
    let create_resp = agent.post(&format!("{}/api/devices", base_url))
        .set("Cookie", &cookie)
        .send_form(&[
            ("tenant_id", &alice_id.to_string()),
            ("mqtt_topic", device_topic),
            ("name", device_name),
        ])
        .expect("Device creation failed");

    assert_eq!(create_resp.status(), 200);
    let json: serde_json::Value = create_resp.into_json().unwrap();
    assert_eq!(json["status"], "ok");
    let device_id_str = json["id"].as_str().expect("id field missing or not a string");
    let device_id = Uuid::parse_str(device_id_str).expect("Invalid UUID in response");

    // 3. Verify in database
    let mut db_client = Client::connect(&harness.config.database_url, NoTls).unwrap();
    let row = db_client.query_one("SELECT name, tenant_id FROM devices WHERE id = $1", &[&device_id]).expect("Device not found in DB");
    let db_name: String = row.get(0);
    let db_tenant_id: Uuid = row.get(1);
    assert_eq!(db_name, device_name);
    assert_eq!(db_tenant_id, alice_id);

    // 4. Verify non-admin (alice) cannot create a device
    let alice_login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "alice"), ("password", "alice123")])
        .expect("Alice login failed");
    let alice_cookie = alice_login_resp.header("Set-Cookie").expect("No session cookie set").to_string();

    let forbidden_resp = agent.post(&format!("{}/api/devices", base_url))
        .set("Cookie", &alice_cookie)
        .send_form(&[
            ("tenant_id", &alice_id.to_string()),
            ("mqtt_topic", "stolen-topic"),
            ("name", "Stolen Device"),
        ]);
    
    match forbidden_resp {
        Err(ureq::Error::Status(403, _)) => (), // Expected
        other => panic!("Expected 403 Forbidden, got {:?}", other),
    }
}

#[test]
fn test_admin_toggle_others_device() {
    let harness = TestHarness::new(8008, true);
    let _admin_id = harness.create_user("admin", "admin123");
    let alice_id = harness.create_user("alice", "alice123");
    let device_id = harness.create_device(alice_id, "Alice Light", "shelly-alice-456");

    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // 1. Login as admin
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "admin"), ("password", "admin123")])
        .expect("Admin login failed");
    let cookie = login_resp.header("Set-Cookie").expect("No session cookie set").to_string();

    // 2. Toggle Alice's device as admin
    let toggle_resp = agent.post(&format!("{}/api/devices/{}/toggle", base_url, device_id))
        .set("Cookie", &cookie)
        .call()
        .expect("Admin failed to toggle Alice's device");

    assert_eq!(toggle_resp.status(), 200);
}

#[test]
fn test_global_read_access() {
    let harness = TestHarness::new(8009, true);
    let admin_id = harness.create_user("admin", "admin123");
    let _alice_id = harness.create_user("alice", "alice123");
    
    // Create a device owned by admin
    let device_id = harness.create_device(admin_id, "Admin Device", "shelly-admin-789");
    
    // Insert telemetry for admin's device
    let mut db_client = Client::connect(&harness.config.database_url, NoTls).unwrap();
    db_client.execute(
        "INSERT INTO telemetry (source, device_id, value) VALUES ('DEVICE_STATE', $1, 1.0)",
        &[&device_id]
    ).expect("Failed to insert telemetry");

    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // Login as Alice (regular user)
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "alice"), ("password", "alice123")])
        .expect("Alice login failed");
    let cookie = login_resp.header("Set-Cookie").expect("No session cookie set").to_string();

    // 1. Alice should see ALL devices
    let devices_resp = agent.get(&format!("{}/api/devices", base_url))
        .set("Cookie", &cookie)
        .call()
        .expect("Failed to fetch devices");
    let devices: serde_json::Value = devices_resp.into_json().unwrap();
    assert!(devices.as_array().unwrap().len() >= 1);
    
    // 2. Alice should see ALL history (even admin's device telemetry)
    let history_resp = agent.get(&format!("{}/api/history", base_url))
        .set("Cookie", &cookie)
        .call()
        .expect("Failed to fetch history");
    let history: serde_json::Value = history_resp.into_json().unwrap();
    assert!(history.as_array().unwrap().len() >= 1);
}

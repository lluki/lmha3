mod common;
use common::TestHarness;
use rumqttc::{Client, MqttOptions, QoS, Event, Packet};
use serde_json::json;
use std::time::Duration;
use std::thread;
use ureq;

#[test]
fn test_list_scripts_rpc_bridge() {
    let port = 8012;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("bob", "password123");
    let device_topic = "shellypro1-rpc-test";
    let device_id = harness.create_device(tenant_id, "Bob's Boiler", device_topic);

    // 1. Setup separate MQTT client to simulate the Shelly device
    let mut mqtt_options = MqttOptions::new("shelly-mock-rpc", &harness.config.mqtt_host, harness.config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    if let (Some(u), Some(p)) = (harness.config.mqtt_user.as_ref(), harness.config.mqtt_password.as_ref()) {
        mqtt_options.set_credentials(u, p);
    }
    
    let (client, mut connection) = Client::new(mqtt_options, 10);
    client.subscribe(format!("{}/rpc", device_topic), QoS::AtLeastOnce).unwrap();

    let client_clone = client.clone();
    let device_topic_str = device_topic.to_string();
    thread::spawn(move || {
        for notification in connection.iter() {
            if let Ok(Event::Incoming(Packet::Publish(publish))) = notification {
                let payload: serde_json::Value = serde_json::from_slice(&publish.payload).unwrap();
                if payload["method"] == "Script.List" {
                    let id = payload["id"].as_u64().unwrap();
                    let response = json!({
                        "id": id,
                        "src": device_topic_str,
                        "result": {
                            "scripts": [
                                {"id": 1, "name": "test_script", "running": true}
                            ]
                        }
                    });
                    client_clone.publish(format!("{}/rpc-response/rpc", device_topic_str), QoS::AtLeastOnce, false, response.to_string()).unwrap();
                }
            }
        }
    });

    // 2. Perform login
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "bob"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // 3. Call the Script.List endpoint
    let scripts_resp = agent.get(&format!("http://localhost:{}/api/admin/devices/{}/scripts", port, device_id))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    
    let scripts: serde_json::Value = scripts_resp.into_json::<serde_json::Value>().unwrap();
    assert_eq!(scripts["result"]["scripts"][0]["name"], "test_script");
    assert_eq!(scripts["result"]["scripts"][0]["running"], true);
}

#[test]
fn test_script_start_stop_rpc_bridge() {
    let port = 8013;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("alice", "password123");
    let device_topic = "shellypro1-alice";
    let device_id = harness.create_device(tenant_id, "Alice's Boiler", device_topic);

    let mut mqtt_options = MqttOptions::new("shelly-mock-rpc-alice", &harness.config.mqtt_host, harness.config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    if let (Some(u), Some(p)) = (harness.config.mqtt_user.as_ref(), harness.config.mqtt_password.as_ref()) {
        mqtt_options.set_credentials(u, p);
    }
    
    let (client, mut connection) = Client::new(mqtt_options, 10);
    client.subscribe(format!("{}/rpc", device_topic), QoS::AtLeastOnce).unwrap();

    let client_clone = client.clone();
    let device_topic_str = device_topic.to_string();
    thread::spawn(move || {
        for notification in connection.iter() {
            if let Ok(Event::Incoming(Packet::Publish(publish))) = notification {
                let payload: serde_json::Value = serde_json::from_slice(&publish.payload).unwrap();
                let id = payload["id"].as_u64().unwrap();
                let method = payload["method"].as_str().unwrap();
                
                if method == "Script.Start" || method == "Script.Stop" {
                    let response = json!({
                        "id": id,
                        "src": device_topic_str,
                        "result": { "was_running": method == "Script.Stop" }
                    });
                    client_clone.publish(format!("{}/rpc-response/rpc", device_topic_str), QoS::AtLeastOnce, false, response.to_string()).unwrap();
                }
            }
        }
    });

    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "alice"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Test Start
    let start_resp = agent.post(&format!("http://localhost:{}/api/admin/devices/{}/scripts/1/start", port, device_id))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert_eq!(start_resp.status(), 200);

    // Test Stop
    let stop_resp = agent.post(&format!("http://localhost:{}/api/admin/devices/{}/scripts/1/stop", port, device_id))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    assert_eq!(stop_resp.status(), 200);
}

#[test]
fn test_rpc_timeout() {
    let port = 8014;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("dave", "password123");
    let device_topic = "shelly-timeout";
    let device_id = harness.create_device(tenant_id, "Dave's Boiler", device_topic);

    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "dave"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Call endpoint - should timeout after 5s as no mock Shelly is responding
    let scripts_resp = agent.get(&format!("http://localhost:{}/api/admin/devices/{}/scripts", port, device_id))
        .set("Cookie", &cookie)
        .call();
    
    assert!(scripts_resp.is_err());
    let err = scripts_resp.unwrap_err();
    if let ureq::Error::Status(status, response) = err {
        assert_eq!(status, 500);
        assert_eq!(response.into_string().unwrap(), "RPC timeout");
    } else {
        panic!("Expected Status error, got {:?}", err);
    }
}

#[test]
fn test_script_authorization() {
    let port = 8015;
    let harness = TestHarness::new(port, true); 
    let alice_id = harness.create_user("alice_auth", "pass");
    let _bob_id = harness.create_user("bob_auth", "pass");
    let device_topic = "shelly-auth-test";
    let device_id = harness.create_device(alice_id, "Alice's Device", device_topic);

    // Login as Bob (not the owner)
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "bob_auth"), ("password", "pass")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Bob tries to list Alice's scripts - should be 403
    let scripts_resp = agent.get(&format!("http://localhost:{}/api/admin/devices/{}/scripts", port, device_id))
        .set("Cookie", &cookie)
        .call();
    
    assert!(scripts_resp.is_err());
    if let ureq::Error::Status(status, _) = scripts_resp.unwrap_err() {
        assert_eq!(status, 403);
    } else {
        panic!("Expected 403");
    }

    // Now login as Admin
    let login_admin_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "admin"), ("password", "admin")])
        .unwrap();
    let admin_cookie = login_admin_resp.header("Set-Cookie").unwrap().to_string();

    // Admin tries to list Alice's scripts - should NOT be 403 (might be 500/timeout if no device responds, but not 403)
    let admin_resp = agent.get(&format!("http://localhost:{}/api/admin/devices/{}/scripts", port, device_id))
        .set("Cookie", &admin_cookie)
        .call();
    
    match admin_resp {
        Ok(_) => {}, // Unlikely without mock device
        Err(ureq::Error::Status(status, _)) => {
            assert_ne!(status, 403);
        }
        _ => {}
    }
}

#[test]
fn test_get_put_script_code() {
    let port = 8016;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("editor", "password123");
    let device_topic = "shellypro1-editor";
    let device_id = harness.create_device(tenant_id, "Editor's Device", device_topic);

    let mut mqtt_options = MqttOptions::new("shelly-mock-editor", &harness.config.mqtt_host, harness.config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    if let (Some(u), Some(p)) = (harness.config.mqtt_user.as_ref(), harness.config.mqtt_password.as_ref()) {
        mqtt_options.set_credentials(u, p);
    }
    
    let (client, mut connection) = Client::new(mqtt_options, 10);
    client.subscribe(format!("{}/rpc", device_topic), QoS::AtLeastOnce).unwrap();

    let client_clone = client.clone();
    let device_topic_str = device_topic.to_string();
    thread::spawn(move || {
        for notification in connection.iter() {
            if let Ok(Event::Incoming(Packet::Publish(publish))) = notification {
                let payload: serde_json::Value = serde_json::from_slice(&publish.payload).unwrap();
                let id = payload["id"].as_u64().unwrap();
                let method = payload["method"].as_str().unwrap();
                
                if method == "Script.GetCode" {
                    let response = json!({
                        "id": id,
                        "src": device_topic_str,
                        "result": { "code": "console.log('hello world');" }
                    });
                    client_clone.publish(format!("{}/rpc-response/rpc", device_topic_str), QoS::AtLeastOnce, false, response.to_string()).unwrap();
                } else if method == "Script.PutCode" {
                    assert_eq!(payload["params"]["code"], "console.log('updated');");
                    let response = json!({
                        "id": id,
                        "src": device_topic_str,
                        "result": { "status": "ok" }
                    });
                    client_clone.publish(format!("{}/rpc-response/rpc", device_topic_str), QoS::AtLeastOnce, false, response.to_string()).unwrap();
                }
            }
        }
    });

    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "editor"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // 1. Get Code
    let get_resp = agent.get(&format!("http://localhost:{}/api/admin/devices/{}/scripts/1/code", port, device_id))
        .set("Cookie", &cookie)
        .call()
        .unwrap();
    let get_data: serde_json::Value = get_resp.into_json().unwrap();
    assert_eq!(get_data["result"]["code"], "console.log('hello world');");

    // 2. Put Code
    let put_resp = agent.put(&format!("http://localhost:{}/api/admin/devices/{}/scripts/1/code", port, device_id))
        .set("Cookie", &cookie)
        .send_string("console.log('updated');")
        .unwrap();
    assert_eq!(put_resp.status(), 200);
}

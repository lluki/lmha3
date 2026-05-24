mod common;
use common::TestHarness;
use rumqttc::{Client, MqttOptions, QoS};
use serde_json::json;
use std::time::Duration;
use std::thread;
use ureq;

#[test]
fn test_status_update_on_and_off() {
    let port = 8011;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("charlie", "password123");
    let device_topic = format!("test-device-{}", uuid::Uuid::new_v4().simple());
    let _device_id = harness.create_device(tenant_id, "Charlie's Heater", &device_topic);

    // 1. Setup separate MQTT client to simulate the Shelly device
    let mut mqtt_options = MqttOptions::new("shelly-mock", &harness.config.mqtt_host, harness.config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(harness.config.mqtt_user.as_ref().unwrap(), harness.config.mqtt_password.as_ref().unwrap());
    
    let (mut client, mut connection) = Client::new(mqtt_options, 10);
    thread::spawn(move || {
        for _ in connection.iter() {}
    });

    // 2. Perform login
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/login", port))
        .send_form(&[("username", "charlie"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // 3. Mock Shelly sending ON
    let payload_on = json!({
        "method": "NotifyStatus",
        "params": {
            "switch:0": { "output": true }
        }
    });
    let topic = format!("{}/events/rpc", device_topic);
    println!("Mock Shelly: Publishing ON to {}", topic);
    client.publish(&topic, QoS::AtLeastOnce, false, payload_on.to_string()).unwrap();

    // Wait and verify ON
    let mut found_on = false;
    let mut last_body = String::new();
    for i in 0..20 {
        thread::sleep(Duration::from_millis(500));
        let body = agent.get(&format!("http://localhost:{}", port)).set("Cookie", &cookie).call().unwrap().into_string().unwrap();
        last_body = body.clone();
        if body.contains("ON") { 
            println!("Success! Found ON in dashboard on attempt {}", i);
            found_on = true; 
            break; 
        }
    }
    if !found_on {
        println!("FAIL: Dashboard body:\n{}", last_body);
    }
    assert!(found_on, "Dashboard did not update to 'ON'");

    // 4. Mock Shelly sending OFF
    let payload_off = json!({
        "method": "NotifyStatus",
        "params": {
            "switch:0": { "output": false }
        }
    });
    println!("Mock Shelly: Publishing OFF to {}", topic);
    client.publish(topic, QoS::AtLeastOnce, false, payload_off.to_string()).unwrap();

    // Wait and verify OFF
    let mut found_off = false;
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        let body = agent.get(&format!("http://localhost:{}", port)).set("Cookie", &cookie).call().unwrap().into_string().unwrap();
        if body.contains("OFF") { found_off = true; break; }
    }
    assert!(found_off, "Dashboard did not update to 'OFF'");
}

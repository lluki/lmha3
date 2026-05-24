mod common;
use common::TestHarness;
use rumqttc::{Client, MqttOptions, QoS};
use serde_json::json;
use std::time::Duration;
use std::thread;
use ureq;

#[test]
fn test_history_recording() {
    let port = 8012;
    let harness = TestHarness::new(port, true); 
    let tenant_id = harness.create_user("telemetry_user", "password123");
    let device_topic = format!("test-device-{}", uuid::Uuid::new_v4().simple());
    let _device_id = harness.create_device(tenant_id, "History Device", &device_topic);

    let mut mqtt_options = MqttOptions::new("shelly-mock-history", &harness.config.mqtt_host, harness.config.mqtt_port);
    mqtt_options.set_credentials(harness.config.mqtt_user.as_ref().unwrap(), harness.config.mqtt_password.as_ref().unwrap());
    
    let (client, mut connection) = Client::new(mqtt_options, 10);
    thread::spawn(move || {
        for _ in connection.iter() {}
    });

    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let login_resp = agent.post(&format!("http://localhost:{}/api/login", port))
        .send_form(&[("username", "telemetry_user"), ("password", "password123")])
        .unwrap();
    let cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // 1. Send ON event + Power
    let payload_on = json!({
        "method": "NotifyStatus",
        "params": {
            "switch:0": { "output": true, "apower": 50.5 }
        }
    });
    client.publish(format!("{}/events/rpc", device_topic), QoS::AtLeastOnce, false, payload_on.to_string()).unwrap();

    // 2. Wait and check history
    let mut history_ok = false;
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        let body_resp = agent.get(&format!("http://localhost:{}/api/history", port)).set("Cookie", &cookie).call().unwrap();
        let history: Vec<serde_json::Value> = body_resp.into_json::<Vec<serde_json::Value>>().unwrap();
        
        let has_state = history.iter().any(|h| h["source"] == "DEVICE_STATE" && h["value"] == 1.0);
        let has_power = history.iter().any(|h| h["source"] == "DEVICE_CONSUMPTION" && h["value"] == 50.5);
        
        if has_state && has_power {
            history_ok = true;
            break;
        }
    }
    assert!(history_ok, "History did not record state change and power consumption");
}

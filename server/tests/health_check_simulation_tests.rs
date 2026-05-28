mod common;
use common::TestHarness;
use ureq;
use serde_json::{Value, json};
use rumqttc::{Client, MqttOptions, QoS, Event, Packet};
use std::thread;
use std::time::Duration;

#[test]
fn test_healthcheck_simulation() {
    let harness = TestHarness::new(8008, true);
    let admin_id = harness.create_user("admin", "password123");
    
    // Create two devices
    let good_device_topic = "shelly-good-123";
    let bad_device_topic = "shelly-bad-456";
    harness.create_device(admin_id, "Good Device", good_device_topic);
    harness.create_device(admin_id, "Bad Device", bad_device_topic);

    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .build();
    let base_url = format!("http://localhost:{}", harness.port);

    // Login as admin
    let login_resp = agent.post(&format!("{}/api/login", base_url))
        .send_form(&[("username", "admin"), ("password", "password123")])
        .unwrap();
    let admin_cookie = login_resp.header("Set-Cookie").unwrap().to_string();

    // Start a "mock device" thread for the good device
    let good_topic = good_device_topic.to_string();
    let mqtt_host = harness.config.mqtt_host.clone();
    let mqtt_port = harness.config.mqtt_port;
    let mqtt_user = harness.config.mqtt_user.clone();
    let mqtt_pass = harness.config.mqtt_password.clone();

    thread::spawn(move || {
        let mut mqttoptions = MqttOptions::new("mock-device", mqtt_host, mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        if let (Some(u), Some(p)) = (mqtt_user, mqtt_pass) {
            mqttoptions.set_credentials(u, p);
        }

        let (mut client, mut connection) = Client::new(mqttoptions, 10);
        client.subscribe(format!("{}/rpc", good_topic), QoS::AtMostOnce).unwrap();

        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let payload: Value = serde_json::from_slice(&publish.payload).unwrap();
                    if payload["method"] == "Shelly.GetStatus" {
                        let src = payload["src"].as_str().unwrap();
                        println!("Mock Device: Received healthcheck, responding to {}", src);
                        client.publish(src, QoS::AtMostOnce, false, "{\"status\":\"ok\"}").unwrap();
                    }
                }
                Ok(_) => {},
                Err(_) => break,
            }
        }
    });

    // Trigger Healthcheck
    println!("Triggering healthcheck...");
    let resp = agent.get(&format!("{}/api/admin/healthcheck", base_url))
        .set("Cookie", &admin_cookie)
        .call()
        .unwrap();
    
    assert_eq!(resp.status(), 200);
    let json: Value = resp.into_json().unwrap();
    
    println!("Healthcheck result: {}", serde_json::to_string_pretty(&json).unwrap());

    let devices = json["devices"]["details"].as_array().unwrap();
    
    let good_res = devices.iter().find(|d| d["topic"] == good_device_topic).expect("Good device not found in results");
    let bad_res = devices.iter().find(|d| d["topic"] == bad_device_topic).expect("Bad device not found in results");

    assert_eq!(good_res["status"], "ok", "Good device should be OK");
    assert_eq!(bad_res["status"], "error", "Bad device should be ERROR");
}

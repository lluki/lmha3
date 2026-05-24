use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::{verify_password, Session, DeviceState};
use rouille::{Request, Response, router};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS, Event, Packet};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    no_scheduler: bool,
    #[arg(long)]
    no_home_assistant: bool,
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
}

struct AppState {
    db: Mutex<Db>,
    config: Config,
    no_home_assistant: bool,
    no_scheduler: bool,
    mqtt_client: Mutex<Client>,
}

fn main() {
    let args = Args::parse();
    let config = Config::from_env();
    let db = Db::connect(&config).expect("Failed to connect to database");

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        config: config.clone(),
        no_home_assistant: args.no_home_assistant,
        no_scheduler: args.no_scheduler,
        mqtt_client: Mutex::new(dummy_client()),
    });

    let loop_state = state.clone();
    thread::spawn(move || {
        run_main_loop(loop_state);
    });

    println!("LMHA3 Server Starting on 0.0.0.0:{}...", args.port);
    
    rouille::start_server(format!("0.0.0.0:{}", args.port), move |request| {
        let state = state.clone();
        
        router!(request,
            (GET) (/) => {
                let session = get_session(request, &state);
                if let Some(s) = session {
                    let mut db = state.db.lock().unwrap();
                    let tenants = db.list_tenants().unwrap_or_default();
                    let devices = db.list_devices().unwrap_or_default();
                    
                    let mut html = format!("<h1>Admin Dashboard</h1><p>Logged in as: {}</p>", s.tenant_id);
                    html.push_str("<h2>Tenants</h2><ul>");
                    for t in tenants {
                        html.push_str(&format!("<li>{} ({})</li>", t.username, t.id));
                    }
                    html.push_str("</ul><h2>Devices</h2><table border='1'><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Last Seen</th><th>Action</th></tr>");
                    for d in devices {
                        let last_seen = d.last_heartbeat.map(|h| h.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Never".to_string());
                        let state_str = match d.current_state {
                            DeviceState::On => "ON",
                            DeviceState::Off => "OFF",
                            DeviceState::Unknown => "Unknown",
                        };
                        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>", d.name, d.tenant_id, d.mqtt_topic, state_str, last_seen));
                        
                        if d.tenant_id == s.tenant_id {
                            html.push_str(&format!("<td><form method='POST' action='/devices/{}/toggle'><button type='submit'>Toggle</button></form></td>", d.id));
                        } else {
                            html.push_str("<td>-</td>");
                        }
                        html.push_str("</tr>");
                    }
                    html.push_str("</table><br><form method='POST' action='/logout'><button type='submit'>Logout</button></form>");
                    Response::html(html)
                } else {
                    Response::redirect_303("/login")
                }
            },

            (GET) (/login) => {
                Response::html("<h1>Login</h1><form method='POST' action='/login'><input name='username' placeholder='Username'/><input name='password' type='password' placeholder='Password'/><button type='submit'>Login</button></form>")
            },

            (POST) (/login) => {
                let data = rouille::post_input!(request, {
                    username: String,
                    password: String,
                }).expect("Invalid login form");

                let mut db = state.db.lock().unwrap();
                if let Some(tenant) = db.get_tenant_by_username(&data.username) {
                    if verify_password(&data.password, &tenant.password_hash) {
                        let session_id = db.create_session(tenant.id).expect("Failed to create session");
                        return Response::redirect_303("/")
                            .with_additional_header("Set-Cookie", format!("session_id={}; HttpOnly; Path=/; SameSite=Lax", session_id));
                    }
                }
                Response::text("Invalid credentials").with_status_code(401)
            },

            (POST) (/devices/{id: Uuid}/toggle) => {
                let session = get_session(request, &state);
                if let Some(s) = session {
                    let mut db = state.db.lock().unwrap();
                    let devices = db.list_devices().unwrap_or_default();
                    if let Some(device) = devices.into_iter().find(|d| d.id == id && d.tenant_id == s.tenant_id) {
                        let new_on = device.current_state != DeviceState::On;
                        let payload = json!({
                            "id": 1,
                            "src": "lmha3",
                            "method": "Switch.Set",
                            "params": {"id": 0, "on": new_on}
                        });
                        let topic = format!("{}/rpc", device.mqtt_topic);
                        println!("API: Publishing toggle to {} | ON: {}", topic, new_on);
                        let client = state.mqtt_client.lock().unwrap();
                        let _ = client.publish(topic, QoS::AtLeastOnce, false, payload.to_string());
                        Response::redirect_303("/")
                    } else {
                        Response::text("Forbidden or Not Found").with_status_code(403)
                    }
                } else {
                    Response::redirect_303("/login")
                }
            },

            (POST) (/logout) => {
                if let Some(session_id) = get_session_id(request) {
                    let mut db = state.db.lock().unwrap();
                    let _ = db.delete_session(session_id);
                }
                Response::redirect_303("/login").with_additional_header("Set-Cookie", "session_id=; HttpOnly; Path=/; Max-Age=0")
            },

            _ => Response::empty_404()
        )
    });
}

fn dummy_client() -> Client {
    let (c, _) = Client::new(MqttOptions::new("dummy", "localhost", 1883), 1);
    c
}

fn get_session_id(request: &Request) -> Option<Uuid> {
    rouille::input::cookies(request)
        .find(|&(ref k, _)| *k == "session_id")
        .and_then(|(_, v)| Uuid::parse_str(&v).ok())
}

fn get_session(request: &Request, state: &AppState) -> Option<Session> {
    let session_id = get_session_id(request)?;
    let mut db = state.db.lock().unwrap();
    db.get_session(session_id)
}

fn run_main_loop(state: Arc<AppState>) {
    loop {
        println!("MQTT Loop starting/reconnecting...");
        let mut mqtt_options = MqttOptions::new("lmha3-server", &state.config.mqtt_host, state.config.mqtt_port);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        if let (Some(u), Some(p)) = (&state.config.mqtt_user, &state.config.mqtt_password) {
            mqtt_options.set_credentials(u, p);
        }
        
        let (client, mut connection) = Client::new(mqtt_options, 50);
        {
            let mut shared_client = state.mqtt_client.lock().unwrap();
            *shared_client = client.clone();
        }

        let subs = ["+/online", "+/status/sys", "+/status/switch:0", "+/events/rpc", "shellies/+/status/switch:0"];
        for sub in subs {
            let _ = client.subscribe(sub, QoS::AtMostOnce);
        }

        if !state.no_scheduler {
            let sch_state = state.clone();
            thread::spawn(move || {
                loop {
                    if !sch_state.no_home_assistant {
                        // TODO: HA
                    }
                    thread::sleep(Duration::from_secs(300));
                }
            });
        }

        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let topic = publish.topic;
                    let payload_str = String::from_utf8_lossy(&publish.payload);
                    
                    let parts: Vec<&str> = topic.split('/').collect();
                    let (base_topic, is_gen1) = if parts.len() > 1 && parts[0] == "shellies" {
                        (parts[1], true)
                    } else {
                        (parts[0], false)
                    };

                    let mut db = state.db.lock().unwrap();
                    
                    if topic.ends_with("/online") || topic.contains("/status/") {
                        let _ = db.update_device_heartbeat(base_topic);
                    }
                    
                    let mut new_state = None;

                    // 1. Check direct status topics
                    if topic.ends_with("/status/switch:0") {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                            // Gen 2
                            if let Some(on) = val.get("output").and_then(|v| v.as_bool()) {
                                new_state = Some(if on { DeviceState::On } else { DeviceState::Off });
                            }
                            // Gen 1
                            else if let Some(on) = val.get("ison").and_then(|v| v.as_bool()) {
                                new_state = Some(if on { DeviceState::On } else { DeviceState::Off });
                            }
                        } else {
                            // Non-JSON status (like Gen 1 "on"/"off")
                            if payload_str == "on" { new_state = Some(DeviceState::On); }
                            else if payload_str == "off" { new_state = Some(DeviceState::Off); }
                        }
                    } 
                    // 2. Check Event RPC topics (Gen 2)
                    else if !is_gen1 && topic.ends_with("/events/rpc") {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                            let method = val.get("method").and_then(|v| v.as_str());
                            if method == Some("NotifyStatus") || method == Some("NotifyFullStatus") {
                                if let Some(params) = val.get("params") {
                                    if let Some(sw) = params.get("switch:0") {
                                        if let Some(on) = sw.get("output").and_then(|v| v.as_bool()) {
                                            new_state = Some(if on { DeviceState::On } else { DeviceState::Off });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(s) = new_state {
                        println!("MQTT State Update: {} -> {:?}", base_topic, s);
                        if let Err(e) = db.update_device_state(base_topic, s) {
                            eprintln!("DB Error updating {}: {}", base_topic, e);
                        }
                    }
                }
                Ok(_) => {},
                Err(e) => {
                    eprintln!("MQTT Loop Error: {:?}. Reconnecting...", e);
                    break;
                }
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

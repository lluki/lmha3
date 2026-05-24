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
        
        // 1. Try to serve static files from various possible locations
        let mut asset_response = rouille::match_assets(request, "server/public");
        if !asset_response.is_success() {
            asset_response = rouille::match_assets(request, "public");
        }
        
        if asset_response.is_success() {
            return asset_response;
        }

        router!(request,
            (GET) (/) => {
                // Return embedded index.html if file matching fails
                let index = include_str!("../public/index.html");
                Response::html(index)
            },

            // --- API Endpoints ---

            (GET) (/api/me) => {
                if let Some(s) = get_session(request, &state) {
                    Response::json(&s)
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/tenants) => {
                if let Some(s) = get_session(request, &state) {
                    println!("API: Fetching tenants for {}", s.tenant_id);
                    let mut db = state.db.lock().unwrap();
                    match db.list_tenants() {
                        Ok(tenants) => {
                            let public_tenants: Vec<lmha_core::TenantPublic> = tenants.into_iter().map(|t| lmha_core::TenantPublic {
                                id: t.id,
                                username: t.username,
                                created_at: t.created_at,
                            }).collect();
                            Response::json(&public_tenants)
                        },
                        Err(e) => {
                            eprintln!("DB Error listing tenants: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/devices) => {
                if let Some(s) = get_session(request, &state) {
                    println!("API: Fetching devices for {}", s.tenant_id);
                    let mut db = state.db.lock().unwrap();
                    match db.list_devices() {
                        Ok(devices) => Response::json(&devices),
                        Err(e) => {
                            eprintln!("DB Error listing devices: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/login) => {
                let data = rouille::post_input!(request, {
                    username: String,
                    password: String,
                }).expect("Invalid login form");

                let mut db = state.db.lock().unwrap();
                if let Some(tenant) = db.get_tenant_by_username(&data.username) {
                    if verify_password(&data.password, &tenant.password_hash) {
                        let session_id = db.create_session(tenant.id).expect("Failed to create session");
                        return Response::json(&json!({"status": "ok", "tenant_id": tenant.id}))
                            .with_additional_header("Set-Cookie", format!("session_id={}; HttpOnly; Path=/; SameSite=Lax", session_id));
                    }
                }
                Response::json(&json!({"error": "Invalid credentials"})).with_status_code(401)
            },

            (POST) (/api/devices/{id: Uuid}/toggle) => {
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
                        Response::json(&json!({"status": "ok"}))
                    } else {
                        Response::json(&json!({"error": "Forbidden"})).with_status_code(403)
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/logout) => {
                if let Some(session_id) = get_session_id(request) {
                    let mut db = state.db.lock().unwrap();
                    let _ = db.delete_session(session_id);
                }
                Response::json(&json!({"status": "ok"}))
                    .with_additional_header("Set-Cookie", "session_id=; HttpOnly; Path=/; Max-Age=0")
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
                    println!("MQTT Incoming: {} | Payload: {}", topic, payload_str);

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
                    if topic.ends_with("/status/switch:0") {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                            let output = val.get("output").and_then(|v| v.as_bool())
                                .or_else(|| val.get("ison").and_then(|v| v.as_bool()))
                                .or_else(|| {
                                    if payload_str == "on" { Some(true) }
                                    else if payload_str == "off" { Some(false) }
                                    else { None }
                                });
                            if let Some(on) = output {
                                new_state = Some(if on { DeviceState::On } else { DeviceState::Off });
                            }
                        }
                    } else if !is_gen1 && topic.ends_with("/events/rpc") {
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

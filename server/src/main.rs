use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::{verify_password, Session, DeviceState};
use rouille::{Request, Response, router};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS, Event, Packet, LastWill};
use serde::{Serialize};
use serde_json::json;
use std::collections::{VecDeque, HashMap};
use tracing::{info, error, trace, debug};
use tracing_subscriber::{fmt, prelude::*, Layer};

#[derive(Serialize, Clone, Debug)]
struct LogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    level: String,
    message: String,
    target: String,
}

struct LogBuffer {
    entries: VecDeque<LogEntry>,
    max_size: usize,
}

impl LogBuffer {
    fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }
}

struct BufferLayer {
    buffer: Arc<Mutex<LogBuffer>>,
}

impl<S> Layer<S> for BufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut fields = std::collections::HashMap::new();
        let mut visitor = FieldVisitor(&mut fields);
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: event.metadata().level().to_string(),
            message: fields.get("message").cloned().unwrap_or_else(|| "No message".to_string()),
            target: event.metadata().target().to_string(),
        };

        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(entry);
        }
    }
}

struct FieldVisitor<'a>(&'a mut std::collections::HashMap<String, String>);

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_string(), format!("{:?}", value));
    }
}

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
    log_buffer: Arc<Mutex<LogBuffer>>,
    is_passive: Mutex<bool>,
    other_instances: Mutex<HashMap<String, u32>>,
}

fn main() {
    let log_buffer = Arc::new(Mutex::new(LogBuffer::new(1000)));

    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into())
        .add_directive("server=trace".parse().unwrap())
        .add_directive("lmha_core=trace".parse().unwrap());

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter.clone()))
        .with(BufferLayer { buffer: log_buffer.clone() }.with_filter(filter))
        .init();

    let args = Args::parse();
    let config = Config::from_env();
    let mut db = Db::connect(&config).expect("Failed to connect to database");
    
    // Auto-run migrations
    info!("Running database migrations...");
    let migrations_dir = std::env::var("LMHA_MIGRATIONS_DIR").unwrap_or_else(|_| "migrations".to_string());
    if let Err(e) = db.run_migrations(&migrations_dir) {
        error!("Failed to run migrations: {}", e);
    }

    if let Err(e) = db.ensure_seeded() {
        error!("Failed to seed database: {}", e);
    }

    // Initial state alignment: set desired_state = current_state for all devices
    info!("Performing initial state alignment...");
    {
        // We need a raw client or a method on Db for this. Since we already have db, 
        // let's just use it if we can get the client, but Db doesn't expose it.
        // Actually, let's add a method to Db or just use a temporary client like before.
        let mut client = postgres::Client::connect(&config.database_url, postgres::NoTls).expect("Failed to connect for alignment");
        client.execute("UPDATE devices SET desired_state = current_state", &[]).ok();
    }

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        config: config.clone(),
        no_home_assistant: args.no_home_assistant,
        no_scheduler: args.no_scheduler,
        mqtt_client: Mutex::new(dummy_client()),
        log_buffer,
        is_passive: Mutex::new(false),
        other_instances: Mutex::new(HashMap::new()),
    });

    if !state.no_scheduler {
        let sch_state = state.clone();
        thread::spawn(move || {
            run_scheduler_loop(sch_state);
        });
    }

    let loop_state = state.clone();
    thread::spawn(move || {
        run_main_loop(loop_state);
    });

    let health_state = state.clone();
    thread::spawn(move || {
        run_health_check_loop(health_state);
    });

    let ib_state = state.clone();
    thread::spawn(move || {
        run_instance_heartbeat_loop(ib_state);
    });

    info!("LMHA3 Server Starting on 0.0.0.0:{}...", args.port);
    
    rouille::start_server(format!("0.0.0.0:{}", args.port), move |request| {
        let state = state.clone();
        
        // 1. Try to serve static files from various possible locations
        let public_dir = std::env::var("LMHA_PUBLIC_DIR").unwrap_or_else(|_| "server/public".to_string());
        let mut asset_response = rouille::match_assets(request, &public_dir);
        
        if !asset_response.is_success() && public_dir == "server/public" {
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
                if let Some(user) = get_user(request, &state) {
                    let is_passive = *state.is_passive.lock().unwrap();
                    Response::json(&json!({
                        "username": user.username,
                        "tenant_id": user.tenant_id,
                        "house_id": user.house_id,
                        "is_admin": user.is_admin,
                        "is_passive": is_passive
                    }))
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/version) => {
                Response::json(&json!({ "version": env!("CARGO_PKG_VERSION") }))
            },

            (GET) (/api/houses) => {
                if let Some(_) = get_session(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    match db.list_houses() {
                        Ok(houses) => Response::json(&houses),
                        Err(e) => {
                            error!("DB Error listing houses: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/houses) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        name: String,
                        ha_url: String,
                        ha_token: String,
                        ha_pv_entity_id: String,
                        ha_consumption_entity_id: String,
                    }).expect("Invalid house creation form");

                    let mut db = state.db.lock().unwrap();
                    match db.create_house(&data.name, &data.ha_url, &data.ha_token, &data.ha_pv_entity_id, &data.ha_consumption_entity_id) {
                        Ok(id) => Response::json(&json!({"status": "ok", "id": id})),
                        Err(e) => {
                            error!("DB Error creating house: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (DELETE) (/api/houses/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let mut db = state.db.lock().unwrap();
                    match db.delete_house(id) {
                        Ok(_) => Response::json(&json!({"status": "ok"})),
                        Err(e) => Response::text(e).with_status_code(400),
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/admin/select-house) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data: serde_json::Value = match rouille::input::json_input(request) {
                        Ok(p) => p,
                        Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                    };
                    let house_id_str = data.get("house_id").and_then(|v| v.as_str()).unwrap_or("");
                    let house_id = match Uuid::parse_str(house_id_str) {
                        Ok(id) => id,
                        Err(_) => return Response::text("Invalid UUID").with_status_code(400),
                    };

                    let mut db = state.db.lock().unwrap();
                    if let Err(e) = db.update_session_view_house(user.session_id, house_id) {
                        error!("DB Error updating session house: {}", e);
                        return Response::text("DB Error").with_status_code(500);
                    }
                    Response::json(&json!({"status": "ok"}))
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/admin/discover-devices) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    
                    let mut discovered_topics = std::collections::HashSet::new();
                    // Scan current and old logs
                    for log_file in ["logs/mqtt.log", "logs/mqtt2.log", "logs/mqtt3.log"] {
                        if let Ok(content) = std::fs::read_to_string(log_file) {
                            for line in content.lines() {
                                if let Some(idx) = line.find("MQTT Incoming: ") {
                                    let rest = &line[idx + 15..];
                                    let topic = rest.split('|').next().unwrap_or("").trim();
                                    let base_topic = topic.split('/').next().unwrap_or("");
                                    if base_topic.starts_with("shelly") {
                                        discovered_topics.insert(base_topic.to_string());
                                    }
                                }
                            }
                        }
                    }

                    let mut db = state.db.lock().unwrap();
                    let existing_devices = db.list_devices(None).unwrap_or_default();
                    let existing_topics: std::collections::HashSet<_> = existing_devices.iter().map(|d| d.mqtt_topic.as_str()).collect();

                    let mut new_topics: Vec<_> = discovered_topics.into_iter()
                        .filter(|t| !existing_topics.contains(t.as_str()))
                        .collect();
                    new_topics.sort();

                    Response::json(&new_topics)
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/admin/healthcheck) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }

                    let start_time = chrono::Utc::now();
                    
                    // 1. PV Check
                    let mut pv_results = Vec::new();
                    let houses = {
                        let mut db = state.db.lock().unwrap();
                        db.list_houses().unwrap_or_default()
                    };

                    for house in &houses {
                        let ha_url = &house.ha_url;
                        let pv_id = &house.ha_pv_entity_id;
                        match lmha_core::ha::fetch_ha_state(ha_url, &house.ha_token, pv_id) {
                            Ok(_) => pv_results.push(json!({"house": house.name, "status": "ok"})),
                            Err(e) => pv_results.push(json!({"house": house.name, "status": "error", "message": e})),
                        }
                    }

                    let pv_success = pv_results.iter().all(|r| r["status"] == "ok");
                    let pv_msg = if pv_success {
                        format!("Fetched PV for {}/{} houses", pv_results.len(), houses.len())
                    } else {
                        let failed_count = pv_results.iter().filter(|r| r["status"] == "error").count();
                        format!("PV fetch failed for {}/{} houses", failed_count, houses.len())
                    };

                    // 2. MQTT Check
                    let mqtt_client = state.mqtt_client.lock().unwrap();
                    let mqtt_status = match mqtt_client.publish("lmha3/healthcheck/ping", QoS::AtMostOnce, false, "ping") {
                        Ok(_) => "ok",
                        Err(_) => "error",
                    };
                    let mqtt_msg = if mqtt_status == "ok" {
                        "MQTT Broker reachable and accepting messages"
                    } else {
                        "MQTT Broker unreachable or client not connected"
                    };

                    // 3. Device Check
                    let devices = {
                        let mut db = state.db.lock().unwrap();
                        db.list_devices(None).unwrap_or_default()
                    };

                    for d in &devices {
                        let topic = format!("{}/rpc", d.mqtt_topic);
                        let payload = json!({
                            "id": 99,
                            "src": format!("{}/rpc-response", d.mqtt_topic),
                            "method": "Shelly.GetStatus"
                        }).to_string();
                        let _ = mqtt_client.publish(topic, QoS::AtMostOnce, false, payload);
                    }
                    
                    // Drop the lock before sleeping
                    drop(mqtt_client);
                    
                    info!("Healthcheck: Sleeping 12s for device responses...");
                    thread::sleep(Duration::from_secs(12));

                    let mut device_results = Vec::new();
                    let updated_devices = {
                        let mut db = state.db.lock().unwrap();
                        db.list_devices(None).unwrap_or_default()
                    };

                    let check_start = start_time;
                    for d in updated_devices {
                        let responsive = d.last_feedback_time.map(|t| t >= check_start).unwrap_or(false);
                        device_results.push(json!({
                            "name": d.name,
                            "topic": d.mqtt_topic,
                            "status": if responsive { "ok" } else { "error" }
                        }));
                    }

                    let responsive_count = device_results.iter().filter(|r| r["status"] == "ok").count();
                    let device_msg = format!("{}/{} devices responded via MQTT", responsive_count, devices.len());

                    Response::json(&json!({
                        "pv": {
                            "status": if pv_success { "ok" } else { "error" },
                            "message": pv_msg,
                            "details": pv_results
                        },
                        "mqtt": {
                            "status": mqtt_status,
                            "message": mqtt_msg
                        },
                        "devices": {
                            "status": if responsive_count == devices.len() { "ok" } else { "error" },
                            "message": device_msg,
                            "details": device_results
                        }
                    }))
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/logs) => {
                if let Some(_) = get_session(request, &state) {
                    let buffer = state.log_buffer.lock().unwrap();
                    Response::json(&buffer.entries)
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/tenants) => {
                if let Some(user) = get_user(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    match db.list_tenants() {
                        Ok(tenants) => {
                            let public_tenants: Vec<lmha_core::TenantPublic> = tenants.into_iter()
                                .filter(|t| user.is_admin || t.house_id == user.house_id)
                                .map(|t| lmha_core::TenantPublic {
                                    id: t.id,
                                    house_id: t.house_id,
                                    username: t.username,
                                    is_admin: t.is_admin,
                                    created_at: t.created_at,
                                }).collect();
                            Response::json(&public_tenants)
                        },
                        Err(e) => {
                            error!("DB Error listing tenants: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/tenants) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        username: String,
                        password: Option<String>,
                        house_id: String,
                        is_admin: Option<String>,
                    }).expect("Invalid tenant creation form");

                    let house_id = match Uuid::parse_str(&data.house_id) {
                        Ok(id) => id,
                        Err(_) => return Response::text("Invalid UUID").with_status_code(400),
                    };

                    let is_admin = data.is_admin.map(|s| s == "true").unwrap_or(false);

                    let password = data.password.unwrap_or_else(|| data.username.clone());
                    let hashed = lmha_core::hash_password(&password).expect("Failed to hash password");

                    let mut db = state.db.lock().unwrap();
                    match db.create_tenant(&data.username, &hashed, house_id, is_admin) {
                        Ok(id) => Response::json(&json!({"status": "ok", "id": id})),
                        Err(e) => {
                            error!("DB Error creating tenant: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (DELETE) (/api/tenants/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let mut db = state.db.lock().unwrap();
                    match db.delete_tenant(id) {
                        Ok(_) => Response::json(&json!({"status": "ok"})),
                        Err(e) => Response::text(e).with_status_code(400),
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/metrics) => {
                if let Some(user) = get_user(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    match db.get_latest_metrics(user.house_id) {
                        Ok((pv, cons)) => Response::json(&json!({ "pv": pv, "consumption": cons })),
                        Err(e) => {
                            error!("DB Error fetching metrics: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/devices) => {
                if let Some(user) = get_user(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    match db.list_devices(Some(user.house_id)) {
                        Ok(devices) => {
                            let mut results = Vec::new();
                            for d in devices {
                                let runtime = if d.scheduling_type == lmha_core::SchedulingType::Boiler {
                                    db.calc_boiler_runtime_24h(d.id).unwrap_or(0)
                                } else {
                                    0
                                };
                                let mut d_json = serde_json::to_value(&d).unwrap();
                                d_json.as_object_mut().unwrap().insert("runtime_24h".to_string(), json!(runtime));
                                results.push(d_json);
                            }
                            Response::json(&results)
                        },
                        Err(e) => {
                            error!("DB Error listing devices: {:?} | Error message: {}", e, e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/devices) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        tenant_id: String,
                        mqtt_topic: String,
                        name: String,
                    }).expect("Invalid device creation form");

                    let tenant_id = match Uuid::parse_str(&data.tenant_id) {
                        Ok(id) => id,
                        Err(_) => return Response::text("Invalid UUID").with_status_code(400),
                    };

                    let mut db = state.db.lock().unwrap();
                    match db.create_device(tenant_id, &data.mqtt_topic, &data.name, user.house_id) {
                        Ok(id) => Response::json(&json!({"status": "ok", "id": id})),
                        Err(e) => {
                            error!("DB Error creating device: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (PATCH) (/api/houses/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        name: String,
                        ha_url: String,
                        ha_token: String,
                        ha_pv_entity_id: String,
                        ha_consumption_entity_id: String,
                    }).expect("Invalid house update form");

                    let mut db = state.db.lock().unwrap();
                    match db.update_house(id, &data.name, &data.ha_url, &data.ha_token, &data.ha_pv_entity_id, &data.ha_consumption_entity_id) {
                        Ok(_) => Response::json(&json!({"status": "ok"})),
                        Err(e) => {
                            error!("DB Error updating house: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (PATCH) (/api/tenants/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        username: String,
                        house_id: String,
                        password: Option<String>,
                        is_admin: Option<String>,
                    }).expect("Invalid tenant update form");

                    let house_id = match Uuid::parse_str(&data.house_id) {
                        Ok(hid) => hid,
                        Err(_) => return Response::text("Invalid UUID").with_status_code(400),
                    };

                    let is_admin = data.is_admin.map(|s| s == "true").unwrap_or(false);

                    let mut db = state.db.lock().unwrap();
                    if let Err(e) = db.update_tenant_admin(id, &data.username, house_id, is_admin) {
                        error!("DB Error updating tenant: {}", e);
                        return Response::text("DB Error").with_status_code(500);
                    }

                    if let Some(password) = data.password {
                        if !password.is_empty() {
                            let hashed = lmha_core::hash_password(&password).expect("Failed to hash password");
                            if let Err(e) = db.update_tenant_password_admin(id, &hashed) {
                                error!("DB Error updating tenant password: {}", e);
                                return Response::text("DB Error").with_status_code(500);
                            }
                        }
                    }

                    Response::json(&json!({"status": "ok"}))
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (POST) (/api/tenants) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let data = rouille::post_input!(request, {
                        username: String,
                        password: Option<String>,
                        house_id: String,
                        is_admin: Option<String>,
                    }).expect("Invalid tenant creation form");

                    let house_id = match Uuid::parse_str(&data.house_id) {
                        Ok(id) => id,
                        Err(_) => return Response::text("Invalid UUID").with_status_code(400),
                    };

                    let is_admin = data.is_admin.map(|s| s == "true").unwrap_or(false);

                    let password = data.password.unwrap_or_else(|| data.username.clone());
                    let hashed = lmha_core::hash_password(&password).expect("Failed to hash password");

                    let mut db = state.db.lock().unwrap();
                    match db.create_tenant(&data.username, &hashed, house_id, is_admin) {
                        Ok(id) => Response::json(&json!({"status": "ok", "id": id})),
                        Err(e) => {
                            error!("DB Error creating tenant: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },
            (DELETE) (/api/devices/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    if !user.is_admin {
                        return Response::text("Forbidden").with_status_code(403);
                    }
                    let mut db = state.db.lock().unwrap();
                    match db.delete_device(id) {
                        Ok(_) => Response::json(&json!({"status": "ok"})),
                        Err(e) => {
                            error!("DB Error deleting device: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (PATCH) (/api/devices/{id: Uuid}) => {
                if let Some(user) = get_user(request, &state) {
                    // Logic: Must be admin OR own the device
                    let mut db = state.db.lock().unwrap();
                    let devices = db.list_devices(Some(user.house_id)).unwrap_or_default();
                    let device = devices.into_iter().find(|d| d.id == id);
                    
                    if let Some(d) = device {
                        if !user.is_admin && d.tenant_id != user.tenant_id {
                            return Response::text("Forbidden").with_status_code(403);
                        }

                        #[derive(serde::Deserialize)]
                        struct DevicePatch {
                            name: Option<String>,
                            mqtt_topic: Option<String>,
                            tenant_id: Option<Uuid>,
                            expected_load: Option<i32>,
                            scheduling_type: Option<lmha_core::SchedulingType>,
                            full_charge_n_day: Option<i32>,
                            min_daily_charge: Option<i32>,
                        }

                        let patch: DevicePatch = match rouille::input::json_input(request) {
                            Ok(p) => p,
                            Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                        };

                        if user.is_admin {
                            let name = patch.name.unwrap_or(d.name);
                            let mqtt_topic = patch.mqtt_topic.unwrap_or(d.mqtt_topic);
                            let tenant_id = patch.tenant_id.unwrap_or(d.tenant_id);
                            let load = patch.expected_load.unwrap_or(d.expected_load);
                            let full_charge = patch.full_charge_n_day.unwrap_or(d.full_charge_n_day);
                            let min_charge = patch.min_daily_charge.unwrap_or(d.min_daily_charge);

                            if let Err(e) = db.update_device_config_admin(id, &name, &mqtt_topic, tenant_id, load, full_charge, min_charge) {
                                error!("DB Error updating admin config: {}", e);
                                return Response::text("DB Error").with_status_code(500);
                            }
                        } else if patch.expected_load.is_some() || patch.full_charge_n_day.is_some() || patch.min_daily_charge.is_some() {
                            let load = patch.expected_load.unwrap_or(d.expected_load);
                            let full_charge = patch.full_charge_n_day.unwrap_or(d.full_charge_n_day);
                            let min_charge = patch.min_daily_charge.unwrap_or(d.min_daily_charge);
                            
                            if let Err(e) = db.update_device_config(id, load, full_charge, min_charge) {
                                error!("DB Error updating config: {}", e);
                                return Response::text("DB Error").with_status_code(500);
                            }
                        }

                        if let Some(sch) = patch.scheduling_type {
                            if let Err(e) = db.update_device_scheduling(id, sch) {
                                error!("DB Error updating scheduling: {}", e);
                                return Response::text("DB Error").with_status_code(500);
                            }
                        }

                        Response::json(&json!({"status": "ok"}))
                    } else {
                        Response::empty_404()
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/history) => {
                if let Some(user) = get_user(request, &state) {
                    let events_only = request.get_param("events_only").map(|v| v == "true").unwrap_or(false);
                    let mut db = state.db.lock().unwrap();
                    match db.list_telemetry(user.house_id, 100, events_only) {
                        Ok(t) => Response::json(&t),
                        Err(e) => {
                            error!("DB Error listing telemetry: {}", e);
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
                if let Some(user) = get_user(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    let devices = db.list_devices(Some(user.house_id)).unwrap_or_default();
                    if let Some(device) = devices.into_iter().find(|d| d.id == id && (user.is_admin || d.tenant_id == user.tenant_id)) {
                        let new_on = device.current_state != DeviceState::On;
                        let new_state = if new_on { DeviceState::On } else { DeviceState::Off };
                        let until = chrono::Utc::now() + chrono::Duration::hours(1);
                        let new_scheduling = if new_on {
                            lmha_core::SchedulingType::ForceOn { until }
                        } else {
                            lmha_core::SchedulingType::ForceOff { until }
                        };
                        
                        // Update desired state and scheduling in DB
                        if let Err(e) = db.update_device_desired_state(id, new_state) {
                            error!("DB Error updating desired state: {}", e);
                        }
                        if let Err(e) = db.update_device_scheduling(id, new_scheduling) {
                            error!("DB Error updating scheduling: {}", e);
                        }

                        let payload = json!({
                            "id": 1,
                            "src": format!("{}/rpc-response", device.mqtt_topic),
                            "method": "Switch.Set",
                            "params": {"id": 0, "on": new_on}
                        });
                        let topic = format!("{}/rpc", device.mqtt_topic);
                        let payload_str = payload.to_string();
                        info!("API: Publishing toggle to topic '{}' | Payload: {}", topic, payload_str);
                        let client = state.mqtt_client.lock().unwrap();
                        let _ = client.publish(topic, QoS::AtMostOnce, false, payload_str);

                        // Follow-up status poll
                        let poll_payload = json!({
                            "id": 103, "src": format!("{}/rpc-response", device.mqtt_topic), "method": "Shelly.GetStatus"
                        }).to_string();
                        let _ = client.publish(format!("{}/rpc", device.mqtt_topic), QoS::AtMostOnce, false, poll_payload);

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

            (POST) (/api/me/password) => {
                if let Some(user) = get_user(request, &state) {
                    let data = rouille::post_input!(request, {
                        password: String,
                    }).expect("Invalid password update form");

                    if data.password.is_empty() {
                        return Response::text("Password cannot be empty").with_status_code(400);
                    }

                    let hashed = lmha_core::hash_password(&data.password).expect("Failed to hash password");
                    let mut db = state.db.lock().unwrap();
                    match db.update_tenant_password_admin(user.tenant_id, &hashed) {
                        Ok(_) => Response::json(&json!({"status": "ok"})),
                        Err(e) => {
                            error!("DB Error updating self password: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
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

fn get_user(request: &Request, state: &AppState) -> Option<lmha_core::UserInfo> {
    let session_id = get_session_id(request)?;
    let mut db = state.db.lock().unwrap();
    db.get_user_info(session_id)
}

fn run_scheduler_loop(state: Arc<AppState>) {
    let (interval, debounce) = if std::env::var("LMHA_SCHEDULER_DEBUG").is_ok() {
        info!("Scheduler Debug mode enabled: 10s interval, 15s debounce");
        (Duration::from_secs(10), 15)
    } else {
        (Duration::from_secs(300), 300)
    };

    loop {
        let houses = {
            let mut db = state.db.lock().unwrap();
            db.list_houses().unwrap_or_default()
        };

        for house in houses {
            trace!("Processing House: {}", house.name);
            
            if !state.no_home_assistant {
                let ha_url = &house.ha_url;
                let pv_id = &house.ha_pv_entity_id;
                let cons_id = &house.ha_consumption_entity_id;

                match lmha_core::ha::fetch_ha_state(ha_url, &house.ha_token, pv_id) {
                    Ok(val) => {
                        let mut db = state.db.lock().unwrap();
                        if let Err(e) = db.insert_telemetry(lmha_core::TelemetrySource::PvProduction, None, val, None, house.id) {
                            error!("DB Error saving PV telemetry for {}: {}", house.name, e);
                        } else {
                            tracing::debug!("Saved PV telemetry for {}: {}", house.name, val);
                        }
                    }
                    Err(e) => error!("HA Error polling PV for {} ({}): {}", house.name, pv_id, e),
                }

                match lmha_core::ha::fetch_ha_state(ha_url, &house.ha_token, cons_id) {
                    Ok(val) => {
                        let mut db = state.db.lock().unwrap();
                        if let Err(e) = db.insert_telemetry(lmha_core::TelemetrySource::HouseConsumption, None, val, None, house.id) {
                            error!("DB Error saving consumption telemetry for {}: {}", house.name, e);
                        } else {
                            tracing::debug!("Saved Consumption telemetry for {}: {}", house.name, val);
                        }
                    }
                    Err(e) => error!("HA Error polling Consumption for {} ({}): {}", house.name, cons_id, e),
                }
            }
            
            // Run Scheduler Logic for this house
            {
                let mut db = state.db.lock().unwrap();
                let devices = db.list_devices(Some(house.id)).unwrap_or_default();
                let (pv_production, house_consumption) = db.get_latest_metrics(house.id).unwrap_or((0, 0));
                
                let now = chrono::Utc::now();
                let history_since = now - chrono::Duration::days(8);
                let mut history = std::collections::HashMap::new();

                for d in &devices {
                    if d.scheduling_type == lmha_core::SchedulingType::Boiler {
                        let device_history = db.get_device_history(d.id, history_since).unwrap_or_default();
                        history.insert(d.id, device_history);
                    }
                }

                let input = lmha_core::scheduler::SchedulerInput {
                    pv_production,
                    house_consumption,
                    devices: devices.iter().map(|d| {
                        let last_change = history.get(&d.id)
                            .and_then(|h| h.last().map(|e| e.timestamp))
                            .or(d.last_feedback_time);
                            
                        lmha_core::scheduler::DeviceContext {
                            id: d.id,
                            current_state: d.current_state,
                            last_state_change: last_change,
                            is_enabled: d.is_enabled,
                            expected_load: d.expected_load,
                            scheduling_type: d.scheduling_type.clone(),
                            full_charge_n_day: d.full_charge_n_day,
                            min_daily_charge: d.min_daily_charge,
                        }
                    }).collect(),
                    history,
                    now,
                    debounce_duration_secs: debounce,
                    rng: rand::thread_rng(),
                };

                trace!("Scheduler invoked for {}: PV={:.1}kW, Cons={:.1}kW, Devices={}",
                    house.name,
                    pv_production as f64 / 1000.0,
                    house_consumption as f64 / 1000.0,
                    input.devices.len()
                );

                if tracing::enabled!(tracing::Level::TRACE) {
                    for (id, evs) in &input.history {
                        if let Some(d) = devices.iter().find(|d| d.id == *id) {
                            let ev_str = evs.iter()
                                .map(|e| format!("{}@{}", match e.state {
                                    lmha_core::DeviceState::On => "ON",
                                    lmha_core::DeviceState::Off => "OFF",
                                    _ => "??",
                                }, e.timestamp.format("%H:%M")))
                                .collect::<Vec<_>>()
                                .join(",");
                            trace!("History for {}: [{}]", d.name, ev_str);
                        }
                    }
                }

                let action = lmha_core::scheduler::decide_action(input);                if action != lmha_core::scheduler::SchedulerAction::Nothing {
                    info!("Scheduler action for {}: {:?}", house.name, action);
                } else {
                    trace!("Scheduler action for {}: {:?}", house.name, action);
                }

                let is_passive = *state.is_passive.lock().unwrap();

                match action {
                    lmha_core::scheduler::SchedulerAction::SwitchOn(id) => {
                        if let Some(device) = devices.iter().find(|d| d.id == id) {
                            if let Err(e) = db.update_device_desired_state(id, DeviceState::On) {
                                error!("DB Error updating desired state (On) for {}: {}", id, e);
                            }
                            if is_passive {
                                debug!("Passive Mode: Skipping SwitchOn for {}", device.name);
                                continue;
                            }
                            let mqtt_client = state.mqtt_client.lock().unwrap();
                            let payload = json!({
                                "id": 1, "src": format!("{}/rpc-response", device.mqtt_topic), "method": "Switch.Set", "params": {"id": 0, "on": true}
                            }).to_string();
                            let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, payload);

                            // Follow-up status poll
                            let poll_payload = json!({
                                "id": 101, "src": format!("{}/rpc-response", device.mqtt_topic), "method": "Shelly.GetStatus"
                            }).to_string();
                            let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, poll_payload);
                        }
                    }
                    lmha_core::scheduler::SchedulerAction::SwitchOff(id) => {
                        if let Some(device) = devices.iter().find(|d| d.id == id) {
                            if let Err(e) = db.update_device_desired_state(id, DeviceState::Off) {
                                error!("DB Error updating desired state (Off) for {}: {}", id, e);
                            }
                            if is_passive {
                                debug!("Passive Mode: Skipping SwitchOff for {}", device.name);
                                continue;
                            }
                            let mqtt_client = state.mqtt_client.lock().unwrap();
                            let payload = json!({
                                "id": 1, "src": format!("{}/rpc-response", device.mqtt_topic), "method": "Switch.Set", "params": {"id": 0, "on": false}
                            }).to_string();
                            let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, payload);

                            // Follow-up status poll
                            let poll_payload = json!({
                                "id": 102, "src": format!("{}/rpc-response", device.mqtt_topic), "method": "Shelly.GetStatus"
                            }).to_string();
                            let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, poll_payload);
                        }
                    }
                    lmha_core::scheduler::SchedulerAction::UpdateScheduling(id, new_type) => {
                        if let Some(device) = devices.iter().find(|d| d.id == id) {
                            info!("Device '{}' ({}) transitioning from {:?} to {:?}", device.name, device.id, device.scheduling_type, new_type);
                            if let Err(e) = db.update_device_scheduling(id, new_type) {
                                error!("DB Error updating scheduling for {}: {}", id, e);
                            }
                        }
                    }
                    lmha_core::scheduler::SchedulerAction::Nothing => {}
                }
            }
        }
        thread::sleep(interval);
    }
}

fn run_instance_heartbeat_loop(state: Arc<AppState>) {
    loop {
        // 1. Publish our heartbeat (Retained)
        {
            let client = state.mqtt_client.lock().unwrap();
            let payload = json!({
                "priority": state.config.instance_priority,
                "status": "online",
                "timestamp": chrono::Utc::now()
            }).to_string();
            // Use retain=true to reduce chatter (instances only need to publish once, but we refresh every 5 min)
            let _ = client.publish(format!("lmha3/instances/{}", state.config.instance_id), QoS::AtMostOnce, true, payload);
        }

        // We no longer check timeout here because we use LWT and explicit state tracking in the MQTT handler.
        thread::sleep(Duration::from_secs(300));
    }
}

fn run_health_check_loop(state: Arc<AppState>) {
    loop {
        thread::sleep(Duration::from_secs(300));
        info!("Running periodic device health check poll...");
        let devices = {
            let mut db = state.db.lock().unwrap();
            db.list_devices(None).unwrap_or_default()
        };

        let now = chrono::Utc::now();
        let is_passive = *state.is_passive.lock().unwrap();
        for d in devices {
            if !d.is_enabled { continue; }
            let last_feedback = d.last_feedback_time.unwrap_or_else(|| now - chrono::Duration::days(365));
            if now - last_feedback > chrono::Duration::seconds(300) {
                if is_passive {
                    debug!("Passive Mode: Skipping Health Check Poll for {}", d.name);
                    continue;
                }
                debug!("Polling inactive device: {}", d.name);
                let topic = format!("{}/rpc", d.mqtt_topic);
                let payload = json!({
                    "id": 99,
                    "src": format!("{}/rpc-response", d.mqtt_topic),
                    "method": "Shelly.GetStatus"
                }).to_string();
                let client = state.mqtt_client.lock().unwrap();
                let _ = client.publish(topic, QoS::AtMostOnce, false, payload);
            }
        }
    }
}

fn run_main_loop(state: Arc<AppState>) {
    loop {
        let client_id = format!("lmha3-server-{}", Uuid::new_v4().to_string().get(0..8).unwrap());
        let mut mqtt_options = MqttOptions::new(&client_id, &state.config.mqtt_host, state.config.mqtt_port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));
        if let (Some(u), Some(p)) = (&state.config.mqtt_user, &state.config.mqtt_password) {
            mqtt_options.set_credentials(u, p);
        }

        // Set Last Will for instance heartbeat
        let lwt_topic = format!("lmha3/instances/{}", state.config.instance_id);
        let lwt_payload = json!({
            "priority": 0,
            "status": "offline",
            "timestamp": chrono::Utc::now()
        }).to_string();
        mqtt_options.set_last_will(LastWill::new(lwt_topic, lwt_payload, QoS::AtMostOnce, true));
        
        let (client, mut connection) = Client::new(mqtt_options, 50);
        {
            let mut shared_client = state.mqtt_client.lock().unwrap();
            *shared_client = client.clone();
        }

        let subs = ["+/online", "+/status/#", "+/events/rpc", "shellies/#", "+/rpc-response/#", "lmha3/instances/+"];
        for sub in subs {
            info!("MQTT: Subscribing to {}", sub);
            let _ = client.subscribe(sub, QoS::AtMostOnce);
        }

        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let topic = publish.topic.clone();
                    let payload_str = String::from_utf8_lossy(&publish.payload);
                    trace!("MQTT Incoming: {} | Payload length: {}", topic, publish.payload.len());

                    let parts: Vec<&str> = topic.split('/').collect();
                    
                    if topic.starts_with("lmha3/instances/") {
                        if let Some(other_id) = parts.get(2) {
                            let other_id = other_id.to_string();
                            if other_id != state.config.instance_id {
                                if publish.payload.is_empty() {
                                    let mut other_instances = state.other_instances.lock().unwrap();
                                    other_instances.remove(&other_id);
                                    info!("Instance {} disappeared (cleared).", other_id);
                                    
                                    let mut is_passive = state.is_passive.lock().unwrap();
                                    let has_higher = other_instances.values().any(|&p| p > state.config.instance_priority);
                                    if !has_higher && *is_passive {
                                        info!("No more high priority instances detected. Resuming control mode.");
                                        *is_passive = false;
                                    }
                                } else if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                                    let priority = val.get("priority").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                    let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("online");
                                    
                                    let mut other_instances = state.other_instances.lock().unwrap();
                                    if status == "offline" || priority == 0 {
                                        other_instances.remove(&other_id);
                                        info!("Instance {} went offline.", other_id);
                                    } else {
                                        other_instances.insert(other_id.clone(), priority);
                                    }

                                    let mut is_passive = state.is_passive.lock().unwrap();
                                    let has_higher = other_instances.values().any(|&p| p > state.config.instance_priority);
                                    
                                    if has_higher && !*is_passive {
                                        error!("!!! BIG FAT WARNING: HIGHER PRIORITY INSTANCE DETECTED !!!");
                                        error!("ENTERING PASSIVE MODE. ACTIVE CONTROL DISABLED.");
                                        *is_passive = true;
                                    } else if !has_higher && *is_passive {
                                        info!("No more high priority instances detected. Resuming control mode.");
                                        *is_passive = false;
                                    }
                                }
                            }
                        }
                    }

                    let (base_topic, is_gen1) = if parts.len() > 1 && parts[0] == "shellies" {
                        (parts[1], true)
                    } else {
                        (parts[0], false)
                    };

                    if topic.ends_with("/online") || topic.contains("/status/") || topic.contains("/rpc-response") {
                        if topic.contains("/rpc-response") {
                            info!("MQTT Heartbeat (RPC Response) for {}: {}", base_topic, topic);
                        }
                        let mut db = state.db.lock().unwrap();
                        let _ = db.update_device_feedback(base_topic);

                        if topic.ends_with("/online") {
                            if payload_str == "false" {
                                info!("Device {} went OFFLINE (LWT). Setting state to UNKNOWN.", base_topic);
                                let _ = db.update_device_state(base_topic, DeviceState::Unknown);
                            } else if payload_str == "true" {
                                info!("Device {} came ONLINE. Triggering immediate status poll.", base_topic);
                                // Force a poll even if we think we are in sync, because our local current_state might be stale
                                let poll_payload = json!({
                                    "id": 104,
                                    "src": format!("{}/rpc-response", base_topic),
                                    "method": "Shelly.GetStatus"
                                }).to_string();
                                let client = state.mqtt_client.lock().unwrap();
                                let _ = client.publish(format!("{}/rpc", base_topic), QoS::AtMostOnce, false, poll_payload);
                            }
                        }

                        // Event-driven sync: if device comes online or sends status, check if sync is needed
                        if let Ok(devices) = db.list_devices(None) {
                            if let Some(d) = devices.iter().find(|d| d.mqtt_topic == base_topic) {
                                if d.desired_state != d.current_state && d.is_enabled {
                                    if *state.is_passive.lock().unwrap() {
                                        debug!("Passive Mode: Skipping Event-driven sync for {}", d.name);
                                    } else {
                                        info!("Event-driven Sync for {}: current={:?}, desired={:?}", d.name, d.current_state, d.desired_state);
                                        let payload = json!({
                                            "id": 1, "src": format!("{}/rpc-response", d.mqtt_topic), "method": "Switch.Set", 
                                            "params": {"id": 0, "on": d.desired_state == DeviceState::On}
                                        }).to_string();
                                        let client = state.mqtt_client.lock().unwrap();
                                        let _ = client.publish(format!("{}/rpc", d.mqtt_topic), QoS::AtMostOnce, false, payload);
                                        
                                        // Follow-up status poll to ensure local state is updated even if NotifyStatus is lost
                                        let poll_payload = json!({
                                            "id": 100,
                                            "src": format!("{}/rpc-response", d.mqtt_topic),
                                            "method": "Shelly.GetStatus"
                                        }).to_string();
                                        let _ = client.publish(format!("{}/rpc", d.mqtt_topic), QoS::AtMostOnce, false, poll_payload);

                                        let _ = db.update_device_request_time(d.id);
                                    }
                                }
                            }
                        }
                    }
                    
                    let mut db = state.db.lock().unwrap();
                    let mut new_state = None;
                    let mut apower = None;
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
                            apower = val.get("apower").and_then(|v| v.as_f64());
                        }
                    } else if topic.contains("/rpc-response") {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                            if let Some(result) = val.get("result") {
                                if let Some(sw) = result.get("switch:0") {
                                    if let Some(on) = sw.get("output").and_then(|v| v.as_bool()) {
                                        new_state = Some(if on { DeviceState::On } else { DeviceState::Off });
                                    }
                                    apower = sw.get("apower").and_then(|v| v.as_f64());
                                }
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
                                        apower = sw.get("apower").and_then(|v| v.as_f64());
                                    }
                                }
                            }
                        }
                    }

                    if new_state.is_some() || apower.is_some() {
                        let devices = db.list_devices(None).unwrap_or_default();
                        if let Some(d) = devices.iter().find(|d| d.mqtt_topic == base_topic) {
                            if let Some(s) = new_state {
                                if s != d.current_state {
                                    info!("MQTT State Update for {}: {:?} -> {:?}", d.name, d.current_state, s);
                                    if let Err(e) = db.update_device_state(base_topic, s) {
                                        error!("DB Error updating {}: {}", base_topic, e);
                                    }
                                    let val = if s == DeviceState::On { 1 } else { 0 };
                                    let _ = db.insert_telemetry(lmha_core::TelemetrySource::DeviceState, Some(d.id), val, None, d.house_id);
                                }
                            }
                            if let Some(p) = apower {
                                let _ = db.insert_telemetry(lmha_core::TelemetrySource::DeviceConsumption, Some(d.id), p as i32, None, d.house_id);
                            }
                        } else {
                            trace!("MQTT: Received state/power for unknown device topic: {}", base_topic);
                        }
                    }
                }
                Ok(_) => {},
                Err(e) => {
                    error!("MQTT Loop Error: {:?}. Reconnecting...", e);
                    break;
                }
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

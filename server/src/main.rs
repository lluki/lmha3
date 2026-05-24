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
use serde::{Serialize};
use serde_json::json;
use std::collections::VecDeque;
use tracing::{info, error};
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
}

fn main() {
    let log_buffer = Arc::new(Mutex::new(LogBuffer::new(1000)));

    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter.clone()))
        .with(BufferLayer { buffer: log_buffer.clone() }.with_filter(filter))
        .init();

    let args = Args::parse();
    if std::env::var("HA_TOKEN").is_err() {
        if let Ok(token) = std::fs::read_to_string("secrets/ha-token.md") {
            std::env::set_var("HA_TOKEN", token.trim());
        }
    }
    let config = Config::from_env();
    let db = Db::connect(&config).expect("Failed to connect to database");
    
    // Auto-run migrations
    info!("Running database migrations...");
    let mut client = postgres::Client::connect(&config.database_url, postgres::NoTls).expect("Failed to connect for migrations");
    let migrations = [
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_add_sessions.sql"),
        include_str!("../../migrations/003_add_device_heartbeat.sql"),
        include_str!("../../migrations/004_add_device_consumption.sql"),
        include_str!("../../migrations/005_add_expected_load.sql"),
    ];
    for migration in migrations {
        client.batch_execute(migration).ok(); // Batch execute will ignore errors if columns already exist
    }

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        config: config.clone(),
        no_home_assistant: args.no_home_assistant,
        no_scheduler: args.no_scheduler,
        mqtt_client: Mutex::new(dummy_client()),
        log_buffer,
    });

    let loop_state = state.clone();
    thread::spawn(move || {
        run_main_loop(loop_state);
    });

    info!("LMHA3 Server Starting on 0.0.0.0:{}...", args.port);
    
    rouille::start_server(format!("0.0.0.0:{}", args.port), move |request| {
        let state = state.clone();
        
        // 1. Try to serve static files from various possible locations
        let public_dir = std::env::var("LMHA3_PUBLIC_DIR").unwrap_or_else(|_| "server/public".to_string());
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
                if let Some(s) = get_user(request, &state) {
                    Response::json(&s)
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/version) => {
                Response::json(&json!({ "version": env!("CARGO_PKG_VERSION") }))
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
                if let Some(s) = get_session(request, &state) {
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
                            error!("DB Error listing tenants: {}", e);
                            Response::text("DB Error").with_status_code(500)
                        }
                    }
                } else {
                    Response::text("Unauthorized").with_status_code(401)
                }
            },

            (GET) (/api/metrics) => {
                if let Some(_) = get_session(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    match db.get_latest_metrics() {
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
                if let Some(_) = get_session(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    // Global Read: All users can see all devices
                    match db.list_devices() {
                        Ok(devices) => Response::json(&devices),
                        Err(e) => {
                            error!("DB Error listing devices: {}", e);
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
                    match db.create_device(tenant_id, &data.mqtt_topic, &data.name) {
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

            (GET) (/api/history) => {
                if let Some(_) = get_user(request, &state) {
                    let mut db = state.db.lock().unwrap();
                    // Global Read: All users see all history unconditionally
                    match db.list_telemetry(None, 100) {
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
                    let devices = db.list_devices().unwrap_or_default();
                    if let Some(device) = devices.into_iter().find(|d| d.id == id && (user.is_admin || d.tenant_id == user.tenant_id)) {
                        let new_on = device.current_state != DeviceState::On;
                        let payload = json!({
                            "id": 1,
                            "src": "lmha3",
                            "method": "Switch.Set",
                            "params": {"id": 0, "on": new_on}
                        });
                        let topic = format!("{}/rpc", device.mqtt_topic);
                        let payload_str = payload.to_string();
                        info!("API: Publishing toggle to topic '{}' | Payload: {}", topic, payload_str);
                        let client = state.mqtt_client.lock().unwrap();
                        let _ = client.publish(topic, QoS::AtLeastOnce, false, payload_str);
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

fn get_user(request: &Request, state: &AppState) -> Option<lmha_core::UserInfo> {
    let session_id = get_session_id(request)?;
    let mut db = state.db.lock().unwrap();
    db.get_user_info(session_id)
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

        let subs = ["+/online", "+/status/#", "+/events/rpc", "shellies/#"];
        for sub in subs {
            info!("MQTT: Subscribing to {}", sub);
            let _ = client.subscribe(sub, QoS::AtMostOnce);
        }

        if !state.no_scheduler {
            let sch_state = state.clone();
            thread::spawn(move || {
                loop {
                    if !sch_state.no_home_assistant {
                        let config = &sch_state.config;
                        if let Some(pv_id) = &config.ha_pv_entity_id {
                            match lmha_core::ha::fetch_ha_state(config, pv_id) {
                                Ok(val) => {
                                    let mut db = sch_state.db.lock().unwrap();
                                    if let Err(e) = db.insert_telemetry(lmha_core::TelemetrySource::PvProduction, None, val, None) {
                                        error!("DB Error saving PV telemetry: {}", e);
                                    } else {
                                        tracing::debug!("Saved PV telemetry: {}", val);
                                    }
                                }
                                Err(e) => error!("HA Error polling PV ({}): {}", pv_id, e),
                            }
                        }
                        if let Some(cons_id) = &config.ha_consumption_entity_id {
                            match lmha_core::ha::fetch_ha_state(config, cons_id) {
                                Ok(val) => {
                                    let mut db = sch_state.db.lock().unwrap();
                                    if let Err(e) = db.insert_telemetry(lmha_core::TelemetrySource::HouseConsumption, None, val, None) {
                                        error!("DB Error saving consumption telemetry: {}", e);
                                    } else {
                                        tracing::debug!("Saved Consumption telemetry: {}", val);
                                    }
                                }
                                Err(e) => error!("HA Error polling Consumption ({}): {}", cons_id, e),
                            }
                        }
                    }
                    thread::sleep(Duration::from_secs(10));
                    
                    // Run Scheduler Logic
                    let mut db = sch_state.db.lock().unwrap();
                    let devices = db.list_devices().unwrap_or_default();
                    let (pv_production, house_consumption) = db.get_latest_metrics().unwrap_or((0.0, 0.0));

                    let input = lmha_core::scheduler::SchedulerInput {
                        pv_production,
                        house_consumption,
                        devices: devices.iter().map(|d| lmha_core::scheduler::DeviceContext {
                            id: d.id,
                            current_state: d.current_state,
                            last_state_change: d.last_heartbeat,
                            is_enabled: d.is_enabled,
                            expected_load: d.expected_load,
                        }).collect(),
                        now: chrono::Utc::now(),
                        debounce_duration_secs: 300,
                        rng: rand::thread_rng(),
                    };

                    info!("Scheduler invoked: PV={:.1}kW, Cons={:.1}kW, Devices={}", pv_production, house_consumption, input.devices.len());
                    tracing::debug!("Scheduler input: {:?}", input);

                    let action = lmha_core::scheduler::decide_action(input);
                    info!("Scheduler action: {:?}", action);

                    match action {
                        lmha_core::scheduler::SchedulerAction::SwitchOn(id) => {
                            if let Some(device) = devices.iter().find(|d| d.id == id) {
                                let mqtt_client = sch_state.mqtt_client.lock().unwrap();
                                let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, "on");
                            }
                        }
                        lmha_core::scheduler::SchedulerAction::SwitchOff(id) => {
                            if let Some(device) = devices.iter().find(|d| d.id == id) {
                                let mqtt_client = sch_state.mqtt_client.lock().unwrap();
                                let _ = mqtt_client.publish(format!("{}/rpc", device.mqtt_topic), rumqttc::QoS::AtMostOnce, false, "off");
                            }
                        }
                        lmha_core::scheduler::SchedulerAction::Nothing => {}
                    }
                }
            });
        }

        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let topic = publish.topic;
                    let payload_str = String::from_utf8_lossy(&publish.payload);
                    info!("MQTT Incoming: {} | Payload: {}", topic, payload_str);

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
                        let devices = db.list_devices().unwrap_or_default();
                        if let Some(d) = devices.iter().find(|d| d.mqtt_topic == base_topic) {
                            if let Some(s) = new_state {
                                if s != d.current_state {
                                    info!("MQTT State Update for {}: {:?} -> {:?}", d.name, d.current_state, s);
                                    if let Err(e) = db.update_device_state(base_topic, s) {
                                        error!("DB Error updating {}: {}", base_topic, e);
                                    }
                                    let val = if s == DeviceState::On { 1.0 } else { 0.0 };
                                    let _ = db.insert_telemetry(lmha_core::TelemetrySource::DeviceState, Some(d.id), val, None);
                                }
                            }
                            if let Some(p) = apower {
                                let _ = db.insert_telemetry(lmha_core::TelemetrySource::DeviceConsumption, Some(d.id), p, None);
                            }
                        } else {
                            info!("MQTT: Received state/power for unknown device topic: {}", base_topic);
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

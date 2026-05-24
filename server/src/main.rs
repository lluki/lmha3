use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::{verify_password, Session};
use rouille::{Request, Response, router};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Disable the background scheduler thread
    #[arg(long)]
    no_scheduler: bool,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
}

struct AppState {
    db: Mutex<Db>,
    config: Config,
}

fn main() {
    let args = Args::parse();
    let config = Config::from_env();
    let db = Db::connect(&config).expect("Failed to connect to database");
    
    let state = Arc::new(AppState {
        db: Mutex::new(db),
        config,
    });

    if !args.no_scheduler {
        let scheduler_state = state.clone();
        thread::spawn(move || {
            run_scheduler_loop(scheduler_state);
        });
    } else {
        println!("Scheduler thread disabled via --no-scheduler");
    }

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
                    html.push_str("</ul><h2>Devices</h2><table border='1'><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Action</th></tr>");
                    for d in devices {
                        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td>", d.name, d.tenant_id, d.mqtt_topic, d.current_state));
                        
                        // Only show toggle for owner
                        if d.tenant_id == s.tenant_id {
                            html.push_str(&format!("<td><form method='POST' action='/devices/{}/toggle'><button type='submit'>Toggle</button></form></td>", d.id));
                        } else {
                            html.push_str("<td>-</td>");
                        }
                        html.push_str("</tr>");
                    }
                    html.push_str("</table>");
                    html.push_str("<br><form method='POST' action='/logout'><button type='submit'>Logout</button></form>");
                    
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
                        println!("Manual toggle triggered for device: {}", device.name);
                        // TODO: Implement actual MQTT toggle logic
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

fn run_scheduler_loop(state: Arc<AppState>) {
    println!("Scheduler background thread started.");
    loop {
        println!("Background Polling Home Assistant at {}...", state.config.ha_url);
        // TODO: Implement HA polling and MQTT logic
        thread::sleep(Duration::from_secs(300));
    }
}

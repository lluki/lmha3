use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::{verify_password, Session};
use rouille::{Request, Response, router};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct AppState {
    db: Mutex<Db>,
}

fn main() {
    let config = Config::from_env();
    let db = Db::connect(&config).expect("Failed to connect to database");
    
    let state = Arc::new(AppState {
        db: Mutex::new(db),
    });

    println!("LMHA3 API Starting on 0.0.0.0:8000...");
    
    rouille::start_server("0.0.0.0:8000", move |request| {
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
                    html.push_str("</ul><h2>Devices</h2><table border='1'><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th></tr>");
                    for d in devices {
                        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>", d.name, d.tenant_id, d.mqtt_topic, d.current_state));
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

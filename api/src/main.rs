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
                    Response::text(format!("Welcome tenant_id: {}", s.tenant_id))
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
                        return Response::text(format!("Welcome tenant_id: {}", tenant.id))
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

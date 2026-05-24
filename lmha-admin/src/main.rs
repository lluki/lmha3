use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::hash_password;
use std::io::{self, Write};
use rpassword::read_password;

fn main() {
    let config = Config::from_env();
    let mut db = Db::connect(&config).expect("Failed to connect to database");

    print!("Enter username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    print!("Enter password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    print!("Confirm password: ");
    io::stdout().flush().unwrap();
    let confirm = read_password().unwrap();

    if password != confirm {
        eprintln!("Passwords do not match");
        return;
    }

    let hashed = hash_password(&password).expect("Failed to hash password");
    
    match db.create_tenant(username, &hashed) {
        Ok(id) => println!("Tenant created successfully with ID: {}", id),
        Err(e) => eprintln!("Failed to create tenant: {}", e),
    }
}

use lmha_core::config::Config;
use lmha_core::db::Db;
use lmha_core::hash_password;
use std::io::{self, Write};
use rpassword::read_password;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Username to create
    #[arg(short, long)]
    username: Option<String>,

    /// Password for the user (if omitted, will prompt)
    #[arg(short, long)]
    password: Option<String>,

    /// Give admin privileges
    #[arg(short, long)]
    admin: bool,
}

fn main() {
    let args = Args::parse();
    let config = Config::from_env();
    let mut db = Db::connect(&config).expect("Failed to connect to database");

    let migrations_dir = std::env::var("LMHA_MIGRATIONS_DIR").unwrap_or_else(|_| "migrations".to_string());
    if let Err(e) = db.run_migrations(&migrations_dir) {
        eprintln!("Warning: Failed to run migrations: {}", e);
    }

    if let Err(e) = db.ensure_seeded() {
        eprintln!("Warning: Failed to ensure database is seeded: {}", e);
    }

    let username = match args.username {
        Some(u) => u,
        None => {
            print!("Enter username: ");
            io::stdout().flush().unwrap();
            let mut u = String::new();
            io::stdin().read_line(&mut u).unwrap();
            u.trim().to_string()
        }
    };

    let password = match args.password {
        Some(p) => p,
        None => {
            print!("Enter password: ");
            io::stdout().flush().unwrap();
            let p = read_password().unwrap();
            
            print!("Confirm password: ");
            io::stdout().flush().unwrap();
            let confirm = read_password().unwrap();

            if p != confirm {
                eprintln!("Passwords do not match");
                return;
            }
            p
        }
    };

    let hashed = hash_password(&password).expect("Failed to hash password");
    
    let houses = db.list_houses().expect("Failed to list houses");
    if houses.is_empty() {
        eprintln!("No houses found in database. Please create a house first.");
        return;
    }
    let house_id = houses[0].id; // Use first house as default

    match db.create_tenant(&username, &hashed, house_id, args.admin) {
        Ok(id) => println!("Tenant '{}' created successfully with ID: {}", username, id),
        Err(e) => eprintln!("Failed to create tenant: {}", e),
    }
}

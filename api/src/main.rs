use lmha_core::config::Config;

fn main() {
    let config = Config::from_env();
    println!("LMHA3 API Starting on 0.0.0.0:8000...");
    
    rouille::start_server("0.0.0.0:8000", move |_request| {
        rouille::Response::text("LMHA3 API MVP")
    });
}

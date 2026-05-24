use lmha_core::config::Config;
use std::thread;
use std::time::Duration;

fn main() {
    let config = Config::from_env();
    println!("LMHA3 Scheduler Starting...");
    
    loop {
        println!("Polling Home Assistant at {} and checking loads...", config.ha_url);
        thread::sleep(Duration::from_secs(300)); // 5 minutes
    }
}

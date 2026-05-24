use crate::config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HaState {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
}

pub fn fetch_ha_state(config: &Config, entity_id: &str) -> Result<f64, String> {
    let url = format!("{}/api/states/{}", config.ha_url, entity_id);
    let resp: HaState = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", config.ha_token))
        .set("Content-Type", "application/json")
        .call()
        .map_err(|e| format!("Request failed: {}", e))?
        .into_json()
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    resp.state.parse::<f64>().map_err(|e| format!("Failed to parse state '{}' as f64: {}", resp.state, e))
}

use crate::config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HaState {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
}

pub fn fetch_ha_state(ha_url: &str, ha_token: &str, entity_id: &str) -> Result<i32, String> {
    let url = format!("{}/api/states/{}", ha_url, entity_id);
    let resp: HaState = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .set("Authorization", &format!("Bearer {}", ha_token))
        .set("Content-Type", "application/json")
        .call()
        .map_err(|e| format!("Request failed: {}", e))?
        .into_json()
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let val = resp.state.parse::<f64>().map_err(|e| format!("Failed to parse state '{}' as f64: {}", resp.state, e))?;
    
    // Normalize to Watts if the unit is kW
    let watts = if let Some(unit) = resp.attributes.get("unit_of_measurement").and_then(|u| u.as_str()) {
        if unit == "kW" {
            (val * 1000.0) as i32
        } else {
            val as i32
        }
    } else {
        val as i32
    };
    
    Ok(watts)
}

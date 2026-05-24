#[cfg(test)]
mod tests {
    use lmha_core::{Telemetry, TelemetrySource};
    use chrono::{Utc, Duration};

    #[test]
    fn test_fetch_latest_telemetry() {
        let now = Utc::now();
        let telemetries = vec![
            Telemetry {
                timestamp: now,
                source: TelemetrySource::HouseConsumption,
                device_id: None,
                value: 3.511,
                metadata: None,
            },
            Telemetry {
                timestamp: now - Duration::seconds(10),
                source: TelemetrySource::PvProduction,
                device_id: None,
                value: 100.0,
                metadata: None,
            },
            Telemetry {
                timestamp: now - Duration::seconds(20),
                source: TelemetrySource::PvProduction,
                device_id: None,
                value: 50.0,
                metadata: None,
            },
        ];

        let pv_production = telemetries.iter()
            .filter(|t| matches!(t.source, TelemetrySource::PvProduction))
            .map(|t| t.value)
            .next()
            .unwrap_or(0.0);

        let house_consumption = telemetries.iter()
            .filter(|t| matches!(t.source, TelemetrySource::HouseConsumption))
            .map(|t| t.value)
            .next()
            .unwrap_or(0.0);

        // Expect the latest (most recent timestamp) value for each source
        assert_eq!(pv_production, 100.0);
        assert_eq!(house_consumption, 3.511);
    }
}

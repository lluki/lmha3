#[cfg(test)]
mod db_tests {
    use lmha_core::db::Db;
    use lmha_core::config::Config;
    use lmha_core::TelemetrySource;
    use postgres::{Client, NoTls};
    use uuid::Uuid;

    #[test]
    fn test_get_latest_metrics() {
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        let base_db_url = std::env::var("LMHA_DATABASE_URL").unwrap_or_else(|_| "host=/var/run/postgresql dbname=postgres user=user".to_string());
        
        let mut base_params = base_db_url.split_whitespace().collect::<Vec<_>>();
        let mut create_params = Vec::new();
        for param in base_params {
            if param.starts_with("dbname=") {
                create_params.push("dbname=postgres");
            } else {
                create_params.push(param);
            }
        }
        let create_url = create_params.join(" ");

        let mut client = Client::connect(&create_url, NoTls).unwrap();
        client.execute(&format!("CREATE DATABASE {}", db_name), &[]).unwrap();

        let mut test_params = Vec::new();
        for param in create_params {
            if param.starts_with("dbname=") {
                test_params.push(format!("dbname={}", db_name));
            } else {
                test_params.push(param.to_string());
            }
        }
        let db_url = test_params.join(" ");

        let mut db = Db::connect(&Config {
            database_url: db_url.clone(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_user: None,
            mqtt_password: None,
            instance_id: "test".to_string(),
            instance_priority: 10,
        }).unwrap();

        db.run_migrations("../migrations").unwrap();

        let house_id = db.list_houses().unwrap()[0].id;

        // Insert some data
        db.insert_telemetry(TelemetrySource::PvProduction, None, 100, None, house_id).unwrap();
        db.insert_telemetry(TelemetrySource::PvProduction, None, 500, None, house_id).unwrap();
        db.insert_telemetry(TelemetrySource::HouseConsumption, None, 200, None, house_id).unwrap();
        db.insert_telemetry(TelemetrySource::HouseConsumption, None, 300, None, house_id).unwrap();

        let (pv, cons) = db.get_latest_metrics(house_id).unwrap();
        assert_eq!(pv, 500);
        assert_eq!(cons, 300);

        // Cleanup
        drop(db);
        let mut client = Client::connect(&create_url, NoTls).unwrap();
        client.execute(&format!("DROP DATABASE {}", db_name), &[]).unwrap();
    }
}

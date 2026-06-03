use lmha_core::ha::fetch_ha_state;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn test_ha_fetch_plain_http() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let n = stream.read(&mut buf).unwrap();
            let _request = String::from_utf8_lossy(&buf[..n]);
            
            // Check if it's a plain HTTP request or TLS handshake
            // TLS handshake starts with 0x16 (22)
            if buf[0] == 0x16 {
                panic!("Received TLS handshake on plain HTTP port!");
            }

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"entity_id\": \"test\", \"state\": \"100\", \"attributes\": {}}";
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    let result = fetch_ha_state(&url, "token", "sensor.test");
    assert!(result.is_ok(), "Result should be OK, got: {:?}", result.err());
    assert_eq!(result.unwrap(), 100);
}

pub mod http;


#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, Instant};

    #[test]
    fn test_http_endpoints_and_latency() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();

        let mut cfg = dlna_config::AppConfig::default();
        cfg.port = port;
        let cfg = Arc::new(cfg);
        let index = Arc::new(RwLock::new(dlna_media_library::MediaIndex::default()));
        let running = Arc::new(AtomicBool::new(true));

        let run_c = Arc::clone(&running);
        let handle = std::thread::spawn(move || http::run(listener, cfg, index, run_c));

        std::thread::sleep(Duration::from_millis(50));

        let t0 = Instant::now();
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        s.write_all(b"GET /device.xml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        let elapsed = t0.elapsed();
        let text = String::from_utf8_lossy(&resp);
        assert!(text.contains("200 OK"), "device.xml failed: {text}");
        assert!(text.contains("MediaServer:1"), "missing MediaServer:1");
        assert!(elapsed < Duration::from_millis(100), "too slow: {elapsed:?}");

        let mut s2 = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s2.write_all(b"GET /icon.png HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut icon_resp = Vec::new();
        s2.read_to_end(&mut icon_resp).unwrap();
        assert!(String::from_utf8_lossy(&icon_resp).contains("200 OK"), "icon.png failed");
        assert!(icon_resp.len() > 500, "icon.png empty");

        running.store(false, Ordering::SeqCst);
        let _ = handle.join();
    }
}

#[cfg(test)]
mod chaos_stress_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    #[test]
    fn test_20_concurrent_clients_and_malformed_requests() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();

        let mut cfg = dlna_config::AppConfig::default();
        cfg.port = port;
        let cfg = Arc::new(cfg);
        let index = Arc::new(RwLock::new(dlna_media_library::MediaIndex::default()));
        let running = Arc::new(AtomicBool::new(true));

        let run_c = Arc::clone(&running);
        let handle = std::thread::spawn(move || http::run(listener, cfg, index, run_c));
        std::thread::sleep(Duration::from_millis(50));

        let mut workers = Vec::new();
        for i in 0..20 {
            workers.push(std::thread::spawn(move || {
                let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
                s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let req = match i % 4 {
                    0 => b"GET /device.xml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".as_slice(),
                    1 => b"GET /media/non_existent_id HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes=0-1024\r\nConnection: close\r\n\r\n".as_slice(),
                    2 => b"GARBAGE_BROKEN_CLIENT_PACKET\r\n\r\n".as_slice(),
                    _ => b"POST /cds/control HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
                };
                let _ = s.write_all(req);
                let mut buf = Vec::new();
                let _ = s.read_to_end(&mut buf);
            }));
        }

        for w in workers {
            w.join().expect("Worker thread panicked under load");
        }

        // Проверяем, что после шторма сервер остался жив и отвечает 200 OK
        let mut check = TcpStream::connect(("127.0.0.1", port)).unwrap();
        check.write_all(b"GET /device.xml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut ok_buf = Vec::new();
        check.read_to_end(&mut ok_buf).unwrap();
        assert!(String::from_utf8_lossy(&ok_buf).contains("200 OK"), "Server degraded after stress test");

        running.store(false, Ordering::SeqCst);
        let _ = handle.join();
    }
}

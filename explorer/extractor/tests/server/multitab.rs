use super::*;
use std::sync::{Arc, Barrier};

fn open(server: &Server) -> (String, BufReader<TcpStream>) {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    write!(
        stream,
        "GET /api/events HTTP/1.1\r\nHost: localhost\r\n\r\n"
    )
    .unwrap();
    let mut reader = BufReader::new(stream);
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if line == "\r\n" {
            break;
        }
        headers.push_str(&line);
    }
    assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(headers.contains("Content-Type: text/event-stream\r\n"));
    (headers, reader)
}

fn overflow(headers: &str, reader: &mut impl Read, version: u64) {
    assert!(
        headers.contains("Connection: close\r\n"),
        "Overflow must release its socket: {headers}"
    );
    assert!(headers.contains("Cache-Control: no-store\r\n"));
    assert!(headers.contains("Access-Control-Allow-Origin: *\r\n"));
    let length: usize = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .unwrap()
        .parse()
        .unwrap();
    let expected = format!("retry: 1000\ndata: {{\"version\":{version}}}\n\n");
    assert_eq!(length, expected.len());
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    assert_eq!(body, expected.as_bytes());
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap(),
        0,
        "Overflow response must reach EOF"
    );
}

#[test]
fn multitab_overflow_closes_catches_up_and_promotes_after_disconnect() {
    let server = Server::start();
    let mut held = Vec::new();
    let mut version = 0;
    for _ in 0..5 {
        let (headers, mut reader) = open(&server);
        assert!(headers.contains("Connection: keep-alive\r\n"));
        let initial = next_version(&mut reader);
        if version == 0 {
            version = initial;
        }
        assert_eq!(initial, version);
        held.push(reader);
    }
    for _ in 0..3 {
        let (headers, mut reader) = open(&server);
        overflow(&headers, &mut reader, version);
    }
    std::thread::sleep(Duration::from_millis(600));
    server.config("project", &config("Overflow catch-up"));
    let next = next_version(&mut held[0]);
    assert!(next > version);
    for reader in &mut held[1..] {
        assert_eq!(next_version(reader), next);
    }
    let (headers, mut reader) = open(&server);
    overflow(&headers, &mut reader, next);
    let released = held.pop().unwrap();
    released.get_ref().shutdown(Shutdown::Both).unwrap();
    drop(released);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let (headers, mut reader) = open(&server);
        if headers.contains("Connection: keep-alive\r\n") {
            assert_eq!(next_version(&mut reader), next);
            held.push(reader);
            break;
        }
        overflow(&headers, &mut reader, next);
        assert!(
            Instant::now() < deadline,
            "Disconnected viewer did not release capacity"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let (headers, mut reader) = open(&server);
    overflow(&headers, &mut reader, next);
    for reader in held {
        reader.get_ref().shutdown(Shutdown::Both).unwrap();
    }
}

#[test]
fn multitab_concurrent_admission_never_exceeds_five_persistent_streams() {
    let server = Arc::new(Server::start());
    let barrier = Arc::new(Barrier::new(12));
    let workers: Vec<_> = (0..12)
        .map(|_| {
            let server = Arc::clone(&server);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                open(&server)
            })
        })
        .collect();
    let mut held = Vec::new();
    for worker in workers {
        let (headers, mut reader) = worker.join().unwrap();
        if headers.contains("Connection: keep-alive\r\n") {
            assert_eq!(next_version(&mut reader), 1);
            held.push(reader);
        } else {
            overflow(&headers, &mut reader, 1);
        }
    }
    assert_eq!(
        held.len(),
        5,
        "Concurrent requests exceeded persistent capacity"
    );
    for reader in held {
        reader.get_ref().shutdown(Shutdown::Both).unwrap();
    }
}

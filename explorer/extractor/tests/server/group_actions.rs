use super::*;

fn action_request(server: &Server, extra: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let request = format!(
        "POST /api/group-actions HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\n{}\r\n{}",
        server.port,
        body.len(),
        extra,
        body
    );
    stream.write_all(request.as_bytes()).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    // A rejected request can close with unread request bytes and reset the
    // socket on Linux. Read the complete framed response, not through EOF.
    let mut reader = BufReader::new(stream);
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
        assert!(!line.is_empty(), "Incomplete response headers");
        headers.push_str(&line);
    }
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .unwrap()
        .parse()
        .unwrap();
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    format!("{headers}\r\n{}", String::from_utf8(body).unwrap())
}

#[test]
fn group_actions_require_same_origin_json_and_reject_electrical_edits() {
    let server = Server::start();
    assert_eq!(server.request("/api/group-actions").0, 422);
    let result = action_request(&server, "Content-Type: application/json\r\nX-CoHDL-Group-Action: 1\r\nOrigin: https://external.example\r\n", "{\"action\":\"focus\",\"run\":\"missing\"}");
    assert!(result.contains("Same-origin"));
    let result = action_request(&server, "Content-Type: application/json\r\n", "{}");
    assert!(result.contains("Same-origin"));
    let result = action_request(
        &server,
        "Content-Type: application/json\r\nX-CoHDL-Group-Action: 1\r\n",
        "{\"action\":\"connect\"}",
    );
    assert!(result.contains("Invalid display group action"));
    let result = action_request(
        &server,
        "Content-Type: application/json\r\nX-CoHDL-Group-Action: 1\r\n",
        "{\"action\":\"focus\",\"run\":\"../escape\"}",
    );
    assert!(result.contains("Invalid group run ID"));
}

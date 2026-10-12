use super::*;

fn action_request(server: &Server, extra: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "POST /api/group-actions HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\n{}\r\n{}",
        server.port,
        body.len(),
        extra,
        body
    )
    .unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let mut result = String::new();
    stream.read_to_string(&mut result).unwrap();
    result
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

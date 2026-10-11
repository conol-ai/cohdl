use super::*;

fn documents(server: &Server) -> Value {
    std::fs::create_dir_all(server.root.join("project/explorer")).unwrap();
    std::fs::create_dir_all(server.root.join("project/docs")).unwrap();
    std::fs::write(
        server.root.join("project/docs/support.md"),
        "# Support\nLiteral evidence\n",
    )
    .unwrap();
    let model: Value = serde_json::from_str(&server.request("/api/model").2).unwrap();
    let instance = model["instances"][0]["path"].clone();
    json!({"schema":1,"documents":[{
        "id":"support","title":"Supporting evidence","kind":"markdown","path":"docs/support.md",
        "revision":null,"sha256":cohdl::hash::sha256_hex(b"# Support\nLiteral evidence\n"),"pages":null,
        "review":"unreviewed","uncertainty":"Original PDF unavailable",
        "applicability":[{"instances":[instance],"groups":[],"relation":"exact","section":"Support","facts":{"mpn":""}}]
    }],"constraints":[]})
}

fn registry(server: &Server, value: &Value) {
    std::fs::write(
        server.root.join("project/explorer/documents.json"),
        value.to_string(),
    )
    .unwrap();
}

#[test]
fn document_api_derives_local_hash_and_preserves_literal_evidence() {
    let server = Server::start();
    assert_eq!(server.request("/api/documents").0, 404);
    let mut value = documents(&server);
    let mut missing = value["documents"][0].clone();
    missing["id"] = json!("original");
    missing["kind"] = json!("pdf");
    missing["path"] = Value::Null;
    missing["applicability"][0]
        .as_object_mut()
        .unwrap()
        .remove("section");
    missing["pages"] = json!(10);
    missing["applicability"][0]["page"] = json!(3);
    value["documents"].as_array_mut().unwrap().push(missing);
    registry(&server, &value);
    let response = server.request("/api/documents");
    assert_eq!(response.0, 200, "{response:?}");
    let body: Value = serde_json::from_str(&response.2).unwrap();
    let support = &body["documents"][0];
    assert_eq!(support["status"], "ready");
    assert_eq!(support["actual_sha256"], value["documents"][0]["sha256"]);
    assert_eq!(support["applicability"][0]["facts"]["mpn"], "");
    assert_eq!(support["revision"], Value::Null);
    assert_eq!(
        server.request(support["open_url"].as_str().unwrap()).2,
        "# Support\nLiteral evidence\n"
    );
    assert_eq!(body["documents"][1]["status"], "missing");
    assert_eq!(body["documents"][1]["actual_sha256"], Value::Null);
    assert_eq!(body["documents"][1]["open_url"], Value::Null);
    assert_eq!(server.request("/api/documents/original").0, 404);
    std::fs::write(server.root.join("project/docs/support.md"), "changed").unwrap();
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "invalid");
    assert_ne!(
        body["documents"][0]["actual_sha256"],
        support["actual_sha256"]
    );
    assert_eq!(body["documents"][0]["open_url"], Value::Null);
    assert_eq!(server.request("/api/documents/support").0, 404);
}

#[test]
fn document_schema_errors_do_not_change_circuit() {
    let server = Server::start();
    let baseline = server.request("/api/model").2;
    let value = documents(&server);
    for (field, bad) in [
        ("schema", json!(2)),
        ("documents", json!({})),
        ("extra", json!(true)),
    ] {
        let mut invalid = value.clone();
        invalid[field] = bad;
        registry(&server, &invalid);
        assert_eq!(server.request("/api/documents").0, 422, "{invalid}");
        assert_eq!(server.request("/api/model").2, baseline);
    }
    for (field, bad) in [
        ("review", json!("verified")),
        ("revision", json!("")),
        ("pages", json!(0)),
        ("sha256", json!("bad")),
    ] {
        let mut invalid = value.clone();
        invalid["documents"][0][field] = bad;
        registry(&server, &invalid);
        let response = server.request("/api/documents");
        assert!(
            response.0 == 422
                || serde_json::from_str::<Value>(&response.2).unwrap()["documents"][0]["status"]
                    == "invalid",
            "{field}: {response:?}"
        );
    }
    registry(&server, &value);
    assert_eq!(server.request("/api/documents").0, 200);
}

#[test]
fn source_api_decodes_once_and_bounds_actual_cohdl_lines() {
    let server = Server::start();
    let response = server.request("/api/source?path=src%2fmain.cohdl&line=1&count=2");
    assert_eq!(response.0, 200, "{response:?}");
    let body: Value = serde_json::from_str(&response.2).unwrap();
    let source = include_str!("../fixtures/parameterized/src/main.cohdl");
    assert_eq!(body["sha256"], cohdl::hash::sha256_hex(source.as_bytes()));
    assert_eq!(
        body["text"],
        source.lines().take(2).collect::<Vec<_>>().join("\n")
    );
    for query in [
        "path=../secret.json",
        "path=%2e%2e%2fsecret.json",
        "path=%252e%252e%252fsecret.cohdl",
        "path=src%2fmain.cohdl&line=0",
        "path=src%2fmain.cohdl&count=201",
        "path=src%2fmain.cohdl&line=18446744073709551615",
        "path=src%2fmain.cohdl&count=-1",
        "path=src%2fmain.cohdl&line=1&line=2",
        "path=%FF",
        "path=%00",
        "path=/etc/passwd",
        "path=src/main.cohdl&unexpected=1",
    ] {
        assert_ne!(
            server.request(&format!("/api/source?{query}")).0,
            200,
            "{query}"
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            server.root.join("secret.json"),
            server.root.join("project/src/escape.cohdl"),
        )
        .unwrap();
        assert_ne!(server.request("/api/source?path=src/escape.cohdl").0, 200);
    }
}

#[test]
fn registry_and_document_changes_emit_sse_without_reextracting() {
    let server = Server::start();
    let baseline = server.request("/api/model").2;
    let value = documents(&server);
    let mut events = server.events();
    let mut version = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    registry(&server, &value);
    let next = next_version(&mut events);
    assert!(next > version);
    version = next;
    let path = server.root.join("project/docs/support.md");
    let original = std::fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed[0] = b'!';
    std::fs::write(&path, changed).unwrap();
    let next = next_version(&mut events);
    assert!(next > version);
    version = next;
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "invalid");
    std::fs::remove_file(&path).unwrap();
    let next = next_version(&mut events);
    assert!(next > version);
    version = next;
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "missing");
    std::fs::write(&path, original).unwrap();
    assert!(next_version(&mut events) > version);
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "ready");
    assert_eq!(server.request("/api/model").2, baseline);
}

#[test]
fn rejected_review_disables_document_open() {
    let server = Server::start();
    let mut value = documents(&server);
    value["documents"][0]["review"] = json!("rejected");
    registry(&server, &value);
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "invalid");
    assert_eq!(body["documents"][0]["open_url"], Value::Null);
}

#[test]
fn document_paths_citations_and_targets_never_gain_false_validity() {
    let server = Server::start();
    let value = documents(&server);
    for path in [
        "../secret.md",
        "/etc/secret.md",
        "docs\\support.md",
        ".env",
        "docs/support.pdf",
    ] {
        let mut invalid = value.clone();
        invalid["documents"][0]["path"] = json!(path);
        registry(&server, &invalid);
        let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
        assert_eq!(body["documents"][0]["open_url"], Value::Null, "{path}");
        assert_eq!(server.request("/api/documents/support").0, 404);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            server.root.join("secret.json"),
            server.root.join("project/docs/escape.md"),
        )
        .unwrap();
        let mut invalid = value.clone();
        invalid["documents"][0]["path"] = json!("docs/escape.md");
        registry(&server, &invalid);
        assert_eq!(server.request("/api/documents/support").0, 404);
    }
    for field in ["page", "instances", "groups"] {
        let mut invalid = value.clone();
        invalid["documents"][0]["applicability"][0][field] = match field {
            "page" => json!(1),
            "instances" => json!(["Board::invented"]),
            _ => json!([{"view":"Unknown","region":"Core"}]),
        };
        registry(&server, &invalid);
        let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
        assert_eq!(body["documents"][0]["status"], "invalid", "{field}");
    }
    let mut pdf = value.clone();
    pdf["documents"][0]["kind"] = json!("pdf");
    pdf["documents"][0]["path"] = Value::Null;
    pdf["documents"][0]["pages"] = json!(5);
    pdf["documents"][0]["applicability"][0]["page"] = json!(6);
    registry(&server, &pdf);
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert!(body["documents"][0]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("Citation page")));
    pdf["documents"][0]["pages"] = Value::Null;
    registry(&server, &pdf);
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert!(!body["documents"][0]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("Citation page")));
}

#[test]
fn absent_applicability_or_insecure_source_url_cannot_be_opened() {
    let server = Server::start();
    let value = documents(&server);
    for (field, bad) in [
        ("applicability", json!([])),
        ("source_url", json!("http://example.com/manual.pdf")),
        ("source_url", json!("https://")),
    ] {
        let mut invalid = value.clone();
        invalid["documents"][0][field] = bad;
        registry(&server, &invalid);
        let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
        assert_eq!(
            body["documents"][0]["status"], "invalid",
            "{field}: {invalid}"
        );
    }
}

#[test]
fn document_and_source_size_limits_fail_closed() {
    let server = Server::start();
    let value = documents(&server);
    registry(&server, &value);
    std::fs::OpenOptions::new()
        .write(true)
        .open(server.root.join("project/docs/support.md"))
        .unwrap()
        .set_len(4 * 1024 * 1024 + 1)
        .unwrap();
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "invalid");
    assert_eq!(body["documents"][0]["actual_sha256"], Value::Null);
    let mut pdf = value.clone();
    pdf["documents"][0]["kind"] = json!("pdf");
    pdf["documents"][0]["path"] = json!("docs/large.pdf");
    pdf["documents"][0]["applicability"][0]
        .as_object_mut()
        .unwrap()
        .remove("section");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(server.root.join("project/docs/large.pdf"))
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    registry(&server, &pdf);
    let body: Value = serde_json::from_str(&server.request("/api/documents").2).unwrap();
    assert_eq!(body["documents"][0]["status"], "invalid");
    assert_eq!(body["documents"][0]["actual_sha256"], Value::Null);
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(server.root.join("project/src/large.cohdl"))
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    assert_eq!(server.request("/api/source?path=src/large.cohdl").0, 422);
    std::fs::write(
        server.root.join("project/src/wide.cohdl"),
        "a".repeat(64 * 1024 + 1),
    )
    .unwrap();
    assert_eq!(
        server.request("/api/source?path=src/wide.cohdl&count=1").0,
        422
    );
    std::fs::OpenOptions::new()
        .write(true)
        .open(server.root.join("project/explorer/documents.json"))
        .unwrap()
        .set_len(4 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(server.request("/api/documents").0, 422);
}

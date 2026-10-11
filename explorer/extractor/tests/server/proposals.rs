use super::*;

fn proposal_directory(server: &Server, id: &str) -> PathBuf {
    let directory = server.root.join("project/explorer/proposals").join(id);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn proposal_api_empty_and_invalid_runs_are_explicit_and_read_only() {
    let server = Server::start();
    let model = server.request("/api/model").2;
    let response = server.request("/api/proposals");
    assert_eq!(response.0, 200);
    assert_eq!(
        serde_json::from_str::<Value>(&response.2).unwrap(),
        json!({"schema":1,"runs":[]})
    );
    let directory = proposal_directory(&server, "broken");
    std::fs::write(directory.join("run.json"), "{broken").unwrap();
    let body: Value = serde_json::from_str(&server.request("/api/proposals").2).unwrap();
    assert_eq!(body["runs"][0]["id"], "broken");
    assert_eq!(body["runs"][0]["status"], "rejected");
    assert_eq!(body["runs"][0]["freshness"], "unknown");
    assert!(!body["runs"][0]["issues"].as_array().unwrap().is_empty());
    assert_eq!(server.request("/api/proposals/broken").0, 200);
    for path in [
        "/api/proposals/missing",
        "/api/proposals/%2e%2e",
        "/api/proposals/broken/apply",
        "/api/proposals/broken/artifacts/run.json",
    ] {
        assert_eq!(server.request(path).0, 404, "{path}");
    }
    assert_eq!(server.request("/api/model").2, model);
}

#[test]
fn proposal_artifacts_are_allowlisted_bounded_and_never_follow_symlinks() {
    let server = Server::start();
    let directory = proposal_directory(&server, "logs");
    std::fs::write(directory.join("provider.stdout.log"), "actual retained log").unwrap();
    assert_eq!(
        server
            .request("/api/proposals/logs/artifacts/provider.stdout.log")
            .2,
        "actual retained log"
    );
    std::fs::OpenOptions::new()
        .write(true)
        .open(directory.join("provider.stdout.log"))
        .unwrap()
        .set_len(2 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(
        server
            .request("/api/proposals/logs/artifacts/provider.stdout.log")
            .0,
        404
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            server.root.join("secret.json"),
            directory.join("provider.stderr.log"),
        )
        .unwrap();
        assert_eq!(
            server
                .request("/api/proposals/logs/artifacts/provider.stderr.log")
                .0,
            404
        );
        std::os::unix::fs::symlink(&directory, directory.parent().unwrap().join("alias")).unwrap();
        assert_eq!(server.request("/api/proposals/alias").0, 404);
    }
}

#[test]
fn proposal_record_changes_emit_sse_without_model_changes() {
    let server = Server::start();
    let model = server.request("/api/model").2;
    let mut events = server.events();
    let previous = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    let directory = proposal_directory(&server, "pending");
    std::fs::write(directory.join("run.json"), "{}").unwrap();
    assert!(next_version(&mut events) > previous);
    assert_eq!(server.request("/api/model").2, model);
}

fn save(directory: &std::path::Path, name: &str, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    std::fs::write(directory.join(name), &bytes).unwrap();
    cohdl::hash::sha256_hex(&bytes)
}

fn retained_run(server: &Server, id: &str) -> (PathBuf, Value, Value, Value, Value) {
    let directory = proposal_directory(server, id);
    let model: Value = serde_json::from_str(&server.request("/api/model").2).unwrap();
    let source = std::fs::read_to_string(server.root.join("project/src/main.cohdl")).unwrap();
    let config = std::fs::read_to_string(server.root.join("project/cohdl.toml")).unwrap();
    let source_hash = cohdl::hash::sha256_hex(source.as_bytes());
    let config_hash = cohdl::hash::sha256_hex(config.as_bytes());
    let snapshot = json!([
        {"path":"cohdl.lock","sha256":null},{"path":"cohdl.toml","sha256":config_hash},
        {"path":"design.lock","sha256":null},{"path":"explorer/documents.json","sha256":null},
        {"path":"src/main.cohdl","sha256":source_hash},{"path":"views/Board.view.json","sha256":null}
    ]);
    let bundle = json!({"schema":1,"project":server.root.join("project").canonicalize().unwrap(),"design":"Board",
        "selection":{"schema":1,"instances":[],"groups":[]},"snapshot":snapshot,
        "sources":[{"path":"cohdl.toml","sha256":config_hash,"text":config},{"path":"src/main.cohdl","sha256":source_hash,"text":source}],
        "model":model,"views":null,"registry":null,"documents":[],"constraints":[],"omissions":[]});
    let proposal = json!({"schema":1,"kind":"connect","title":"Disposable HTTP regression proposal","rationale":"Explicit fixture only","uncertainty":"Not real AI",
        "citations":[{"document":null,"page":null,"section":null,"source":{"path":"src/main.cohdl","line":1,"sha256":source_hash}}],
        "changes":[{"path":"src/main.cohdl","before_sha256":source_hash,"old":"name never used","new":""}]});
    let mut proposal = proposal;
    proposal["changes"][0]["old"] = json!("mfr: \"Fixture\"");
    proposal["changes"][0]["new"] = json!("mfr: \"Reviewed fixture\"");
    let bundle_hash = save(&directory, "bundle.json", &bundle);
    let proposal_hash = save(&directory, "proposal.json", &proposal);
    let process = json!({"command":["declared-disposable-fixture"],"cwd":"/tmp/fixture","started_at":"2026-10-11T00:00:00Z","finished_at":"2026-10-11T00:00:01Z","exit_code":0,"signal":null,"error":null});
    let mut provider = process.clone();
    provider["engine"] = json!("codex");
    provider["heartbeat_at"] = Value::Null;
    let run = json!({"schema":1,"id":id,"kind":"connect","created_at":"2026-10-11T00:00:00Z","bundle_sha256":bundle_hash,"proposal_sha256":proposal_hash,"provider":provider});
    save(&directory, "run.json", &run);
    let mut after = snapshot.clone();
    let edited = source.replacen("mfr: \"Fixture\"", "mfr: \"Reviewed fixture\"", 1);
    after[4]["sha256"] = json!(cohdl::hash::sha256_hex(edited.as_bytes()));
    let mut compiler = process;
    compiler["result"] = json!({"schema_version":1,"verdict":"pass","diagnostics":[]});
    let validation = json!({"schema":1,"id":id,"proposal_sha256":proposal_hash,"bundle_sha256":bundle_hash,"started_at":"2026-10-11T00:00:00Z","finished_at":"2026-10-11T00:00:01Z","decision":"PASS","issues":[],"before":snapshot,"after":after,"compiler":compiler});
    (directory, run, bundle, proposal, validation)
}

fn detail(server: &Server, id: &str) -> Value {
    let response = server.request(&format!("/api/proposals/{id}"));
    assert_eq!(response.0, 200, "{response:?}");
    serde_json::from_str(&response.2).unwrap()
}

#[test]
fn proposal_lifecycle_requires_matching_hashes_compiler_and_current_snapshot() {
    let server = Server::start();
    let (directory, run, _, _, validation) = retained_run(&server, "lifecycle");
    let body = detail(&server, "lifecycle");
    assert_eq!(body["status"], "proposed", "{body}");
    assert_eq!(body["freshness"], "fresh", "{body}");
    save(&directory, "validation.json", &validation);
    assert_eq!(detail(&server, "lifecycle")["status"], "validated");
    for (key, bad) in [
        ("exit_code", json!(1)),
        ("finished_at", Value::Null),
        ("signal", json!("SIGTERM")),
        ("error", json!("compiler failed")),
    ] {
        let mut invalid = validation.clone();
        invalid["compiler"][key] = bad;
        save(&directory, "validation.json", &invalid);
        assert_eq!(detail(&server, "lifecycle")["status"], "rejected", "{key}");
    }
    save(&directory, "validation.json", &validation);
    std::fs::write(
        server.root.join("project/src/added.cohdl"),
        "// added input",
    )
    .unwrap();
    let body = detail(&server, "lifecycle");
    assert_eq!(body["status"], "validated");
    assert_eq!(body["freshness"], "stale");
    std::fs::remove_file(server.root.join("project/src/added.cohdl")).unwrap();
    let mut bytes = std::fs::read(directory.join("proposal.json")).unwrap();
    bytes.push(b'\n');
    std::fs::write(directory.join("proposal.json"), bytes).unwrap();
    assert_eq!(detail(&server, "lifecycle")["status"], "rejected");
    let mut failed = run;
    failed["provider"]["exit_code"] = json!(1);
    save(&directory, "run.json", &failed);
    assert_eq!(detail(&server, "lifecycle")["status"], "failed");
}

#[test]
fn proposal_heartbeat_is_bounded_and_never_success() {
    let server = Server::start();
    let (directory, mut run, _, _, _) = retained_run(&server, "heartbeat");
    std::fs::remove_file(directory.join("proposal.json")).unwrap();
    run["proposal_sha256"] = Value::Null;
    run["provider"]["finished_at"] = Value::Null;
    run["provider"]["exit_code"] = Value::Null;
    let now: chrono::DateTime<chrono::Utc> = SystemTime::now().into();
    run["provider"]["heartbeat_at"] = json!(now.to_rfc3339());
    save(&directory, "run.json", &run);
    assert_eq!(detail(&server, "heartbeat")["status"], "pending");
    run["provider"]["heartbeat_at"] = json!("2026-10-10T00:00:00Z");
    save(&directory, "run.json", &run);
    assert_eq!(detail(&server, "heartbeat")["status"], "interrupted");
    run["provider"]["heartbeat_at"] = json!("2099-10-10T00:00:00Z");
    save(&directory, "run.json", &run);
    assert_eq!(detail(&server, "heartbeat")["status"], "interrupted");
}

#[test]
fn proposal_receipt_reports_guarded_apply_revert_and_incomplete_history() {
    let server = Server::start();
    let (directory, run, bundle, proposal, validation) = retained_run(&server, "receipt");
    let validation_hash = save(&directory, "validation.json", &validation);
    let before = bundle["sources"][1]["text"].as_str().unwrap();
    let after = before.replacen(
        proposal["changes"][0]["old"].as_str().unwrap(),
        proposal["changes"][0]["new"].as_str().unwrap(),
        1,
    );
    let operation = json!({"started_at":"2026-10-11T00:00:00Z","finished_at":"2026-10-11T00:00:01Z","outcome":"applied","error":null});
    let mut receipt = json!({"schema":1,"id":"receipt","bundle_sha256":run["bundle_sha256"],"proposal_sha256":run["proposal_sha256"],"validation_sha256":validation_hash,"apply_validation":validation,
        "before":validation["before"],"after":validation["after"],"files":[{"path":"src/main.cohdl","before_sha256":proposal["changes"][0]["before_sha256"],"after_sha256":cohdl::hash::sha256_hex(after.as_bytes()),"before":before,"after":after}],"apply":operation,"revert":null});
    std::fs::write(server.root.join("project/src/main.cohdl"), &after).unwrap();
    save(&directory, "receipt.json", &receipt);
    let body = detail(&server, "receipt");
    assert_eq!(body["status"], "applied", "{body}");
    assert_eq!(body["freshness"], "fresh");
    receipt["revert"] = operation.clone();
    receipt["revert"]["outcome"] = json!("reverting");
    receipt["revert"]["finished_at"] = Value::Null;
    save(&directory, "receipt.json", &receipt);
    assert_eq!(detail(&server, "receipt")["status"], "interrupted");
    std::fs::write(server.root.join("project/src/main.cohdl"), before).unwrap();
    receipt["revert"] = operation;
    receipt["revert"]["outcome"] = json!("reverted");
    save(&directory, "receipt.json", &receipt);
    let body = detail(&server, "receipt");
    assert_eq!(body["status"], "reverted", "{body}");
    assert_eq!(body["freshness"], "fresh");
    receipt["files"][0]["after"] = json!("tampered");
    save(&directory, "receipt.json", &receipt);
    assert_eq!(detail(&server, "receipt")["status"], "rejected");
}

#[test]
fn proposal_json_and_run_count_bounds_fail_before_hydration() {
    let server = Server::start();
    let directory = proposal_directory(&server, "large");
    std::fs::File::create(directory.join("run.json"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    let body = detail(&server, "large");
    assert_eq!(body["status"], "rejected");
    assert!(body["issues"].to_string().contains("size"));
    for n in 0..128 {
        proposal_directory(&server, &format!("run{n}"));
    }
    assert_eq!(server.request("/api/proposals").0, 422);
}

#[test]
fn proposal_semantic_failures_never_inherit_a_saved_pass() {
    let server = Server::start();
    let (directory, run, _, proposal, validation) = retained_run(&server, "invalid-proof");
    for mutation in ["path", "line", "old", "duplicate"] {
        let mut bad = proposal.clone();
        match mutation {
            "path" => bad["changes"][0]["path"] = json!("cohdl.toml"),
            "line" => bad["citations"][0]["source"]["line"] = json!(u64::MAX),
            "old" => bad["changes"][0]["old"] = json!("never occurs"),
            _ => {
                let duplicate = bad["changes"][0].clone();
                bad["changes"].as_array_mut().unwrap().push(duplicate);
            }
        }
        let hash = save(&directory, "proposal.json", &bad);
        let mut metadata = run.clone();
        metadata["proposal_sha256"] = json!(hash);
        let mut report = validation.clone();
        report["proposal_sha256"] = json!(hash);
        save(&directory, "run.json", &metadata);
        save(&directory, "validation.json", &report);
        assert_eq!(
            detail(&server, "invalid-proof")["status"],
            "rejected",
            "{mutation}"
        );
    }
}

#[test]
fn proposal_writeahead_without_fresh_validation_is_interrupted() {
    let server = Server::start();
    let (directory, run, bundle, proposal, validation) = retained_run(&server, "writeahead");
    let validation_hash = save(&directory, "validation.json", &validation);
    let before = bundle["sources"][1]["text"].as_str().unwrap();
    let after = before.replacen(
        proposal["changes"][0]["old"].as_str().unwrap(),
        proposal["changes"][0]["new"].as_str().unwrap(),
        1,
    );
    let receipt = json!({"schema":1,"id":"writeahead","bundle_sha256":run["bundle_sha256"],"proposal_sha256":run["proposal_sha256"],"validation_sha256":validation_hash,"apply_validation":null,
        "before":validation["before"],"after":validation["after"],"files":[{"path":"src/main.cohdl","before_sha256":proposal["changes"][0]["before_sha256"],"after_sha256":cohdl::hash::sha256_hex(after.as_bytes()),"before":before,"after":after}],
        "apply":{"started_at":"2026-10-11T00:00:00Z","finished_at":null,"outcome":"applying","error":null},"revert":null});
    save(&directory, "receipt.json", &receipt);
    let body = detail(&server, "writeahead");
    assert_eq!(body["status"], "interrupted", "{body}");
    assert_eq!(body["freshness"], "unknown");
}

#[test]
fn proposal_document_citations_bind_registry_bytes_and_current_document_hash() {
    let server = Server::start();
    let (directory, mut run, mut bundle, mut proposal, _) = retained_run(&server, "documents");
    std::fs::create_dir_all(server.root.join("project/docs")).unwrap();
    let document_hash = cohdl::hash::sha256_hex(b"# Proof\n");
    std::fs::write(server.root.join("project/docs/proof.md"), "# Proof\n").unwrap();
    let application = json!({"instances":[bundle["model"]["instances"][0]["path"]],"groups":[],"relation":"exact","section":"Proof"});
    let original = json!({"id":"proof","title":"Declared fixture document","kind":"markdown","path":"docs/proof.md","revision":null,"sha256":document_hash,"pages":null,"review":"reviewed","uncertainty":null,"applicability":[application]});
    let registry = json!({"schema":1,"documents":[original],"constraints":[]});
    let registry_hash = save(
        &server.root.join("project/explorer"),
        "documents.json",
        &registry,
    );
    let registry_text =
        std::fs::read_to_string(server.root.join("project/explorer/documents.json")).unwrap();
    let mut document = original;
    document["actual_sha256"] = json!(document_hash);
    document["status"] = json!("ready");
    document["issues"] = json!([]);
    document["excerpts"] = json!([]);
    bundle["registry"] = registry;
    bundle["documents"] = json!([document]);
    bundle["sources"].as_array_mut().unwrap().push(
        json!({"path":"explorer/documents.json","sha256":registry_hash,"text":registry_text}),
    );
    for entry in bundle["snapshot"].as_array_mut().unwrap() {
        if entry["path"] == "explorer/documents.json" {
            entry["sha256"] = json!(registry_hash);
        }
    }
    bundle["snapshot"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"docs/proof.md","sha256":document_hash}));
    bundle["snapshot"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    proposal["citations"]
        .as_array_mut()
        .unwrap()
        .push(json!({"document":"proof","page":null,"section":"Proof","source":null}));
    run["bundle_sha256"] = json!(save(&directory, "bundle.json", &bundle));
    run["proposal_sha256"] = json!(save(&directory, "proposal.json", &proposal));
    save(&directory, "run.json", &run);
    let body = detail(&server, "documents");
    assert_eq!(body["status"], "proposed", "{body}");
    assert_eq!(body["freshness"], "fresh");
    std::fs::write(server.root.join("project/docs/proof.md"), "# Other\n").unwrap();
    assert_eq!(detail(&server, "documents")["freshness"], "stale");
    bundle["documents"][0]["actual_sha256"] = json!("b".repeat(64));
    run["bundle_sha256"] = json!(save(&directory, "bundle.json", &bundle));
    save(&directory, "run.json", &run);
    let body = detail(&server, "documents");
    assert_eq!(body["status"], "rejected");
    assert_eq!(body["run"]["bundle"], Value::Null);
}

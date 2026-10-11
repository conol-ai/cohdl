use super::*;

const SOURCE: &str = include_str!("../fixtures/parameterized/src/main.cohdl");

fn semantic_failure() -> String {
    SOURCE.replace("B: 2 [passive]", "B: 2 [passive], required C: 3 [passive]")
}

fn extract(server: &Server) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
        .arg(server.root.join("project"))
        .output()
        .unwrap()
}

fn model(server: &Server) -> Value {
    serde_json::from_str(&server.request("/api/model").2).unwrap()
}

fn edit(server: &Server, events: &mut impl BufRead, version: &mut u64, source: &str) {
    std::fs::write(server.root.join("project/src/main.cohdl"), source).unwrap();
    let next = next_version(events);
    assert!(next > *version, "Source edit did not notify SSE");
    *version = next;
}

#[test]
fn semantic_failure_retains_last_passing_circuit() {
    let server = Server::start();
    let original = model(&server);
    assert_eq!(original["verdict"], "pass");
    let mut events = server.events();
    let mut version = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    edit(&server, &mut events, &mut version, &semantic_failure());
    let standalone = extract(&server);
    assert!(standalone.status.success());
    let failed: Value = serde_json::from_slice(&standalone.stdout).unwrap();
    assert_eq!(failed["verdict"], "fail");
    assert!(!failed["instances"].as_array().unwrap().is_empty());
    assert!(failed["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "E701"));
    let mut live = model(&server);
    let error = live
        .as_object_mut()
        .unwrap()
        .remove("live_error")
        .expect("Semantic failure must expose live diagnostics");
    assert!(error.as_str().unwrap().contains("E701"));
    assert!(error.as_str().unwrap().contains("main.cohdl"));
    assert_eq!(
        live, original,
        "Failed semantic extraction replaced the passing circuit"
    );
    let recovered = SOURCE.replace("mpn: \"D\"", "mpn: \"Recovered\"");
    edit(&server, &mut events, &mut version, &recovered);
    let live = model(&server);
    assert!(live.get("live_error").is_none());
    assert_eq!(live["verdict"], "pass");
    assert_ne!(live, original, "Valid edit must publish a fresh model");
    assert_eq!(
        live,
        serde_json::from_slice::<Value>(&extract(&server).stdout).unwrap()
    );
}

#[test]
fn parse_failure_reports_real_diagnostics_on_retained_circuit() {
    let server = Server::start();
    let original = model(&server);
    let mut events = server.events();
    let mut version = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    edit(&server, &mut events, &mut version, "@");
    let standalone = extract(&server);
    assert!(
        !standalone.status.success(),
        "Parse failure must stay an extraction error"
    );
    let mut live = model(&server);
    let error = live.as_object_mut().unwrap().remove("live_error").unwrap();
    assert!(
        error.as_str().unwrap().contains("main.cohdl"),
        "Actual source diagnostic is missing: {error}"
    );
    assert!(error.as_str().unwrap().contains("error["));
    assert_eq!(live, original);
    edit(&server, &mut events, &mut version, SOURCE);
    assert_eq!(model(&server), original);
}

#[test]
fn explicit_top_parse_failure_preserves_diagnostics_and_selection_errors() {
    let configured = "[package]\nname = \"parameterized\"\nversion = \"0.1.0\"\n[design]\ntop = \"Board\"\n[dependencies]\n";
    let server = Server::start_with_manifest(SOURCE, configured);
    let manifest = server.root.join("project/cohdl.toml");
    let original = model(&server);
    let mut events = server.events();
    let mut version = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    edit(&server, &mut events, &mut version, "@\n");
    let standalone = extract(&server);
    assert!(!standalone.status.success());
    assert!(standalone.stdout.is_empty());
    let stderr = String::from_utf8(standalone.stderr).unwrap();
    assert!(
        stderr.contains("E001"),
        "Parser diagnostic hidden: {stderr}"
    );
    assert!(stderr.contains("main.cohdl:1:1"));
    assert!(stderr.contains("design selection failed"));
    let mut live = model(&server);
    let error = live.as_object_mut().unwrap().remove("live_error").unwrap();
    let error = error.as_str().unwrap();
    assert!(error.contains("E001"));
    assert!(error.contains("main.cohdl:1:1"));
    assert_eq!(live, original);
    edit(&server, &mut events, &mut version, SOURCE);
    assert_eq!(model(&server), original);
    std::fs::write(&manifest, configured.replace("Board", "Missing")).unwrap();
    let selection = extract(&server);
    assert!(!selection.status.success());
    let error = String::from_utf8(selection.stderr).unwrap();
    assert!(error.contains("design selection failed"));
    assert!(error.contains("no design named `Missing`"));
    assert!(
        !error.contains("error["),
        "Selection-only behavior changed: {error}"
    );
}

#[test]
fn initial_semantic_failure_stays_visible_until_passing_edit() {
    let server = Server::start_with_source(&semantic_failure());
    let failed = model(&server);
    assert_eq!(failed["verdict"], "fail");
    assert!(failed.get("live_error").is_none());
    assert_eq!(
        failed,
        serde_json::from_slice::<Value>(&extract(&server).stdout).unwrap()
    );
    let mut events = server.events();
    let mut version = next_version(&mut events);
    std::thread::sleep(Duration::from_millis(600));
    edit(
        &server,
        &mut events,
        &mut version,
        &semantic_failure().replace("mpn: \"D\"", "mpn: \"Invalid\""),
    );
    let current = model(&server);
    assert_eq!(current["verdict"], "fail");
    assert_ne!(
        current, failed,
        "Without a passing circuit, current failed IR remains inspectable"
    );
    edit(&server, &mut events, &mut version, SOURCE);
    assert_eq!(model(&server)["verdict"], "pass");
    edit(&server, &mut events, &mut version, &semantic_failure());
    let live = model(&server);
    assert_eq!(live["verdict"], "pass");
    assert!(live["live_error"].as_str().unwrap().contains("E701"));
}

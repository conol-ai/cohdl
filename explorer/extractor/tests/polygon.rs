#[path = "../../../tests/common/polygon_fixture.rs"]
mod fixture;
#[path = "../../../tests/common/source_identity.rs"]
mod identity;

#[test]
fn native_vertices_and_repeated_contact_placements_survive_extraction() {
    let dir = std::env::temp_dir().join(format!("cohdl-native-polygon-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"native-polygon\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.cohdl"), fixture::source()).unwrap();
    let model = cohdl_explorer::project_model::extract(&dir).unwrap();
    let json = serde_json::to_value(model).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cohdl-explorer"))
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        json,
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    );
    assert_eq!(json["verdict"], "pass", "{json}");
    for record in fixture::records().as_array().unwrap() {
        let name = record["ref"].as_str().unwrap();
        let pads = json["footprints"][format!("native_polygon::{name}FP")]["pads"]
            .as_array()
            .unwrap();
        assert_eq!(pads.len(), record["pads"].as_array().unwrap().len());
        for (actual, raw) in pads.iter().zip(record["pads"].as_array().unwrap()) {
            let raw = raw.as_str().unwrap();
            assert_eq!(actual["number"], raw.split('~').nth(8).unwrap());
            if raw.starts_with("PAD~POLYGON") {
                assert_eq!(actual["shape"], "polygon");
                let expected: Vec<_> = fixture::vertices(raw)
                    .iter()
                    .map(|(x, y)| [*x as f64 / 1e15, *y as f64 / 1e15])
                    .collect();
                assert_eq!(actual["vertices"], serde_json::to_value(expected).unwrap());
            } else {
                assert!(actual.get("vertices").is_none());
            }
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn explicitly_unknown_procurement_fields_remain_empty_in_model() {
    let dir = std::env::temp_dir().join(format!("cohdl-source-identity-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"identity\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.cohdl"), identity::source()).unwrap();
    let json = serde_json::to_value(cohdl_explorer::project_model::extract(&dir).unwrap()).unwrap();
    assert_eq!(json["verdict"], "pass");
    let insts = json["instances"].as_array().unwrap();
    assert_eq!(insts[0]["part"]["mfr"], "KnownMaker");
    assert_eq!(insts[0]["part"]["mpn"], "");
    assert_eq!(insts[1]["part"]["mfr"], "");
    assert_eq!(insts[1]["part"]["mpn"], "KnownMPN");
    assert_eq!(insts[2]["part"]["mpn"], "");
    assert_eq!(insts[2]["part"]["mfr"], "");
    std::fs::remove_dir_all(dir).unwrap();
}

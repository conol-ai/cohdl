use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[test]
fn loop_placement_errors_keep_provenance_in_the_explorer_message() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "cohdl-explorer-loop-diagnostics-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    std::fs::create_dir(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"loop_diagnostics\"\nversion = \"0.1.0\"\n[dependencies]\n",
    )
    .unwrap();
    let library = include_str!("fixtures/parameterized/src/main.cohdl")
        .split("pub fn attach")
        .next()
        .unwrap();
    for (body, iterations) in [
        (
            "for pos: i in 0..2 { place a[i] at (0mm, 0mm) place a[i] at (1mm, 0mm) }",
            [0, 1],
        ),
        (
            "for pos: i in 0..4 { place a[i] at (0mm, 0mm) rotate 200 * i }",
            [2, 3],
        ),
    ] {
        let source = format!(
            "{library}\ndesign B {{ inst a: [P; 4] net _: a[0..=3].A, a[0..=3].B layout {{ {body} }} }}"
        );
        std::fs::write(dir.join("src/main.cohdl"), &source).unwrap();
        let model = cohdl_explorer::project_model::extract(&dir).unwrap();
        assert_eq!(model.verdict, "fail");
        let errors: Vec<_> = model
            .diagnostics
            .iter()
            .filter(|d| d.severity == "error")
            .collect();
        assert_eq!(errors.len(), 2);
        for (error, i) in errors.iter().zip(iterations) {
            assert_eq!(error.code, "E1007");
            for detail in [
                format!("B::a_{i}"),
                format!("B::__for_pos_{i}"),
                format!("i = {i}"),
            ] {
                assert!(
                    error.message.contains(&detail),
                    "Explorer message lost {detail}: {}",
                    error.message
                );
            }
            if i >= 2 {
                assert!(error.message.contains(&format!("rotate {}", 200 * i)));
            }
        }
        assert_ne!(errors[0].message, errors[1].message);
        assert_eq!(
            std::fs::read_to_string(dir.join("src/main.cohdl")).unwrap(),
            source
        );
        for path in ["cohdl.lock", "design.lock", "out"] {
            assert!(!dir.join(path).exists());
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn expanded_instances_keep_complete_layout_connectivity_and_source_locations() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parameterized");
    let source = std::fs::read_to_string(dir.join("src/main.cohdl")).unwrap();
    let extract =
        || serde_json::to_value(cohdl_explorer::project_model::extract(&dir).unwrap()).unwrap();
    let model = extract();
    assert_eq!(model, extract(), "read-only extraction is deterministic");
    assert_eq!(model["verdict"], "pass", "{}", model["diagnostics"]);
    assert_eq!(model["nc"], serde_json::json!([]));

    let mut expected = BTreeMap::from([("Board::source".to_string(), "inst source: P")]);
    for i in 0..3 {
        expected.insert(format!("Board::bank::chips_{i}"), "inst chips: [P; N]");
    }
    for i in 0..2 {
        expected.insert(
            format!("Board::__for_helpers_{i}::__fn0_attach::helper"),
            "inst helper: P",
        );
    }
    let instances = model["instances"].as_array().unwrap();
    let actual: BTreeSet<_> = instances
        .iter()
        .map(|i| i["path"].as_str().unwrap())
        .collect();
    assert_eq!(actual, expected.keys().map(String::as_str).collect());
    let expected_span = |text: &str| {
        let offset = source.find(text).unwrap();
        let before = &source[..offset];
        serde_json::json!({
            "file": "src/main.cohdl",
            "line": before.bytes().filter(|b| *b == b'\n').count() + 1,
            "col": before.rsplit('\n').next().unwrap().len() + 1,
        })
    };
    for inst in instances {
        assert_eq!(
            inst["span"],
            expected_span(expected[inst["path"].as_str().unwrap()])
        );
        assert_eq!(inst["part"]["mpn"], "D");
        assert!(inst["designator"].as_str().unwrap().starts_with('R'));
    }
    let subs = model["subdesigns"].as_array().unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0]["path"], "Board::bank");
    assert_eq!(subs[0]["span"], expected_span("subdesign bank: Bank<3>"));
    assert_eq!(subs[0]["local_placements"].as_array().unwrap().len(), 3);

    let placements: BTreeMap<_, _> = model["layout"]["placements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["instance"].as_str().unwrap(),
                (
                    p["at_mm"].clone(),
                    p["rotate"].as_u64().unwrap(),
                    p["side"].as_str().unwrap(),
                ),
            )
        })
        .collect();
    assert_eq!(
        placements,
        BTreeMap::from([
            ("Board::source", (serde_json::json!(["0", "0"]), 0, "top")),
            (
                "Board::bank::chips_0",
                (serde_json::json!(["20", "20"]), 90, "top")
            ),
            // Authoring coordinates are +y-down; positive rotation is CCW.
            (
                "Board::bank::chips_1",
                (serde_json::json!(["20", "15"]), 90, "top")
            ),
            (
                "Board::bank::chips_2",
                (serde_json::json!(["40", "40"]), 0, "top")
            ),
        ])
    );
    let nets = model["nets"].as_array().unwrap();
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0]["name"], "BUS");
    let members: BTreeSet<_> = nets[0]["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["instance_path"].as_str().unwrap().to_string(),
                p["logical_pin"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let expected_members: BTreeSet<_> = expected
        .keys()
        .flat_map(|path| ["A", "B"].map(|pin| (path.clone(), pin.to_string())))
        .collect();
    assert_eq!(members, expected_members);
    assert_eq!(
        source,
        std::fs::read_to_string(dir.join("src/main.cohdl")).unwrap()
    );
    for path in ["cohdl.lock", "design.lock", "out"] {
        assert!(!dir.join(path).exists(), "extraction wrote {path}");
    }
}

//! S1 structural assertions against the two real reference designs.

use std::path::Path;

fn extract(example: &str) -> serde_json::Value {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../cohdl/examples")
        .join(example);
    let model = cohdl_explorer::project_model::extract(&dir).expect("extract");
    serde_json::to_value(&model).expect("serialize")
}

fn common_asserts(v: &serde_json::Value) {
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["verdict"], "pass");
    let instances = v["instances"].as_array().unwrap();
    assert!(instances.len() > 20, "instances: {}", instances.len());
    // Pin-less mechanical parts (mounting holes) are real instances with
    // real designators; the overwhelming majority still carry pins.
    let mut with_pins = 0;
    for i in instances {
        if !i["pins"].as_array().unwrap().is_empty() {
            with_pins += 1;
        }
        assert!(
            i["designator"].as_str().is_some(),
            "no designator for {}",
            i["path"]
        );
        assert!(i["span"]["line"].as_u64().unwrap() >= 1);
    }
    assert!(with_pins > 20, "instances with pins: {with_pins}");
    let nets = v["nets"].as_array().unwrap();
    assert!(nets.len() > 20, "nets: {}", nets.len());
    for n in nets {
        assert!(!n["members"].as_array().unwrap().is_empty());
    }
    let rails: Vec<&str> = v["derived"]["rails"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert!(rails.contains(&"GND"), "rails: {rails:?}");
    assert!(!v["derived"]["two_terminal"].as_array().unwrap().is_empty());
}

#[test]
fn rpi_pico2_model() {
    let v = extract("rpi-pico2");
    common_asserts(&v);
    assert_eq!(v["design"], "Pico2");
    // Net count must match the committed KiCad netlist (67 nets).
    assert_eq!(v["nets"].as_array().unwrap().len(), 67);
    // The MCU instance carries its part, MPN, docs, and full pin table.
    let mcu = v["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["path"] == "Pico2::mcu")
        .expect("mcu");
    assert_eq!(mcu["part"]["mpn"], "RP2350A");
    assert_eq!(mcu["pins"].as_array().unwrap().len(), 54);
    assert!(mcu["docs"][0]["name"].as_str().unwrap().ends_with(".pdf"));
    assert!(mcu["docs"][0]["abs"].as_str().unwrap().starts_with("/"));
    // fn expansion produces grouping seeds.
    assert!(!v["derived"]["fn_groups"].as_array().unwrap().is_empty());
    // Footprint geometry rides along for sidebar previews: the SWD
    // castellated header's three Ø1.7mm PTH pads at ±2.54mm, Ø1mm drills.
    let fps = v["footprints"].as_object().unwrap();
    assert!(fps.len() > 10, "footprints: {}", fps.len());
    let swd = &fps["connectors::headers::castellated_254::FP_Pico_Castellated_3"];
    let pads = swd["pads"].as_array().unwrap();
    assert_eq!(pads.len(), 3);
    assert_eq!(pads[0]["x"], -2.54);
    assert_eq!(pads[0]["shape"], "circle");
    assert_eq!(pads[0]["drill"][0], 1.0);
    assert_eq!(pads[0]["pth"], true);
    assert_eq!(swd["courtyard"]["size"][0], 8.3);
}

#[test]
fn sf32_miniboard_model() {
    // OpenMicro moved to the openmicrokbd repository; the SF32 miniboard is
    // the second in-repo reference design.
    let v = extract("sf32-miniboard");
    common_asserts(&v);
    assert_eq!(v["design"], "SF32MiniBoard");
    assert!(!v["nc"].as_array().unwrap().is_empty());
}

#[test]
fn snapshot_stability() {
    // Two consecutive extractions must serialize byte-identically.
    let a = serde_json::to_string(&extract("rpi-pico2")).unwrap();
    let b = serde_json::to_string(&extract("rpi-pico2")).unwrap();
    assert_eq!(a, b);
}

#[test]
fn subdesign_hierarchy_ports_and_functions_are_distinct_and_read_only() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/subdesign");
    let extract = || {
        let model = cohdl_explorer::project_model::extract(&dir).expect("extract hierarchy");
        serde_json::to_value(model).unwrap()
    };
    let v = extract();
    assert_eq!(v, extract(), "hierarchy projection must be deterministic");
    assert_eq!(v["verdict"], "pass", "{}", v["diagnostics"]);
    let subs = v["subdesigns"].as_array().unwrap();
    assert_eq!(subs.len(), 9); // 2 banks + 4 cells + 2 port-only links + empty
    let cell = subs
        .iter()
        .find(|s| s["path"] == "Board::banks_1::lanes_0")
        .unwrap();
    assert_eq!(cell["parent"], "Board::banks_1");
    assert_eq!(cell["definition"], "hierarchy::Cell");
    assert!(cell["span"]["line"].as_u64().unwrap() > 0);
    let ports = cell["ports"].as_array().unwrap();
    let port = |name: &str| ports.iter().find(|p| p["name"] == name).unwrap();
    assert_eq!(port("IN")["net"], "INPUT");
    assert_eq!(port("IN")["obligation"], "required");
    assert_eq!(port("IN")["connected"], true);
    assert_eq!(port("TAP")["net"], "INPUT");
    // Exhaustiveness is per electrical class: TAP shares IN's outside link.
    assert_eq!(port("TAP")["connected"], true);
    assert_eq!(port("UNUSED")["connected"], false);
    assert!(port("UNUSED").get("net").is_none());
    let link = subs.iter().find(|s| s["path"] == "Board::links_0").unwrap();
    assert!(link.get("parent").is_none());
    for p in link["ports"].as_array().unwrap() {
        assert_eq!(p["net"], "ONLY_PORTS");
        assert_eq!(p["connected"], true);
    }
    assert!(!v["nets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["name"] == "ONLY_PORTS"));
    let instances = v["instances"].as_array().unwrap();
    assert_eq!(instances.len(), 9);
    for s in subs {
        assert!(s.get("designator").is_none());
        assert!(s.get("part").is_none());
        assert!(!instances.iter().any(|i| i["path"] == s["path"]));
    }
    let groups = v["derived"]["fn_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 4);
    assert!(groups
        .iter()
        .all(|g| g["name"].as_str().unwrap().contains("::__fn")));
    assert!(!dir.join("cohdl.lock").exists());
    assert!(!dir.join("design.lock").exists());
    assert!(!dir.join("out").exists());
}

#[test]
fn physical_layout_uses_resolved_subdesign_coordinates_and_real_geometry() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/layout");
    let model = cohdl_explorer::project_model::extract(&dir).unwrap();
    let v = serde_json::to_value(&model).unwrap();
    assert_eq!(v["verdict"], "pass", "{}", v["diagnostics"]);
    assert_eq!(
        serde_json::to_value(cohdl_explorer::project_model::extract(&dir).unwrap()).unwrap(),
        v
    );
    let layout = &v["layout"];
    assert_eq!(layout["placements"].as_array().unwrap().len(), 3);
    let placement = |path: &str| {
        layout["placements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["instance"] == path)
            .unwrap()
    };
    let rotated = placement("Board::rotated::child::chip");
    assert_eq!(rotated["at_mm"], serde_json::json!(["15", "25"]));
    assert_eq!(rotated["rotate"], 90);
    assert_eq!(rotated["side"], "top");
    assert_eq!(rotated["matrix"], serde_json::json!([0.0, -1.0, 1.0, 0.0]));
    let overridden = placement("Board::overridden::child::chip");
    assert_eq!(overridden["at_mm"], serde_json::json!(["-3.25", "6.5"]));
    assert_eq!(overridden["rotate"], 270);
    assert_eq!(overridden["side"], "bottom");
    assert_eq!(
        overridden["matrix"],
        serde_json::json!([0.0, -1.0, -1.0, 0.0])
    );
    let arbitrary = placement("Board::fixed");
    assert_eq!(arbitrary["rotate"], 37);
    assert!((arbitrary["matrix"][0].as_f64().unwrap() - 0.7986355100472928).abs() < 1e-15);
    assert!((arbitrary["matrix"][1].as_f64().unwrap() + 0.6018150231520483).abs() < 1e-15);
    // Unanchored defaults must not become invented absolute placements.
    assert!(v["instances"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["path"] == "Board::loose::chip"));
    assert!(!layout["placements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["instance"] == "Board::loose::chip"));
    let local = |path: &str| {
        &v["subdesigns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["path"] == path)
            .unwrap()["local_placements"]
    };
    let loose = &local("Board::loose")[0];
    assert_eq!(loose["instance"], "Board::loose::chip");
    assert_eq!(loose["at_mm"], serde_json::json!(["1", "2"]));
    assert_eq!(loose["rotate"], 90);
    assert_eq!(loose["matrix"], serde_json::json!([0.0, -1.0, 1.0, 0.0]));
    for path in ["Board::rotated", "Board::overridden"] {
        let group = local(path);
        assert_eq!(group.as_array().unwrap().len(), 1);
        assert_eq!(group[0]["at_mm"], serde_json::json!(["5", "5"]));
        assert_eq!(group[0]["rotate"], 0);
        assert_eq!(group[0]["side"], "bottom");
        assert_eq!(group[0]["matrix"], serde_json::json!([-1.0, 0.0, 0.0, 1.0]));
    }
    assert_eq!(
        layout["board_outline"]["start"],
        serde_json::json!([-10, -10])
    );
    assert_eq!(
        layout["board_outline"]["segments"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        layout["diff_pairs"][0],
        serde_json::json!({"p":"INPUT", "n":"OUTPUT"})
    );
    assert_eq!(layout["length_matches"][0]["tolerance"], "0.15mm");
    assert_eq!(layout["net_classes"][0]["name"], "Signals");
    let fp = &v["footprints"]["physical::F"];
    assert_eq!(
        fp["pads"][0]["matrix"],
        serde_json::json!([0.0, -1.0, 1.0, 0.0])
    );
    assert_eq!(fp["pads"][1]["drill"], serde_json::json!([0.4, 1.0]));
    assert_eq!(fp["pads"][1]["layer"], "through_all");
    assert_eq!(fp["mount_holes"].as_array().unwrap().len(), 1);
    assert!(!dir.join("out").exists());
    assert!(!dir.join("cohdl.lock").exists());
    assert!(!dir.join("design.lock").exists());
}

#[test]
fn missing_outline_reports_a_layout_issue_without_losing_the_schematic() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/layout");
    let dir =
        std::env::temp_dir().join(format!("cohdl-explorer-no-outline-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::copy(source.join("cohdl.toml"), dir.join("cohdl.toml")).unwrap();
    std::fs::copy(source.join("src/main.cohdl"), dir.join("src/main.cohdl")).unwrap();
    let model = cohdl_explorer::project_model::extract(&dir).unwrap();
    let v = serde_json::to_value(model).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
    assert_eq!(v["verdict"], "pass");
    assert_eq!(v["layout"]["placements"].as_array().unwrap().len(), 3);
    assert!(v["layout"]["board_outline"].is_null());
    assert!(v["layout"]["outline_error"]
        .as_str()
        .unwrap()
        .contains("outline.dxf"));
}

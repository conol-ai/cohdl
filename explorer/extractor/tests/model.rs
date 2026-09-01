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

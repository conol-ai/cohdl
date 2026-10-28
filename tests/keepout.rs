//! Antenna regions must survive compilation as enforceable, all-layer rules.
use cohdl::{emit, lock::LockState, pipeline};

const LIB: &str = r#"
pub pad Land { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }
pub footprint Antenna {
    pad 1: Land at (0mm, 0mm)
    keepout { shape: rect, at: (2mm, -4mm), size: (4mm, 2mm) }
}
pub device Module { pins { P: 1 [passive] } }
pub part M: Module { primary { mfr: "test", mpn: "module", footprint: Antenna } }
"#;

fn checked(source: &str) -> pipeline::Checked {
    pipeline::check_files_in("test", &[("main.cohdl".into(), source.into())], None).unwrap()
}

fn built(source: &str) -> pipeline::Checked {
    let mut c = checked(source);
    pipeline::build_artifacts(&mut c, &LockState::default()).unwrap();
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    c
}

#[test]
fn invalid_keepouts_fail_even_in_unused_library_declarations() {
    for fields in [
        "shape: circle, at: (0mm, 0mm), size: (1mm)",
        "shape: oval, at: (0mm, 0mm), size: (1mm, 2mm)",
        "shape: rect, at: (0mm, 0mm), size: (1mm)",
        "shape: rect, at: (0mm, 0mm), size: (0mm, 1mm)",
        "shape: rect, at: (0mm, 0mm), size: (-1mm, 1mm)",
        "shape: rect, at: (1V, 0mm), size: (1mm, 1mm)",
        "shape: rect, at: (0mm, 0mm), size: (1mm, 1V)",
        "shape: rect, at: (0mm, 0mm), size: (1mm, 1mm), size: (2mm, 2mm)",
        "shape: rect, at: (0mm, 0mm), size: (1mm, 1mm), tracks: allowed",
        "shape: rect, at: (0mm, 0mm)",
    ] {
        let c = checked(&format!("pub footprint Bad {{ keepout {{ {fields} }} }}"));
        assert!(c.diags.has_errors(), "accepted {fields}");
        assert!(
            c.diags.render(&c.sm).contains("E806"),
            "{}",
            c.diags.render(&c.sm)
        );
    }
}

#[test]
fn repeated_regions_format_round_trip_and_keepout_only_is_not_placeholder() {
    let source = "pub footprint F {\n// antenna\nkeepout { size: (3mm,4mm), shape: rect, at:(1mm,2mm) }\n// second\nkeepout { shape:rect,at:(5mm,6mm),size:(1mm,2mm) }\n}\n";
    let fmt = cohdl::fmt::format_source("test", source).unwrap();
    assert_eq!(cohdl::fmt::format_source("test", &fmt).unwrap(), fmt);
    assert!(fmt.contains("// antenna") && fmt.contains("// second"));
    let c = checked(&fmt);
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    let fp = &c.world.footprints["test::F"];
    assert_eq!(fp.keepouts.len(), 2);
    assert!(!cohdl::check::footprints::is_placeholder(fp));
}

#[test]
fn keepouts_use_absolute_board_coordinates_and_all_copper_restrictions() {
    let source = format!(
        r#"{LIB}
design B {{
    inst a: M
    inst b: M
    nc: a.P, b.P
    layout {{
        place a at (10mm, 20mm) rotate 90
        place b at (30mm, 40mm) rotate 90 side bottom
    }}
}}
"#
    );
    let c = built(&source);
    let ir = c.ir.as_ref().unwrap();
    let modules = emit::kicad_mod::emit_kicad_mods(&c.world, ir).unwrap();
    let module = &modules[0].2;
    assert!(module.contains("(xy 0 -5) (xy 4 -5) (xy 4 -3) (xy 0 -3)"));
    let board = emit::kicad_pcb::emit_kicad_pcb(&c.world, ir, "test").unwrap();
    assert!(board.contains("(xy 5 20) (xy 5 16) (xy 7 16) (xy 7 20)"));
    assert!(board.contains("(xy 25 40) (xy 25 44) (xy 27 44) (xy 27 40)"));
    for output in [module, &board] {
        for required in [
            "(layers \"*.Cu\")",
            "(tracks not_allowed)",
            "(vias not_allowed)",
            "(pads not_allowed)",
            "(copperpour not_allowed)",
            "(footprints not_allowed)",
        ] {
            assert!(output.contains(required), "missing {required}");
        }
    }
    assert_eq!(
        board,
        emit::kicad_pcb::emit_kicad_pcb(&c.world, ir, "test").unwrap()
    );
    let error = emit::ipc2581::emit_ipc2581(&c.world, ir, "test")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Antenna") && error.contains("--emit kicad_pcb"),
        "{error}"
    );
}

#[test]
fn api_docs_expose_enforced_keepouts() {
    let c = checked(LIB);
    let rendered = emit::docsjson::render(
        &c,
        &emit::docsjson::PackageMeta {
            name: "test",
            version: "0.1.0",
            description: None,
            license: None,
            repository: None,
        },
        &[],
    );
    let value: serde_json::Value = serde_json::from_str(&rendered.json).unwrap();
    let footprint = value["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "footprint")
        .unwrap();
    let regions = footprint["footprint"]["keepouts"].as_array().unwrap();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0]["layers"], "all_copper");
    assert_eq!(
        regions[0]["prohibits"],
        serde_json::json!(["tracks", "vias", "pads", "copper_pours", "components"])
    );
}

#[test]
fn ipc_checks_used_alternates_but_not_unused_keepout_footprints() {
    let plain = r#"
pub footprint Plain { pad 1: Land at (0mm, 0mm) }
pub part Normal: Module {
    primary { mfr: "test", mpn: "normal", footprint: Plain }
}
design B { inst m: Normal nc: m.P }
"#;
    let c = built(&format!("{LIB}{plain}"));
    emit::ipc2581::emit_ipc2581(&c.world, c.ir.as_ref().unwrap(), "test").unwrap();

    let alternate = plain.replace(
        "mpn: \"normal\", footprint: Plain }",
        "mpn: \"normal\", footprint: Plain } alt { mfr: \"test\", mpn: \"alternate\", footprint: Antenna }",
    );
    let c = built(&format!("{LIB}{alternate}"));
    let error = emit::ipc2581::emit_ipc2581(&c.world, c.ir.as_ref().unwrap(), "test")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Antenna") && error.contains("cannot preserve keepouts"));
}

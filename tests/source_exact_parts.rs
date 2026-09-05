//! Exact parts backed by manufacturer documents and orderable identities.

use cohdl::ast::{PadDrill, PadLayer, PadPlating, PadShape};
use std::path::PathBuf;

fn load_package(path: &str) -> cohdl::pipeline::Checked {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![("std".to_string(), root.join("lib/std"))];
    let project = cohdl::project::load_project_with_deps(&root.join(path), &deps).unwrap();
    let mut checked = cohdl::pipeline::check_files_in_with_deps(
        &project.name,
        &["std".to_string()],
        &project.files,
        None,
    )
    .unwrap();
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn assert_smd_rect(checked: &cohdl::pipeline::Checked, name: &str, size: [&str; 2]) {
    let pad = &checked.world.pads[name];
    assert_eq!(pad.shape.unwrap().0, PadShape::Rect);
    assert_eq!(pad.size[0].text, size[0]);
    assert_eq!(pad.size[1].text, size[1]);
    assert_eq!(pad.layer.unwrap().0, PadLayer::TopCopper);
    assert_eq!(pad.plating.unwrap().0, PadPlating::Smd);
}

fn assert_corner_radius(checked: &cohdl::pipeline::Checked, name: &str, radius: &str) {
    assert_eq!(
        checked.world.pads[name]
            .corner_radius
            .as_ref()
            .map(|(value, _)| value.text.as_str()),
        Some(radius)
    );
}

#[test]
fn exact_source_leds_match_manufacturer_polarity_and_lands() {
    let checked = load_package("lib/led");
    for (part_name, mpn, footprint) in [
        (
            "led::LED_XINGLIGHT_XL_1005UBC",
            "XL-1005UBC",
            "led::FP_XINGLIGHT_XL_1005UBC",
        ),
        (
            "led::LED_NATIONSTAR_NCD0402R1",
            "NCD0402R1",
            "led::FP_NATIONSTAR_NCD0402R1",
        ),
    ] {
        let part = &checked.world.parts[part_name];
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert_eq!(part.primary.footprint.as_ref().unwrap().name, footprint);
    }
    let device = &checked.world.devices["led::ChipLED"];
    let pins = device.pins_for(None);
    assert_eq!(pins[0].name.name, "Cathode");
    assert_eq!(pins[0].numbers[0].text, "1");
    assert_eq!(pins[1].name.name, "Anode");
    assert_eq!(pins[1].numbers[0].text, "2");

    assert_smd_rect(&checked, "led::P_XINGLIGHT_XL_1005UBC", ["0.3mm", "0.55mm"]);
    assert_smd_rect(&checked, "led::P_NATIONSTAR_NCD0402R1", ["0.35mm", "0.5mm"]);
}

#[test]
fn exact_xfcn_header_and_lailan_fpc_match_layouts() {
    let checked = load_package("lib/connectors");
    let pz = &checked.world.parts["connectors::headers::xfcn_pz127v::CON_XFCN_PZ127V_11_04_0720"];
    assert_eq!(pz.primary.field("mpn").unwrap().value, "PZ127V-11-04-0720");
    let pz_fp = &checked.world.footprints["connectors::headers::xfcn_pz127v::FP_XFCN_PZ127V_4P"];
    let pz_x: Vec<_> = pz_fp.pads.iter().map(|pad| pad.x.text.as_str()).collect();
    assert_eq!(pz_x, ["-1.905mm", "-0.635mm", "0.635mm", "1.905mm"]);
    let pz_pad = &checked.world.pads["connectors::headers::xfcn_pz127v::P_XFCN_PZ127V_4P"];
    assert_eq!(pz_pad.size[0].text, "1.0mm");
    match &pz_pad.drill {
        Some((PadDrill::Round(drill), _)) => assert_eq!(drill.text, "0.65mm"),
        other => panic!("unexpected PZ127V drill: {other:?}"),
    }
    assert_eq!(pz_pad.layer.unwrap().0, PadLayer::ThroughAll);
    assert_eq!(pz_pad.plating.unwrap().0, PadPlating::PlatedThroughHole);

    let fpc =
        &checked.world.parts["connectors::fpc::lailan_cx01_31p::CON_LAILAN_FPC_CX01_31P0_3_GW"];
    assert_eq!(
        fpc.primary.field("mpn").unwrap().value,
        "LAIL-FPC-CX01-31P0.3-GW"
    );
    let fpc_fp =
        &checked.world.footprints["connectors::fpc::lailan_cx01_31p::FP_LAILAN_FPC_CX01_31P0_3_GW"];
    assert_eq!(fpc_fp.pads.len(), 33);
    assert_eq!(
        (
            fpc_fp.pads[0].x.text.as_str(),
            fpc_fp.pads[0].y.text.as_str(),
            fpc_fp.pads[1].x.text.as_str(),
            fpc_fp.pads[1].y.text.as_str(),
        ),
        ("-4.5mm", "1.4375mm", "-4.2mm", "-1.4375mm")
    );
    assert_eq!(fpc_fp.pads[31].number.text, "32");
    assert_eq!(fpc_fp.pads[31].x.text, "5.15mm");
    assert_eq!(fpc_fp.pads[32].number.text, "33");
    assert_eq!(fpc_fp.pads[32].x.text, "-5.15mm");
}

#[test]
fn exact_xunpu_switch_matches_normally_open_two_pad_drawing() {
    let checked = load_package("lib/switches");
    let part = &checked.world.parts["switches::SW_XUNPU_TS_1088R_02026"];
    assert_eq!(part.primary.field("mfr").unwrap().value, "XUNPU");
    assert_eq!(part.primary.field("mpn").unwrap().value, "TS-1088R-02026");
    assert_smd_rect(&checked, "switches::P_XUNPU_TS_1088R", ["1.05mm", "2.0mm"]);
    let footprint = &checked.world.footprints["switches::FP_XUNPU_TS_1088R"];
    assert_eq!(footprint.pads[0].x.text, "-2.225mm");
    assert_eq!(footprint.pads[1].x.text, "2.225mm");
}

#[test]
fn hirose_ufl_matches_locked_kicad_roundrect_geometry() {
    let checked = load_package("lib/connectors");
    let module = "connectors::coax::hirose_ufl";
    let part = &checked.world.parts[&format!("{module}::CON_HIROSE_UFL_R_SMT_1_10")];
    assert_eq!(
        part.primary.field("mpn").map(|field| field.value.as_str()),
        Some("U.FL-R-SMT-1(10)")
    );
    assert_smd_rect(
        &checked,
        &format!("{module}::P_HIROSE_UFL_SIGNAL"),
        ["1.05mm", "1.0mm"],
    );
    assert_corner_radius(
        &checked,
        &format!("{module}::P_HIROSE_UFL_SIGNAL"),
        "0.25mm",
    );
    assert_smd_rect(
        &checked,
        &format!("{module}::P_HIROSE_UFL_GROUND"),
        ["2.2mm", "1.05mm"],
    );
    assert_corner_radius(
        &checked,
        &format!("{module}::P_HIROSE_UFL_GROUND"),
        "0.25mm",
    );

    let footprint = &checked.world.footprints[&format!("{module}::FP_HIROSE_UFL_R_SMT_1")];
    assert_eq!(footprint.pads.len(), 3);
    let placements = footprint
        .pads
        .iter()
        .map(|pad| {
            (
                pad.number.text.as_str(),
                pad.x.text.as_str(),
                pad.y.text.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placements,
        [
            ("1", "-1.05mm", "0mm"),
            ("2", "0.475mm", "-1.475mm"),
            ("2", "0.475mm", "1.475mm"),
        ]
    );
}

#[test]
fn max16054_matches_locked_kicad_roundrect_geometry() {
    let checked = load_package("lib/switches");
    let part = &checked.world.parts["switches::PUSHBUTTON_CTRL_MAX16054AZT_T"];
    assert_eq!(
        part.primary.field("mpn").map(|field| field.value.as_str()),
        Some("MAX16054AZT+T")
    );
    assert_smd_rect(
        &checked,
        "switches::P_MAX16054_SOT23_6_LEAD",
        ["1.325mm", "0.6mm"],
    );
    assert_corner_radius(&checked, "switches::P_MAX16054_SOT23_6_LEAD", "0.15mm");

    let footprint = &checked.world.footprints["switches::FP_MAX16054_THIN_SOT23_6"];
    assert_eq!(footprint.pads.len(), 6);
    let placements = footprint
        .pads
        .iter()
        .map(|pad| {
            (
                pad.number.text.as_str(),
                pad.x.text.as_str(),
                pad.y.text.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placements,
        [
            ("1", "-1.1375mm", "-0.95mm"),
            ("2", "-1.1375mm", "0mm"),
            ("3", "-1.1375mm", "0.95mm"),
            ("4", "1.1375mm", "0.95mm"),
            ("5", "1.1375mm", "0mm"),
            ("6", "1.1375mm", "-0.95mm"),
        ]
    );
}

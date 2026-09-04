//! Exact LAILAN LAIL-PHC1.25-2P-01-PB-WT identity and land-pattern checks.

use cohdl::ast::{Obligation, PadDrill, PadLayer, PadPlating, PadShape, PinRole};
use std::path::PathBuf;

fn load_connectors() -> cohdl::pipeline::Checked {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![("std".to_string(), root.join("lib/std"))];
    let project =
        cohdl::project::load_project_with_deps(&root.join("lib/connectors"), &deps).unwrap();
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

#[test]
fn exact_lailan_part_binds_four_passive_pads() {
    let checked = load_connectors();
    let device = &checked.world.devices
        ["connectors::wire_to_board::lailan_phc125::LAILAN_PHC125_2P"];
    let pins = device.pins_for(None);
    for (pin, (name, number)) in pins.iter().zip([
        ("P1", "1"),
        ("P2", "2"),
        ("SHELL3", "3"),
        ("SHELL4", "4"),
    ]) {
        assert_eq!(pin.name.name, name);
        assert_eq!(pin.numbers[0].text, number);
        assert_eq!(pin.role_or_default(), PinRole::Passive);
        assert_eq!(pin.obligation, Obligation::Required);
    }

    let part = &checked.world.parts
        ["connectors::wire_to_board::lailan_phc125::CON_LAILAN_PHC125_2P_01_PB_WT"];
    assert_eq!(part.primary.field("mfr").unwrap().value, "LAILAN");
    assert_eq!(
        part.primary.field("mpn").unwrap().value,
        "LAIL-PHC1.25-2P-01-PB-WT"
    );
    assert_eq!(
        part.primary.footprint.as_ref().unwrap().name,
        "connectors::wire_to_board::lailan_phc125::FP_LAILAN_PHC125_2P_RIGHT_ANGLE"
    );
    assert!(part.alts.is_empty());
}

#[test]
fn lailan_lands_and_chirality_match_the_manufacturer_layout() {
    let checked = load_connectors();
    for (name, size) in [
        ("P_LAILAN_PHC125_CONTACT", ["0.7mm", "1.7mm"]),
        ("P_LAILAN_PHC125_SHELL", ["1.8mm", "2.5mm"]),
    ] {
        let pad = &checked.world.pads
            [&format!("connectors::wire_to_board::lailan_phc125::{name}")];
        assert_eq!(pad.shape.map(|(shape, _)| shape), Some(PadShape::Rect));
        assert_eq!(pad.size[0].text, size[0]);
        assert_eq!(pad.size[1].text, size[1]);
        assert_eq!(pad.layer.map(|(layer, _)| layer), Some(PadLayer::TopCopper));
        assert_eq!(pad.plating.map(|(plating, _)| plating), Some(PadPlating::Smd));
    }

    let footprint = &checked.world.footprints
        ["connectors::wire_to_board::lailan_phc125::FP_LAILAN_PHC125_2P_RIGHT_ANGLE"];
    let placements: Vec<_> = footprint
        .pads
        .iter()
        .map(|pad| {
            (
                pad.number.text.as_str(),
                pad.x.text.as_str(),
                pad.y.text.as_str(),
            )
        })
        .collect();
    assert_eq!(
        placements,
        [
            ("1", "-0.625mm", "0mm"),
            ("2", "0.625mm", "0mm"),
            ("3", "3.03mm", "2.5mm"),
            ("4", "-3.03mm", "2.5mm"),
        ]
    );
}

#[test]
fn exact_lailan_pz127_part_binds_four_passive_pins() {
    let checked = load_connectors();
    let device =
        &checked.world.devices["connectors::headers::lailan_pz127::LAILAN_PZ127_4P"];
    let pins = device.pins_for(None);
    for (pin, (name, number)) in pins.iter().zip([
        ("P1", "1"),
        ("P2", "2"),
        ("P3", "3"),
        ("P4", "4"),
    ]) {
        assert_eq!(pin.name.name, name);
        assert_eq!(pin.numbers[0].text, number);
        assert_eq!(pin.role_or_default(), PinRole::Passive);
        assert_eq!(pin.obligation, Obligation::Required);
    }

    let part =
        &checked.world.parts["connectors::headers::lailan_pz127::CON_LAILAN_PZ1_27_4P_L"];
    assert_eq!(part.primary.field("mfr").unwrap().value, "LAILAN");
    assert_eq!(
        part.primary.field("mpn").unwrap().value,
        "LAIL-PZ1.27-4P-L"
    );
    assert_eq!(
        part.primary.footprint.as_ref().unwrap().name,
        "connectors::headers::lailan_pz127::FP_LAILAN_PZ127_4P"
    );
    assert!(part.alts.is_empty());
}

#[test]
fn lailan_pz127_holes_pitch_and_numbering_match_exact_sources() {
    let checked = load_connectors();
    for (name, shape) in [
        ("P_LAILAN_PZ127_PIN1", PadShape::Rect),
        ("P_LAILAN_PZ127_PIN", PadShape::Circle),
    ] {
        let pad = &checked.world.pads[&format!("connectors::headers::lailan_pz127::{name}")];
        assert_eq!(pad.shape.map(|(actual, _)| actual), Some(shape));
        assert_eq!(pad.size[0].text, "1.0mm");
        assert_eq!(pad.layer.map(|(layer, _)| layer), Some(PadLayer::ThroughAll));
        assert_eq!(
            pad.plating.map(|(plating, _)| plating),
            Some(PadPlating::PlatedThroughHole)
        );
        match pad.drill.as_ref() {
            Some((PadDrill::Round(drill), _)) => assert_eq!(drill.text, "0.70mm"),
            other => panic!("expected a round 0.70 mm drill, got {other:?}"),
        }
    }

    let footprint =
        &checked.world.footprints["connectors::headers::lailan_pz127::FP_LAILAN_PZ127_4P"];
    let placements: Vec<_> = footprint
        .pads
        .iter()
        .map(|pad| {
            (
                pad.number.text.as_str(),
                pad.x.text.as_str(),
                pad.y.text.as_str(),
            )
        })
        .collect();
    assert_eq!(
        placements,
        [
            ("1", "-1.905mm", "0mm"),
            ("2", "-0.635mm", "0mm"),
            ("3", "0.635mm", "0mm"),
            ("4", "1.905mm", "0mm"),
        ]
    );
}

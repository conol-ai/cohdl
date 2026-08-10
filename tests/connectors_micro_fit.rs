//! Molex SD-43045-005 locked checks for the exact 43045-0212, 43045-0412,
//! and 43045-0612 vertical through-hole headers.

use cohdl::ast::{
    MountHoleGeom, MountHolePlating, Obligation, PadDrill, PadLayer, PadPlating, PadShape, PinRole,
};
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
fn exact_micro_fit_parts_bind_the_right_passive_contacts() {
    let checked = load_connectors();
    for (device_name, expected) in [
        (
            "connectors::headers::micro_fit_3::MicroFit3_2Pin",
            vec![("P1", "1"), ("P2", "2")],
        ),
        (
            "connectors::headers::micro_fit_3::MicroFit3_2x2",
            vec![("P1", "1"), ("P2", "2"), ("P3", "3"), ("P4", "4")],
        ),
        (
            "connectors::headers::micro_fit_3::MicroFit3_2x3",
            vec![
                ("P1", "1"),
                ("P2", "2"),
                ("P3", "3"),
                ("P4", "4"),
                ("P5", "5"),
                ("P6", "6"),
            ],
        ),
    ] {
        let pins = checked.world.devices[device_name].pins_for(None);
        assert_eq!(pins.len(), expected.len());
        for (pin, (name, number)) in pins.iter().zip(expected) {
            assert_eq!(pin.name.name, name);
            assert_eq!(
                pin.numbers
                    .iter()
                    .map(|number| number.text.as_str())
                    .collect::<Vec<_>>(),
                [number]
            );
            assert_eq!(pin.role_or_default(), PinRole::Passive);
            assert_eq!(pin.obligation, Obligation::Required);
        }
    }

    for (part_name, device, mpn, footprint) in [
        (
            "connectors::headers::micro_fit_3::MOLEX_43045_0212",
            "connectors::headers::micro_fit_3::MicroFit3_2Pin",
            "43045-0212",
            "connectors::headers::micro_fit_3::FP_Molex_43045_0212",
        ),
        (
            "connectors::headers::micro_fit_3::MOLEX_43045_0412",
            "connectors::headers::micro_fit_3::MicroFit3_2x2",
            "43045-0412",
            "connectors::headers::micro_fit_3::FP_Molex_43045_0412",
        ),
        (
            "connectors::headers::micro_fit_3::MOLEX_43045_0612",
            "connectors::headers::micro_fit_3::MicroFit3_2x3",
            "43045-0612",
            "connectors::headers::micro_fit_3::FP_Molex_43045_0612",
        ),
    ] {
        let part = &checked.world.parts[part_name];
        assert_eq!(part.device.name.name, device);
        assert_eq!(part.primary.field("mfr").unwrap().value, "Molex");
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert_eq!(part.primary.footprint.as_ref().unwrap().name, footprint);
        assert!(part.alts.is_empty());
    }
}

#[test]
fn micro_fit_holes_pitch_numbering_and_courtyards_match_the_sales_drawing() {
    let checked = load_connectors();
    let pad = &checked.world.pads["connectors::headers::micro_fit_3::P_MicroFit3_PTH"];
    assert_eq!(pad.shape.map(|(shape, _)| shape), Some(PadShape::Circle));
    assert_eq!(pad.size[0].text, "1.5mm");
    assert_eq!(
        pad.layer.map(|(layer, _)| layer),
        Some(PadLayer::ThroughAll)
    );
    assert_eq!(
        pad.plating.map(|(plating, _)| plating),
        Some(PadPlating::PlatedThroughHole)
    );
    match &pad.drill {
        Some((PadDrill::Round(drill), _)) => assert_eq!(drill.text, "1.02mm"),
        other => panic!("expected a round 1.02mm drill, found {other:?}"),
    }

    let fp2 = &checked.world.footprints["connectors::headers::micro_fit_3::FP_Molex_43045_0212"];
    assert_eq!(placements(fp2), [("1", "0mm", "0mm"), ("2", "0mm", "3mm")]);
    assert_mount_holes(fp2, [("-3mm", "3.94mm"), ("3mm", "3.94mm")]);
    assert_courtyard(fp2, ("0mm", "1.92mm"), ("8.16mm", "9.78mm"));

    let fp4 = &checked.world.footprints["connectors::headers::micro_fit_3::FP_Molex_43045_0412"];
    assert_eq!(
        placements(fp4),
        [
            ("1", "0mm", "0mm"),
            ("2", "3mm", "0mm"),
            ("3", "0mm", "3mm"),
            ("4", "3mm", "3mm"),
        ]
    );
    assert_mount_holes(fp4, [("-3mm", "3.94mm"), ("6mm", "3.94mm")]);
    assert_courtyard(fp4, ("1.5mm", "1.92mm"), ("11.16mm", "9.78mm"));

    let fp6 = &checked.world.footprints["connectors::headers::micro_fit_3::FP_Molex_43045_0612"];
    assert_eq!(
        placements(fp6),
        [
            ("1", "0mm", "0mm"),
            ("2", "3mm", "0mm"),
            ("3", "6mm", "0mm"),
            ("4", "0mm", "3mm"),
            ("5", "3mm", "3mm"),
            ("6", "6mm", "3mm"),
        ]
    );
    assert_mount_holes(fp6, [("-3mm", "3.94mm"), ("9mm", "3.94mm")]);
    assert_courtyard(fp6, ("3mm", "1.92mm"), ("14.16mm", "9.78mm"));
}

fn placements(fp: &cohdl::ast::FootprintDef) -> Vec<(&str, &str, &str)> {
    fp.pads
        .iter()
        .map(|place| {
            (
                place.number.text.as_str(),
                place.x.text.as_str(),
                place.y.text.as_str(),
            )
        })
        .collect()
}

fn assert_mount_holes(fp: &cohdl::ast::FootprintDef, expected: [(&str, &str); 2]) {
    assert_eq!(fp.mount_holes.len(), 2);
    for (hole, (x, y)) in fp.mount_holes.iter().zip(expected) {
        assert_eq!(hole.plating, MountHolePlating::NonPlated);
        assert_eq!(hole.shape_or_default(), PadShape::Circle);
        assert_eq!((hole.x.text.as_str(), hole.y.text.as_str()), (x, y));
        match &hole.geom {
            MountHoleGeom::Diameter(diameter) => assert_eq!(diameter.text, "1.02mm"),
            other => panic!("expected a round locating hole, found {other:?}"),
        }
    }
}

fn assert_courtyard(fp: &cohdl::ast::FootprintDef, at: (&str, &str), size: (&str, &str)) {
    let courtyard = fp.courtyard.as_ref().unwrap();
    assert_eq!(courtyard.shape.0, PadShape::Rect);
    assert_eq!(
        (courtyard.at.0.text.as_str(), courtyard.at.1.text.as_str()),
        at
    );
    assert_eq!(
        (
            courtyard.size[0].text.as_str(),
            courtyard.size[1].text.as_str(),
        ),
        size
    );
}

//! GCT USB4105 B4 sheet 1 oracles and passive-connector ERC regressions.
use cohdl::ast::{MountHoleGeom, PadDrill, PinRole};
use cohdl::pipeline::{build_artifacts, check_files_in_with_deps};

fn project() -> cohdl::project::Project {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    cohdl::project::load_project_with_deps(
        &root.join("lib/usb"),
        &[("std".into(), root.join("lib/std"))],
    )
    .unwrap()
}

#[test]
fn power_contacts_are_passive_and_gct_pin_table_is_exact() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    for name in [
        "type_c::USB_C_Receptacle_2_0",
        "gct_usb4105::GCT_USB4105_2_0",
    ] {
        let pins = c.world.devices[&format!("usb::connectors::{name}")].pins_for(None);
        for (name, numbers) in [
            ("GND", "A1,A12,B1,B12"),
            ("VBUS", "A4,A9,B4,B9"),
            ("CC1", "A5"),
            ("CC2", "B5"),
            ("DP", "A6,B6"),
            ("DN", "A7,B7"),
            ("SBU1", "A8"),
            ("SBU2", "B8"),
            ("SHIELD", "SH1,SH2,SH3,SH4"),
        ] {
            let pin = pins.iter().find(|p| p.name.name == name).unwrap();
            assert_eq!(
                pin.numbers
                    .iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                numbers
            );
            assert_eq!(
                pin.role_or_default(),
                if name == "DP" || name == "DN" {
                    PinRole::Bidirectional
                } else {
                    PinRole::Passive
                }
            );
        }
        assert_eq!(pins.len(), 9);
    }
}

#[test]
fn gct_contact_lands_slots_and_locating_holes_match_drawing_b4() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    let prefix = "usb::connectors::gct_usb4105";
    let fp = &c.world.footprints[&format!("{prefix}::FP_USB_C_GCT_USB4105")];
    // Drawing contact row, left to right. Four power lands serve paired IDs.
    let row = [
        ("A1 B12", "-3.2", "0.6"),
        ("A4 B9", "-2.4", "0.6"),
        ("B8", "-1.75", "0.3"),
        ("A5", "-1.25", "0.3"),
        ("B7", "-0.75", "0.3"),
        ("A6", "-0.25", "0.3"),
        ("A7", "0.25", "0.3"),
        ("B6", "0.75", "0.3"),
        ("A8", "1.25", "0.3"),
        ("B5", "1.75", "0.3"),
        ("B4 A9", "2.4", "0.6"),
        ("B1 A12", "3.2", "0.6"),
    ];
    assert_eq!(fp.pads.len(), 20);
    for (names, x, width) in row {
        for name in names.split_whitespace() {
            let pad = fp.pads.iter().find(|p| p.number.text == name).unwrap();
            assert_eq!(pad.x.text, format!("{x}mm"));
            assert_eq!(pad.y.text, "-7.355mm");
            let geom = &c.world.pads[&pad.pad.name];
            assert_eq!(
                geom.size.iter().map(|u| u.text.clone()).collect::<Vec<_>>(),
                [format!("{width}mm"), "1.15mm".into()]
            );
        }
    }
    for (name, x, y, height, slot) in [
        ("SH1", "-4.32mm", "-6.78mm", "2.1mm", "1.7mm"),
        ("SH2", "4.32mm", "-6.78mm", "2.1mm", "1.7mm"),
        ("SH3", "-4.32mm", "-2.6mm", "1.8mm", "1.4mm"),
        ("SH4", "4.32mm", "-2.6mm", "1.8mm", "1.4mm"),
    ] {
        let pad = fp.pads.iter().find(|p| p.number.text == name).unwrap();
        assert_eq!((pad.x.text.as_str(), pad.y.text.as_str()), (x, y));
        let geom = &c.world.pads[&pad.pad.name];
        assert_eq!(
            geom.size
                .iter()
                .map(|v| v.text.as_str())
                .collect::<Vec<_>>(),
            ["1mm", height]
        );
        let Some((PadDrill::Slot(w, h), _)) = &geom.drill else {
            panic!("{name} must be a slot")
        };
        assert_eq!((w.text.as_str(), h.text.as_str()), ("0.6mm", slot));
    }
    assert_eq!(fp.mount_holes.len(), 2);
    for (hole, x) in fp.mount_holes.iter().zip(["-2.89mm", "2.89mm"]) {
        assert_eq!((hole.x.text.as_str(), hole.y.text.as_str()), (x, "-6.28mm"));
        let MountHoleGeom::Diameter(d) = &hole.geom else {
            panic!("round locating hole")
        };
        assert_eq!(d.text, "0.65mm");
    }
}

#[test]
fn a_direct_connector_pair_builds_without_a_resistor_workaround() {
    for (part, mpn) in [
        ("type_c::USB_C_HRO_TYPE_C_31_M_12", "TYPE-C-31-M-12"),
        ("gct_usb4105::USB_C_GCT_USB4105_GF_A", "USB4105-GF-A"),
    ] {
        let mut p = project();
        let mut source = format!(
            "design Pair {{ inst a: usb::connectors::{part}\ninst b: usb::connectors::{part}\n"
        );
        for pin in [
            "GND", "VBUS", "CC1", "CC2", "DP", "DN", "SBU1", "SBU2", "SHIELD",
        ] {
            source.push_str(&format!("net {pin}: a.{pin}, b.{pin}\n"));
        }
        source.push_str("}\n");
        p.files.push(("src/smoke.cohdl".into(), source));
        let mut c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
        let out = build_artifacts(&mut c, &cohdl::lock::LockState::default());
        assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
        let out = out.unwrap();
        assert!(out.bom.contains(mpn));
        assert!(out.bom.contains("J1,J2"));
        for pin in [
            "A1", "A4", "A5", "A6", "A7", "A8", "A9", "A12", "B1", "B4", "B5", "B6", "B7", "B8",
            "B9", "B12", "SH1", "SH2", "SH3", "SH4",
        ] {
            assert_eq!(out.netlist.matches(&format!("(pin \"{pin}\")")).count(), 2);
        }
    }
}

#[test]
fn two_real_sources_still_fail_the_driver_rule() {
    let mut p = project();
    p.files.push(("src/conflict.cohdl".into(), "device Supply { pins { required OUT: 1 [power_out] } }\ndesign Conflict { inst a: Supply\ninst b: Supply\nnet VBUS: a.OUT, b.OUT\n}\n".into()));
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    assert!(c.diags.has_errors());
    assert!(c.diags.render(&c.sm).contains("D004"));
}

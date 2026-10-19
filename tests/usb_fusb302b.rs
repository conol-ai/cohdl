//! Manufacturer-oracle regression: FUSB302B/D Fig. 5 p5 and case 510BR p31.
use cohdl::ast::{Obligation, PinRole};
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
fn mlp_pin_groups_addresses_and_manufacturer_lands() {
    let project = project();
    let checked = check_files_in_with_deps("usb", &["std".into()], &project.files, None).unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let prefix = "usb::pd::fusb302b";
    let pins = checked.world.devices[&format!("{prefix}::FUSB302B_MLP")].pins_for(None);
    assert_eq!(pins.len(), 9);
    for (name, numbers, role) in [
        ("CC2", vec!["1", "14"], PinRole::Bidirectional),
        ("VBUS", vec!["2"], PinRole::Input),
        ("VDD", vec!["3", "4"], PinRole::PowerIn),
        ("INT_N", vec!["5"], PinRole::Output),
        ("SCL", vec!["6"], PinRole::Input),
        ("SDA", vec!["7"], PinRole::Bidirectional),
        ("GND", vec!["8", "9", "15"], PinRole::PowerIn),
        ("CC1", vec!["10", "11"], PinRole::Bidirectional),
        ("VCONN", vec!["12", "13"], PinRole::PowerIn),
    ] {
        let pin = pins.iter().find(|pin| pin.name.name == name).unwrap();
        assert_eq!(
            pin.numbers
                .iter()
                .map(|n| n.text.as_str())
                .collect::<Vec<_>>(),
            numbers,
            "{name}"
        );
        assert_eq!(pin.role_or_default(), role);
        assert_eq!(pin.obligation, Obligation::Required);
    }
    for (mpn, address) in [
        ("FUSB302BMPX", "0x22"),
        ("FUSB302B01MPX", "0x23"),
        ("FUSB302B10MPX", "0x24"),
        ("FUSB302B11MPX", "0x25"),
    ] {
        let key = format!("{prefix}::PHY_{mpn}");
        let part = &checked.world.parts[&key];
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert!(
            part.alts.is_empty(),
            "address variants are not drop-in AVL alternates"
        );
        assert!(checked.world.intents[&key].contains(address));
        assert_eq!(checked.world.docs[&key][0], "docs/fusb302b-application.md");
    }
    let fp = &checked.world.footprints[&format!("{prefix}::QFN14N50P250X250_1EP145X145")];
    let positions: Vec<_> = fp
        .pads
        .iter()
        .map(|p| (p.number.text.as_str(), p.x.text.as_str(), p.y.text.as_str()))
        .collect();
    assert_eq!(
        positions,
        vec![
            ("1", "-1.215mm", "-0.5mm"),
            ("2", "-1.215mm", "0mm"),
            ("3", "-1.215mm", "0.5mm"),
            ("4", "-0.75mm", "1.215mm"),
            ("5", "-0.25mm", "1.215mm"),
            ("6", "0.25mm", "1.215mm"),
            ("7", "0.75mm", "1.215mm"),
            ("8", "1.215mm", "0.5mm"),
            ("9", "1.215mm", "0mm"),
            ("10", "1.215mm", "-0.5mm"),
            ("11", "0.75mm", "-1.215mm"),
            ("12", "0.25mm", "-1.215mm"),
            ("13", "-0.25mm", "-1.215mm"),
            ("14", "-0.75mm", "-1.215mm"),
            ("15", "0mm", "0mm"),
        ]
    );
    for (pad, size) in [
        ("P_510BR_H", ["0.58mm", "0.3mm"]),
        ("P_510BR_V", ["0.3mm", "0.58mm"]),
        ("P_510BR_EP", ["1.45mm", "1.45mm"]),
    ] {
        assert_eq!(
            checked.world.pads[&format!("{prefix}::{pad}")]
                .size
                .iter()
                .map(|v| v.text.as_str())
                .collect::<Vec<_>>(),
            size
        );
    }
    let mut diags = cohdl::diag::Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&checked.world, &mut diags);
    assert!(!diags.has_errors(), "{}", diags.render(&checked.sm));
}

#[test]
fn all_address_variants_build_with_exposed_ground_and_exact_bom_identity() {
    for mpn in [
        "FUSB302BMPX",
        "FUSB302B01MPX",
        "FUSB302B10MPX",
        "FUSB302B11MPX",
    ] {
        let mut project = project();
        project.files.push((
            "src/smoke.cohdl".into(),
            format!(
                r#"
        design Smoke {{
            inst phy: usb::pd::fusb302b::PHY_{mpn}
            net VDD [3.3V]: phy.VDD
            net GND [0V]: phy.GND
            net VCONN [3.3V]: phy.VCONN
            net VBUS [5V]: phy.VBUS
            net CC1: phy.CC1
            net CC2: phy.CC2
            net SCL: phy.SCL
            net SDA: phy.SDA
            net IRQ: phy.INT_N
        }}"#
            ),
        ));
        let mut checked =
            check_files_in_with_deps("usb", &["std".into()], &project.files, None).unwrap();
        let out = build_artifacts(&mut checked, &cohdl::lock::LockState::default());
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
        let out = out.expect("real build");
        assert!(out.bom.contains(mpn));
        assert!(
            out.netlist.contains("(pin \"15\")"),
            "exposed ground is a real netlist terminal"
        );
    }
}

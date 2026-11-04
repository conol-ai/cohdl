//! Manufacturer pin-map oracles, not a comparison between generated outputs.
//! See each package's docs/sources.md for the retained PDF revisions/pages.

use cohdl::ast::{Obligation, PinRole};
use cohdl::pipeline::{build_artifacts, check_files_in_with_deps, Checked};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

struct Mcu {
    package: &'static str,
    namespace: &'static str,
    device: &'static str,
    mpn: &'static str,
    manufacturer: &'static str,
    footprint: &'static str,
    // Physical pad order, read independently from the manufacturer figures.
    pins: &'static str,
    supplies: &'static str,
    boot: bool,
    reset_role: PinRole,
}

const MCUS: &[Mcu] = &[
    Mcu {
        package: "@gigadevice/gd32",
        namespace: "gigadevice_gd32",
        device: "GD32E230F8_TSSOP20",
        mpn: "GD32E230F8P6TR",
        manufacturer: "GigaDevice",
        footprint: "soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM",
        // GD32E230xx Rev3.1 Fig2-8 and Table2-11, PDF pp15,35-36.
        pins: "BOOT0 PF0 PF1 NRST VDDA PA0 PA1 PA2 PA3 PA4 PA5 PA6 PA7 PB1 VSS VDD PA9 PA10 PA13 PA14",
        supplies: "VDDA VSS VDD",
        boot: true,
        reset_role: PinRole::Bidirectional,
    },
    Mcu {
        package: "@gigadevice/gd32",
        namespace: "gigadevice_gd32",
        device: "GD32F303CB_LQFP48",
        mpn: "GD32F303CBT6",
        manufacturer: "GigaDevice",
        footprint: "qfp::QFP50P900X900X160_48N",
        // GD32F303xB Rev1.1 Fig2-4 and Table2-5, PDF pp12,30-32.
        pins: "VBAT PC13 PC14 PC15 PD0 PD1 NRST VSSA VDDA PA0 PA1 PA2 PA3 PA4 PA5 PA6 PA7 PB0 PB1 PB2 PB10 PB11 VSS_1 VDD_1 PB12 PB13 PB14 PB15 PA8 PA9 PA10 PA11 PA12 PA13 VSS_2 VDD_2 PA14 PA15 PB3 PB4 PB5 PB6 PB7 BOOT0 PB8 PB9 VSS_3 VDD_3",
        supplies: "VBAT VSSA VDDA VSS_1 VDD_1 VSS_2 VDD_2 VSS_3 VDD_3",
        boot: true,
        reset_role: PinRole::Bidirectional,
    },
    Mcu {
        package: "@wch/ch32",
        namespace: "wch_ch32",
        device: "CH32V003F4_TSSOP20",
        mpn: "CH32V003F4P6",
        manufacturer: "WCH",
        footprint: "soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM",
        // CH32V003DS0 V1.8 section2.1 and Table2-1, PDF pp12-14.
        pins: "PD4 PD5 PD6 PD7 PA1 PA2 VSS PD0 VDD PC0 PC1 PC2 PC3 PC4 PC5 PC6 PC7 PD1 PD2 PD3",
        supplies: "VSS VDD",
        boot: false,
        reset_role: PinRole::Bidirectional,
    },
    Mcu {
        package: "@wch/ch32",
        namespace: "wch_ch32",
        device: "CH32V203C8_LQFP48",
        mpn: "CH32V203C8T6",
        manufacturer: "WCH",
        footprint: "qfp::KICAD_LQFP_48_7X7MM_P0_5MM",
        // CH32V203DS0 V3.0 section3.1 and Table3-1-1, PDF pp21,23-26.
        pins: "VBAT PC13 PC14 PC15 PD0 PD1 NRST VSSA VDDA PA0 PA1 PA2 PA3 PA4 PA5 PA6 PA7 PB0 PB1 PB2 PB10 PB11 VSS_1 VDD_IO_1 PB12 PB13 PB14 PB15 PA8 PA9 PA10 PA11 PA12 PA13 VSS_2 VDD_2 PA14 PA15 PB3 PB4 PB5 PB6 PB7 BOOT0 PB8 PB9 VSS_3 VDD_IO_3",
        supplies: "VBAT VSSA VDDA VSS_1 VDD_IO_1 VSS_2 VDD_2 VSS_3 VDD_IO_3",
        boot: true,
        reset_role: PinRole::Input,
    },
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn project(mcu: &Mcu) -> cohdl::project::Project {
    let root = root();
    let deps = ["std", "qfp", "soic"].map(|name| (name.to_string(), root.join("lib").join(name)));
    cohdl::project::load_project_with_deps(&root.join("lib").join(mcu.package), &deps).unwrap()
}

fn check(mcu: &Mcu, extra: Option<String>) -> Checked {
    let mut project = project(mcu);
    if let Some(extra) = extra {
        project.files.push(("src/consumer.cohdl".into(), extra));
    }
    check_files_in_with_deps(
        &project.name,
        &["std".into(), "qfp".into(), "soic".into()],
        &project.files,
        None,
    )
    .unwrap()
}

fn assert_clean(checked: &Checked) {
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
}

// A structural export fixture, not a reference application circuit.
fn consumer(mcu: &Mcu, omitted: Option<&str>) -> String {
    let mut source = format!(
        "design Smoke {{\n inst mcu: {}::MCU_{}\n",
        mcu.namespace, mcu.mpn
    );
    for pin in mcu.pins.split_whitespace() {
        if Some(pin) != omitted {
            source.push_str(&format!(" net N_{pin}: mcu.{pin}\n"));
        }
    }
    source.push_str("}\n");
    source
}

#[test]
fn every_physical_pin_matches_the_manufacturer_oracle() {
    for mcu in MCUS {
        let checked = check(mcu, None);
        assert_clean(&checked);
        let device = &checked.world.devices[&format!("{}::{}", mcu.namespace, mcu.device)];
        let pins = device.pins_for(None);
        let oracle = mcu.pins.split_whitespace().collect::<Vec<_>>();
        assert_eq!(pins.len(), oracle.len(), "{}: phantom/missing pin", mcu.mpn);
        let supplies = mcu.supplies.split_whitespace().collect::<BTreeSet<_>>();
        for (index, name) in oracle.iter().enumerate() {
            let pin = pins.iter().find(|pin| pin.name.name == *name).unwrap();
            assert_eq!(
                pin.numbers.len(),
                1,
                "{} {name}: no merged supplies",
                mcu.mpn
            );
            assert_eq!(
                pin.numbers[0].text,
                (index + 1).to_string(),
                "{} {name}",
                mcu.mpn
            );
            let required = supplies.contains(name) || *name == "BOOT0";
            assert_eq!(
                pin.obligation,
                if required {
                    Obligation::Required
                } else {
                    Obligation::Optional
                }
            );
            let role = if supplies.contains(name) {
                PinRole::PowerIn
            } else if *name == "BOOT0" {
                PinRole::Input
            } else if *name == "NRST" {
                mcu.reset_role
            } else {
                PinRole::Bidirectional
            };
            assert_eq!(pin.role_or_default(), role, "{} {name}", mcu.mpn);
        }
        let part = &checked.world.parts[&format!("{}::MCU_{}", mcu.namespace, mcu.mpn)];
        assert_eq!(
            part.device.name.name,
            format!("{}::{}", mcu.namespace, mcu.device)
        );
        assert_eq!(part.primary.field("mpn").unwrap().value, mcu.mpn);
        assert_eq!(part.primary.field("mfr").unwrap().value, mcu.manufacturer);
        assert_eq!(part.primary.footprint.as_ref().unwrap().name, mcu.footprint);
        assert!(
            part.alts.is_empty(),
            "cross-vendor or package substitution is not established"
        );
        let footprint = &checked.world.footprints[mcu.footprint];
        let actual = footprint
            .pads
            .iter()
            .map(|p| p.number.text.parse::<usize>().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual,
            (1..=oracle.len()).collect(),
            "no QFN exposed pad in a leaded package"
        );
    }
}

#[test]
fn exact_parts_build_and_every_net_uses_the_expected_package_pin() {
    for mcu in MCUS {
        let mut checked = check(mcu, Some(consumer(mcu, None)));
        assert_clean(&checked);
        let artifacts = build_artifacts(&mut checked, &cohdl::lock::LockState::default());
        assert_clean(&checked);
        let artifacts = artifacts.expect("real netlist and BOM");
        assert!(artifacts.bom.contains(mcu.mpn));
        for (index, pin) in mcu.pins.split_whitespace().enumerate() {
            let name = format!("(name \"N_{pin}\")");
            let tail = artifacts.netlist.split_once(&name).expect(&name).1;
            let net = tail.split("(net (code").next().unwrap();
            assert!(
                net.contains(&format!("(pin \"{}\")", index + 1)),
                "{} {pin}: {net}",
                mcu.mpn
            );
        }
    }
}

#[test]
fn missing_supply_or_boot_connections_are_rejected() {
    for mcu in MCUS {
        let mut required = mcu.supplies.split_whitespace().collect::<Vec<_>>();
        if mcu.boot {
            required.push("BOOT0");
        }
        for pin in required {
            let mut checked = check(mcu, Some(consumer(mcu, Some(pin))));
            let diagnostics = checked.diags.render(&checked.sm);
            assert!(
                checked.diags.has_errors(),
                "{}: omitted {pin} accepted",
                mcu.mpn
            );
            assert!(diagnostics.contains(pin), "{diagnostics}");
            assert!(build_artifacts(&mut checked, &cohdl::lock::LockState::default()).is_none());
        }
    }
}

#[test]
fn footprint_copper_and_orientation_match_documented_bindings() {
    // All four lands sit in the assembler's frame (soic 0.3.0, qfp 0.3.0;
    // library plan W3): pin 1 at the left end of the lower row.
    for (mcu, size, first, last) in [
        (
            &MCUS[0],
            ["1.475mm", "0.4mm"],
            ["-2.925mm", "2.8625mm"],
            ["-2.925mm", "-2.8625mm"],
        ),
        (
            &MCUS[1],
            ["1.2mm", "0.3mm"],
            ["-2.75mm", "4.25mm"],
            ["-4.25mm", "2.75mm"],
        ),
        (
            &MCUS[2],
            ["1.475mm", "0.4mm"],
            ["-2.925mm", "2.8625mm"],
            ["-2.925mm", "-2.8625mm"],
        ),
        (
            &MCUS[3],
            ["1.475mm", "0.3mm"],
            ["-2.75mm", "4.1625mm"],
            ["-4.1625mm", "2.75mm"],
        ),
    ] {
        let checked = check(mcu, None);
        assert_clean(&checked);
        let fp = &checked.world.footprints[mcu.footprint];
        let p1 = &fp.pads[0];
        let pn = fp.pads.last().unwrap();
        let pad = &checked.world.pads[&p1.pad.name];
        assert_eq!([pad.size[0].text.as_str(), pad.size[1].text.as_str()], size);
        assert_eq!([p1.x.text.as_str(), p1.y.text.as_str()], first);
        assert_eq!([pn.x.text.as_str(), pn.y.text.as_str()], last);
        let mut diagnostics = cohdl::diag::Diagnostics::new();
        cohdl::check::footprints::check_pad_consistency(&checked.world, &mut diagnostics);
        assert!(
            !diagnostics.has_errors(),
            "{}",
            diagnostics.render(&checked.sm)
        );
    }
}

#[test]
fn retained_datasheets_and_public_document_links_are_complete() {
    for (path, hash) in [
        (
            "@gigadevice/gd32/docs/gd32e230-rev3.1.pdf",
            "aa564218bdad841f82027ae85559d66301004c204001109e9b646c86f0d172c2",
        ),
        (
            "@gigadevice/gd32/docs/gd32f303xb-rev1.1.pdf",
            "a485cbdce30f6e52f787537220230324421cbfd4864fc74f2d852830bf602e74",
        ),
        (
            "@wch/ch32/docs/ch32v003-v1.8.pdf",
            "186210d35123325ea49305e3a0152cbfa6e377d8e6efc2e610e3497daabe17f8",
        ),
        (
            "@wch/ch32/docs/ch32v203-v3.0.pdf",
            "10af5de2779ce1e985162dceae2e829800b2d633d34491333b1ce90f4ffcbdd2",
        ),
    ] {
        let bytes = std::fs::read(root().join("lib").join(path)).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert_eq!(cohdl::hash::sha256_hex(&bytes), hash, "{path}");
    }
    for mcu in MCUS {
        let checked = check(mcu, None);
        for symbol in [
            format!("{}::{}", mcu.namespace, mcu.device),
            format!("{}::MCU_{}", mcu.namespace, mcu.mpn),
        ] {
            let docs = &checked.world.docs[&symbol];
            assert!(!docs.is_empty());
            for doc in docs {
                assert!(!Path::new(doc).is_absolute());
                assert!(
                    root().join("lib").join(mcu.package).join(doc).is_file(),
                    "{symbol}: {doc}"
                );
            }
        }
    }
}

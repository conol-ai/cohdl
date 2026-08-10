//! Datasheet-locked checks for `ti_power_switch::EFUSE_TPS259823ONRGET`.

use cohdl::ast::{Obligation, PadPaste, PinRole};
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in_with_deps};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_package(with_fixture: bool) -> cohdl::pipeline::Checked {
    let repository = root();
    let deps = vec![("std".to_string(), repository.join("lib/std"))];
    let mut project =
        cohdl::project::load_project_with_deps(&repository.join("lib/@ti/power-switch"), &deps)
            .unwrap();
    if with_fixture {
        project.files.push((
            "src/build_fixture.cohdl".to_string(),
            r#"
design TPS25982_BUILD_FIXTURE {
    inst protection: EFUSE_TPS259823ONRGET
    nc: protection.IN, protection.GND, protection.EN_UVLO,
        protection.ITIMER, protection.ILIM, protection.IMON,
        protection.RETRY_DLY, protection.NRETRY, protection.LDSTRT,
        protection.PG, protection.dVdt, protection.OUT
}
"#
            .to_string(),
        ));
    }
    let mut checked = check_files_in_with_deps(
        &project.name,
        &["std".to_string()],
        &project.files,
        with_fixture.then_some("TPS25982_BUILD_FIXTURE"),
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
fn tps259823onrget_pinout_and_orderable_part_match_rev_d() {
    let checked = load_package(false);
    let device = &checked.world.devices["ti_power_switch::TPS25982"];
    let pins = device.pins_for(None);
    let expected: &[(&str, &[&str], PinRole, Obligation)] = &[
        (
            "IN",
            &["1", "2", "3", "16", "25"],
            PinRole::PowerIn,
            Obligation::Required,
        ),
        (
            "GND",
            &["4", "5", "14", "26"],
            PinRole::PowerIn,
            Obligation::Required,
        ),
        ("EN_UVLO", &["6"], PinRole::Input, Obligation::Required),
        ("ITIMER", &["7"], PinRole::Output, Obligation::Optional),
        ("ILIM", &["8"], PinRole::Output, Obligation::Required),
        ("IMON", &["9"], PinRole::Output, Obligation::Optional),
        ("RETRY_DLY", &["10"], PinRole::Output, Obligation::Optional),
        ("NRETRY", &["11"], PinRole::Output, Obligation::Optional),
        ("LDSTRT", &["12"], PinRole::Input, Obligation::Required),
        ("PG", &["13"], PinRole::Output, Obligation::Optional),
        ("dVdt", &["15"], PinRole::Output, Obligation::Optional),
        (
            "OUT",
            &["17", "18", "19", "20", "21", "22", "23", "24"],
            PinRole::PowerOut,
            Obligation::Required,
        ),
    ];
    assert_eq!(pins.len(), expected.len());
    for (name, numbers, role, obligation) in expected {
        let pin = pins
            .iter()
            .find(|pin| pin.name.name == *name)
            .unwrap_or_else(|| panic!("missing TPS25982 pin {name}"));
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            *numbers,
            "wrong physical assignment for {name}"
        );
        assert_eq!(pin.role_or_default(), *role, "wrong role for {name}");
        assert_eq!(pin.obligation, *obligation, "wrong obligation for {name}");
    }

    let part = &checked.world.parts["ti_power_switch::EFUSE_TPS259823ONRGET"];
    assert_eq!(part.device.name.name, "ti_power_switch::TPS25982");
    assert!(part.device.variant.is_none());
    assert_eq!(
        part.primary.field("mfr").unwrap().value,
        "Texas Instruments"
    );
    assert_eq!(part.primary.field("mpn").unwrap().value, "TPS259823ONRGET");
    assert_eq!(
        part.primary.footprint.as_ref().unwrap().name,
        "ti_power_switch::TI_RGE0024M"
    );
    assert!(part.alts.is_empty());
}

#[test]
fn rge0024m_lands_vias_and_stencil_match_ti_drawing_4223975_b() {
    let checked = load_package(false);
    let world = &checked.world;
    let footprint = &world.footprints["ti_power_switch::TI_RGE0024M"];
    assert_eq!(footprint.pads.len(), 39);
    assert_eq!(
        footprint
            .pads
            .iter()
            .filter(|pad| pad.number.text == "25")
            .count(),
        9
    );
    assert_eq!(
        footprint
            .pads
            .iter()
            .filter(|pad| pad.number.text == "26")
            .count(),
        6
    );
    let electrical_numbers: BTreeSet<_> = footprint
        .pads
        .iter()
        .map(|pad| pad.number.text.clone())
        .collect();
    let expected_numbers: BTreeSet<_> = (1..=26).map(|number| number.to_string()).collect();
    assert_eq!(electrical_numbers, expected_numbers);

    let actual: BTreeSet<_> = footprint
        .pads
        .iter()
        .map(|pad| {
            format!(
                "{}|{}|{}|{}|{}",
                pad.number.text, pad.pad.name, pad.x.text, pad.y.text, pad.rotate
            )
        })
        .collect();
    let expected: BTreeSet<_> = [
        "1|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|-1.25mm|0",
        "2|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|-0.75mm|0",
        "3|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|-0.25mm|0",
        "4|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|0.25mm|0",
        "5|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|0.75mm|0",
        "6|ti_power_switch::P_TI_RGE0024M_LEAD_H|-1.9125mm|1.25mm|0",
        "7|ti_power_switch::P_TI_RGE0024M_LEAD_V|-1.25mm|1.9125mm|0",
        "8|ti_power_switch::P_TI_RGE0024M_LEAD_V|-0.75mm|1.9125mm|0",
        "9|ti_power_switch::P_TI_RGE0024M_LEAD_V|-0.25mm|1.9125mm|0",
        "10|ti_power_switch::P_TI_RGE0024M_LEAD_V|0.25mm|1.9125mm|0",
        "11|ti_power_switch::P_TI_RGE0024M_LEAD_V|0.75mm|1.9125mm|0",
        "12|ti_power_switch::P_TI_RGE0024M_LEAD_V|1.25mm|1.9125mm|0",
        "13|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|1.25mm|0",
        "14|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|0.75mm|0",
        "15|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|0.25mm|0",
        "16|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|-0.25mm|0",
        "17|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|-0.75mm|0",
        "18|ti_power_switch::P_TI_RGE0024M_LEAD_H|1.9125mm|-1.25mm|0",
        "19|ti_power_switch::P_TI_RGE0024M_LEAD_V|1.25mm|-1.9125mm|0",
        "20|ti_power_switch::P_TI_RGE0024M_LEAD_V|0.75mm|-1.9125mm|0",
        "21|ti_power_switch::P_TI_RGE0024M_LEAD_V|0.25mm|-1.9125mm|0",
        "22|ti_power_switch::P_TI_RGE0024M_LEAD_V|-0.25mm|-1.9125mm|0",
        "23|ti_power_switch::P_TI_RGE0024M_LEAD_V|-0.75mm|-1.9125mm|0",
        "24|ti_power_switch::P_TI_RGE0024M_LEAD_V|-1.25mm|-1.9125mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_EP25_COPPER|0mm|-0.625mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_EP25_PASTE|-0.694mm|-0.625mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_EP25_PASTE|0.694mm|-0.625mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|-1.1mm|-1.175mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|0mm|-1.175mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|1.1mm|-1.175mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|-1.1mm|-0.075mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|0mm|-0.075mm|0",
        "25|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|1.1mm|-0.075mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_EP26_COPPER|0mm|0.925mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_EP26_PASTE|-0.694mm|0.925mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_EP26_PASTE|0.694mm|0.925mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|-1.1mm|0.925mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|0mm|0.925mm|0",
        "26|ti_power_switch::P_TI_RGE0024M_THERMAL_VIA|1.1mm|0.925mm|0",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(actual, expected);

    for (name, size) in [
        ("P_TI_RGE0024M_LEAD_H", &["0.575mm", "0.24mm"][..]),
        ("P_TI_RGE0024M_LEAD_V", &["0.24mm", "0.575mm"][..]),
        ("P_TI_RGE0024M_EP25_COPPER", &["2.7mm", "1.45mm"][..]),
        ("P_TI_RGE0024M_EP26_COPPER", &["2.7mm", "0.85mm"][..]),
        ("P_TI_RGE0024M_EP25_PASTE", &["1.188mm", "1.3mm"][..]),
        ("P_TI_RGE0024M_EP26_PASTE", &["1.188mm", "0.76mm"][..]),
        ("P_TI_RGE0024M_THERMAL_VIA", &["0.4mm"][..]),
    ] {
        let pad = &world.pads[&format!("ti_power_switch::{name}")];
        assert_eq!(
            pad.size
                .iter()
                .map(|dimension| dimension.text.as_str())
                .collect::<Vec<_>>(),
            size,
            "wrong geometry for {name}"
        );
    }
    for name in [
        "P_TI_RGE0024M_LEAD_H",
        "P_TI_RGE0024M_LEAD_V",
        "P_TI_RGE0024M_EP25_COPPER",
        "P_TI_RGE0024M_EP26_COPPER",
    ] {
        assert_eq!(
            world.pads[&format!("ti_power_switch::{name}")]
                .mask_expansion
                .as_ref()
                .unwrap()
                .0
                .text,
            "0.07mm"
        );
    }
    for name in ["P_TI_RGE0024M_EP25_COPPER", "P_TI_RGE0024M_EP26_COPPER"] {
        assert!(matches!(
            world.pads[&format!("ti_power_switch::{name}")]
                .paste
                .as_ref()
                .unwrap()
                .0,
            PadPaste::None
        ));
    }
    let via = &world.pads["ti_power_switch::P_TI_RGE0024M_THERMAL_VIA"];
    assert_eq!(via.drill.as_ref().unwrap().0.values()[0].text, "0.2mm");
}

#[test]
fn rge0024m_builds_and_projects_all_39_pad_features() {
    let mut checked = load_package(true);
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        artifacts.is_some() && !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let modules =
        cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, checked.ir.as_ref().unwrap());
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].0, "ti_power_switch::TI_RGE0024M");
    let module = &modules[0].2;
    assert_eq!(module.matches("(pad \"").count(), 39, "{module}");
    assert_eq!(module.matches("(drill 0.2)").count(), 9, "{module}");
    assert_eq!(
        module.matches("(pad \"25\"").count(),
        9,
        "pad 25 features:\n{module}"
    );
    assert_eq!(
        module.matches("(pad \"26\"").count(),
        6,
        "pad 26 features:\n{module}"
    );
    assert!(
        module.contains("(size 2.7 1.45) (layers \"F.Cu\" \"F.Mask\")"),
        "EP25 must suppress full-area paste:\n{module}"
    );
    assert!(
        module.contains("(size 2.7 0.85) (layers \"F.Cu\" \"F.Mask\")"),
        "EP26 must suppress full-area paste:\n{module}"
    );
    assert!(
        module.contains("(size 1.188 1.3) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\")"),
        "EP25 stencil aperture missing:\n{module}"
    );
    assert!(
        module.contains("(size 1.188 0.76) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\")"),
        "EP26 stencil aperture missing:\n{module}"
    );
}

#[test]
fn vendored_tps25982_pdf_is_the_audited_rev_d_file() {
    let pdf = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/@ti/power-switch/docs/tps25982.pdf"),
    )
    .unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert_eq!(
        cohdl::hash::sha256_hex(&pdf),
        "655c21dbf6b91a3c98b7ab26cd3bf0020d3eb7d765f411c71030d22fa4b6bc1f"
    );
}

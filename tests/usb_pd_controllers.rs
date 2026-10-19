//! Independent manufacturer table/land oracles for the additional PD families.
use cohdl::ast::PinRole;
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
fn manufacturer_pin_tables_and_direction_traps() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    // Physical pin order from NXP Table 4 pp8-9, ST DS11254 Table 1 pp3-4,
    // and ST DS12499 Table 1 p4. Ground EP numbers are library conventions.
    for (device, expected) in [
        ("ptn5110::PTN5110_HX2QFN16", "1=FRS_EN;2=EN_SNK1;3=VDD;4=BYPASS;5=SLV_ADDR;6=ILIM_5V_VBUS;7=I2C_SDA;8=I2C_SCL;9=DBG_ACC;10=ALERT_N;11=FAULT_N;12=VCONN_IN;13=CC1;14=CC2;15=VBUS;16=EN_SRC;17=GND"),
        ("stusb1602::STUSB1602A_QFN24", "1=CC1DB;2=CC1;3=VCONN;4=CC2;5=CC2DB;6=RESET;7=SCL;8=SDA;9=ALERT_N;10=GND;11=MOSI;12=NSS;13=ADDR0;14=MISO;15=TX_EN;16=SCLK;17=A_B_SIDE;18=VBUS_SENSE;19=VBUS_EN_SNK;20=VBUS_EN_SRC;21=VREG_1V2;22=VSYS;23=VREG_2V7;24=VDD;25=GND"),
        ("stusb4500::STUSB4500_QFN24", "1=CC1DB;2=CC1;3=NC;4=CC2;5=CC2DB;6=RESET;7=SCL;8=SDA;9=DISCH;10=GND;11=ATTACH;12=ADDR0;13=ADDR1;14=POWER_OK3;15=GPIO;16=VBUS_EN_SNK;17=A_B_SIDE;18=VBUS_VS_DISCH;19=ALERT;20=POWER_OK2;21=VREG_1V2;22=VSYS;23=VREG_2V7;24=VDD;25=GND"),
    ] {
        let pins = c.world.devices[&format!("usb::pd::{device}")].pins_for(None);
        let mut actual: Vec<_> = pins.iter().flat_map(|p| p.numbers.iter().map(move |n| (n.text.parse::<u32>().unwrap(), p.name.name.as_str()))).collect();
        actual.sort();
        assert_eq!(actual.iter().map(|(n,p)| format!("{n}={p}")).collect::<Vec<_>>().join(";"), expected, "{device}");
    }
    for (device, pin, role) in [
        ("stusb1602::STUSB1602A_QFN24", "MOSI", PinRole::Output),
        ("stusb1602::STUSB1602A_QFN24", "MISO", PinRole::Input),
        ("stusb1602::STUSB1602A_QFN24", "SCLK", PinRole::Output),
        ("stusb1602::STUSB1602A_QFN24", "TX_EN", PinRole::Input),
        ("stusb1602::STUSB1602A_QFN24", "VREG_1V2", PinRole::PowerOut),
        (
            "stusb4500::STUSB4500_QFN24",
            "VBUS_VS_DISCH",
            PinRole::Bidirectional,
        ),
        (
            "stusb4500::STUSB4500_QFN24",
            "DISCH",
            PinRole::Bidirectional,
        ),
        ("ptn5110::PTN5110_HX2QFN16", "BYPASS", PinRole::PowerOut),
    ] {
        let pins = c.world.devices[&format!("usb::pd::{device}")].pins_for(None);
        assert_eq!(
            pins.iter()
                .find(|p| p.name.name == pin)
                .unwrap()
                .role_or_default(),
            role,
            "{device}.{pin}"
        );
    }
}

#[test]
fn exact_order_codes_are_not_substitution_groups_and_emit_real_ground_pads() {
    for (family, prefix, device, mpn, ep, expected_intent) in [
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110HQZ",
            "17",
            "TCPC 1.0 v1.1; POR role DRP",
        ),
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110DHQZ",
            "17",
            "TCPC 1.0 v1.1; POR role sink",
        ),
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110THQZ",
            "17",
            "TCPC 1.0 v1.1; POR role source",
        ),
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110NHQZ",
            "17",
            "TCPC 2.0 v1.0; POR role DRP",
        ),
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110NDHQZ",
            "17",
            "TCPC 2.0 v1.0; POR role sink",
        ),
        (
            "ptn5110",
            "TCPC",
            "PTN5110_HX2QFN16",
            "PTN5110NTHQZ",
            "17",
            "TCPC 2.0 v1.0; POR role source",
        ),
        (
            "stusb1602",
            "PHY",
            "STUSB1602A_QFN24",
            "STUSB1602AQTR",
            "25",
            "SPI master",
        ),
        (
            "stusb4500",
            "SINK",
            "STUSB4500_QFN24",
            "STUSB4500QTR",
            "25",
            "negotiates power",
        ),
    ] {
        let mut p = project();
        let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
        let symbol = format!("usb::pd::{family}::{prefix}_{mpn}");
        let part = &c.world.parts[&symbol];
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert!(
            part.alts.is_empty(),
            "startup/interface variants cannot be implicit substitutes"
        );
        assert!(c.world.intents[&symbol].contains(expected_intent));
        assert_eq!(
            c.world.docs[&symbol][0],
            format!("docs/{family}-application.md")
        );
        let pins = c.world.devices[&format!("usb::pd::{family}::{device}")].pins_for(None);
        let mut design = format!("design Smoke {{ inst u: {symbol}\n");
        for pin in pins {
            if pin.name.name == "NC" {
                design.push_str("nc: u.NC\n");
            } else {
                design.push_str(&format!("net N_{}: u.{}\n", pin.name.name, pin.name.name));
            }
        }
        design.push_str("}\n");
        p.files.push(("src/smoke.cohdl".into(), design));
        let mut c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
        let out = build_artifacts(&mut c, &cohdl::lock::LockState::default());
        assert!(!c.diags.has_errors(), "{mpn}: {}", c.diags.render(&c.sm));
        let out = out.unwrap();
        assert!(out.bom.contains(mpn));
        assert!(out.netlist.contains(&format!("(pin \"{ep}\")")));
        if mpn == "STUSB4500QTR" {
            assert!(
                !out.netlist.contains("(pin \"3\")"),
                "NC must not enter any net"
            );
        }
    }
}

#[test]
fn land_oracles_include_non_square_nxp_ep_and_all_top_view_pad_positions() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    for (name, coordinates) in [
        ("ptn5110::HX2QFN16_SOT1883_1", "-1.35,-0.8;-1.35,-0.4;-1.35,0;-1.35,0.4;-1.35,0.8;-0.4,1.35;0,1.35;0.4,1.35;1.35,0.8;1.35,0.4;1.35,0;1.35,-0.4;1.35,-0.8;0.4,-1.35;0,-1.35;-0.4,-1.35;0,0"),
        ("st_qfn24::ST_QFN24_EP_4X4", "-1.9,-1.25;-1.9,-0.75;-1.9,-0.25;-1.9,0.25;-1.9,0.75;-1.9,1.25;-1.25,1.9;-0.75,1.9;-0.25,1.9;0.25,1.9;0.75,1.9;1.25,1.9;1.9,1.25;1.9,0.75;1.9,0.25;1.9,-0.25;1.9,-0.75;1.9,-1.25;1.25,-1.9;0.75,-1.9;0.25,-1.9;-0.25,-1.9;-0.75,-1.9;-1.25,-1.9;0,0"),
    ] {
        let fp = &c.world.footprints[&format!("usb::pd::{name}")];
        let expected: Vec<_> = coordinates.split(';').collect();
        assert_eq!(fp.pads.len(), expected.len());
        for (i, (pad, xy)) in fp.pads.iter().zip(expected).enumerate() {
            assert_eq!(pad.number.text, (i+1).to_string());
            assert_eq!(format!("{},{}",pad.x.text.trim_end_matches("mm"),pad.y.text.trim_end_matches("mm")), xy, "{name} pad {}",i+1);
        }
    }
    for (pad, size) in [
        ("ptn5110::P_SOT1883_H", ["0.9mm", "0.24mm"]),
        ("ptn5110::P_SOT1883_V", ["0.24mm", "0.9mm"]),
        ("ptn5110::P_SOT1883_EP", ["1.1mm", "1mm"]),
        ("st_qfn24::P_ST_QFN24_H", ["0.6mm", "0.25mm"]),
        ("st_qfn24::P_ST_QFN24_V", ["0.25mm", "0.6mm"]),
        ("st_qfn24::P_ST_QFN24_EP", ["2.7mm", "2.7mm"]),
    ] {
        assert_eq!(
            c.world.pads[&format!("usb::pd::{pad}")]
                .size
                .iter()
                .map(|x| x.text.as_str())
                .collect::<Vec<_>>(),
            size
        );
    }
    let mut diags = cohdl::diag::Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&c.world, &mut diags);
    assert!(!diags.has_errors(), "{}", diags.render(&c.sm));
}

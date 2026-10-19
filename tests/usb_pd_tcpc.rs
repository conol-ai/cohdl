//! Pin and land oracles: TI SLLSEW6C pp3,68-69; FUSB307B/D Rev7 pp1,4-5,57.
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
fn exact_pin_tables_preserve_swappable_i2c_and_ground_ep() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    for (device, expected) in [
        ("tusb422::TUSB422_YFP9", "CC2=A1;VBUSIN=A2;VDD=A3;VCONN=B1;INT_N=B2;SCL=B3;CC1=C1;GND=C2;SDA=C3"),
        ("fusb307b::FUSB307B_QFN16", "CC1=1;VCONN=2;CC2=3;ORIENT_I2C_ADDR=4;LDO=5;VBUS=6;VDD=7;DBG_N=8;GPIO2=9;INT_N=10;SCL1_SDA2=11;SDA1_SCL2=12;SRC=13;SNK=14;GND=15,17;GPIO1=16"),
    ] {
        let pins = c.world.devices[&format!("usb::pd::{device}")].pins_for(None);
        let actual = pins.iter().map(|p| format!("{}={}", p.name.name, p.numbers.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(";");
        assert_eq!(actual, expected, "{device}");
        for pin in pins {
            let expected_role = match pin.name.name.as_str() {
                "VBUS" | "VBUSIN" | "VDD" | "VCONN" | "GND" => PinRole::PowerIn,
                "LDO" => PinRole::PowerOut,
                "INT_N" | "SRC" | "SNK" => PinRole::Output,
                _ => PinRole::Bidirectional,
            };
            assert_eq!(pin.role_or_default(), expected_role, "{device}.{}", pin.name.name);
        }
    }
}

#[test]
fn manufacturer_lands_use_top_view_copper_dimensions() {
    let p = project();
    let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
    for (name, expected) in [
        ("tusb422::DSBGA9_YFP0009_C01", "A1=-0.4,-0.4;A2=0,-0.4;A3=0.4,-0.4;B1=-0.4,0;B2=0,0;B3=0.4,0;C1=-0.4,0.4;C2=0,0.4;C3=0.4,0.4"),
        ("fusb307b::WQFN16_510BS", "1=-1.365,-0.75;2=-1.365,-0.25;3=-1.365,0.25;4=-1.365,0.75;5=-0.75,1.365;6=-0.25,1.365;7=0.25,1.365;8=0.75,1.365;9=1.365,0.75;10=1.365,0.25;11=1.365,-0.25;12=1.365,-0.75;13=0.75,-1.365;14=0.25,-1.365;15=-0.25,-1.365;16=-0.75,-1.365;17=0,0"),
    ] {
        let fp = &c.world.footprints[&format!("usb::pd::{name}")];
        let actual = fp.pads.iter().map(|p| format!("{}={},{}",p.number.text,p.x.text.trim_end_matches("mm"),p.y.text.trim_end_matches("mm"))).collect::<Vec<_>>().join(";");
        assert_eq!(actual, expected, "{name}");
    }
    for (name, expected) in [
        ("tusb422::P_YFP0009_LAND", "0.23mm"),
        ("fusb307b::P_510BS_H", "0.57mm,0.3mm"),
        ("fusb307b::P_510BS_V", "0.3mm,0.57mm"),
        ("fusb307b::P_510BS_EP", "1.75mm,1.75mm"),
    ] {
        let pad = &c.world.pads[&format!("usb::pd::{name}")];
        assert_eq!(
            pad.size
                .iter()
                .map(|x| x.text.as_str())
                .collect::<Vec<_>>()
                .join(","),
            expected
        );
    }
    let mut diags = cohdl::diag::Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&c.world, &mut diags);
    assert!(!diags.has_errors(), "{}", diags.render(&c.sm));
}

#[test]
fn every_order_code_builds_with_exact_identity_and_its_documentation() {
    for (family, device, mpn, expected_warning, expected_nodes) in [
        (
            "tusb422",
            "TUSB422_YFP9",
            "TUSB422IYFPR",
            "Extended unsupported",
            9,
        ),
        (
            "tusb422",
            "TUSB422_YFP9",
            "TUSB422IYFPT",
            "Extended unsupported",
            9,
        ),
        (
            "fusb307b",
            "FUSB307B_QFN16",
            "FUSB307BMPX",
            "discontinued",
            17,
        ),
        (
            "fusb307b",
            "FUSB307B_QFN16",
            "FUSB307BVMPX",
            "discontinued",
            17,
        ),
    ] {
        let mut p = project();
        let c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
        let symbol = format!("usb::pd::{family}::TCPC_{mpn}");
        let part = &c.world.parts[&symbol];
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert!(part.alts.is_empty());
        assert!(c.world.intents[&symbol].contains(expected_warning));
        assert!(c.world.intents[&symbol].contains("unpowered Rd"));
        assert_eq!(
            c.world.docs[&symbol][0],
            format!("docs/{family}-application.md")
        );
        let pins = c.world.devices[&format!("usb::pd::{family}::{device}")].pins_for(None);
        let mut design = format!("design Smoke {{ inst u: {symbol}\n");
        for pin in pins {
            design.push_str(&format!("net N_{}: u.{}\n", pin.name.name, pin.name.name));
        }
        design.push_str("}\n");
        p.files.push(("src/smoke.cohdl".into(), design));
        let mut c = check_files_in_with_deps("usb", &["std".into()], &p.files, None).unwrap();
        let out = build_artifacts(&mut c, &cohdl::lock::LockState::default());
        assert!(!c.diags.has_errors(), "{mpn}: {}", c.diags.render(&c.sm));
        let out = out.unwrap();
        assert!(out.bom.contains(mpn));
        assert_eq!(
            out.netlist.matches("(node ").count(),
            expected_nodes,
            "{mpn}"
        );
        if family == "fusb307b" {
            assert!(out.netlist.contains("(pin \"17\")"));
        }
    }
}

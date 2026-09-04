//! RFC-017 library-registry conformance: `#[doc(...)]` reference documents
//! and `footprint` as a resolvable declaration kind (symbol-resolution-
//! complete, format-empty — the body arrives with RFC-018).

use cohdl::check::check_declarations_in;
use cohdl::diag::Diagnostics;
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files, check_files_in};
use cohdl::resolve::ModuleInfo;
use cohdl::span::SourceMap;

fn check(pkg: &str, files: &[(&str, &str)]) -> (cohdl::pipeline::Checked, String) {
    let files: Vec<(String, String)> = files
        .iter()
        .map(|(n, c)| (n.to_string(), c.to_string()))
        .collect();
    let mut checked = check_files_in(pkg, &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn world_of(files: &[(&str, &str, &str, &str)]) -> (cohdl::resolve::World, String) {
    let mut sm = SourceMap::new();
    let mut diags = Diagnostics::new();
    let mut parsed = Vec::new();
    let mut modules = Vec::new();
    for (name, content, package, module) in files {
        let fid = sm.add_file(name.to_string(), content.to_string());
        let tokens = cohdl::lex::lex(fid, sm.text(fid), &mut diags);
        parsed.push(cohdl::parse::parse(tokens, &mut diags));
        modules.push(ModuleInfo {
            package: package.to_string(),
            module: module.to_string(),
        });
    }
    let world = check_declarations_in(parsed, &modules, &mut diags);
    diags.sort(&sm);
    (world, diags.render(&sm))
}

const BOARD: &str = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint FP_0402 {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: FP_0402 } }
design B {
    inst r1: R1
    inst r2: R1
    net N: r1.A, r2.A
    net M: r1.B, r2.B
}
"#;

const ZERO_OHM_ORACLE: [(&str, &str, &str); 8] = [
    ("0201", "R_0R_J_0201", "RC0201JR-070RL"),
    ("0402", "R_0R_J_0402", "RC0402JR-070RL"),
    ("0603", "R_0R_J_0603", "RC0603JR-070RL"),
    ("0805", "R_0R_J_0805", "RC0805JR-070RL"),
    ("1206", "R_0R_J_1206", "RC1206JR-070RL"),
    ("1210", "R_0R_J_1210", "RC1210JR-070RL"),
    ("2010", "R_0R_J_2010", "RC2010JK-070RL"),
    ("2512", "R_0R_J_2512", "RC2512JK-070RL"),
];

struct GeneratedPassives {
    root: std::path::PathBuf,
}

impl Drop for GeneratedPassives {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn regenerate_passives(root: &std::path::Path) -> GeneratedPassives {
    let fixture = GeneratedPassives {
        root: std::env::temp_dir().join(format!("cohdl-zero-ohm-generator-{}", std::process::id())),
    };
    let _ = std::fs::remove_dir_all(&fixture.root);
    std::fs::create_dir_all(fixture.root.join("tools/passive_data")).unwrap();
    std::fs::create_dir_all(fixture.root.join("lib/passive/src")).unwrap();
    std::fs::copy(
        root.join("tools/gen_passive.py"),
        fixture.root.join("tools/gen_passive.py"),
    )
    .unwrap();
    for entry in std::fs::read_dir(root.join("tools/passive_data")).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            std::fs::copy(
                entry.path(),
                fixture
                    .root
                    .join("tools/passive_data")
                    .join(entry.file_name()),
            )
            .unwrap();
        }
    }
    let output = std::process::Command::new("python3")
        .arg("tools/gen_passive.py")
        .current_dir(&fixture.root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "passive generator failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture
}

fn resistor_sizes(dir: &std::path::Path) -> Vec<String> {
    let mut sizes = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_prefix("resistors_")
                .and_then(|name| name.strip_suffix(".cohdl"))
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    sizes.sort();
    sizes
}

fn zero_ohm_identity(source: &str, size: &str) -> (String, String, String) {
    let parts = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("pub part ") && line.contains("ChipResistor<0ohm,"))
        .collect::<Vec<_>>();
    assert_eq!(
        parts.len(),
        1,
        "{size} must emit exactly one zero-ohm resistor"
    );
    let (line_index, declaration) = parts[0];
    let name = declaration
        .strip_prefix("pub part ")
        .and_then(|line| line.split_once(':'))
        .map(|(name, _)| name)
        .unwrap();
    let primary = source.lines().nth(line_index + 1).unwrap();
    let mpn = primary
        .split_once("mpn: \"")
        .and_then(|(_, suffix)| suffix.split_once('"'))
        .map(|(mpn, _)| mpn)
        .unwrap();
    (declaration.to_string(), name.to_string(), mpn.to_string())
}

fn assert_zero_ohm_part(
    root: &std::path::Path,
    generated: &GeneratedPassives,
    size: &str,
    expected_name: &str,
    expected_mpn: &str,
) -> (String, String) {
    let file_name = format!("resistors_{size}.cohdl");
    let committed = std::fs::read_to_string(root.join(&file_name)).unwrap();
    let regenerated =
        std::fs::read_to_string(generated.root.join("lib/passive/src").join(&file_name)).unwrap();
    assert_eq!(
        committed, regenerated,
        "{file_name} must be owned entirely by tools/gen_passive.py"
    );
    let (declaration, actual_name, actual_mpn) = zero_ohm_identity(&committed, size);
    assert_eq!(
        declaration,
        format!("pub part {expected_name}: ChipResistor<0ohm, 5%>[R{size}] {{"),
        "zero ohms must exist only in the J/5% family"
    );
    assert_eq!(actual_mpn, expected_mpn);
    (actual_name, actual_mpn)
}

#[test]
fn t3902_public_part_matches_manufacturer_geometry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mic = std::fs::read_to_string(root.join("lib/@contrib/mic/src/t3902.cohdl")).unwrap();
    let consumer = r#"
pub trait IC {}
design T3902Consumer {
    inst mic: MMICT3902_00_012
    nc: mic.DATA, mic.SELECT, mic.GND, mic.CLK, mic.VDD
}
"#;
    let (checked, rendered) = check(
        "contrib_mic",
        &[("src/ic.cohdl", consumer), ("src/t3902.cohdl", &mic)],
    );
    assert!(!checked.diags.has_errors(), "{rendered}");

    let part = &checked.world.parts["contrib_mic::MMICT3902_00_012"];
    assert_eq!(
        part.primary.field("mfr").map(|field| field.value.as_str()),
        Some("TDK/InvenSense")
    );
    assert_eq!(
        part.primary.field("mpn").map(|field| field.value.as_str()),
        Some("MMICT3902-00-012")
    );
    let fp = &checked.world.footprints["contrib_mic::FP_TDK_T3902"];
    assert_eq!(fp.pads.len(), 5);
    assert_eq!(fp.mount_holes.len(), 1);
    let expected = [
        ("1", 837_500_000_000_000, 1_364_000_000_000_000),
        ("2", 837_500_000_000_000, 542_000_000_000_000),
        ("3", 0, -710_000_000_000_000),
        ("4", -837_500_000_000_000, 542_000_000_000_000),
        ("5", -837_500_000_000_000, 1_364_000_000_000_000),
    ];
    for (place, (number, x, y)) in fp.pads.iter().zip(expected) {
        assert_eq!(place.number.text, number);
        assert_eq!((place.x.femto, place.y.femto), (x, y));
    }
    let signal = &checked.world.pads["contrib_mic::P_T3902_SIGNAL"];
    assert_eq!(
        signal
            .size
            .iter()
            .map(|v| v.text.as_str())
            .collect::<Vec<_>>(),
        ["0.725mm", "0.522mm"]
    );
    let cohdl::ast::PadPaste::Rect(paste_w, paste_h) = &signal.paste.as_ref().unwrap().0 else {
        panic!("T3902 signal paste is not rectangular")
    };
    assert_eq!(
        (paste_w.text.as_str(), paste_h.text.as_str()),
        ("0.625mm", "0.422mm")
    );
    let ground = &checked.world.pads["contrib_mic::P_T3902_GND"];
    assert_eq!(ground.shape.unwrap().0, cohdl::ast::PadShape::Annulus);
    assert_eq!(
        ground
            .size
            .iter()
            .map(|v| v.text.as_str())
            .collect::<Vec<_>>(),
        ["1.625mm", "1.025mm"]
    );

    let mut checked = checked;
    let artifacts =
        build_artifacts(&mut checked, &LockState::default()).expect("real T3902 consumer builds");
    assert!(artifacts.netlist.contains("MMICT3902-00-012"));
    assert!(artifacts.bom.contains("MMICT3902-00-012"));
    let ir = checked.ir.as_ref().unwrap();
    assert_eq!(
        cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir).len(),
        1
    );
    assert!(cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "t3902").contains("<Cutout>"));
}

#[test]
fn std_exports_only_the_core_trait_allowlist() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std/src");
    let mut declarations = Vec::new();
    for entry in std::fs::read_dir(src_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "cohdl") {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        for line in text.lines().map(str::trim) {
            let Some(rest) = line.strip_prefix("pub ") else {
                continue;
            };
            let mut words = rest.split_whitespace();
            let kind = words.next().unwrap_or_default();
            let name = words
                .next()
                .unwrap_or_default()
                .trim_end_matches(':')
                .to_string();
            declarations.push(format!("{kind} {name}"));
        }
    }
    declarations.sort();
    assert_eq!(
        declarations,
        [
            "trait Capacitor",
            "trait Connector",
            "trait Diode",
            "trait IC",
            "trait Polarized",
            "trait Resistor",
            "trait TwoTerminal",
        ]
    );
}

#[test]
fn generated_zero_ohm_resistors_match_the_locked_oracle() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let generated = regenerate_passives(root);
    let mut actual_names = Vec::new();
    let mut actual_mpns = Vec::new();
    let resistor_dir = root.join("lib/passive/src");
    assert_eq!(
        resistor_sizes(&resistor_dir),
        ZERO_OHM_ORACLE
            .iter()
            .map(|(size, _, _)| size.to_string())
            .collect::<Vec<_>>()
    );

    for (size, expected_name, expected_mpn) in ZERO_OHM_ORACLE {
        let (actual_name, actual_mpn) =
            assert_zero_ohm_part(&resistor_dir, &generated, size, expected_name, expected_mpn);
        actual_names.push(actual_name);
        actual_mpns.push(actual_mpn);
    }

    assert_eq!(
        actual_names,
        ZERO_OHM_ORACLE
            .iter()
            .map(|(_, name, _)| name.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual_mpns,
        ZERO_OHM_ORACLE
            .iter()
            .map(|(_, _, mpn)| mpn.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn generated_stm32_catalog_matches_the_pinned_st_snapshot() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("python3")
        .args(["tools/gen_stm32.py", "--check"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "STM32 generator check failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("26 files, 2284 devices, 199920 logical pins, 2389 exact parts"),
        "unexpected STM32 coverage:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let coverage =
        std::fs::read_to_string(root.join("lib/@st/stm32/src/catalog_coverage.cohdl")).unwrap();
    for expected in [
        "Exact ST portfolio order-code rows: 4930",
        "Identities matched exactly once to a pinned pinout: 3398",
        "Matched identities represented by emitted devices: 2731",
        "Exact order-code rows represented by emitted devices: 3708",
        "Source-backed pub parts with concrete footprints: 2389",
        "Exact order-code rows covered by exact parts: 3303",
        "Represented exact rows awaiting fabrication audit: 405",
        "All portfolio rows not emitted as exact parts: 1627",
        "STM32F072C(8-B)Ux.xml is incomplete: UFQFPN48",
        "STM32H553VGZx.xml is incomplete: LQFP100-EP",
    ] {
        assert!(
            coverage.contains(expected),
            "missing coverage fact `{expected}`"
        );
    }
    let catalog =
        std::fs::read_to_string(root.join("lib/@st/stm32/docs/stm32-part-catalog.md")).unwrap();
    assert!(catalog.contains("Electrical identities: 2389"));
    assert!(catalog.contains("Exact order-code rows (including terminal `TR` packaging): 3303"));
}

#[test]
fn generated_stm32_footprints_match_the_pinned_kicad_snapshot() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("python3")
        .args(["tools/gen_stm32_footprints.py", "--check"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "STM32 footprint generator check failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("verified 103 footprints and 9147 exact electrical pad numbers"),
        "unexpected STM32 footprint coverage:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn generated_esp32_footprints_match_the_pinned_manufacturer_snapshot() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("python3")
        .args(["tools/gen_esp32_footprints.py", "--check"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ESP32 footprint generator check failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_esp32_catalog_matches_the_pinned_official_sources() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("python3")
        .args(["tools/gen_esp32.py", "--check"])
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ESP32 catalog generator check failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "318 selector rows; 140 admitted (4 existing), 178 omitted; 34 source symbols"
        ),
        "unexpected ESP32 catalog coverage:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let catalog =
        std::fs::read_to_string(root.join("lib/@espressif/esp32/docs/esp32-part-catalog.md"))
            .unwrap();
    assert!(catalog.contains("**140 admitted**"));
    assert!(catalog.contains("**178 omitted**"));
    assert!(catalog.contains("`140 + 178 = 318`"));
    assert!(catalog.contains("memory variant changes pin availability or I/O voltage"));
}

#[test]
fn every_shipped_component_library_has_consistent_part_footprints() {
    fn assert_pin(
        world: &cohdl::resolve::World,
        device_name: &str,
        variant: Option<&str>,
        pin_name: &str,
        number: &str,
        role: cohdl::ast::PinRole,
    ) {
        let device = &world.devices[device_name];
        let pin = device
            .pins_for(variant)
            .iter()
            .find(|pin| pin.name.name == pin_name)
            .unwrap_or_else(|| panic!("missing source-locked pin `{pin_name}` on `{device_name}`"));
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            [number],
            "wrong physical assignment for `{device_name}.{pin_name}`"
        );
        assert_eq!(
            pin.role_or_default(),
            role,
            "wrong electrical role for `{device_name}.{pin_name}`"
        );
    }

    fn packages_under(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if dir.join("cohdl.toml").is_file() {
            out.push(dir.to_path_buf());
            return;
        }
        let mut children: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .collect();
        children.sort();
        for child in children {
            packages_under(&child, out);
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let std_dir = root.join("lib/std");
    let mut packages = Vec::new();
    let mut qualified_contrib_parts = std::collections::BTreeSet::new();
    packages_under(&root.join("lib"), &mut packages);
    for package in packages {
        if package == std_dir {
            continue;
        }
        let (_, manifest) = cohdl::project::peek_manifest(&package).unwrap();
        let declared = manifest
            .deps_raw
            .expect("every shipped component library must declare dependencies");
        let mut deps = Vec::new();
        for (name, raw_version, _) in declared {
            let wanted = cohdl::deps::parse_exact_version(&raw_version).unwrap();
            let family = root.join("lib").join(&name);
            let (_, dir) = cohdl::deps::available_versions(&family, &name)
                .unwrap()
                .into_iter()
                .find(|(version, _)| *version == wanted)
                .unwrap_or_else(|| {
                    panic!(
                        "`{}` declares unavailable shipped dependency `{} = \"{}\"`",
                        package.display(),
                        name,
                        raw_version
                    )
                });
            deps.push((name, dir));
        }
        deps.sort_by_key(|(name, _)| if name == "std" { 0 } else { 1 });
        let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
        let project = cohdl::project::load_project_with_deps(&package, &deps).unwrap();
        let mut checked = cohdl::pipeline::check_files_in_with_deps(
            &project.name,
            &dep_names,
            &project.files,
            None,
        )
        .unwrap();
        checked.diags.sort(&checked.sm);
        assert!(
            !checked.diags.has_errors(),
            "`{}` declaration check failed:\n{}",
            package.display(),
            checked.diags.render(&checked.sm)
        );

        match project.name.as_str() {
            "@contrib/usb-uart" => {
                for (name, number, role) in [
                    ("VIO", "1", cohdl::ast::PinRole::PowerIn),
                    ("GND", "2", cohdl::ast::PinRole::PowerIn),
                    ("VDD5", "3", cohdl::ast::PinRole::PowerIn),
                    ("TXD", "4", cohdl::ast::PinRole::Output),
                    ("RXD", "5", cohdl::ast::PinRole::Input),
                    ("V3", "6", cohdl::ast::PinRole::PowerOut),
                    ("UDP", "7", cohdl::ast::PinRole::Bidirectional),
                    ("UDN", "8", cohdl::ast::PinRole::Bidirectional),
                    ("VBUS", "9", cohdl::ast::PinRole::Input),
                    ("ACTN", "10", cohdl::ast::PinRole::Output),
                    ("DCD", "11", cohdl::ast::PinRole::Input),
                    ("DTR_TNOW", "12", cohdl::ast::PinRole::Output),
                    ("RTS", "13", cohdl::ast::PinRole::Output),
                    ("DSR", "14", cohdl::ast::PinRole::Input),
                    ("CTS", "15", cohdl::ast::PinRole::Input),
                    ("RI", "16", cohdl::ast::PinRole::Input),
                    ("EPAD", "0", cohdl::ast::PinRole::PowerIn),
                ] {
                    assert_pin(
                        &checked.world,
                        "contrib_usb_uart::CH343P",
                        None,
                        name,
                        number,
                        role,
                    );
                }
            }
            "@contrib/lora" => {
                for (name, number, role) in [
                    ("VDD_IN", "1", cohdl::ast::PinRole::PowerIn),
                    ("GND2", "2", cohdl::ast::PinRole::PowerIn),
                    ("XTA", "3", cohdl::ast::PinRole::Passive),
                    ("XTB", "4", cohdl::ast::PinRole::Passive),
                    ("GND5", "5", cohdl::ast::PinRole::PowerIn),
                    ("DIO3", "6", cohdl::ast::PinRole::Bidirectional),
                    ("VREG", "7", cohdl::ast::PinRole::PowerOut),
                    ("GND8", "8", cohdl::ast::PinRole::PowerIn),
                    ("DCC_SW", "9", cohdl::ast::PinRole::Output),
                    ("VBAT", "10", cohdl::ast::PinRole::PowerIn),
                    ("VBAT_IO", "11", cohdl::ast::PinRole::PowerIn),
                    ("DIO2", "12", cohdl::ast::PinRole::Bidirectional),
                    ("DIO1", "13", cohdl::ast::PinRole::Bidirectional),
                    ("BUSY", "14", cohdl::ast::PinRole::Output),
                    ("NRESET", "15", cohdl::ast::PinRole::Input),
                    ("MISO", "16", cohdl::ast::PinRole::Output),
                    ("MOSI", "17", cohdl::ast::PinRole::Input),
                    ("SCK", "18", cohdl::ast::PinRole::Input),
                    ("NSS", "19", cohdl::ast::PinRole::Input),
                    ("GND20", "20", cohdl::ast::PinRole::PowerIn),
                    ("RFI_P", "21", cohdl::ast::PinRole::Input),
                    ("RFI_N", "22", cohdl::ast::PinRole::Input),
                    ("RFO", "23", cohdl::ast::PinRole::Output),
                    ("VR_PA", "24", cohdl::ast::PinRole::PowerOut),
                    ("EP", "0", cohdl::ast::PinRole::PowerIn),
                ] {
                    assert_pin(
                        &checked.world,
                        "contrib_lora::SX1262",
                        None,
                        name,
                        number,
                        role,
                    );
                }
                for (name, number, role) in [
                    ("VR_PA", "1", cohdl::ast::PinRole::PowerOut),
                    ("VDD_IN", "2", cohdl::ast::PinRole::PowerIn),
                    ("NRESET", "3", cohdl::ast::PinRole::Input),
                    ("XTA", "4", cohdl::ast::PinRole::Passive),
                    ("GND5", "5", cohdl::ast::PinRole::PowerIn),
                    ("XTB", "6", cohdl::ast::PinRole::Passive),
                    ("BUSY", "7", cohdl::ast::PinRole::Output),
                    ("DIO1", "8", cohdl::ast::PinRole::Bidirectional),
                    ("DIO2", "9", cohdl::ast::PinRole::Bidirectional),
                    ("DIO3", "10", cohdl::ast::PinRole::Bidirectional),
                    ("VBAT_IO", "11", cohdl::ast::PinRole::PowerIn),
                    ("DCC_FB", "12", cohdl::ast::PinRole::PowerOut),
                    ("GND13", "13", cohdl::ast::PinRole::PowerIn),
                    ("DCC_SW", "14", cohdl::ast::PinRole::Output),
                    ("VBAT", "15", cohdl::ast::PinRole::PowerIn),
                    ("MISO_TX", "16", cohdl::ast::PinRole::Output),
                    ("MOSI_RX", "17", cohdl::ast::PinRole::Input),
                    ("SCK_RTSN", "18", cohdl::ast::PinRole::Bidirectional),
                    ("NSS_CTSN", "19", cohdl::ast::PinRole::Input),
                    ("GND20", "20", cohdl::ast::PinRole::PowerIn),
                    ("GND21", "21", cohdl::ast::PinRole::PowerIn),
                    ("RFIO", "22", cohdl::ast::PinRole::Bidirectional),
                    ("GND23", "23", cohdl::ast::PinRole::PowerIn),
                    ("GND24", "24", cohdl::ast::PinRole::PowerIn),
                    ("EP", "0", cohdl::ast::PinRole::PowerIn),
                ] {
                    assert_pin(
                        &checked.world,
                        "contrib_lora::SX1280",
                        None,
                        name,
                        number,
                        role,
                    );
                }
            }
            "@contrib/display" => {
                assert_pin(
                    &checked.world,
                    "contrib_display::H0216F002AM",
                    None,
                    "TP_VCC",
                    "6",
                    cohdl::ast::PinRole::PowerOut,
                );
            }
            "@contrib/ldo" => {
                assert_pin(
                    &checked.world,
                    "contrib_ldo::RT9080",
                    Some("TSOT235"),
                    "NC4",
                    "4",
                    cohdl::ast::PinRole::Passive,
                );
            }
            "@contrib/ir-emitter" => {
                assert_pin(
                    &checked.world,
                    "contrib_ir_emitter::VSMY14940",
                    None,
                    "Cathode",
                    "1",
                    cohdl::ast::PinRole::Passive,
                );
                assert_pin(
                    &checked.world,
                    "contrib_ir_emitter::VSMY14940",
                    None,
                    "Anode",
                    "2",
                    cohdl::ast::PinRole::Passive,
                );
            }
            "@contrib/io-expander" => {
                assert_pin(
                    &checked.world,
                    "contrib_io_expander::XL9555",
                    Some("QFN24"),
                    "EPAD",
                    "Y",
                    cohdl::ast::PinRole::PowerIn,
                );
            }
            "@contrib/audio-amp" => {
                assert_pin(
                    &checked.world,
                    "contrib_audio_amp::MAX98357A",
                    None,
                    "EP",
                    "EP",
                    cohdl::ast::PinRole::Passive,
                );
            }
            "@contrib/pmu" => {
                assert_pin(
                    &checked.world,
                    "contrib_pmu::AXP2101",
                    None,
                    "EPAD",
                    "EP",
                    cohdl::ast::PinRole::PowerIn,
                );
            }
            _ => {}
        }

        let own_prefix = format!("{}::", cohdl::pipeline::package_root(&project.name));
        let own_components: Vec<&String> = checked
            .world
            .devices
            .keys()
            .chain(checked.world.parts.keys())
            .filter(|name| {
                name.starts_with(&own_prefix)
                    && checked
                        .world
                        .symbols
                        .get(*name)
                        .is_some_and(|symbol| symbol.is_pub)
            })
            .collect();
        if !own_components.is_empty() {
            assert!(
                package.join("docs/README.md").is_file(),
                "`{}` ships public devices or parts without docs/README.md",
                package.display()
            );
        }
        for component in own_components {
            let docs = checked.world.docs.get(component).unwrap_or_else(|| {
                panic!(
                    "`{}` public component `{component}` has no #[doc(...)] reference",
                    package.display()
                )
            });
            assert!(
                !docs.is_empty(),
                "`{}` public component `{component}` has an empty document list",
                package.display()
            );
            for relative in docs {
                assert!(
                    std::path::Path::new(relative).starts_with("docs"),
                    "`{}` component `{component}` references `{relative}` outside docs/",
                    package.display()
                );
                assert!(
                    package.join(relative).is_file(),
                    "`{}` component `{component}` references missing `{relative}`",
                    package.display()
                );
            }
        }

        let mut footprint_diags = Diagnostics::new();
        cohdl::check::footprints::check_pad_consistency(&checked.world, &mut footprint_diags);
        footprint_diags.sort(&checked.sm);
        assert!(
            !footprint_diags.has_errors(),
            "`{}` has a part/footprint mismatch:\n{}",
            package.display(),
            footprint_diags.render(&checked.sm)
        );

        // Contrib parts are fabrication-facing data, so internal pin/pad-set
        // equality is not enough: their emitted copper must fit inside the
        // authored courtyard and distinct electrical pads must not overlap.
        // Keep this source-independent backstop beside the declaration-wide
        // E807 check so every newly exported contrib part is covered.
        // @sifli/ graduated out of @contrib and stays under the same audit;
        // the generated catalogs (@st/stm32, @espressif/esp32) carry their
        // own pinned-source tests instead.
        if project.name.starts_with("@contrib/") || project.name.starts_with("@sifli/") {
            const QUARANTINED_SHARED_QFN_LANDS: [&str; 8] = [
                "qfn::QFN68N35P700X700_1EP549X549",
                "qfn::QFN24N50P400X400_1EP27X27",
                "qfn::QFN20N40P300X300_1EP170X170",
                "qfn::QFN24N50P400X400_1EP245X245",
                "qfn::QFN16N50P300X300_1EP170X170",
                "qfn::QFN40N40P600X600_1EP44X44",
                "qfn::QFN32N50P500X500_1EP34X34",
                "qfn::QFN14N50P350X350_1EP20X20",
            ];
            for (part_name, part) in checked.world.parts.iter().filter(|(name, _)| {
                name.starts_with(&own_prefix)
                    && checked
                        .world
                        .symbols
                        .get(*name)
                        .is_some_and(|symbol| symbol.is_pub)
            }) {
                let device_name = &part.device.name.name;
                let variant = part
                    .device
                    .variant
                    .as_ref()
                    .map(|variant| variant.name.as_str());
                let mut pin_rows = checked.world.devices[device_name]
                    .pins_for(variant)
                    .iter()
                    .map(|pin| {
                        let mut numbers = pin
                            .numbers
                            .iter()
                            .map(|number| number.text.as_str())
                            .collect::<Vec<_>>();
                        numbers.sort_unstable();
                        format!(
                            "{}={}:{}:{}",
                            pin.name.name,
                            numbers.join(","),
                            pin.role_or_default().name(),
                            pin.obligation.keyword()
                        )
                    })
                    .collect::<Vec<_>>();
                pin_rows.sort_unstable();
                let pin_hash = cohdl::hash::sha256_hex(pin_rows.join("|").as_bytes());
                let device_binding = format!("{}[{}]", device_name, variant.unwrap_or("-"));
                for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
                    let fp_ref = entry
                        .footprint
                        .as_ref()
                        .or(part.primary.footprint.as_ref())
                        .unwrap_or_else(|| {
                            panic!("public contrib part `{part_name}` has no fabrication footprint")
                        });
                    let mpn = entry
                        .field("mpn")
                        .expect("a public contrib part must have an exact MPN");
                    let mfr = entry
                        .field("mfr")
                        .filter(|field| !field.value.trim().is_empty())
                        .expect("a public contrib part must have an exact manufacturer");
                    qualified_contrib_parts.insert(format!(
                        "{}::{}|{}|{}|{}|{}|{}",
                        project.name,
                        cohdl::resolve::short(part_name),
                        mfr.value,
                        mpn.value,
                        fp_ref.name,
                        device_binding,
                        pin_hash
                    ));
                    assert!(
                        !QUARANTINED_SHARED_QFN_LANDS.contains(&fp_ref.name.as_str()),
                        "public contrib part `{part_name}` uses quarantined generic land `{}`; bind a manufacturer-qualified part-specific footprint",
                        fp_ref.name
                    );
                    let fp = &checked.world.footprints[&fp_ref.name];
                    if cohdl::check::footprints::is_placeholder(fp) {
                        panic!(
                            "public contrib part `{part_name}` uses placeholder footprint `{}`",
                            fp_ref.name
                        );
                    }
                    let courtyard = fp.courtyard.as_ref().unwrap_or_else(|| {
                        panic!(
                            "public contrib part `{part_name}` footprint `{}` has no courtyard",
                            fp_ref.name
                        )
                    });
                    assert_eq!(
                        courtyard.shape.0,
                        cohdl::ast::PadShape::Rect,
                        "public contrib part `{part_name}` footprint `{}` needs a rectangular courtyard for deterministic containment checking",
                        fp_ref.name
                    );
                    let [court_w, court_h] = courtyard.size.as_slice() else {
                        panic!(
                            "public contrib part `{part_name}` footprint `{}` has malformed courtyard dimensions",
                            fp_ref.name
                        );
                    };
                    let court_left = 2 * courtyard.at.0.femto - court_w.femto;
                    let court_right = 2 * courtyard.at.0.femto + court_w.femto;
                    let court_top = 2 * courtyard.at.1.femto - court_h.femto;
                    let court_bottom = 2 * courtyard.at.1.femto + court_h.femto;

                    let mut copper = Vec::new();
                    for place in &fp.pads {
                        let pad = &checked.world.pads[&place.pad.name];
                        let (mut width, mut height) = match pad.size.as_slice() {
                            [diameter] => (diameter.femto, diameter.femto),
                            [width, height] => (width.femto, height.femto),
                            _ => panic!("validated pad geometry has invalid arity"),
                        };
                        if matches!(place.rotate, 90 | 270) {
                            std::mem::swap(&mut width, &mut height);
                        }
                        let bounds = (
                            2 * place.x.femto - width,
                            2 * place.x.femto + width,
                            2 * place.y.femto - height,
                            2 * place.y.femto + height,
                        );
                        assert!(
                            bounds.0 >= court_left
                                && bounds.1 <= court_right
                                && bounds.2 >= court_top
                                && bounds.3 <= court_bottom,
                            "public contrib part `{part_name}` footprint `{}` pad `{}` escapes its courtyard",
                            fp_ref.name,
                            place.number.text
                        );
                        let layer = pad
                            .layer
                            .map(|(layer, _)| layer)
                            .expect("validated pad geometry has no copper layer");
                        copper.push((place.number.text.as_str(), layer, bounds));
                    }
                    for hole in &fp.mount_holes {
                        let (width, height) = match &hole.geom {
                            cohdl::ast::MountHoleGeom::Diameter(diameter) => {
                                (diameter.femto, diameter.femto)
                            }
                            cohdl::ast::MountHoleGeom::Size(size, _) => {
                                let [width, height] = size.as_slice() else {
                                    panic!("validated mount-hole geometry has invalid arity");
                                };
                                (width.femto, height.femto)
                            }
                        };
                        let bounds = (
                            2 * hole.x.femto - width,
                            2 * hole.x.femto + width,
                            2 * hole.y.femto - height,
                            2 * hole.y.femto + height,
                        );
                        assert!(
                            bounds.0 >= court_left
                                && bounds.1 <= court_right
                                && bounds.2 >= court_top
                                && bounds.3 <= court_bottom,
                            "public contrib part `{part_name}` footprint `{}` mount hole `{}` escapes its courtyard",
                            fp_ref.name,
                            hole.number.text
                        );
                    }
                    for left in 0..copper.len() {
                        for right in (left + 1)..copper.len() {
                            let (left_number, left_layer, a) = copper[left];
                            let (right_number, right_layer, b) = copper[right];
                            let shares_layer = left_layer == right_layer
                                || left_layer == cohdl::ast::PadLayer::ThroughAll
                                || right_layer == cohdl::ast::PadLayer::ThroughAll;
                            let bounding_boxes_overlap =
                                a.0 < b.1 && b.0 < a.1 && a.2 < b.3 && b.2 < a.3;
                            assert!(
                                left_number == right_number
                                    || !shares_layer
                                    || !bounding_boxes_overlap,
                                "public contrib part `{part_name}` footprint `{}` pads `{left_number}` and `{right_number}` have overlapping copper bounds on a shared layer",
                                fp_ref.name
                            );
                        }
                    }
                }
            }
        }
    }

    // This is intentionally an allowlist, not a count assertion: adding a
    // purchasable contrib part requires a reviewer to record its exact
    // manufacturer, MPN, fully-qualified footprint and a digest of its exact
    // device/variant pin names, numbers, roles, and obligations here.
    // Device-only declarations do not appear.
    let expected: std::collections::BTreeSet<String> = [
        "@contrib/analog-switch::SW_RS2257XC6|Run-IC|RS2257XC6|contrib_analog_switch::SOT6P65X210X125N|contrib_analog_switch::RS2257[-]|5a20578cdfedc3394e46c840590d006c69d6be1878605375403e9cfc2c9a46f4",
        "@contrib/analog-switch::SW_RS2257XH|Run-IC|RS2257XH|contrib_analog_switch::SOT6P95X290X160N|contrib_analog_switch::RS2257[-]|5a20578cdfedc3394e46c840590d006c69d6be1878605375403e9cfc2c9a46f4",
        "@contrib/audio-amp::AMP_MAX98357A|Analog Devices|MAX98357AETE+|contrib_audio_amp::QFN16N50P300X300_1EP123X123|contrib_audio_amp::MAX98357A[-]|a235edfeb77483f6ce49859d1508a02037c924c3165314e2168b59421b4b7aa0",
        "@contrib/audio-amp::AMP_NS4150B|Nsiway|NS4150B|contrib_audio_amp::SOP8P65X490X110N|contrib_audio_amp::NS4150B[-]|68c467ea216f68d6c7f2fe1439b14bed81dae3b5cbec8a04b5b45bafc4dc6f19",
        "@contrib/charger::CHARGER_BQ25185DLHR|Texas Instruments|BQ25185DLHR|contrib_charger::FP_TI_DLH0010A|contrib_charger::BQ25185[-]|cf11afa6d86f56bdf247261f4d68b09632916cae85d2e0c82b51e74553a1ea28",
        "@contrib/charger::CHARGER_SGM41562B|SG Micro|SGM41562BXG/TR|contrib_charger::BGA9C50P3X3_152X152X60N|contrib_charger::SGM41562B[-]|2fc6771cc0150a98f6ca02490534f5cfc8f529cef28651d676ddd39ac8e69d79",
        "@contrib/env::ENV_BME280|Bosch Sensortec|BME280|contrib_env::FP_Bosch_BME280_LGA8_2_5x2_5mm|contrib_env::BME280[-]|3383b88fba1dfc37e0ab29f9c06b5ea75ef23e02ffb0c151c33c7e0fd4a532cc",
        "@contrib/esd::ESD_GBLC05C|ProTek|GBLC05C-LF-T7|contrib_esd::FP_SOD323_2P5X1P25mm|contrib_esd::GBLC05C[-]|8728da7595e160597f639c23bd3e6d2de2099a14d567437c396215734b86cd55",
        "@contrib/esd::ESD_ULC0511C|Tergy|ULC0511C|contrib_esd::QFN2N65P60X100|contrib_esd::ULC0511C[-]|8728da7595e160597f639c23bd3e6d2de2099a14d567437c396215734b86cd55",
        "@contrib/fuel-gauge::FGAUGE_CW2015CHBD|CellWise Microelectronics|CW2015CHBD|contrib_fuel_gauge::FP_CellWise_TDFN8_2X3|contrib_fuel_gauge::CW2015[-]|f3f4c521a3e4e50df26b5141d9b02468eee841e9d0fe3e3b0631f81020b6bd82",
        "@contrib/gnss::GNSS_L76K|Quectel|L76K|contrib_gnss::FP_Quectel_L76K_18LCC_101x97mm|contrib_gnss::L76K[-]|58c60801994860f9f08f7e1cc9c21d17128f8c2f10f6c4272d8d550b21985b21",
        "@contrib/gnss::GNSS_MIA_M10Q|u-blox|MIA-M10Q-00B|contrib_gnss::FP_Ublox_MIA_M10Q_MLGA53_45x45mm|contrib_gnss::MIA_M10Q[-]|10c63eb92f8f9d8d5cb02c643e40817f4f9fcc3a59520c37e30368f3744f8ed9",
        "@contrib/haptic::HAPTIC_AW86224|Awinic|AW86224AFCR|contrib_haptic::QFN9N40P137X137|contrib_haptic::AW86224[-]|a243e2e20b41f64545620918dcbc20ab5293f487c3db74df83d7ced63e6bcbbc",
        "@contrib/haptic::HAPTIC_DRV2605|Texas Instruments|DRV2605YZFR|contrib_haptic::BGA9C50P3X3_145X145X50N|contrib_haptic::DRV2605[-]|3f27e5ab72d051a631cca7886c663c47443161dde6e7d695fa3fe279a9278c99",
        "@contrib/imu::IMU_BHI260AP|Bosch Sensortec|BHI260AP|contrib_imu::LGA44P40_360X410|contrib_imu::BHI260AP[-]|ee1867a4965abf5cebcdd6645b1730f58f27b9ac87615d687c2c5113aac55cbb",
        "@contrib/imu::IMU_LSM6DS3TR_C|STMicroelectronics|LSM6DS3TR-C|contrib_imu::LGA14P50_300X250X86N|contrib_imu::LSM6DS3TR_C[-]|2f2a0807fc29b31dde1e7b041559d4b86211eba60abf218215d40fcb7d2334d9",
        "@contrib/imu::MAG_MMC5603NJ|MEMSIC|MMC5603NJ|contrib_imu::WLP4C40P2X2_82X82X40N|contrib_imu::MMC5603NJ[-]|4cc4cc655d3788e9f93526e6db78f983d3ad69890680effef545ff115b17e2e6",
        "@contrib/io-expander::IOEXP_XL9555|Xinluda|XL9555|contrib_io_expander::SOP24P65X640X120N|contrib_io_expander::XL9555[TSSOP24]|828026a20b79c80dc8710194390ea3c1f0babb682b9105268ed7e25548088c9e",
        "@contrib/ir-emitter::IR_VSMY14940|Vishay|VSMY14940|contrib_ir_emitter::FP_VSMY14940_3P0X2P51mm|contrib_ir_emitter::VSMY14940[-]|47809dba08bd124beed8a95e1a84adc2edb8a56869dfd3541499756a90857b46",
        "@contrib/keyscan::KEYSCAN_TCA8418|Texas Instruments|TCA8418RTWR|contrib_keyscan::QFN24N50P400X400_1EP245X245|contrib_keyscan::TCA8418[-]|e6a3ac51dc59733ef5b19704192c0a1532dd283277c3729ac36c7297f3dcdcef",
        "@contrib/ldo::LDO_ETA5060V330S8F|ETA Solutions|ETA5060V330S8F|contrib_ldo::FP_ETA_SOT89_5|contrib_ldo::ETA5060[-]|00c7c7994b6db754a34ea74774b6c333ffafb7e17689b782806fb57fcffc1386",
        "@contrib/ldo::LDO_ME6211C15M5G_N|Microne|ME6211C15M5G-N|contrib_ldo::FP_Microne_ME6211_SOT23_5|contrib_ldo::ME6211C_M5G_N[-]|7f61983fc64e6376c5d9ea993d0082f9604d2798855935eda06ff745bb17c89a",
        "@contrib/ldo::LDO_ME6211C18M5G_N|Microne|ME6211C18M5G-N|contrib_ldo::FP_Microne_ME6211_SOT23_5|contrib_ldo::ME6211C_M5G_N[-]|7f61983fc64e6376c5d9ea993d0082f9604d2798855935eda06ff745bb17c89a",
        "@contrib/ldo::LDO_ME6211C28M5G_N|Microne|ME6211C28M5G-N|contrib_ldo::FP_Microne_ME6211_SOT23_5|contrib_ldo::ME6211C_M5G_N[-]|7f61983fc64e6376c5d9ea993d0082f9604d2798855935eda06ff745bb17c89a",
        "@contrib/ldo::LDO_RT9080_33|Richtek|RT9080-33GJ5|contrib_ldo::SOT5P95X290X160N|contrib_ldo::RT9080[TSOT235]|6dbd8af8e6a20e00cb2e3ffa13af396f04f9aa9c5104208ea2d9b5f9f223f543",
        "@contrib/ldo::LDO_RT9080_33_ZQFN|Richtek|RT9080-33GQZ|contrib_ldo::FP_Richtek_ZQFN4L_1X1|contrib_ldo::RT9080[ZQFN4L]|14066b95d4fc1864b8f120c80b2b0632482e810d7e21c61c97c40f36768e4565",
        "@contrib/ldo::LDO_XC6206P182MR|Torex Semiconductor|XC6206P182MR-G|contrib_ldo::SOT3P190X290X160N|contrib_ldo::XC6206[SOT23]|0e5219461d83760afc2356ad46efb87eeb2e841033237fefe2cf164633d6fd8e",
        "@contrib/led-driver::LEDDRV_AW21009|Awinic|AW21009QNR|contrib_led_driver::QFN20N40P300X300_1EP170X170|contrib_led_driver::AW21009[-]|14b2dff0bbf6db9eaa321f200c693958ddf42654d78aab634dd7deb286c7f11a",
        "@contrib/level-shifter::LS_RS0104|Run-IC|RS0104YQ|soic::SOP14P65X640X120N|contrib_level_shifter::RS0104[TSSOP14]|bfbb765be1b02fb0c0bf15d35ffada722b9082bb2558410aa599a7ad7a3d6503",
        "@contrib/level-shifter::LS_RS0104_QFN12_2X1P7|Run-IC|RS0104YUTQH12|contrib_level_shifter::QFN12N40P200X170|contrib_level_shifter::RS0104[QFN12_2X1P7]|dcfe43c301b14263277a06f5a935ccd7d8889dee8864bd231df586bb5acc9a38",
        "@contrib/level-shifter::LS_RS0104_QFN12_2X2|Run-IC|RS0104YTQE12|contrib_level_shifter::QFN12N40P200X200_1EP120X120|contrib_level_shifter::RS0104[QFN12_2X2]|e3fffe03513c199b7e90cbfecbda27af40e8ff25972c19f6aadf0134e0780ca8",
        "@contrib/level-shifter::LS_RS0104_QFN14|Run-IC|RS0104YTQF14|contrib_level_shifter::QFN14N50P350X350_1EP200X150|contrib_level_shifter::RS0104[QFN14]|3630fb2e789c29f2450aec1bd2bd6238985bd13cd5216dc4d4b0abe693f24f46",
        "@contrib/lora::LORA_SX1262|Semtech|SX1262IMLTRT|contrib_lora::QFN24N50P400X400_1EP270X270|contrib_lora::SX1262[-]|0283921c57536ef05472323278e94728e22d738f6ff31b8937a11f8df12a734c",
        "@contrib/lora::LORA_SX1280|Semtech|SX1280IMLTRT|contrib_lora::QFN24N50P400X400_1EP270X270|contrib_lora::SX1280[-]|57f667bbeac1ea0682f2b5ef0b09d53da8e0b4d9d7d644a9348b5721dc9da567",
        "@contrib/mic::MMICT3902_00_012|TDK/InvenSense|MMICT3902-00-012|contrib_mic::FP_TDK_T3902|contrib_mic::T3902[-]|1060ef92c73635da819e96b7b7262a8f818a578028575cd21022352e09ebe22d",
        "@contrib/nfc::NFC_ST25R3916|STMicroelectronics|ST25R3916-AQET|contrib_nfc::QFN32N50P500X500_1EP345X345|contrib_nfc::ST25R3916[-]|aa52ed734bb60338e6a1457e28efbfae87a0f33826c43894528c3aa4c6f3cb22",
        "@contrib/rtc::RTC_PCF85063AT|NXP|PCF85063AT/AY|contrib_rtc::SOIC8P127X600X175N|contrib_rtc::PCF85063A[SO8]|e946e95965556a8151ea8372d6a59363ea770ea1929c31bbae73e95b81fdb759",
        "@contrib/rtc::RTC_PCF85063ATL|NXP|PCF85063ATL/1,118|contrib_rtc::FP_NXP_DFN2626_10_SOT1197_1|contrib_rtc::PCF85063A[DFN10]|0adb4e66d4b7917d6471953d8775cc5a085ed22aa53452b53826f8f1ad495abc",
        "@contrib/rtc::RTC_PCF85063ATT|NXP|PCF85063ATT/AJ|contrib_rtc::SOP8P65X490X110N|contrib_rtc::PCF85063A[TSSOP8]|e946e95965556a8151ea8372d6a59363ea770ea1929c31bbae73e95b81fdb759",
        "@contrib/sd-card::CONN_MICROSD|Wurth Elektronik|693070010811|contrib_sd_card::FP_MicroSD_Wurth_693070010811|contrib_sd_card::MICROSD_SOCKET[-]|a927f96998d32e2e6ae630deb905da79cb9bdf0b7d504ee70441b294296dcec4",
        "@sifli/sf32::MCU_SF32LB52EUB6|SiFli Technologies|SF32LB52EUB6|sifli_sf32::FP_SiFli_SF32LB52X_QFN68L_7X7|sifli_sf32::SF32LB52X[-]|11de47e07e71b155bf398a53e99c7011ce044a580436db4bab4df7a8a7c69be5",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        qualified_contrib_parts, expected,
        "the public contrib part set changed; verify exact MPN, pin map, and manufacturer land before updating this allowlist"
    );
}

#[test]
fn promoted_w25q128_variants_use_the_datasheet_pinouts_and_exact_lands() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![
        ("std".to_string(), root.join("lib/std")),
        ("qfn".to_string(), root.join("lib/qfn")),
        ("soic".to_string(), root.join("lib/soic")),
    ];
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project = cohdl::project::load_project_with_deps(&root.join("lib/flash"), &deps).unwrap();
    let checked =
        cohdl::pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    let flash = checked.world.devices.get("flash::W25Q128JV").unwrap();
    for variant in ["SOIC8", "WSON8_6X5"] {
        let pins = flash.pins_for(Some(variant));
        for (name, number) in [
            ("CS", "1"),
            ("DO", "2"),
            ("WP", "3"),
            ("GND", "4"),
            ("DI", "5"),
            ("CLK", "6"),
            ("HOLD", "7"),
            ("VCC", "8"),
        ] {
            let pin = pins.iter().find(|pin| pin.name.name == name).unwrap();
            assert_eq!(
                pin.numbers
                    .iter()
                    .map(|number| number.text.as_str())
                    .collect::<Vec<_>>(),
                [number],
                "wrong W25Q128JV {variant} physical mapping for {name}"
            );
        }
    }
    let wson_pins = flash.pins_for(Some("WSON8_6X5"));
    let ep = wson_pins.iter().find(|pin| pin.name.name == "EP").unwrap();
    assert_eq!(
        ep.numbers
            .iter()
            .map(|number| number.text.as_str())
            .collect::<Vec<_>>(),
        ["9"],
        "wrong W25Q128JV WSON exposed-pad mapping"
    );
    assert!(checked
        .world
        .parts
        .contains_key("flash::FLASH_W25Q128JVSIQ"));
    assert!(checked
        .world
        .parts
        .contains_key("flash::FLASH_W25Q128JVPIQ"));
    assert!(checked
        .world
        .footprints
        .contains_key("soic::SOIC8P127X790X216N"));
    assert!(checked
        .world
        .footprints
        .contains_key("flash::FP_W25Q128JV_WSON8_6X5"));
}

#[test]
fn ti_sn74lvc8t245pwr_uses_exact_pinout_part_and_pw0024a_land() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![
        ("std".to_string(), root.join("lib/std")),
        ("soic".to_string(), root.join("lib/soic")),
    ];
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project =
        cohdl::project::load_project_with_deps(&root.join("lib/@ti/logic"), &deps).unwrap();
    let checked =
        cohdl::pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    let device = &checked.world.devices["ti_logic::SN74LVC8T245"];
    let pins = device.pins_for(None);
    assert_eq!(
        pins.len(),
        21,
        "24 package pads collapse to 21 logical pins"
    );
    for (name, number, role) in [
        ("VCCA", "1", cohdl::ast::PinRole::PowerIn),
        ("DIR", "2", cohdl::ast::PinRole::Input),
        ("A1", "3", cohdl::ast::PinRole::Bidirectional),
        ("A2", "4", cohdl::ast::PinRole::Bidirectional),
        ("A3", "5", cohdl::ast::PinRole::Bidirectional),
        ("A4", "6", cohdl::ast::PinRole::Bidirectional),
        ("A5", "7", cohdl::ast::PinRole::Bidirectional),
        ("A6", "8", cohdl::ast::PinRole::Bidirectional),
        ("A7", "9", cohdl::ast::PinRole::Bidirectional),
        ("A8", "10", cohdl::ast::PinRole::Bidirectional),
        ("B8", "14", cohdl::ast::PinRole::Bidirectional),
        ("B7", "15", cohdl::ast::PinRole::Bidirectional),
        ("B6", "16", cohdl::ast::PinRole::Bidirectional),
        ("B5", "17", cohdl::ast::PinRole::Bidirectional),
        ("B4", "18", cohdl::ast::PinRole::Bidirectional),
        ("B3", "19", cohdl::ast::PinRole::Bidirectional),
        ("B2", "20", cohdl::ast::PinRole::Bidirectional),
        ("B1", "21", cohdl::ast::PinRole::Bidirectional),
        ("OE", "22", cohdl::ast::PinRole::Input),
    ] {
        let pin = pins.iter().find(|pin| pin.name.name == name).unwrap();
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            [number],
            "wrong SN74LVC8T245 physical mapping for {name}"
        );
        assert_eq!(pin.role_or_default(), role, "wrong role for {name}");
        assert_eq!(pin.obligation, cohdl::ast::Obligation::Required);
    }
    for (name, numbers) in [
        ("GND", &["11", "12", "13"][..]),
        ("VCCB", &["23", "24"][..]),
    ] {
        let pin = pins.iter().find(|pin| pin.name.name == name).unwrap();
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            numbers,
            "wrong SN74LVC8T245 physical mapping for {name}"
        );
        assert_eq!(pin.role_or_default(), cohdl::ast::PinRole::PowerIn);
        assert_eq!(pin.obligation, cohdl::ast::Obligation::Required);
    }

    let part = &checked.world.parts["ti_logic::LS_SN74LVC8T245PWR"];
    assert_eq!(part.device.name.name, "ti_logic::SN74LVC8T245");
    assert!(part.device.variant.is_none());
    assert_eq!(
        part.primary.field("mfr").unwrap().value,
        "Texas Instruments"
    );
    assert_eq!(part.primary.field("mpn").unwrap().value, "SN74LVC8T245PWR");
    assert_eq!(
        part.primary.footprint.as_ref().unwrap().name,
        "soic::TSSOP24P65_TI_PW0024A"
    );
    assert!(
        part.alts.is_empty(),
        "the orderable MPN is a standalone part"
    );

    let footprint = &checked.world.footprints["soic::TSSOP24P65_TI_PW0024A"];
    assert_eq!(footprint.pads.len(), 24);
    let placements: Vec<_> = footprint
        .pads
        .iter()
        .map(|place| {
            (
                place.number.text.as_str(),
                place.x.text.as_str(),
                place.y.text.as_str(),
            )
        })
        .collect();
    assert_eq!(
        placements,
        [
            ("1", "-2.9mm", "-3.575mm"),
            ("2", "-2.9mm", "-2.925mm"),
            ("3", "-2.9mm", "-2.275mm"),
            ("4", "-2.9mm", "-1.625mm"),
            ("5", "-2.9mm", "-0.975mm"),
            ("6", "-2.9mm", "-0.325mm"),
            ("7", "-2.9mm", "0.325mm"),
            ("8", "-2.9mm", "0.975mm"),
            ("9", "-2.9mm", "1.625mm"),
            ("10", "-2.9mm", "2.275mm"),
            ("11", "-2.9mm", "2.925mm"),
            ("12", "-2.9mm", "3.575mm"),
            ("13", "2.9mm", "3.575mm"),
            ("14", "2.9mm", "2.925mm"),
            ("15", "2.9mm", "2.275mm"),
            ("16", "2.9mm", "1.625mm"),
            ("17", "2.9mm", "0.975mm"),
            ("18", "2.9mm", "0.325mm"),
            ("19", "2.9mm", "-0.325mm"),
            ("20", "2.9mm", "-0.975mm"),
            ("21", "2.9mm", "-1.625mm"),
            ("22", "2.9mm", "-2.275mm"),
            ("23", "2.9mm", "-2.925mm"),
            ("24", "2.9mm", "-3.575mm"),
        ]
    );
    assert!(footprint.pads.iter().all(|place| {
        place.pad.name == "soic::P_TSSOP24P65_TI_PW0024A_LEAD" && place.rotate == 0
    }));

    let lead = &checked.world.pads["soic::P_TSSOP24P65_TI_PW0024A_LEAD"];
    assert_eq!(
        lead.shape.map(|(shape, _)| shape),
        Some(cohdl::ast::PadShape::Rect)
    );
    assert_eq!(
        lead.size
            .iter()
            .map(|value| value.text.as_str())
            .collect::<Vec<_>>(),
        ["1.5mm", "0.45mm"]
    );
    assert_eq!(
        lead.layer.map(|(layer, _)| layer),
        Some(cohdl::ast::PadLayer::TopCopper)
    );
    assert_eq!(
        lead.plating.map(|(plating, _)| plating),
        Some(cohdl::ast::PadPlating::Smd)
    );

    let courtyard = footprint.courtyard.as_ref().unwrap();
    assert_eq!(courtyard.at.0.text, "0mm");
    assert_eq!(courtyard.at.1.text, "0mm");
    assert_eq!(
        courtyard
            .size
            .iter()
            .map(|value| value.text.as_str())
            .collect::<Vec<_>>(),
        ["7.8mm", "8.4mm"]
    );

    let mut footprint_diags = Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&checked.world, &mut footprint_diags);
    assert!(
        !footprint_diags.has_errors(),
        "{}",
        footprint_diags.render(&checked.sm)
    );
}

#[test]
fn esp32_s3_wroom_uses_exact_segmented_ep_and_distinct_memory_parts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![
        ("std".to_string(), root.join("lib/std")),
        ("qfn".to_string(), root.join("lib/qfn")),
    ];
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project =
        cohdl::project::load_project_with_deps(&root.join("lib/@espressif/esp32"), &deps).unwrap();
    let checked =
        cohdl::pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    let footprint = &checked.world.footprints["espressif_esp32::FP_ESP32_S3_WROOM_1"];
    let pad_41: Vec<_> = footprint
        .pads
        .iter()
        .filter(|place| place.number.text == "41")
        .collect();
    assert_eq!(
        footprint.pads.len(),
        61,
        "40 perimeter lands + 21 EP features"
    );
    assert_eq!(pad_41.len(), 21, "nine islands + twelve thermal vias");

    let island_name = "espressif_esp32::P_ESP32_S3_WROOM_EP_ISLAND";
    let via_name = "espressif_esp32::P_ESP32_S3_WROOM_EP_VIA";
    let islands: Vec<_> = pad_41
        .iter()
        .copied()
        .filter(|place| place.pad.name == island_name)
        .collect();
    let vias: Vec<_> = pad_41
        .iter()
        .copied()
        .filter(|place| place.pad.name == via_name)
        .collect();
    assert_eq!(islands.len(), 9);
    assert_eq!(vias.len(), 12);

    let island_pad = &checked.world.pads[island_name];
    assert_eq!(
        island_pad.shape.map(|(shape, _)| shape),
        Some(cohdl::ast::PadShape::Rect)
    );
    assert_eq!(
        island_pad
            .size
            .iter()
            .map(|value| value.text.as_str())
            .collect::<Vec<_>>(),
        ["0.9mm", "0.9mm"]
    );
    assert_eq!(
        island_pad.layer.map(|(layer, _)| layer),
        Some(cohdl::ast::PadLayer::TopCopper)
    );
    assert_eq!(
        island_pad.plating.map(|(plating, _)| plating),
        Some(cohdl::ast::PadPlating::Smd)
    );
    assert!(
        island_pad.paste.is_none(),
        "the library keeps the default nominal paste aperture; stencil reduction is process-specific"
    );

    let island_coords: std::collections::BTreeSet<_> = islands
        .iter()
        .map(|place| (place.x.text.as_str(), place.y.text.as_str()))
        .collect();
    let expected_islands = [
        ("-2.9mm", "1.06mm"),
        ("-1.5mm", "1.06mm"),
        ("-0.1mm", "1.06mm"),
        ("-2.9mm", "2.46mm"),
        ("-1.5mm", "2.46mm"),
        ("-0.1mm", "2.46mm"),
        ("-2.9mm", "3.86mm"),
        ("-1.5mm", "3.86mm"),
        ("-0.1mm", "3.86mm"),
    ]
    .into_iter()
    .collect();
    assert_eq!(island_coords, expected_islands);

    let via_pad = &checked.world.pads[via_name];
    assert_eq!(
        via_pad.shape.map(|(shape, _)| shape),
        Some(cohdl::ast::PadShape::Circle)
    );
    assert_eq!(
        via_pad
            .size
            .iter()
            .map(|value| value.text.as_str())
            .collect::<Vec<_>>(),
        ["0.5mm"]
    );
    assert_eq!(
        via_pad.layer.map(|(layer, _)| layer),
        Some(cohdl::ast::PadLayer::ThroughAll)
    );
    assert_eq!(
        via_pad.plating.map(|(plating, _)| plating),
        Some(cohdl::ast::PadPlating::PlatedThroughHole)
    );
    match via_pad.drill.as_ref().map(|(drill, _)| drill) {
        Some(cohdl::ast::PadDrill::Round(diameter)) => {
            assert_eq!(diameter.text, "0.25mm")
        }
        _ => panic!("ESP32-S3-WROOM thermal vias need a 0.25mm round drill"),
    }

    let via_coords: std::collections::BTreeSet<_> = vias
        .iter()
        .map(|place| (place.x.text.as_str(), place.y.text.as_str()))
        .collect();
    let expected_vias = [
        ("-2.2mm", "1.06mm"),
        ("-0.8mm", "1.06mm"),
        ("-2.2mm", "2.46mm"),
        ("-0.8mm", "2.46mm"),
        ("-2.2mm", "3.86mm"),
        ("-0.8mm", "3.86mm"),
        ("-2.9mm", "1.76mm"),
        ("-2.9mm", "3.16mm"),
        ("-1.5mm", "1.76mm"),
        ("-1.5mm", "3.16mm"),
        ("-0.1mm", "1.76mm"),
        ("-0.1mm", "3.16mm"),
    ]
    .into_iter()
    .collect();
    assert_eq!(via_coords, expected_vias);

    for (name, mpn) in [
        (
            "espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1_N8",
            "ESP32-S3-WROOM-1-N8",
        ),
        (
            "espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1_N8R2",
            "ESP32-S3-WROOM-1-N8R2",
        ),
    ] {
        let part = &checked.world.parts[name];
        assert_eq!(part.primary.field("mfr").unwrap().value, "Espressif");
        assert_eq!(part.primary.field("mpn").unwrap().value, mpn);
        assert_eq!(
            part.primary.footprint.as_ref().unwrap().name,
            "espressif_esp32::FP_ESP32_S3_WROOM_1"
        );
        assert!(
            part.alts.is_empty(),
            "memory variants are exact standalone parts, not AVL alternates"
        );
    }

    let esp_parts = checked
        .world
        .parts
        .iter()
        .filter(|(name, _)| name.starts_with("espressif_esp32::"))
        .collect::<Vec<_>>();
    assert_eq!(
        esp_parts.len(),
        140,
        "every admitted Product Selector identity must be exported as one part"
    );
    let exact_mpns = esp_parts
        .iter()
        .map(|(_, part)| {
            assert_eq!(
                part.primary.field("mfr").map(|field| field.value.as_str()),
                Some("Espressif")
            );
            assert!(
                part.alts.is_empty(),
                "ESP32 exact MPNs must not be AVL alts"
            );
            part.primary.field("mpn").unwrap().value.as_str()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(exact_mpns.len(), 140, "every ESP32 MPN must be unique");

    let c3 = &checked.world.parts["espressif_esp32::ESP32_C3"];
    assert_eq!(c3.device.name.name, "espressif_esp32::DEV_ESP32_C3");
    assert_eq!(
        c3.primary.footprint.as_ref().unwrap().name,
        "qfn::ESPRESSIF_QFN32_0P5_5"
    );
    assert_eq!(
        checked.world.docs["espressif_esp32::ESP32_C3"],
        ["docs/esp32-part-catalog.md"]
    );
    assert!(
        !checked
            .world
            .parts
            .contains_key("espressif_esp32::ESP32_S3_WROOM_1_N8R8"),
        "an Octal-PSRAM row must stay omitted until its pin availability has an exact model"
    );

    let c5 = &checked.world.devices["espressif_esp32::DEV_ESP32_C5"];
    let c5_pin_names = c5
        .pins_for(None)
        .iter()
        .map(|pin| pin.name.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(c5_pin_names.contains("RESERVED_25"));
    assert!(c5_pin_names.contains("RESERVED_32"));
    assert!(
        !c5_pin_names.contains("GPIO15"),
        "upstream no-connect die aliases must not become connectable-looking GPIO APIs"
    );
}

// ---------------------------------------------------------------------------
// footprint: a resolvable declaration kind.

#[test]
fn footprint_resolves_like_every_other_declaration() {
    // Cross-package: a library's pub footprint, imported and qualified.
    let (world, rendered) = world_of(&[
        (
            "sparkfun/src/footprints/qfn.cohdl",
            "pub footprint FP_QFN10_3x3 {}\n",
            "sparkfun",
            "sparkfun::footprints::qfn",
        ),
        (
            "app/src/main.cohdl",
            "use sparkfun::footprints::qfn::FP_QFN10_3x3;\n\
             pub device D { pins { A: 1 [passive] } }\n\
             pub part P1: D { primary { mfr: \"m\", mpn: \"a\", footprint: FP_QFN10_3x3 } }\n\
             pub part P2: D { primary { mfr: \"m\", mpn: \"b\", footprint: sparkfun::footprints::qfn::FP_QFN10_3x3 } }\n",
            "app",
            "app",
        ),
    ]);
    assert!(!rendered.contains("error"), "{}", rendered);
    assert!(world
        .footprints
        .contains_key("sparkfun::footprints::qfn::FP_QFN10_3x3"));
    // Both references resolved to the same fq symbol.
    for part in ["app::P1", "app::P2"] {
        assert_eq!(
            world.parts[part].primary.footprint.as_ref().unwrap().name,
            "sparkfun::footprints::qfn::FP_QFN10_3x3"
        );
    }
}

#[test]
fn non_pub_footprint_is_invisible_cross_package() {
    let (_world, rendered) = world_of(&[
        ("lib/src/main.cohdl", "footprint Hidden {}\n", "lib", "lib"),
        (
            "app/src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\n\
             pub part P: D { primary { mfr: \"m\", mpn: \"n\", footprint: lib::Hidden } }\n",
            "app",
            "app",
        ),
    ]);
    assert!(rendered.contains("E209"), "{}", rendered);
}

#[test]
fn footprint_reference_must_be_a_footprint() {
    // A device where a footprint is required: wrong kind, E205.
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Res { pins { A: 1 [passive] } }\npub part P: Res { primary { mfr: \"m\", mpn: \"n\", footprint: Res } }\n",
        )],
    );
    assert!(rendered.contains("E205"), "{}", rendered);
    assert!(rendered.contains("not a footprint"), "{}", rendered);

    // Unknown symbol: E202 with the closest-match suggestion.
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Res { pins { A: 1 [passive] } }\npub footprint FP_0402 {}\npub part P: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_0403 } }\n",
        )],
    );
    assert!(rendered.contains("unknown footprint"), "{}", rendered);
}

#[test]
fn footprint_string_gets_the_migration_error() {
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Res { pins { A: 1 [passive] } }\npub part P: Res { primary { mfr: \"m\", mpn: \"n\", footprint: \"Lib:Name\" } }\n",
        )],
    );
    assert!(
        rendered.contains("references a footprint SYMBOL"),
        "{}",
        rendered
    );
    assert!(
        rendered.contains("pub footprint"),
        "targeted help:\n{}",
        rendered
    );
}

#[test]
fn footprint_body_is_real_since_rfc018() {
    // RFC-018 gave the body content: a malformed placement is a precise
    // grammar error, not the old "format not yet specified" deferral.
    let (_checked, rendered) = check(
        "board",
        &[("src/main.cohdl", "pub footprint FP { pad 1 }\n")],
    );
    assert!(rendered.contains("E010"), "{}", rendered);
    assert!(
        !rendered.contains("not yet specified"),
        "the deferral message is retired:\n{}",
        rendered
    );
}

#[test]
fn netlist_emits_the_resolved_footprint_symbol() {
    let files = vec![("src/main.cohdl".to_string(), BOARD.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    assert!(!checked.diags.has_errors());
    let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
    assert!(
        artifacts.netlist.contains("(footprint \"board::FP_0402\")"),
        "the .net carries the fq footprint symbol:\n{}",
        artifacts.netlist
    );
}

// ---------------------------------------------------------------------------
// #[doc(...)]: multiple per declaration, zero compilation impact.

#[test]
fn docs_are_zero_impact_and_recorded() {
    let plain = BOARD;
    let documented = BOARD.replace(
        "pub device Res",
        "#[doc(\"datasheets/res.pdf\")]\n#[doc(\"app-notes/res-layout.pdf\")]\npub device Res",
    );
    let build = |src: &str| {
        let files = vec![("src/main.cohdl".to_string(), src.to_string())];
        let mut checked = check_files_in("board", &files, None).expect("selection");
        let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
        checked.diags.sort(&checked.sm);
        (
            checked.diags.render(&checked.sm),
            artifacts.netlist,
            artifacts.bom,
            artifacts.lock.render(),
            checked,
        )
    };
    let (d1, n1, b1, l1, _c1) = build(plain);
    let (d2, n2, b2, l2, c2) = build(&documented);
    assert_eq!(d1, d2, "docs changed diagnostics");
    assert_eq!(n1, n2, "docs changed the netlist");
    assert_eq!(b1, b2, "docs changed the BOM");
    assert_eq!(l1, l2, "docs changed the designator lock");
    // …and the paths are recorded for tooling.
    assert_eq!(
        c2.world.docs.get("board::Res").map(Vec::as_slice),
        Some(
            &[
                "datasheets/res.pdf".to_string(),
                "app-notes/res-layout.pdf".to_string()
            ][..]
        )
    );
}

#[test]
fn doc_attr_shape_is_validated() {
    // No argument.
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "#[doc]\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("exactly one string"), "{}", rendered);
    // Two arguments in one attribute.
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "#[doc(\"a.pdf\", \"b.pdf\")]\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("exactly one string"), "{}", rendered);
    // On a use import: rejected.
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub footprint F {}\n#[doc(\"x.pdf\")]\nuse board::F;\n",
        )],
    );
    assert!(rendered.contains("not valid on a `use`"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// fmt round-trips the new constructs.

#[test]
fn fmt_round_trips_footprint_and_docs() {
    use cohdl::fmt::format_source;
    let src = "#[doc(\"ds.pdf\")]\n#[doc(\"an.pdf\")]\npub device D { pins { A: 1 [passive] } }\npub footprint FP_X {} // placeholder\npub part P: D { primary { mfr: \"m\", mpn: \"n\", footprint: FP_X } }\n";
    let once = format_source("lib.cohdl", src).unwrap();
    assert!(
        once.contains("#[doc(\"ds.pdf\")]\n#[doc(\"an.pdf\")]\n"),
        "{}",
        once
    );
    assert!(
        once.contains("pub footprint FP_X {} // placeholder"),
        "{}",
        once
    );
    assert!(
        once.contains("footprint: FP_X"),
        "unquoted symbol:\n{}",
        once
    );
    let twice = format_source("lib.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);
}

// Backstop: the compat single-package path still accepts everything.
#[test]
fn compat_entry_supports_footprints() {
    let files = vec![("f.cohdl".to_string(), BOARD.to_string())];
    let checked = check_files(&files, None).expect("selection");
    assert!(!checked.diags.has_errors());
    assert!(checked.world.footprints.contains_key("main::FP_0402"));
}

// ---------------------------------------------------------------------------
// Adversarial-verification regressions (RFC-017 round 1).

// Finding (high/medium): panic-mode recovery swallowed a following bare
// `footprint` declaration (sync sets knew `use` but not `footprint`),
// manufacturing phantom E202s.
#[test]
fn recovery_stops_at_footprint_declarations() {
    let (checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "garbage\nfootprint FP_X {}\npub device Res { pins { A: 1 [passive] } }\npub part P: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_X } }\n",
        )],
    );
    assert!(rendered.contains("E010"), "{}", rendered);
    assert!(
        !rendered.contains("unknown footprint"),
        "the footprint decl must survive recovery:\n{}",
        rendered
    );
    assert!(checked.world.footprints.contains_key("board::FP_X"));
}

// Finding (medium): a misplaced footprint decl inside a design body
// misparsed as a fn call and destroyed the rest of the body.
#[test]
fn footprint_in_a_body_gets_a_targeted_error() {
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\ndesign B {\n    footprint FP {}\n    inst d: D\n    net N: d.A\n}\n",
        )],
    );
    assert!(rendered.contains("top-level"), "{}", rendered);
    assert!(
        !rendered.contains("expected `(`"),
        "no fn-call misparse:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("unknown instance"),
        "the body keeps parsing:\n{}",
        rendered
    );
}

// Finding (medium): invalid attributes on an inst inside a NEVER-CALLED fn
// were silently accepted (attr validation only ran at expansion).
#[test]
fn inst_attrs_are_validated_at_parse_even_in_uncalled_fns() {
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\nfn unused(p: Pin) {\n    #[frobnicate(\"x\")]\n    inst d: D\n    net _: p, d.A\n}\n",
        )],
    );
    assert!(
        rendered.contains("unrecognized attribute `frobnicate`"),
        "{}",
        rendered
    );
}

// Finding (low, RFC-017 round; semantics updated for RFC-018): malformed
// body content must not cascade past the closing brace or loop forever.
#[test]
fn footprint_body_recovery_is_contained() {
    // Malformed placements: errors stay inside the body, following
    // declarations survive, and the parser always makes progress.
    let (checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub footprint FP { pad 1, pad 2 }\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("E010"), "{}", rendered);
    assert!(
        !rendered.contains("expected a top-level declaration"),
        "body content must not cascade to file scope:\n{}",
        rendered
    );
    assert!(checked.world.devices.contains_key("board::D"));
}

// Finding (low): `footprint {}` (missing name) got the generic
// expected-a-declaration message.
#[test]
fn footprint_missing_name_is_named() {
    let (_checked, rendered) = check("board", &[("src/main.cohdl", "pub footprint {}\n")]);
    assert!(rendered.contains("needs a name"), "{}", rendered);
}

// Finding (low): #[doc] on an impl was silently dropped (impls are unnamed
// — the paths were recorded nowhere).
#[test]
fn doc_on_impl_is_rejected_with_the_reason() {
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub trait T { pins { required A: pin } }\npub device D { pins { A: 1 [passive] } }\n#[doc(\"impl-notes.pdf\")]\nimpl T for D {}\n",
        )],
    );
    assert!(rendered.contains("impls are unnamed"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Fifth-review (2026-07-15) regressions.

// R5-7(a): a duplicate singleton AVL field (mpn/mfr) is rejected, not
// silently first-wins (which dropped the shadowed value from the BOM/AVL).
#[test]
fn duplicate_avl_field_is_rejected() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\n\
             pub footprint FP {}\n\
             pub part P: D { primary { mfr: \"A\", mpn: \"X\", mpn: \"Y\", footprint: FP } }\n",
        )],
    );
    assert!(
        r.contains("E802") && r.contains("duplicate AVL field `mpn`"),
        "{}",
        r
    );
}

// R5-7(b): two parts sharing (manufacturer, MPN) but describing different
// components (different device/value) are rejected — one part number names
// one component, and the lossy BOM grouping would hide the disagreement.
#[test]
fn inconsistent_parts_sharing_mfr_mpn_are_rejected() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Da { pins { A: 1 [passive] } spec { resistance: 1kohm } }\n\
             pub device Db { pins { A: 1 [passive] } spec { resistance: 2kohm } }\n\
             pub footprint FP {}\n\
             pub part PA: Da { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n\
             pub part PB: Db { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n",
        )],
    );
    assert!(
        r.contains("E802") && r.contains("describes a different component"),
        "{}",
        r
    );
}

// R5-7(b): two parts that genuinely ARE the same component (identical device,
// binding, footprint) may share (manufacturer, MPN) without error.
#[test]
fn consistent_parts_sharing_mfr_mpn_are_allowed() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } spec { resistance: 1kohm } }\n\
             pub footprint FP {}\n\
             pub part PA: D { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n\
             pub part PB: D { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n",
        )],
    );
    assert!(!r.contains("E802"), "identical parts may share MPN:\n{}", r);
}

// R5-9: a `#[doc]` path must be package-relative — absolute, parent-escape,
// empty, and URL forms are rejected lexically.
#[test]
fn doc_paths_must_be_package_relative() {
    for (path, _why) in [
        ("/etc/passwd", "absolute"),
        ("../../outside.pdf", "parent escape"),
        ("", "empty"),
        ("https://example.com/x.pdf", "url"),
    ] {
        let (_c, r) = check(
            "board",
            &[(
                "src/main.cohdl",
                &format!(
                    "#[doc(\"{}\")]\npub device D {{ pins {{ A: 1 [passive] }} }}\n",
                    path
                ),
            )],
        );
        assert!(
            r.contains("not a package-relative path"),
            "doc path `{}` must be rejected:\n{}",
            path,
            r
        );
    }
    // A normal relative path is fine.
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "#[doc(\"datasheets/d.pdf\")]\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(!r.contains("not a package-relative"), "{}", r);
}

// ---------------------------------------------------------------------------
// Sixth-review (2026-07-15) regressions.

// R6-2: the same-MPN identity comparison uses FULLY-QUALIFIED names — two
// parts whose devices share a leaf name but differ in module (and value) are
// distinct components and must be rejected, not collapsed by short().
#[test]
fn same_mpn_distinct_fq_devices_are_rejected() {
    let (_c, r) = check(
        "board",
        &[
            (
                "src/a/dev.cohdl",
                "pub device D { pins { A: 1 [passive] } spec { resistance: 1kohm } }\npub footprint FP {}\n",
            ),
            (
                "src/b/dev.cohdl",
                "pub device D { pins { A: 1 [passive] } spec { resistance: 2kohm } }\npub footprint FP {}\n",
            ),
            (
                "src/main.cohdl",
                "pub part PA: board::a::dev::D { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: board::a::dev::FP } }\n\
                 pub part PB: board::b::dev::D { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: board::b::dev::FP } }\n\
                 design B {}\n",
            ),
        ],
    );
    assert!(
        r.contains("E802") && r.contains("different component"),
        "distinct fq devices under one MPN must be rejected:\n{}",
        r
    );
}

// R6-2: generic bindings compare NORMALIZED unit values — `1kohm` and
// `1000ohm` are the same component, so sharing an MPN is allowed.
#[test]
fn equivalent_unit_spellings_are_the_same_component() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device R<V: Resistance> { pins { A: 1 [passive] } }\npub footprint FP {}\n\
             pub part PA: R<1kohm> { primary { mfr: \"Y\", mpn: \"SHARED\", footprint: FP } }\n\
             pub part PB: R<1000ohm> { primary { mfr: \"Y\", mpn: \"SHARED\", footprint: FP } }\n",
        )],
    );
    assert!(
        !r.contains("E802"),
        "1kohm and 1000ohm are the same value — no conflict:\n{}",
        r
    );
}

// R6-2: an ALTERNATE AVL entry sharing another part's manufacturer/MPN with a
// different component is also caught (not only primary entries).
#[test]
fn alt_entry_mpn_conflict_is_checked() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Da { pins { A: 1 [passive] } spec { resistance: 1kohm } }\n\
             pub device Db { pins { A: 1 [passive] } spec { resistance: 2kohm } }\n\
             pub footprint FP {}\n\
             pub part PA: Da { primary { mfr: \"Z\", mpn: \"SHARED\", footprint: FP } }\n\
             pub part PB: Db { primary { mfr: \"Z\", mpn: \"OTHER\", footprint: FP }\n\
                 alt { mfr: \"Z\", mpn: \"SHARED\", footprint: FP } }\n",
        )],
    );
    assert!(
        r.contains("E802") && r.contains("different component"),
        "an alt entry conflict must be caught:\n{}",
        r
    );
}

// R6-6: doc-path validation rejects drive roots and every URI-scheme form,
// not just the four originally named.
#[test]
fn doc_paths_reject_drive_roots_and_uri_schemes() {
    for path in [
        "C:/Windows/System32/manual.pdf",
        "file:/etc/passwd",
        "mailto:docs@example.com",
        "data:text/plain,hello",
        "docs\\win.pdf",
    ] {
        let (_c, r) = check(
            "board",
            &[(
                "src/main.cohdl",
                &format!(
                    "#[doc(\"{}\")]\npub device D {{ pins {{ A: 1 [passive] }} }}\n",
                    path
                ),
            )],
        );
        assert!(
            r.contains("not a package-relative path"),
            "doc path `{}` must be rejected:\n{}",
            path,
            r
        );
    }
    // A relative path with a colon deeper (not first segment) is still fine.
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "#[doc(\"notes/a:b.txt\")]\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(!r.contains("not a package-relative"), "{}", r);
}

// R7-3: the same-MPN identity resolves generic DEFAULTS — a part relying on a
// default and one writing the same value explicitly are the same component.
#[test]
fn default_equivalent_generics_are_the_same_component() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device R<V: Resistance = 1kohm> { pins { A: 1 [passive] } }\npub footprint FP {}\n\
             pub part PA: R { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n\
             pub part PB: R<1kohm> { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: FP } }\n",
        )],
    );
    assert!(
        !r.contains("E802"),
        "default 1kohm and explicit 1kohm are the same component:\n{}",
        r
    );
}

// R7-3: an alt entry that omits its optional footprint inherits the primary's
// effective footprint, so it is not falsely compared as empty.
#[test]
fn omitted_alt_footprint_inherits_primary() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\npub footprint FP {}\n\
             pub part PX: D { primary { mfr: \"Z\", mpn: \"AAA\", footprint: FP } }\n\
             pub part PY: D { primary { mfr: \"Z\", mpn: \"BBB\", footprint: FP }\n\
                 alt { mfr: \"Z\", mpn: \"AAA\" } }\n",
        )],
    );
    // PY's alt (mfr Z, mpn AAA, inheriting FP) matches PX (mfr Z, mpn AAA, FP)
    // — same component, no false conflict.
    assert!(
        !r.contains("E802"),
        "omitted alt footprint inherits primary:\n{}",
        r
    );
}

// R7-5: doc-path validation rejects `./`, empty components, and trailing
// separators — not just direct scheme/drive forms.
#[test]
fn doc_paths_reject_dot_slash_and_empty_components() {
    for path in [
        "./file:/etc/passwd",
        "./C:/Windows/System32/manual.pdf",
        "docs//manual.pdf",
        "docs/",
        "./docs/x.pdf",
    ] {
        let (_c, r) = check(
            "board",
            &[(
                "src/main.cohdl",
                &format!(
                    "#[doc(\"{}\")]\npub device D {{ pins {{ A: 1 [passive] }} }}\n",
                    path
                ),
            )],
        );
        assert!(
            r.contains("not a package-relative path"),
            "doc path `{}` must be rejected:\n{}",
            path,
            r
        );
    }
}

// The canonical STM32F072 pin map is library data, not board data: pin names
// are the datasheet's PA/PB/PC ports on their LQFP-48 numbers, and board-level
// aliases must never leak into the manufacturer package. (These assertions
// lived in the OpenMicro example's exit-criteria test until that project moved
// to its own repository.)
#[test]
fn stm32f072_parts_use_canonical_pins_and_audited_geometry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps = vec![
        ("std".to_string(), root.join("lib/std")),
        ("bga".to_string(), root.join("lib/bga")),
        ("csp".to_string(), root.join("lib/csp")),
        ("qfp".to_string(), root.join("lib/qfp")),
        ("soic".to_string(), root.join("lib/soic")),
    ];
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project =
        cohdl::project::load_project_with_deps(&root.join("lib/@st/stm32"), &deps).unwrap();
    let checked =
        cohdl::pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    let mcu = &checked.world.devices["st_stm32::STM32F072CBTx"];
    let pins = mcu.pins_for(None);
    for (name, number) in [
        ("PA9", "30"),
        ("PA11", "32"),
        ("PA12", "33"),
        ("PA13", "34"),
        ("PA14", "37"),
        ("PB14", "27"),
    ] {
        let pin = pins.iter().find(|pin| pin.name.name == name).unwrap();
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            [number],
            "wrong physical mapping for {name}"
        );
    }
    for board_alias in ["ROW0", "USB_DM", "SWDIO", "LED_DATA_UG"] {
        assert!(
            pins.iter().all(|pin| pin.name.name != board_alias),
            "board alias `{board_alias}` leaked into the manufacturer library"
        );
    }

    assert!(
        !checked
            .world
            .devices
            .contains_key("st_stm32::f0::stm32f072cb::STM32F072CBT6"),
        "the duplicate authored F072 device must be removed"
    );

    let audited_parts = [
        (
            "MCU_STM32F072C8T6",
            "STM32F072C8Tx",
            "STM32F072C8T6",
            Some("STM32F072C8T6TR"),
            "qfp::QFP50P900X900X160_48N",
        ),
        (
            "MCU_STM32F072C8T7",
            "STM32F072C8Tx",
            "STM32F072C8T7",
            None,
            "qfp::QFP50P900X900X160_48N",
        ),
        (
            "MCU_STM32F072CBT6",
            "STM32F072CBTx",
            "STM32F072CBT6",
            Some("STM32F072CBT6TR"),
            "qfp::QFP50P900X900X160_48N",
        ),
        (
            "MCU_STM32F072CBT7",
            "STM32F072CBTx",
            "STM32F072CBT7",
            None,
            "qfp::QFP50P900X900X160_48N",
        ),
        (
            "MCU_STM32F072R8T6",
            "STM32F072R8Tx",
            "STM32F072R8T6",
            Some("STM32F072R8T6TR"),
            "qfp::QFP50P1200X1200X160_64N",
        ),
        (
            "MCU_STM32F072R8T7",
            "STM32F072R8Tx",
            "STM32F072R8T7",
            None,
            "qfp::QFP50P1200X1200X160_64N",
        ),
        (
            "MCU_STM32F072RBH6",
            "STM32F072RBHx",
            "STM32F072RBH6",
            Some("STM32F072RBH6TR"),
            "bga::BGA64C50P8X8_500X500X60N",
        ),
        (
            "MCU_STM32F072RBT6",
            "STM32F072RBTx",
            "STM32F072RBT6",
            Some("STM32F072RBT6TR"),
            "qfp::QFP50P1200X1200X160_64N",
        ),
        (
            "MCU_STM32F072RBT7",
            "STM32F072RBTx",
            "STM32F072RBT7",
            Some("STM32F072RBT7TR"),
            "qfp::QFP50P1200X1200X160_64N",
        ),
        (
            "MCU_STM32F072V8H6",
            "STM32F072V8Hx",
            "STM32F072V8H6",
            None,
            "bga::BGA100C50P12X12_700X700X60N",
        ),
        (
            "MCU_STM32F072V8T6",
            "STM32F072V8Tx",
            "STM32F072V8T6",
            None,
            "qfp::QFP50P1600X1600X160_100N",
        ),
        (
            "MCU_STM32F072VBH6",
            "STM32F072VBHx",
            "STM32F072VBH6",
            Some("STM32F072VBH6TR"),
            "bga::BGA100C50P12X12_700X700X60N",
        ),
        (
            "MCU_STM32F072VBH7",
            "STM32F072VBHx",
            "STM32F072VBH7",
            None,
            "bga::BGA100C50P12X12_700X700X60N",
        ),
        (
            "MCU_STM32F072VBT6",
            "STM32F072VBTx",
            "STM32F072VBT6",
            Some("STM32F072VBT6TR"),
            "qfp::QFP50P1600X1600X160_100N",
        ),
    ];
    assert_eq!(
        checked
            .world
            .parts
            .keys()
            .filter(|name| name.starts_with("st_stm32::"))
            .count(),
        2389,
        "every source-joined exact STM32 identity must be public"
    );
    let audited_part_names = audited_parts
        .iter()
        .map(|(symbol, ..)| format!("st_stm32::{symbol}"))
        .collect::<std::collections::BTreeSet<_>>();
    for (symbol, device, primary_mpn, alt_mpn, footprint) in audited_parts {
        let part_name = format!("st_stm32::{symbol}");
        let part = checked
            .world
            .parts
            .get(&part_name)
            .unwrap_or_else(|| panic!("missing audited part `{part_name}`"));
        assert_eq!(part.device.name.name, format!("st_stm32::{device}"));
        assert_eq!(
            part.primary.field("mfr").map(|field| field.value.as_str()),
            Some("STMicroelectronics")
        );
        assert_eq!(
            part.primary.field("mpn").map(|field| field.value.as_str()),
            Some(primary_mpn)
        );
        assert_eq!(part.primary.footprint.as_ref().unwrap().name, footprint);
        match alt_mpn {
            Some(expected) => {
                assert_eq!(part.alts.len(), 1);
                assert_eq!(
                    part.alts[0].field("mpn").map(|field| field.value.as_str()),
                    Some(expected)
                );
            }
            None => assert!(part.alts.is_empty()),
        }
        assert_eq!(
            checked.world.docs[&part_name],
            ["docs/stm32f072cb-datasheet.pdf"]
        );
    }

    let exact_parts = checked
        .world
        .parts
        .iter()
        .filter(|(name, _)| name.starts_with("st_stm32::"))
        .collect::<Vec<_>>();
    assert_eq!(
        exact_parts
            .iter()
            .map(|(_, part)| 1 + part.alts.len())
            .sum::<usize>(),
        3303,
        "terminal-TR rows must be preserved as AVL alternates"
    );
    for (name, part) in exact_parts {
        assert_eq!(
            part.primary.field("mfr").map(|field| field.value.as_str()),
            Some("STMicroelectronics")
        );
        let footprint = &part.primary.footprint.as_ref().unwrap().name;
        assert!(
            ["bga::", "csp::", "qfp::", "soic::"]
                .iter()
                .any(|prefix| footprint.starts_with(prefix)),
            "STM32 part `{name}` has non-focused footprint `{footprint}`"
        );
        if !audited_part_names.contains(name) {
            assert_eq!(
                checked.world.docs[name],
                ["docs/stm32-part-catalog.md"],
                "catalog part `{name}` lost its local provenance index"
            );
        }
    }

    let assert_place = |footprint: &str, number: &str, x: &str, y: &str| {
        let place = checked.world.footprints[footprint]
            .pads
            .iter()
            .find(|place| place.number.text == number)
            .unwrap_or_else(|| panic!("missing `{footprint}` pad `{number}`"));
        assert_eq!(
            (place.x.text.as_str(), place.y.text.as_str()),
            (x, y),
            "wrong placement for `{footprint}` pad `{number}`"
        );
    };
    for (footprint, count, samples) in [
        (
            "qfp::QFP50P900X900X160_48N",
            48,
            &[
                ("1", "-4.25mm", "-2.75mm"),
                ("13", "-2.75mm", "4.25mm"),
                ("25", "4.25mm", "2.75mm"),
                ("37", "2.75mm", "-4.25mm"),
            ][..],
        ),
        (
            "qfp::QFP50P1200X1200X160_64N",
            64,
            &[
                ("1", "-5.75mm", "-3.75mm"),
                ("17", "-3.75mm", "5.75mm"),
                ("33", "5.75mm", "3.75mm"),
                ("49", "3.75mm", "-5.75mm"),
            ][..],
        ),
        (
            "qfp::QFP50P1600X1600X160_100N",
            100,
            &[
                ("1", "-7.75mm", "-6mm"),
                ("26", "-6mm", "7.75mm"),
                ("51", "7.75mm", "6mm"),
                ("76", "6mm", "-7.75mm"),
            ][..],
        ),
    ] {
        assert_eq!(checked.world.footprints[footprint].pads.len(), count);
        for (number, x, y) in samples {
            assert_place(footprint, number, x, y);
        }
    }

    let bga64 = &checked.world.footprints["bga::BGA64C50P8X8_500X500X60N"];
    let bga100 = &checked.world.footprints["bga::BGA100C50P12X12_700X700X60N"];
    assert_eq!(bga64.pads.len(), 64);
    assert_eq!(bga100.pads.len(), 100);
    assert_place("bga::BGA64C50P8X8_500X500X60N", "A1", "-1.75mm", "-1.75mm");
    assert_place("bga::BGA64C50P8X8_500X500X60N", "H8", "1.75mm", "1.75mm");
    assert_place(
        "bga::BGA100C50P12X12_700X700X60N",
        "A1",
        "-2.75mm",
        "-2.75mm",
    );
    assert_place(
        "bga::BGA100C50P12X12_700X700X60N",
        "M12",
        "2.75mm",
        "2.75mm",
    );
    for absent in ["C6", "C7", "F3", "F10", "I1"] {
        assert!(
            bga100.pads.iter().all(|place| place.number.text != absent),
            "sparse UFBGA100 map must not fabricate ball `{absent}`"
        );
    }
    for (device, footprint) in [
        ("st_stm32::STM32F072RBHx", bga64),
        ("st_stm32::STM32F072VBHx", bga100),
    ] {
        let device_pads = checked.world.devices[device]
            .pins_for(None)
            .iter()
            .flat_map(|pin| pin.numbers.iter().map(|number| number.text.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        let footprint_pads = footprint
            .pads
            .iter()
            .map(|place| place.number.text.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            footprint_pads, device_pads,
            "`{device}` and its audited BGA ball map diverged"
        );
    }

    // Broad generated coverage is source-locked by representative classic,
    // current, and high-pin-count families. Exact part emission remains a
    // fail-closed subset when no concrete pad-set-proven footprint exists.
    for name in [
        "st_stm32::STM32C531CBT6",
        "st_stm32::STM32F103C8Tx",
        "st_stm32::STM32G071KBTx",
        "st_stm32::STM32H743VITx",
        "st_stm32::STM32N657X0HxQ",
        "st_stm32::STM32U585AIIx",
        "st_stm32::STM32WB55VCQx",
    ] {
        assert!(checked.world.devices.contains_key(name), "missing `{name}`");
    }
    assert_eq!(
        checked
            .world
            .devices
            .keys()
            .filter(|name| name.starts_with("st_stm32::"))
            .count(),
        2284,
        "the duplicate authored F072 device must be replaced by its generated model"
    );
    for quarantined in [
        "st_stm32::STM32C531CBU6",
        "st_stm32::STM32F072CBUx",
        "st_stm32::STM32H553VGZx",
    ] {
        assert!(
            !checked.world.devices.contains_key(quarantined),
            "incomplete exposed-pad device `{quarantined}` must stay quarantined"
        );
    }

    let remappable = &checked.world.devices["st_stm32::STM32G071KBTx"];
    let remappable_pins = remappable.pins_for(None);
    for (name, number) in [("PA9", "19"), ("PA11", "22")] {
        let pin = remappable_pins
            .iter()
            .find(|pin| pin.name.name == name)
            .unwrap();
        assert_eq!(pin.numbers[0].text, number);
    }

    let distinct = &checked.world.devices["st_stm32::STM32H743XIHx"];
    let distinct_pins = distinct.pins_for(None);
    for (name, number) in [
        ("PC2", "M3"),
        ("PC2_C", "R1"),
        ("PC3", "M4"),
        ("PC3_C", "R2"),
    ] {
        let pin = distinct_pins
            .iter()
            .find(|pin| pin.name.name == name)
            .unwrap();
        assert_eq!(pin.numbers[0].text, number);
    }

    // The generator's conservative electrical policy is part of the library
    // contract, not merely a source-count check. Keep representative roles,
    // obligations, grouped supplies, aliases, and inherited C5 descriptors
    // pinned here so a broad upstream type bucket cannot silently weaken it.
    let assert_catalog_pin = |device_name: &str,
                              pin_name: &str,
                              numbers: &[&str],
                              role: cohdl::ast::PinRole,
                              obligation: cohdl::ast::Obligation| {
        let device = &checked.world.devices[device_name];
        let pin = device
            .pins_for(None)
            .iter()
            .find(|pin| pin.name.name == pin_name)
            .unwrap_or_else(|| panic!("missing `{device_name}.{pin_name}`"));
        assert_eq!(
            pin.numbers
                .iter()
                .map(|number| number.text.as_str())
                .collect::<Vec<_>>(),
            numbers,
            "wrong physical mapping for `{device_name}.{pin_name}`"
        );
        assert_eq!(
            pin.role_or_default(),
            role,
            "wrong role for `{device_name}.{pin_name}`"
        );
        assert_eq!(
            pin.obligation, obligation,
            "wrong obligation for `{device_name}.{pin_name}`"
        );
    };

    let optional = cohdl::ast::Obligation::Optional;
    let required = cohdl::ast::Obligation::Required;
    let input = cohdl::ast::PinRole::Input;
    let output = cohdl::ast::PinRole::Output;
    let passive = cohdl::ast::PinRole::Passive;
    let bidirectional = cohdl::ast::PinRole::Bidirectional;
    let power_in = cohdl::ast::PinRole::PowerIn;

    for (name, number) in [
        ("NC1_1", "1"),
        ("NC2_2", "2"),
        ("NC3_3", "3"),
        ("NC4_4", "4"),
        ("NC5_5", "5"),
        ("NC6_50", "50"),
        ("NC7_51", "51"),
        ("NC9_53", "53"),
    ] {
        assert_catalog_pin(
            "st_stm32::STM32WBA6MOIHx",
            name,
            &[number],
            passive,
            optional,
        );
    }
    for (name, number, role) in [
        ("ANT_OUT", "45", output),
        ("ANT_IN", "47", input),
        ("PD7", "17", bidirectional),
        ("PD6", "18", bidirectional),
    ] {
        assert_catalog_pin("st_stm32::STM32WBA6MOIHx", name, &[number], role, optional);
    }
    for (name, number, role) in [("OSC_IN", "12", input), ("OSC_OUT", "13", output)] {
        assert_catalog_pin("st_stm32::STM32F100V8Tx", name, &[number], role, optional);
    }
    for (device, name, number) in [
        ("st_stm32::STM32F030CCTx", "PC14OSC32_IN", "3"),
        ("st_stm32::STM32F030CCTx", "PC15OSC32_OUT", "4"),
        ("st_stm32::STM32F091CBTx", "PF11BOOT0", "44"),
        ("st_stm32::STM32H5E5IJKxQ", "OTG_HS_DM", "G15"),
        ("st_stm32::STM32H5E5IJKxQ", "OTG_HS_DP", "F15"),
    ] {
        assert_catalog_pin(device, name, &[number], bidirectional, optional);
    }
    assert_catalog_pin(
        "st_stm32::STM32L151QCHx",
        "OPAMP1_VINM",
        &["M3"],
        input,
        optional,
    );
    for (name, number, role) in [
        ("OSCIN", "K1", input),
        ("OSCOUT", "M1", output),
        ("RF1", "M5", passive),
    ] {
        assert_catalog_pin("st_stm32::STM32WB05TZFx", name, &[number], role, optional);
    }
    for (name, number, role) in [
        ("OTG1_HSDM", "A5", bidirectional),
        ("OTG1_ID", "B4", input),
        ("UCPD1_CC1", "B6", bidirectional),
        ("CSI_CKN", "M3", input),
        ("CSI_REXT", "N5", passive),
    ] {
        assert_catalog_pin("st_stm32::STM32N645A0HxQ", name, &[number], role, optional);
    }
    assert_catalog_pin(
        "st_stm32::STM32F469AEHx",
        "DSIHOST_CKN",
        &["H13"],
        output,
        optional,
    );

    for (name, numbers, role, obligation) in [
        ("NRST", &["7"][..], input, optional),
        ("BOOT0", &["44"][..], input, required),
        ("VSS", &["23", "35", "47"][..], power_in, required),
        ("VDD", &["24", "36", "48"][..], power_in, required),
    ] {
        assert_catalog_pin("st_stm32::STM32F103C8Tx", name, numbers, role, obligation);
    }

    assert_catalog_pin(
        "st_stm32::STM32C531FBP6",
        "PA1_OR_PA2_OR_PA3",
        &["7"],
        bidirectional,
        optional,
    );
    // This C5 variant has no variant-local descriptor in the PDSC; its
    // LQFP80 pinout is inherited from the containing device declaration.
    for (name, numbers, role) in [
        ("VSS", &["6", "22", "39", "59", "79"][..], power_in),
        ("VDD", &["7", "23", "40", "60", "80"][..], power_in),
        ("VCAP", &["38"][..], passive),
    ] {
        assert_catalog_pin("st_stm32::STM32C551MCT6", name, numbers, role, required);
    }
}

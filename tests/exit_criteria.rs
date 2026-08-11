//! Fixture tests mapped 1:1 to the MVP Definition's exit criteria
//! (docs/design/09-mvp-definition.md, "Exit criteria"). Each test name says
//! which criterion it proves. The demo-scenario criterion (AI loop) lives in
//! harness/, not here.

use cohdl::diag::Diagnostics;
use cohdl::lock::{assign_designators, LockState};
use cohdl::pipeline::{build_artifacts, check_files};

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("fixture.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("design selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

/// Load an example board the way the CLI does: every library it pins.
fn load_example(dir: &std::path::Path) -> (cohdl::project::Project, Vec<String>) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let (_, manifest) = cohdl::project::peek_manifest(dir).unwrap();
    let mut names: Vec<String> = manifest
        .deps_raw
        .unwrap_or_default()
        .into_iter()
        .map(|(name, _, _)| name)
        .collect();
    names.sort();
    if let Some(pos) = names.iter().position(|name| name == "std") {
        let std = names.remove(pos);
        names.insert(0, std);
    }
    let deps: Vec<(String, std::path::PathBuf)> = names
        .iter()
        .map(|name| (name.clone(), root.join("lib").join(name)))
        .collect();
    (
        cohdl::project::load_project_with_deps(dir, &deps).unwrap(),
        names,
    )
}

/// Load the real std library + a board source, as the CLI would.
fn check_with_std(board_src: &str) -> (cohdl::pipeline::Checked, String) {
    let mut files = Vec::new();
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std/src");
    let mut entries: Vec<_> = std::fs::read_dir(&std_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "cohdl"))
        .collect();
    entries.sort();
    for p in entries {
        files.push((
            format!("std/{}", p.file_name().unwrap().to_string_lossy()),
            std::fs::read_to_string(&p).unwrap(),
        ));
    }
    files.push(("board.cohdl".to_string(), board_src.to_string()));
    let mut checked = check_files(&files, None).expect("design selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

// ---------------------------------------------------------------------------
// Criterion: "Grammar parses every construct listed in 'In scope', on at
// least the demo board's actual source."

#[test]
fn grammar_parses_demo_board_and_std() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let (proj, dep_names) = load_example(&root.join("examples/rpi-pico2"));
    let checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )
    .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    assert!(checked.ir.is_some());
}

#[test]
fn rpi_pico2_uses_reusable_catalog_packages_and_canonical_chip_pins() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let (proj, dep_names) = load_example(&root.join("examples/rpi-pico2"));
    let checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )
    .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    for part in [
        "raspberrypi_mcu::RP2350A_QFN60",
        "richtek_dcdc::buck_boost::rt6150b::BUCKBOOST_RT6150B",
        "flash::FLASH_W25Q32",
        "osc::XTAL_12MHZ",
        "usb::connectors::micro_b::USB_MICRO_B",
        "connectors::headers::castellated_254::HEADER_SWD_3W",
        "diode::SCHOTTKY_PMEG6010",
        "mosfet::FET_DMG1012T",
        "passive::IND_2U2",
        "passive::IND_3U3",
        "led::LED_GREEN",
    ] {
        assert!(
            checked.world.parts.contains_key(part),
            "missing extracted part `{part}`"
        );
    }
    for stale in [
        "rpi_pico2::RP2350A_QFN60",
        "rpi_pico2::BUCKBOOST_RT6150B",
        "rpi_pico2::FLASH_W25Q32",
        "rpi_pico2::XTAL_12MHZ",
        "rpi_pico2::USB_MICRO_B",
        "rpi_pico2::HEADER_SWD_3W",
        "rpi_pico2::SCHOTTKY_PMEG6010",
        "rpi_pico2::FET_DMG1012T",
        "rpi_pico2::IND_2U2",
        "rpi_pico2::IND_3U3",
        "rpi_pico2::LED_GREEN",
    ] {
        assert!(
            !checked.world.parts.contains_key(stale),
            "`{stale}` must not be reintroduced locally"
        );
    }

    let mcu = checked
        .world
        .devices
        .get("raspberrypi_mcu::RP2350A")
        .unwrap();
    let pins = mcu.pins_for(None);
    for (name, number) in [
        ("GPIO0", "2"),
        ("GPIO23", "35"),
        ("GPIO24", "36"),
        ("GPIO25", "37"),
        ("GPIO29", "43"),
        ("USB_DM", "51"),
        ("USB_DP", "52"),
        ("QSPI_SS", "60"),
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

    let flash = checked.world.devices.get("flash::W25Q32").unwrap();
    let flash_pins = flash.pins_for(None);
    for board_alias in ["SD0", "SD1", "SD2", "SD3", "SCLK"] {
        assert!(
            flash_pins.iter().all(|pin| pin.name.name != board_alias),
            "board alias `{board_alias}` leaked into the flash library"
        );
    }
}

// A larger, real board: the Raspberry Pi Pico 2 (RP2350A). Exercises the
// compiler at scale (≈50 instances, ≈60 nets) and the post-MVP RFC features it
// uses — package variants (RFC-008), `#[intent]` (RFC-012), and a `layout {}`
// block with `#[placement_hint]` (RFC-013) — all the way to a clean build.
#[test]
fn rpi_pico2_example_builds_cleanly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let (proj, dep_names) = load_example(&root.join("examples/rpi-pico2"));
    // The package-aware entry (the CLI's own path): project-local footprints
    // carry the real package name (`rpi_pico2::…`), not the compat `main::…`.
    let mut checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )
    .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "pico2 should check cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    let artifacts =
        build_artifacts(&mut checked, &LockState::default()).expect("pico2 should build");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    // A real netlist, and the RFC-013 layout artifact (it has a layout {} block).
    assert!(
        artifacts.netlist.contains("(export"),
        "expected a KiCad netlist"
    );
    let layout = artifacts.layout.expect("pico2 declares layout constraints");
    assert!(
        layout.contains("\"diff_pairs\""),
        "USB diff pair in layout.json"
    );
}

// A mixed-signal, multi-rail reference board: native USB, stereo microphone
// capture, I2S playback, a translated 1.8 V IMU domain, and eight servo PWM
// outputs. Keep its protected power path, safe keyed harnesses, and exact
// memory variant buildable.
#[test]
#[ignore = "examples/robot-dog-mainboard is not in the repository yet; remove this attribute when it lands"]
fn robot_dog_mainboard_builds_cleanly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("examples/robot-dog-mainboard");
    let (proj, dep_names) = load_example(&dir);
    let mut checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )
    .unwrap();
    assert!(
        !checked.diags.has_errors(),
        "robot-dog mainboard should check cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );

    // Lock the safety-critical connector and control topology, not only the
    // human-readable net names. Molex 4-circuit BEC cannot mate with a leg;
    // each 6-circuit leg dedicates one complete row to each servo.
    {
        let ir = checked.ir.as_ref().unwrap();
        let members = |name: &str| {
            &ir.nets
                .iter()
                .find(|net| net.name == name)
                .unwrap_or_else(|| panic!("missing robot-dog net `{name}`"))
                .members
        };
        for (net, path, pin) in [
            ("BEC_5V_RAW", "RobotDogMainboard::bec_input", "P1"),
            ("BEC_5V_RAW", "RobotDogMainboard::bec_input", "P2"),
            ("BEC_5V_RAW", "RobotDogMainboard::actuator_switch", "IN"),
            ("GND", "RobotDogMainboard::bec_input", "P3"),
            ("GND", "RobotDogMainboard::bec_input", "P4"),
            (
                "ACTUATOR_AUDIO_5V",
                "RobotDogMainboard::actuator_switch",
                "OUT",
            ),
            (
                "ACTUATOR_AUDIO_5V",
                "RobotDogMainboard::leg_front_left",
                "P2",
            ),
            (
                "ACTUATOR_AUDIO_5V",
                "RobotDogMainboard::leg_front_left",
                "P5",
            ),
            ("GND", "RobotDogMainboard::leg_front_left", "P3"),
            ("GND", "RobotDogMainboard::leg_front_left", "P6"),
            ("SERVO_FL_HIP", "RobotDogMainboard::leg_front_left", "P1"),
            ("SERVO_FL_KNEE", "RobotDogMainboard::leg_front_left", "P4"),
            ("EFUSE_ENABLE_MCU", "RobotDogMainboard::mcu", "IO48"),
            ("EFUSE_POWER_GOOD", "RobotDogMainboard::mcu", "IO41"),
            ("EFUSE_CURRENT_MONITOR", "RobotDogMainboard::mcu", "IO1"),
            ("USB_VBUS_SENSE", "RobotDogMainboard::mcu", "IO2"),
            ("SERVO_LEVEL_OEN", "RobotDogMainboard::mcu", "IO47"),
            ("IMU_LEVEL_OE", "RobotDogMainboard::mcu", "IO42"),
            ("AMP_I2S_BCLK", "RobotDogMainboard::amp_level", "B1"),
        ] {
            assert!(
                members(net).contains(&(path.to_string(), pin.to_string())),
                "`{net}` must contain `{path}.{pin}`"
            );
        }
    }

    let lock_text = std::fs::read_to_string(dir.join("design.lock")).unwrap();
    let prior = LockState::parse(&lock_text).unwrap();
    let artifacts =
        build_artifacts(&mut checked, &prior).expect("robot-dog mainboard should build");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    assert_eq!(artifacts.lock.render(), lock_text);
    assert!(
        artifacts.bom.contains("ESP32-S3-WROOM-1-N8R2"),
        "the exact 8 MB flash + 2 MB PSRAM module must stay bound"
    );
    for mpn in [
        "TPS259823ONRGET",
        "SN74LVC8T245PWR",
        "43045-0212",
        "43045-0412",
        "43045-0612",
    ] {
        assert!(
            artifacts.bom.contains(mpn),
            "missing robot-dog safety-critical BOM item `{mpn}`"
        );
    }
    for net in [
        "BEC_5V_RAW",
        "ACTUATOR_AUDIO_5V",
        "V5_LOGIC",
        "USB_VBUS_SENSE",
        "EFUSE_ENABLE",
        "EFUSE_POWER_GOOD",
        "EFUSE_CURRENT_MONITOR",
        "I2S_MIC_SD",
        "AMP_I2S_BCLK",
        "AMP_SD_MODE",
        "IMU_LEVEL_OE",
        "SERVO_LEVEL_OEN",
        "MCU_BOOT",
        "SERVO_RR_KNEE",
    ] {
        assert!(
            artifacts.netlist.contains(net),
            "missing robot-dog interface/power net `{net}`"
        );
    }
    let layout = artifacts
        .layout
        .expect("robot-dog mainboard declares layout constraints");
    assert!(layout.contains("USBC_DP") && layout.contains("USBC_DM"));
    assert!(layout.contains("BEC_5V_RAW"));
    assert!(layout.contains("ACTUATOR_AUDIO_5V"));
    assert!(!artifacts.netlist.contains("VSERVO_V5_AUDIO"));
}

// ---------------------------------------------------------------------------
// Criterion: "Unit-type checking fires correctly: a fixture with a
// deliberately wrong-unit spec produces the correct diagnostic, naming the
// expected vs. actual unit type."

#[test]
fn wrong_unit_spec_names_expected_and_actual() {
    // Wrong unit at a generic instantiation site.
    let (_, rendered) = check(
        r#"
pub device MLCC<C: Capacitance, V: Voltage = 10V> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V }
}
design B {
    inst c1: MLCC<3.3V, 16V>
    net X: c1.A, c1.B
}
"#,
    );
    // RFC-011: unit-mismatch at a generic site is an E1xx (unit) diagnostic,
    // E112 — not the retired E402 (which lived in the E4xx generics block).
    assert!(rendered.contains("E112"), "{}", rendered);
    assert!(
        rendered.contains("expected `Capacitance`, found `Voltage`"),
        "{}",
        rendered
    );

    // Wrong unit between a trait requirement and a device spec field.
    let (_, rendered) = check(
        r#"
pub trait Capacitor {
    spec { capacitance: Capacitance }
}
pub device Weird {
    pins { A: 1 [passive] }
    spec { capacitance: 10V }
}
impl Capacitor for Weird {}
"#,
    );
    assert!(rendered.contains("E301"), "{}", rendered);
    assert!(
        rendered.contains("must be `Capacitance`") && rendered.contains("`Voltage`"),
        "{}",
        rendered
    );
}

#[test]
fn bare_number_where_unit_expected() {
    let (_, rendered) = check(
        r#"
pub device MLCC<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
design B {
    inst c1: MLCC<100>
    net X: c1.A, c1.B
}
"#,
    );
    // RFC-011: bare number at a generic site is E113 (retired E404).
    assert!(rendered.contains("E113"), "{}", rendered);
    assert!(
        rendered.contains("100nF") || rendered.contains("write the unit"),
        "{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Criterion: "Pin exhaustiveness fires correctly: a fixture with an
// unresolved required pin produces the correct diagnostic; a fixture with a
// pin in both net and nc produces the contradictory-declaration diagnostic."

#[test]
fn unresolved_required_pin_diagnostic() {
    let (_, rendered) = check(
        r#"
pub device MCU {
    pins { required VDD: 1 [passive], required GND: 2 [passive], optional TP: 3 [passive] }
}
pub device R2 { pins { A: 1 [passive], B: 2 [passive] } }
design B {
    inst mcu: MCU
    inst r: R2
    net VDD: mcu.VDD, r.A
    net X: r.B, mcu.GND
}
"#,
    );
    // TP is optional — must NOT fire. VDD/GND connected. Everything clean.
    assert!(!rendered.contains("E701"), "{}", rendered);

    let (_, rendered) = check(
        r#"
pub device MCU {
    pins { required VDD: 1 [passive], required GND: 2 [passive] }
}
pub device R2 { pins { A: 1 [passive], B: 2 [passive] } }
design B {
    inst mcu: MCU
    inst r: R2
    net VDD: mcu.VDD, r.A, r.B
}
"#,
    );
    assert!(rendered.contains("E701"), "{}", rendered);
    assert!(
        rendered.contains("`B::mcu.GND` is unresolved"),
        "{}",
        rendered
    );
    assert!(
        rendered.contains("add it to a `net` or explicitly mark it `nc`"),
        "{}",
        rendered
    );
}

#[test]
fn contradictory_net_and_nc_diagnostic() {
    let (_, rendered) = check(
        r#"
pub device MCU {
    pins { required VDD: 1 [passive], required GND: 2 [passive] }
}
pub device R2 { pins { A: 1 [passive], B: 2 [passive] } }
design B {
    inst mcu: MCU
    inst r: R2
    net VDD: mcu.VDD, r.A
    net G: mcu.GND, r.B
    nc: mcu.GND
}
"#,
    );
    assert!(rendered.contains("E702"), "{}", rendered);
    assert!(rendered.contains("contradictory"), "{}", rendered);
    assert!(
        rendered.contains("cannot be both connected and explicitly not-connected"),
        "{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Criterion: "Trait satisfaction fires correctly: a fixture with a device
// missing a required pin/spec for its claimed impl produces the correct
// diagnostic naming the trait and the gap; a fixture with a missing
// sub-trait-bound impl produces the correct chain diagnostic."

#[test]
fn impl_missing_pin_and_spec_names_trait_and_gap() {
    let (_, rendered) = check(
        r#"
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub device OnePin {
    pins { A: 1 [passive] }
}
impl TwoTerminal for OnePin {}
"#,
    );
    assert!(rendered.contains("E301"), "{}", rendered);
    assert!(
        rendered.contains("impl `TwoTerminal` for `OnePin`"),
        "{}",
        rendered
    );
    assert!(rendered.contains("requires pin role `B`"), "{}", rendered);

    let (_, rendered) = check(
        r#"
pub trait Capacitor {
    spec { capacitance: Capacitance, voltage_rating: Voltage }
}
pub device NoRating<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
impl Capacitor for NoRating {}
"#,
    );
    assert!(rendered.contains("E301"), "{}", rendered);
    assert!(
        rendered.contains("requires spec field `voltage_rating` (`Voltage`)"),
        "{}",
        rendered
    );
}

#[test]
fn missing_subtrait_impl_chain_diagnostic() {
    let (_, rendered) = check(
        r#"
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub trait Capacitor: TwoTerminal {
    spec { capacitance: Capacitance }
}
pub device Cap<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
impl Capacitor for Cap {}
"#,
    );
    assert!(rendered.contains("E302"), "{}", rendered);
    assert!(
        rendered.contains("impl `Capacitor` for `Cap` requires `impl TwoTerminal for Cap`, which was not found in scope"),
        "{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Criterion: "Generic trait-bound checking fires correctly: a fixture
// instantiating a generic with a type argument lacking the required impl
// produces the correct diagnostic."

#[test]
fn generic_trait_bound_violation_diagnostic() {
    let (_, rendered) = check(
        r#"
pub trait Capacitor { spec { capacitance: Capacitance } }
pub device Resistor<R: Resistance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R }
}
pub device MCU { pins { required VDD: 1 [passive] } }
fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A
}
design B {
    inst mcu: MCU
    inst r1: Resistor<10kohm>
    add_decoupling::<Resistor>(r1, mcu.VDD)
    net X: r1.A, r1.B
}
"#,
    );
    assert!(rendered.contains("E403"), "{}", rendered);
    assert!(
        rendered.contains("`Resistor` does not implement `Capacitor`"),
        "{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Criterion: "Nested fn calls expand correctly to at least 2 levels in a
// fixture, with correct substitution threading and no naming collisions; a
// cyclic-call fixture produces the cycle diagnostic."

#[test]
fn nested_fn_three_levels_substitution_threading() {
    let (checked, rendered) = check(
        r#"
pub device MLCC<C: Capacitance, V: Voltage> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V }
}
pub device MCU { pins { required VDD: 1 [passive], required GND: 2 [passive] } }

fn level3<V: Voltage>(p: Pin, g: Pin) {
    inst c: MLCC<100nF, V>
    net _: p, c.A
    net _: g, c.B
}
fn level2<V: Voltage>(p: Pin, g: Pin) {
    level3::<V>(p, g)
}
fn level1<V: Voltage>(p: Pin, g: Pin) {
    level2::<V>(p, g)
}
design B {
    inst mcu: MCU
    level1::<3.3V>(mcu.VDD, mcu.GND)
    level1::<5V>(mcu.VDD, mcu.GND)
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.unwrap();
    let caps: Vec<_> = ir
        .instances
        .values()
        .filter(|i| cohdl::resolve::short(&i.device) == "MLCC")
        .collect();
    assert_eq!(caps.len(), 2, "two call chains → two instances");
    let mut voltages: Vec<&str> = caps
        .iter()
        .map(|i| i.specs["voltage_rating"].text.as_str())
        .collect();
    voltages.sort();
    // Substitution threaded through three levels, per chain.
    assert_eq!(voltages, ["3.3V", "5V"]);
    // Full call-chain naming, no collisions.
    let paths: Vec<&String> = ir.instances.keys().collect();
    assert_eq!(
        paths.len(),
        ir.instances.len(),
        "paths unique by construction: {paths:?}"
    );
    assert!(
        caps.iter().all(|c| c.path.matches("::__fn").count() == 3),
        "3 chain segments each: {paths:?}"
    );
}

#[test]
fn cyclic_call_full_chain_diagnostic() {
    let (_, rendered) = check(
        r#"
pub device MCU { pins { required VDD: 1 [passive] } }
fn a(p: Pin) { b(p) }
fn b(p: Pin) { c(p) }
fn c(p: Pin) { a(p) }
design B {
    inst mcu: MCU
    a(mcu.VDD)
}
"#,
    );
    assert!(rendered.contains("E501"), "{}", rendered);
    assert!(rendered.contains("a → b → c → a"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Criterion: "The designator allocator fixture test confirms no collisions
// across at least one fixture with multiple same-prefix instances (the
// esd/ldo33-style case)."

#[test]
fn designator_same_prefix_no_collision() {
    // Two devices with no prefix-mapped trait — both default to "U", the
    // exact v1 esd/ldo33 collision class.
    let (mut checked, rendered) = check(
        r#"
pub device ESD_USB { pins { required VCC: 1 [passive] } }
pub device AP2112K { pins { required VOUT: 1 [passive] } }
design B {
    inst esd: ESD_USB
    inst ldo33: AP2112K
    net X: esd.VCC, ldo33.VOUT
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let mut diags = Diagnostics::new();
    let ir = checked.ir.as_mut().unwrap();
    let lock = assign_designators(&checked.world, ir, &LockState::default(), &mut diags);
    assert!(!diags.has_errors());
    let d_esd = &lock.designators["B::esd"];
    let d_ldo = &lock.designators["B::ldo33"];
    assert_ne!(d_esd, d_ldo, "same-prefix instances must not collide");
    assert_eq!(d_esd, "U1");
    assert_eq!(d_ldo, "U2");
}

#[test]
fn designator_stability_tombstones_and_overrides() {
    let src_v1 = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst first: D1
    inst second: D1
    net X: first.P, second.P
}
"#;
    let (mut checked, _) = check(src_v1);
    let mut diags = Diagnostics::new();
    let lock1 = assign_designators(
        &checked.world,
        checked.ir.as_mut().unwrap(),
        &LockState::default(),
        &mut diags,
    );
    assert_eq!(lock1.designators["B::first"], "U1");
    assert_eq!(lock1.designators["B::second"], "U2");

    // Remove `first`, add `third`: `second` keeps U2 (stability), `first` is
    // tombstoned, and U1 is never reused — `third` gets U3.
    let src_v2 = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst second: D1
    inst third: D1
    net X: second.P, third.P
}
"#;
    let (mut checked, _) = check(src_v2);
    let mut diags = Diagnostics::new();
    let lock2 = assign_designators(
        &checked.world,
        checked.ir.as_mut().unwrap(),
        &lock1,
        &mut diags,
    );
    assert_eq!(
        lock2.designators["B::second"], "U2",
        "stability across rebuilds"
    );
    assert_eq!(lock2.tombstones["B::first"], "U1", "removed → tombstoned");
    assert_eq!(
        lock2.designators["B::third"], "U3",
        "tombstoned designator never reused"
    );

    // Explicit override wins and collision with it is detected.
    let src_v3 = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    #[designator("U2")]
    inst fourth: D1
    inst second: D1
    net X: second.P, fourth.P
}
"#;
    let (mut checked, _) = check(src_v3);
    let mut diags = Diagnostics::new();
    let _ = assign_designators(
        &checked.world,
        checked.ir.as_mut().unwrap(),
        &lock2,
        &mut diags,
    );
    // `second` holds U2 from the lock; the override collides → E803.
    let rendered = {
        diags.sort(&checked.sm);
        diags.render(&checked.sm)
    };
    assert!(rendered.contains("E803"), "{}", rendered);
}

#[test]
fn designator_assignment_is_order_independent() {
    // RFC-005 Tooling & operations: same live set in a different collection
    // order produces the identical assignment. Instance paths are the same
    // set regardless of source order, so assignments must match exactly.
    let forward = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst alpha: D1
    inst beta: D1
    inst gamma: D1
    net X: alpha.P, beta.P, gamma.P
}
"#;
    let backward = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst gamma: D1
    inst beta: D1
    inst alpha: D1
    net X: alpha.P, beta.P, gamma.P
}
"#;
    let mut locks = Vec::new();
    for src in [forward, backward] {
        let (mut checked, _) = check(src);
        let mut diags = Diagnostics::new();
        locks.push(assign_designators(
            &checked.world,
            checked.ir.as_mut().unwrap(),
            &LockState::default(),
            &mut diags,
        ));
    }
    assert_eq!(locks[0], locks[1], "collection order must not matter");
}

// ---------------------------------------------------------------------------
// Criterion: "The residual DRC engine's 4 rules each fire on a dedicated
// fixture designed to trigger them."

#[test]
fn drc_d001_voltage_exceed() {
    let (_, rendered) = check(
        r#"
pub device MLCC<C: Capacitance, V: Voltage> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V }
}
pub device Source { pins { required OUT: 1 [power_out], required GND: 2 } }
design B {
    inst src: Source
    inst c1: MLCC<100nF, 3V>
    net VBUS [5V]: src.OUT, c1.A
    net GND [gnd]: src.GND, c1.B
}
"#,
    );
    assert!(rendered.contains("D001"), "{}", rendered);
    assert!(rendered.contains("rated `3V`"), "{}", rendered);
    assert!(rendered.contains("annotated `5V`"), "{}", rendered);
}

#[test]
fn drc_d002_polarity_mismatch() {
    let (_, rendered) = check(
        r#"
pub trait Polarized {
    pins { required Anode: pin, required Cathode: pin }
}
pub device TantalumCap {
    pins { Anode: 1 [passive], Cathode: 2 [passive] }
}
impl Polarized for TantalumCap {}
pub device Source { pins { required OUT: 1 [power_out], required GND: 2 } }
design B {
    inst src: Source
    inst c1: TantalumCap
    net V: src.OUT, c1.Cathode
    net GND [gnd]: src.GND, c1.Anode
}
"#,
    );
    assert!(rendered.contains("D002"), "{}", rendered);
    assert!(rendered.contains("anode pin `B::c1.Anode`"), "{}", rendered);
}

#[test]
fn drc_d003_single_driver() {
    let (_, rendered) = check(
        r#"
pub device MCU { pins { required TX: 1 [output], required GND: 2 } }
pub device R2 { pins { A: 1 [passive], B: 2 [passive] } }
design B {
    inst mcu: MCU
    inst r: R2
    net LONELY: mcu.TX
    net GND: mcu.GND, r.A, r.B
}
"#,
    );
    assert!(rendered.contains("D003"), "{}", rendered);
    assert!(rendered.contains("only one connected pin"), "{}", rendered);
    assert!(rendered.contains("warning"), "{}", rendered);
}

#[test]
fn drc_d004_multi_driver() {
    let (_, rendered) = check(
        r#"
pub device MCU { pins { required TX: 1 [output], required GND: 2 } }
design B {
    inst a: MCU
    inst b: MCU
    net BUS: a.TX, b.TX
    net GND: a.GND, b.GND
}
"#,
    );
    assert!(rendered.contains("D004"), "{}", rendered);
    assert!(rendered.contains("2 driver-type pins"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Build artifacts: netlist + BOM byte-stability against the committed golden
// files (reproducibility hard constraint).

#[test]
fn example_build_matches_committed_golden_output() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["rpi-pico2", "sf32-miniboard"] {
        let dir = root.join("examples").join(name);
        let (proj, dep_names) = load_example(&dir);
        // The package-aware entry (the CLI's own path): project-local
        // footprints carry the real package name (`rpi_pico2::…`), not the
        // compat `main::…`.
        let mut checked = cohdl::pipeline::check_files_in_with_deps(
            &proj.name,
            &dep_names,
            &proj.files,
            proj.top.as_deref(),
        )
        .unwrap();
        assert!(!checked.diags.has_errors());
        let lock_text = std::fs::read_to_string(dir.join("design.lock")).unwrap();
        let prior = LockState::parse(&lock_text).unwrap();
        let artifacts = build_artifacts(&mut checked, &prior).expect("build succeeds");
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );

        let golden_net =
            std::fs::read_to_string(dir.join("out").join(format!("{}.net", name))).unwrap();
        let golden_bom =
            std::fs::read_to_string(dir.join("out").join(format!("{}-bom.csv", name))).unwrap();
        assert_eq!(
            artifacts.netlist, golden_net,
            "[{}] netlist bytes must be stable",
            name
        );
        assert_eq!(
            artifacts.bom, golden_bom,
            "[{}] BOM bytes must be stable",
            name
        );
        assert_eq!(
            artifacts.lock.render(),
            lock_text,
            "[{}] lock must be stable",
            name
        );

        // Every instance carries a real MPN in the BOM (no <UNSPECIFIED>,
        // ever) and every cell is filled.
        assert!(
            !artifacts.bom.contains("\"\""),
            "[{}] no empty cells:\n{}",
            name,
            artifacts.bom
        );
    }
}

// ---------------------------------------------------------------------------
// Part binding: an unbound instance is an E801 build error (the BOM must
// not lie), while `check` alone passes.

#[test]
fn unbound_instance_is_a_build_error() {
    let (mut checked, rendered) = check_with_std(
        r#"
pub device Chip<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}

design B {
    inst c1: Chip<47nF>
    inst r1: Chip<47nF>
    net X: c1.A, r1.A
    net Y: c1.B, r1.B
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    assert!(artifacts.is_none());
    let rendered = checked.diags.render(&checked.sm);
    assert!(rendered.contains("E801"), "{}", rendered);
    assert!(rendered.contains("no part binding"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Compliance-audit regressions (confirmed deviations from the RFCs, fixed):
// see docs/compliance-report.md.

#[test]
fn net_annotation_must_be_voltage_typed() {
    // RFC-001 comparison discipline: the D001 comparison input must be
    // Voltage-typed, so `[100nF]` in the annotation slot is a unit-type error.
    let (_, rendered) = check(
        r#"
pub device R2 { pins { A: 1 [passive], B: 2 [passive] } }
design B {
    inst r: R2
    net X [100nF]: r.A, r.B
}
"#,
    );
    assert!(rendered.contains("E110"), "{}", rendered);
    assert!(
        rendered.contains("expected `Voltage`, found `Capacitance`"),
        "{}",
        rendered
    );
}

#[test]
fn drc_d001_never_compares_across_unit_types() {
    // A device whose `voltage_rating` field is (bizarrely but legally)
    // non-Voltage must not be magnitude-compared against a Voltage net
    // annotation.
    let (_, rendered) = check(
        r#"
pub device Weird<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { voltage_rating: C }
}
pub device Source { pins { required OUT: 1 [power_out], required GND: 2 } }
design B {
    inst src: Source
    inst w: Weird<100nF>
    net V [5V]: src.OUT, w.A
    net G [gnd]: src.GND, w.B
}
"#,
    );
    assert!(!rendered.contains("D001"), "{}", rendered);
}

#[test]
fn overridden_paths_prior_designator_stays_reserved() {
    // RFC-005 Step 3: the reserved set is built from ALL prior assignments —
    // a prior number shadowed by an override on the same path is never handed
    // to a fresh instance.
    let src_v1 = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst a: D1
    net X: a.P, a.P
}
"#;
    let (mut checked, _) = check(src_v1);
    let mut diags = Diagnostics::new();
    let lock1 = assign_designators(
        &checked.world,
        checked.ir.as_mut().unwrap(),
        &LockState::default(),
        &mut diags,
    );
    assert_eq!(lock1.designators["B::a"], "U1");

    let src_v2 = r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    #[designator("U9")]
    inst a: D1
    inst b: D1
    net X: a.P, b.P
}
"#;
    let (mut checked, _) = check(src_v2);
    let mut diags = Diagnostics::new();
    let lock2 = assign_designators(
        &checked.world,
        checked.ir.as_mut().unwrap(),
        &lock1,
        &mut diags,
    );
    assert_eq!(lock2.designators["B::a"], "U9", "override wins");
    assert_eq!(
        lock2.designators["B::b"], "U2",
        "a's prior U1 stays reserved for this run — fresh b must not take it"
    );
}

#[test]
fn dunder_names_are_reserved() {
    let (_, rendered) = check(
        r#"
pub device D1 { pins { required P: 1 [passive] } }
design B {
    inst __fn0_x: D1
    net __net0: __fn0_x.P, __fn0_x.P
}
"#,
    );
    assert!(rendered.contains("E206"), "{}", rendered);
    assert!(
        rendered.contains("reserved for compiler-generated"),
        "{}",
        rendered
    );
}

#[test]
fn ambiguous_part_binding_is_deterministic_and_noted() {
    let (mut checked, rendered) = check(
        r#"
pub device MLCC<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
pub footprint TFP {}
pub part ZPart: MLCC<100nF> {
    primary { mpn: "Z-1", footprint: TFP }
}
pub part APart: MLCC<100nF> {
    primary { mpn: "A-1", footprint: TFP }
}
design B {
    inst c: MLCC<100nF>
    net X: c.A, c.B
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let artifacts = build_artifacts(&mut checked, &LockState::default()).unwrap();
    assert!(
        artifacts.bom.contains("A-1") && !artifacts.bom.contains("Z-1"),
        "lexicographically-smallest part name wins:\n{}",
        artifacts.bom
    );
    assert_eq!(artifacts.notes.len(), 1);
    assert!(
        artifacts.notes[0].contains("APart, ZPart"),
        "{}",
        artifacts.notes[0]
    );
}

// ---------------------------------------------------------------------------
// RFC-008: exhaustive pattern-matching over structural variants.

#[test]
fn rfc008_missing_pin_role_is_e901_listing_roles() {
    let (_, rendered) = check(
        r#"
pub device Bare { pins { required VDD: 1 } }
"#,
    );
    assert!(rendered.contains("E901"), "{}", rendered);
    assert!(
        rendered.contains("pin `VDD` has no role annotation"),
        "{}",
        rendered
    );
    for role in [
        "[input]",
        "[output]",
        "[bidirectional]",
        "[passive]",
        "[power_in]",
        "[power_out]",
    ] {
        assert!(
            rendered.contains(role),
            "missing {role} in help: {rendered}"
        );
    }
}

#[test]
fn rfc008_variant_without_pins_block_is_e902_naming_it() {
    let (_, rendered) = check(
        r#"
pub device V3<C: Capacitance> {
    variants { C0402, C0603, C0805 }
    pins[C0402] { A: 1 [passive], B: 2 [passive] }
    pins[C0603] { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
"#,
    );
    assert!(rendered.contains("E902"), "{}", rendered);
    assert!(
        rendered.contains("variant `C0805` of device `V3` has no `pins[C0805]` block"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_undeclared_variant_selected_is_e903_with_valid_set() {
    let (_, rendered) = check(
        r#"
pub device V2 {
    variants { A1, A2 }
    pins[A1] { P: 1 [passive] }
    pins[A2] { P: 1 [passive] }
}
design B {
    inst x: V2[A9]
    net N: x.P, x.P
}
"#,
    );
    assert!(rendered.contains("E903"), "{}", rendered);
    assert!(rendered.contains("no variant named `A9`"), "{}", rendered);
    assert!(
        rendered.contains("valid variants are: A1, A2"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_omitted_selector_is_e904_no_implicit_default() {
    let (_, rendered) = check(
        r#"
pub device V2 {
    variants { A1, A2 }
    pins[A1] { P: 1 [passive] }
    pins[A2] { P: 1 [passive] }
}
design B {
    inst x: V2
    net N: x.P, x.P
}
"#,
    );
    assert!(rendered.contains("E904"), "{}", rendered);
    assert!(rendered.contains("no implicit default"), "{}", rendered);
    assert!(
        rendered.contains("valid variants are: A1, A2"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_selector_on_plain_device_is_e905() {
    let (_, rendered) = check(
        r#"
pub device Plain { pins { P: 1 [passive] } }
design B {
    inst x: Plain[C0402]
    net N: x.P, x.P
}
"#,
    );
    assert!(rendered.contains("E905"), "{}", rendered);
    assert!(
        rendered.contains("has no `variants { }` block"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_duplicate_variant_is_e906() {
    let (_, rendered) = check(
        r#"
pub device V {
    variants { C0402, C0402 }
    pins[C0402] { P: 1 [passive] }
}
"#,
    );
    assert!(rendered.contains("E906"), "{}", rendered);
    assert!(
        rendered.contains("duplicate variant `C0402`"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_undeclared_qualifier_is_e907() {
    let (_, rendered) = check(
        r#"
pub device V {
    variants { C0402 }
    pins[C0402] { P: 1 [passive] }
    spec[C9999] { }
}
"#,
    );
    assert!(rendered.contains("E907"), "{}", rendered);
    assert!(
        rendered.contains("no variant named `C9999`"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_bare_pins_on_variant_device_is_e908() {
    let (_, rendered) = check(
        r#"
pub device V {
    variants { C0402 }
    pins[C0402] { P: 1 [passive] }
    pins { Q: 2 [passive] }
}
"#,
    );
    assert!(rendered.contains("E908"), "{}", rendered);
    assert!(rendered.contains("must be qualified"), "{}", rendered);
}

#[test]
fn rfc008_variant_spec_merge_override_and_addition() {
    let (checked, rendered) = check(
        r#"
pub device V<C: Capacitance> {
    variants { SMALL, BIG }
    pins[SMALL] { A: 1 [passive], B: 2 [passive] }
    pins[BIG] { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: 10V }
    spec[BIG] { voltage_rating: 25V, max_ripple: 100mA }
}
design B {
    inst small: V<100nF>[SMALL]
    inst big: V<100nF>[BIG]
    net N: small.A, big.A
    net M: small.B, big.B
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.unwrap();
    let small = &ir.instances["B::small"];
    let big = &ir.instances["B::big"];
    // Base value for SMALL; override + addition for BIG (RFC-008 merge).
    assert_eq!(small.specs["voltage_rating"].text, "10V");
    assert!(!small.specs.contains_key("max_ripple"));
    assert_eq!(big.specs["voltage_rating"].text, "25V");
    assert_eq!(big.specs["max_ripple"].text, "100mA");
}

#[test]
fn rfc008_per_variant_pin_numbers_reach_the_netlist() {
    let (mut checked, rendered) = check(
        r#"
pub device Dual {
    variants { QFN, DIP }
    pins[QFN] { required SIG: 7 [passive], required GND: 8 [passive] }
    pins[DIP] { required SIG: 1 [passive], required GND: 2 [passive] }
}
pub footprint TFP {}
pub part DUAL_QFN: Dual[QFN] {
    primary { mpn: "D-QFN", footprint: TFP }
}
pub part DUAL_DIP: Dual[DIP] {
    primary { mpn: "D-DIP", footprint: TFP }
}
design B {
    inst q: DUAL_QFN
    inst d: DUAL_DIP
    net S: q.SIG, d.SIG
    net G: q.GND, d.GND
}
"#,
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let artifacts = build_artifacts(&mut checked, &LockState::default()).unwrap();
    // Same logical pin, different physical pads per variant.
    assert!(
        artifacts.netlist.contains("(pin \"7\")"),
        "{}",
        artifacts.netlist
    );
    assert!(
        artifacts.netlist.contains("(pin \"1\")"),
        "{}",
        artifacts.netlist
    );
    // Variant-aware part binding matched each instance to its own part.
    assert!(
        artifacts.bom.contains("D-QFN") && artifacts.bom.contains("D-DIP"),
        "{}",
        artifacts.bom
    );
}

#[test]
fn rfc008_impl_satisfaction_must_hold_for_every_variant() {
    let (_, rendered) = check(
        r#"
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub device Lopsided {
    variants { GOOD, BAD }
    pins[GOOD] { A: 1 [passive], B: 2 [passive] }
    pins[BAD] { A: 1 [passive] }
}
impl TwoTerminal for Lopsided {}
"#,
    );
    assert!(rendered.contains("E301"), "{}", rendered);
    assert!(rendered.contains("(variant `BAD`)"), "{}", rendered);
    assert!(rendered.contains("no pin with that name"), "{}", rendered);
}

#[test]
fn rfc008_part_must_select_variant() {
    let (_, rendered) = check(
        r#"
pub device V<C: Capacitance> {
    variants { C0402 }
    pins[C0402] { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
pub footprint TFP {}
pub part P1: V<100nF> {
    primary { mpn: "X", footprint: TFP }
}
"#,
    );
    assert!(rendered.contains("E904"), "{}", rendered);
    assert!(
        rendered.contains("part `P1` binds device `V`"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_selector_on_part_instantiation_is_rejected() {
    let (_, rendered) = check_with_std(
        r#"
pub device Chip {
    variants { C0402, C0603 }
    pins[C0402] { A: 1 [passive], B: 2 [passive] }
    pins[C0603] { A: 1 [passive], B: 2 [passive] }
}

pub footprint CHIP_0402 {}

pub part ChipPart: Chip[C0402] {
    primary { mfr: "m", mpn: "n", footprint: CHIP_0402 }
}

design B {
    inst c: ChipPart[C0603]
    net N: c.A, c.B
}
"#,
    );
    assert!(rendered.contains("E905"), "{}", rendered);
    assert!(
        rendered.contains("already selects its variant"),
        "{}",
        rendered
    );
}

#[test]
fn rfc008_wildcard_variant_arm_is_rejected() {
    let (_, rendered) = check(
        r#"
pub device V {
    variants { C0402, _ }
    pins[C0402] { P: 1 [passive] }
}
"#,
    );
    assert!(
        rendered.contains("no wildcard/catch-all arms"),
        "{}",
        rendered
    );
}

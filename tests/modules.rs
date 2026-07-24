//! RFC-016 module-system conformance.
//!
//! The mandatory regression (RFC-016 Gradeability): two packages sharing a
//! colliding unqualified name, confirming both resolve via their qualified
//! paths and that no unqualified ambiguity leaks through. Plus: the
//! load-bearing compatibility property (single-package projects unchanged),
//! `use` imports, file-tree module paths, E207/E208/E209, `pub`
//! enforcement across package boundaries, and the std prelude.

use cohdl::check::check_declarations_in;
use cohdl::diag::Diagnostics;
use cohdl::pipeline::{check_files, check_files_in};
use cohdl::resolve::ModuleInfo;
use cohdl::span::SourceMap;

/// Full pipeline over a project package (with module-bearing displays) —
/// the project is `pkg`; any `std/`-displayed file is the std package.
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

/// Declaration-stage world over ARBITRARY packages (the two-third-party-
/// package shape RFC-017 will make real; the pipeline's CLI surface only
/// loads project+std today, so this drives the resolver directly).
fn world_of(files: &[(&str, &str, &str, &str)]) -> (cohdl::resolve::World, String, SourceMap) {
    // (display, content, package, module)
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
    let rendered = diags.render(&sm);
    (world, rendered, sm)
}

const DEV: &str = "pub device Chip { pins { A: 1 [passive], B: 2 [passive] } }\n";

// ---------------------------------------------------------------------------
// The mandatory two-package collision regression.

#[test]
fn colliding_names_across_packages_resolve_via_qualified_paths() {
    // Two packages both declare `TPS62840`; a consumer reaches each through
    // its qualified path — no collision, no ambiguity leak.
    let (world, rendered, _sm) = world_of(&[
        (
            "sparkfun/src/power/buck.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "sparkfun",
            "sparkfun::power::buck",
        ),
        (
            "acme/src/power.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "acme",
            "acme::power",
        ),
        (
            "consumer/src/main.cohdl",
            "use sparkfun::power::buck::TPS62840;\n\
             pub device Local { pins { A: 1 [passive] } }\n",
            "consumer",
            "consumer",
        ),
    ]);
    assert!(!rendered.contains("error"), "{}", rendered);
    assert!(world
        .devices
        .contains_key("sparkfun::power::buck::TPS62840"));
    assert!(world.devices.contains_key("acme::power::TPS62840"));
    assert!(world.devices.contains_key("consumer::Local"));
}

#[test]
fn same_module_duplicate_is_still_an_error() {
    let (_world, rendered, _sm) = world_of(&[(
        "p/src/main.cohdl",
        "pub device D { pins { A: 1 [passive] } }\npub device D { pins { A: 1 [passive] } }\n",
        "p",
        "p",
    )]);
    assert!(rendered.contains("E201"), "{}", rendered);
    assert!(rendered.contains("in module `p`"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// The load-bearing compatibility property + std prelude.

#[test]
fn single_package_projects_are_unchanged() {
    // Multiple files, no use statements, names visible unqualified across
    // files — exactly today's ergonomics.
    let (checked, rendered) = check(
        "board",
        &[
            ("src/parts.cohdl", "pub device Res { pins { A: 1 [passive], B: 2 [passive] } }\npub footprint TFP {}\npub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n"),
            ("src/main.cohdl", "design B {\n    inst r1: R1\n    inst r2: R1\n    net N: r1.A, r2.A\n    net M: r1.B, r2.B\n}\n"),
        ],
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    assert!(checked.ir.is_some());
}

#[test]
fn std_prelude_stays_implicitly_visible_and_qualified_paths_reach_it() {
    // A project uses a std name unqualified (the prelude) AND via its
    // qualified `std::…` path.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(root.join("std/src")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|e| e == "cohdl") {
            files.push((
                format!("std/{}", p.file_name().unwrap().to_string_lossy()),
                std::fs::read_to_string(&p).unwrap(),
            ));
        }
    }
    files.sort();
    files.push((
        "src/main.cohdl".to_string(),
        "design B {\n    inst c1: MLCC_100nF_16V_0402\n    inst c2: std::MLCC_100nF_16V_0402\n    net N: c1.A, c2.A\n    net GND [gnd]: c1.B, c2.B\n}\n"
            .to_string(),
    ));
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.unwrap();
    assert_eq!(ir.instances.len(), 2);
    for inst in ir.instances.values() {
        assert_eq!(inst.part.as_deref(), Some("std::MLCC_100nF_16V_0402"));
    }
}

#[test]
fn project_name_shadows_std_prelude() {
    // A project declaring its own `MLCC` coexists with std's: the bare name
    // resolves to the project's (own package before prelude); `std::MLCC`
    // still reaches std's. No duplicate-declaration error (different
    // packages, different module paths).
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(root.join("std/src")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|e| e == "cohdl") {
            files.push((
                format!("std/{}", p.file_name().unwrap().to_string_lossy()),
                std::fs::read_to_string(&p).unwrap(),
            ));
        }
    }
    files.sort();
    files.push((
        "src/main.cohdl".to_string(),
        "pub device MLCC { pins { A: 1 [passive], B: 2 [passive] } }\n\
         pub footprint TFP {}\n\
         pub part M1: MLCC { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n\
         design B {\n    inst c: M1\n    net N: c.A\n    net G: c.B\n}\n"
            .to_string(),
    ));
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    assert!(
        !rendered.contains("E201"),
        "no cross-package dup:\n{}",
        rendered
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.unwrap();
    let inst = ir.instances.values().next().unwrap();
    assert_eq!(inst.device, "board::MLCC", "own package wins over prelude");
}

// ---------------------------------------------------------------------------
// Module paths mirror the file tree.

#[test]
fn file_tree_becomes_module_tree() {
    let (checked, rendered) = check(
        "pkg",
        &[
            (
                "src/power/buck.cohdl",
                "pub device Buck { pins { SW: 1 [output] } }\n",
            ),
            ("src/main.cohdl", DEV),
        ],
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    // Nested file → pkg::power::buck::Buck; root file → pkg::Chip.
    assert!(checked.world.devices.contains_key("pkg::power::buck::Buck"));
    assert!(checked.world.devices.contains_key("pkg::Chip"));
}

#[test]
fn qualified_and_imported_references_work_intra_package() {
    let (checked, rendered) = check(
        "pkg",
        &[
            (
                "src/parts/res.cohdl",
                "pub device Res { pins { A: 1 [passive], B: 2 [passive] } }\npub footprint TFP {}\npub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n",
            ),
            (
                "src/main.cohdl",
                "use pkg::parts::res::R1;\n\
                 design B {\n    inst a: R1\n    inst b: pkg::parts::res::R1\n    net N: a.A, b.A\n    net M: a.B, b.B\n}\n",
            ),
        ],
    );
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.unwrap();
    for inst in ir.instances.values() {
        assert_eq!(inst.part.as_deref(), Some("pkg::parts::res::R1"));
    }
}

// ---------------------------------------------------------------------------
// New diagnostics: E207 ambiguity, E208 use collision, E209 pub violation.

#[test]
fn intra_package_cross_module_collision_is_ambiguous_at_use_site() {
    // Same name in two modules of ONE package: legal declarations, but an
    // unqualified reference is ambiguous — E207 names both candidates.
    let (_checked, rendered) = check(
        "pkg",
        &[
            (
                "src/a/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/b/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/main.cohdl",
                "design B {\n    inst d: Dup\n    net N: d.A\n}\n",
            ),
        ],
    );
    assert!(rendered.contains("E207"), "{}", rendered);
    assert!(rendered.contains("pkg::a::x::Dup"), "{}", rendered);
    assert!(rendered.contains("pkg::b::x::Dup"), "{}", rendered);

    // Qualifying resolves it.
    let (_checked, rendered) = check(
        "pkg",
        &[
            (
                "src/a/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/b/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/main.cohdl",
                "design B {\n    inst d: pkg::a::x::Dup\n    net N: d.A\n}\n",
            ),
        ],
    );
    assert!(!rendered.contains("E207"), "{}", rendered);
}

#[test]
fn use_collision_is_e208_at_the_use_site() {
    let (_checked, rendered) = check(
        "pkg",
        &[
            (
                "src/a/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/b/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/main.cohdl",
                "use pkg::a::x::Dup;\nuse pkg::b::x::Dup;\n",
            ),
        ],
    );
    assert!(rendered.contains("E208"), "{}", rendered);
    assert!(rendered.contains("pkg::a::x::Dup"), "{}", rendered);
    // Identical re-import is NOT a collision (RFC: "from different paths").
    let (_checked, rendered) = check(
        "pkg",
        &[
            (
                "src/a/x.cohdl",
                "pub device Dup { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/main.cohdl",
                "use pkg::a::x::Dup;\nuse pkg::a::x::Dup;\n",
            ),
        ],
    );
    assert!(!rendered.contains("E208"), "{}", rendered);
}

#[test]
fn non_pub_cross_package_reference_is_e209() {
    let (_world, rendered, _sm) = world_of(&[
        (
            "lib/src/main.cohdl",
            "device Hidden { pins { A: 1 [passive] } }\npub device Shown { pins { A: 1 [passive] } }\n",
            "lib",
            "lib",
        ),
        (
            "app/src/main.cohdl",
            "use lib::Hidden;\n",
            "app",
            "app",
        ),
    ]);
    assert!(rendered.contains("E209"), "{}", rendered);
    assert!(rendered.contains("not `pub`"), "{}", rendered);

    // Qualified reference to a non-pub item: same violation.
    let (_world, rendered, _sm) = world_of(&[
        (
            "lib/src/main.cohdl",
            "device Hidden { pins { A: 1 [passive] } }\n",
            "lib",
            "lib",
        ),
        (
            "app/src/main.cohdl",
            "design B {\n    inst d: lib::Hidden\n    net N: d.A\n}\n",
            "app",
            "app",
        ),
    ]);
    assert!(rendered.contains("E209"), "{}", rendered);

    // Intra-package: pub is irrelevant (unchanged from today).
    let (_world, rendered, _sm) = world_of(&[(
        "lib/src/main.cohdl",
        "device Hidden { pins { A: 1 [passive] } }\ndesign B {\n    inst d: Hidden\n    net N: d.A\n}\n",
        "lib",
        "lib",
    )]);
    assert!(!rendered.contains("E209"), "{}", rendered);
}

#[test]
fn unresolved_use_path_suggests_closest_match() {
    let (_checked, rendered) = check(
        "pkg",
        &[
            (
                "src/a/x.cohdl",
                "pub device Thing { pins { A: 1 [passive] } }\n",
            ),
            ("src/main.cohdl", "use pkg::wrong::Thing;\n"),
        ],
    );
    assert!(rendered.contains("E202"), "{}", rendered);
    assert!(
        rendered.contains("use pkg::a::x::Thing"),
        "closest-match suggestion:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Grammar details.

#[test]
fn use_requires_semicolon_and_qualified_path() {
    let (_checked, rendered) = check("pkg", &[("src/main.cohdl", "use pkg::Chip\n")]);
    assert!(rendered.contains("E010"), "missing `;`:\n{}", rendered);

    let (_checked, rendered) = check("pkg", &[("src/main.cohdl", "use Chip;\n")]);
    assert!(
        rendered.contains("no package segment"),
        "single-segment use rejected:\n{}",
        rendered
    );

    let (_checked, rendered) = check(
        "pkg",
        &[(
            "src/main.cohdl",
            "pub use pkg::Chip;\npub device Chip { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("pub use"), "{}", rendered);
}

#[test]
fn compat_entry_defaults_to_main_package() {
    // The old check_files entry: bare display names land in package `main`,
    // and the flat-scope behavior holds.
    let files = vec![(
        "f.cohdl".to_string(),
        format!(
            "{}design B {{\n    inst c: Chip\n    net N: c.A\n    net M: c.B\n}}\n",
            DEV
        ),
    )];
    let mut checked = check_files(&files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    // No part declared -> build would fail, but checking is clean.
    assert!(!rendered.contains("error"), "{}", rendered);
    assert!(checked.world.devices.contains_key("main::Chip"));
}

// ---------------------------------------------------------------------------
// Adversarial-verification regressions (RFC-016 round 1).

// Finding (high): designator prefixes compared FULLY-QUALIFIED trait names,
// so a project trait could jump ahead of a std trait (bare `IC` < `Watchdog`
// but `board::Watchdog` < `std::IC`) — changing designators/netlist bytes
// for an existing project. "Smallest" now compares SHORT names.
#[test]
fn designator_prefix_ignores_module_paths() {
    let files: Vec<(String, String)> = vec![
        (
            "std/ic.cohdl".to_string(),
            "pub trait IC { designator_prefix: \"U\" pins { required VDD: pin } }\n".to_string(),
        ),
        (
            "src/main.cohdl".to_string(),
            "pub trait Watchdog { designator_prefix: \"WD\" pins { required VDD: pin } }\n\
             pub device Chip { pins { VDD: 1 [power_in], A: 2 [passive] } }\n\
             impl IC for Chip {}\n\
             impl Watchdog for Chip {}\n\
             pub footprint TFP {}\n\
             pub part P: Chip { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n\
             design B {\n    inst u1: P\n    net N: u1.VDD\n    net M: u1.A\n}\n"
                .to_string(),
        ),
    ];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let artifacts =
        cohdl::pipeline::build_artifacts(&mut checked, &cohdl::lock::LockState::default())
            .expect("build");
    // Flat-model order: `IC` < `Watchdog` → prefix `U`, exactly as before
    // RFC-016 (fq order would have picked `board::Watchdog` → `WD1`).
    assert!(
        artifacts.lock.render().contains("\"U1\""),
        "designator must not depend on module paths:\n{}",
        artifacts.lock.render()
    );
}

// Finding (medium): ambiguous part binding picked the smallest FQ path, so
// moving a part between modules changed which MPN was bought (and the note
// contradicted its own short-name display). Now compares short names.
#[test]
fn part_binding_order_ignores_module_paths() {
    let (checked_flat, r1) = check(
        "bind",
        &[(
            "src/main.cohdl",
            "pub device Res { pins { A: 1 [passive], B: 2 [passive] } spec { r: 10kohm } }\n\
             pub footprint TFP {}\n\
             pub part AAA_RES: Res { primary { mfr: \"Yageo\", mpn: \"AAA-10K\", footprint: TFP } }\n\
             pub part BBB_RES: Res { primary { mfr: \"Vishay\", mpn: \"BBB-10K\", footprint: TFP } }\n\
             design B {\n    inst r: Res\n    net N: r.A\n    net M: r.B\n}\n",
        )],
    );
    assert!(!r1.contains("error"), "{}", r1);
    let (checked_nested, r2) = check(
        "bind",
        &[
            (
                "src/zz/parts.cohdl",
                "pub part AAA_RES: Res { primary { mfr: \"Yageo\", mpn: \"AAA-10K\", footprint: TFP } }\n",
            ),
            (
                "src/main.cohdl",
                "pub device Res { pins { A: 1 [passive], B: 2 [passive] } spec { r: 10kohm } }\n\
                 pub footprint TFP {}\n\
                 pub part BBB_RES: Res { primary { mfr: \"Vishay\", mpn: \"BBB-10K\", footprint: TFP } }\n\
                 design B {\n    inst r: Res\n    net N: r.A\n    net M: r.B\n}\n",
            ),
        ],
    );
    assert!(!r2.contains("error"), "{}", r2);
    for mut checked in [checked_flat, checked_nested] {
        let artifacts =
            cohdl::pipeline::build_artifacts(&mut checked, &cohdl::lock::LockState::default())
                .expect("build");
        assert!(
            artifacts.bom.contains("AAA-10K"),
            "the short-name-smallest part wins regardless of module layout:\n{}",
            artifacts.bom
        );
    }
}

// Finding (medium): a project named `std` merged into the standard library's
// namespace (cascading errors with spans inside std/ files). Reserved now.
#[test]
fn std_package_name_is_reserved() {
    let tmp = std::env::temp_dir().join(format!("cohdl-mod-std-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(tmp.join("cohdl.toml"), "[package]\nname = \"std\"\n").unwrap();
    std::fs::write(tmp.join("src/main.cohdl"), DEV).unwrap();
    let err = cohdl::project::load_project(&tmp, None).expect_err("std name rejected");
    assert!(err.contains("reserved"), "{}", err);
    let _ = std::fs::remove_dir_all(&tmp);
}

// Finding (medium): unresolved qualified paths at REFERENCE sites carried no
// closest-match suggestion (only the use site did; World::suggest was dead
// code). Every unknown-name site now suggests.
#[test]
fn unknown_references_suggest_closest_match() {
    let (_checked, rendered) = check(
        "board",
        &[
            (
                "src/parts/lib.cohdl",
                "pub device Widget { pins { A: 1 [passive] } }\n",
            ),
            (
                "src/main.cohdl",
                "design B {\n    inst d: board::wrong::Widget\n    net N: d.A\n}\n",
            ),
        ],
    );
    assert!(rendered.contains("E202"), "{}", rendered);
    assert!(
        rendered.contains("did you mean `board::parts::lib::Widget`?"),
        "qualified reference miss must suggest:\n{}",
        rendered
    );
}

// Finding (low): a design and a declaration sharing a name in ONE package
// was silently legal (the flat model errored). It errors again, scoped to
// the package (std shadowing stays allowed).
#[test]
fn design_vs_declaration_collision_in_package_is_e201() {
    let (_checked, rendered) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Foo { pins { A: 1 [passive] } }\ndesign Foo {\n    inst d: Foo\n    net N: d.A\n}\n",
        )],
    );
    assert!(rendered.contains("E201"), "{}", rendered);
    assert!(rendered.contains("a design and a device"), "{}", rendered);
}

// Finding (low): a project trait named `Polarized` shaded the std one out of
// the D002 polarity check. Every short-named candidate is checked now.
#[test]
fn project_polarized_trait_does_not_shade_d002() {
    let files: Vec<(String, String)> = vec![
        (
            "std/pol.cohdl".to_string(),
            "pub trait Polarized { pins { required Anode: pin required Cathode: pin } }\n"
                .to_string(),
        ),
        (
            "src/main.cohdl".to_string(),
            "pub trait Polarized { pins { required Cathode: pin } }\n\
             pub device Cap { pins { Anode: 1 [passive], Cathode: 2 [passive] } }\n\
             impl std::Polarized for Cap {}\n\
             impl board::Polarized for Cap {}\n\
             pub footprint TFP {}\n\
             pub part CP: Cap { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n\
             design B {\n    inst c: CP\n    net GND [gnd]: c.Anode\n    net N: c.Cathode\n}\n"
                .to_string(),
        ),
    ];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    assert!(
        rendered.contains("D002"),
        "the std Polarized check must not be shaded by a same-named project trait:\n{}",
        rendered
    );
}

// Finding (low): `use` of a design said "nothing is declared there" — false.
#[test]
fn use_of_a_design_names_the_real_rule() {
    let (_checked, rendered) = check(
        "board",
        &[
            (
                "src/sub/boards.cohdl",
                "pub device D { pins { A: 1 [passive] } }\ndesign Board {\n    inst d: D\n    net N: d.A\n}\n",
            ),
            ("src/main.cohdl", "use board::sub::boards::Board;\n"),
        ],
    );
    assert!(
        rendered.contains("designs are project-global and cannot be imported"),
        "{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Adversarial-verification round 2 regressions.

// The RFC's mandatory two-package regression, strengthened to REFERENCE
// level (round-2 finding: the original only asserted declaration keys):
// both colliding declarations are reached via their qualified paths from a
// consuming package, and a bare reference resolves to NEITHER (no silent
// pick, no false ambiguity — it is simply unknown outside both packages,
// with the closest-match suggestion).
#[test]
fn colliding_names_resolve_at_reference_level() {
    let (world, rendered, _sm) = world_of(&[
        (
            "sparkfun/src/power/buck.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "sparkfun",
            "sparkfun::power::buck",
        ),
        (
            "acme/src/power.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "acme",
            "acme::power",
        ),
        (
            "consumer/src/main.cohdl",
            "pub footprint TFP {}\npub part PA: sparkfun::power::buck::TPS62840 { primary { mfr: \"m\", mpn: \"a\", footprint: TFP } }\n\
             pub part PB: acme::power::TPS62840 { primary { mfr: \"m\", mpn: \"b\", footprint: TFP } }\n",
            "consumer",
            "consumer",
        ),
    ]);
    assert!(!rendered.contains("error"), "{}", rendered);
    assert_eq!(
        world.parts["consumer::PA"].device.name.name,
        "sparkfun::power::buck::TPS62840"
    );
    assert_eq!(
        world.parts["consumer::PB"].device.name.name,
        "acme::power::TPS62840"
    );

    // Bare reference from the consumer: unknown (E202 with a suggestion),
    // never a silent pick of either package.
    let (_world, rendered, _sm) = world_of(&[
        (
            "sparkfun/src/power/buck.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "sparkfun",
            "sparkfun::power::buck",
        ),
        (
            "acme/src/power.cohdl",
            "pub device TPS62840 { pins { VIN: 1 [power_in] } }\n",
            "acme",
            "acme::power",
        ),
        (
            "consumer/src/main.cohdl",
            "pub footprint TFP {}\npub part PX: TPS62840 { primary { mfr: \"m\", mpn: \"x\", footprint: TFP } }\n",
            "consumer",
            "consumer",
        ),
    ]);
    assert!(rendered.contains("E202"), "{}", rendered);
    assert!(rendered.contains("did you mean"), "{}", rendered);
}

// Round 2 (medium): `#[intent]` on a use / `pub use` / lone-segment spans
// anchor at the construct itself, and a broken use resynchronizes.
#[test]
fn use_grammar_errors_anchor_and_recover() {
    // Keyword inside a path must not misparse trailing tokens as a phantom
    // declaration — exactly one E010 for the use, and the following device
    // still parses.
    let (checked, rendered) = check(
        "pkg",
        &[(
            "src/main.cohdl",
            "use a::device::b;\npub device Good { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("E010"), "{}", rendered);
    assert!(
        checked.world.devices.contains_key("pkg::Good"),
        "recovery must not swallow the next declaration:\n{}",
        rendered
    );

    // #[intent] on a use anchors at the attribute (line 1), not line 2.
    let (_checked, rendered) = check(
        "pkg",
        &[(
            "src/main.cohdl",
            "#[intent(\"why\")]\nuse pkg::x::Y;\npub device D { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(
        rendered.contains("not valid on a `use`") && rendered.contains("main.cohdl:1:"),
        "intent error anchors at the attribute:\n{}",
        rendered
    );

    // A stray `;` instead of a body must not swallow following declarations.
    let (checked, rendered) = check(
        "pkg",
        &[(
            "src/main.cohdl",
            "device Bad;\npub device Good { pins { A: 1 [passive] } }\n",
        )],
    );
    assert!(rendered.contains("E010"), "{}", rendered);
    assert!(
        checked.world.devices.contains_key("pkg::Good"),
        "stray `;` must not swallow the file:\n{}",
        rendered
    );

    // `use` inside a design body: targeted message, no cascade.
    let (_checked, rendered) = check(
        "pkg",
        &[(
            "src/main.cohdl",
            "pub device D { pins { A: 1 [passive] } }\ndesign B {\n    use pkg::x::Y;\n    inst d: D\n    net N: d.A\n}\n",
        )],
    );
    assert!(
        rendered.contains("file-level"),
        "targeted use-in-body error:\n{}",
        rendered
    );
    assert!(
        !rendered.contains("expected `(`"),
        "no fn-call misparse:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// Fifth-review (2026-07-15) regressions.

// R5-2: an unresolved qualified path in an UNCALLED function must be
// diagnosed at the rewrite pass — expansion never runs on a dead fn, so
// relying on it produced a false-clean verdict.
#[test]
fn unresolved_reference_in_uncalled_fn_is_caught() {
    let (checked, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "fn dead(p: Pin) {\n    inst d: missing::module::Device\n    net _: p, d.A\n}\n\
             pub device Real { pins { A: 1 [passive] } }\n\
             design B { inst x: Real  net N: x.A }\n",
        )],
    );
    assert!(
        r.contains("E202") && r.contains("missing::module::Device"),
        "a dead fn's unresolved reference must fail:\n{}",
        r
    );
    assert!(checked.diags.has_errors());
}

// R5-2: an unresolved reference reported by BOTH the rewrite pass and
// expansion (a design body) is deduped to a single diagnostic.
#[test]
fn unresolved_reference_is_reported_once() {
    let (_c, r) = check(
        "board",
        &[("src/main.cohdl", "design B { inst d: Nope  net N: d.A }\n")],
    );
    assert_eq!(
        r.matches("unknown device or part `Nope`").count(),
        1,
        "exactly one diagnostic:\n{}",
        r
    );
}

// R5-3: a subdirectory named with a reserved keyword indexes declarations at
// an identity no qualified path can spell — diagnosed (E210).
#[test]
fn keyword_directory_is_unspellable_e210() {
    let (_c, r) = check(
        "board",
        &[
            (
                "src/device/x.cohdl",
                "pub device Thing { pins { A: 1 [passive] } }\n",
            ),
            ("src/main.cohdl", "design B { inst d: Thing  net N: d.A }\n"),
        ],
    );
    assert!(
        r.contains("E210") && r.contains("reserved keyword"),
        "keyword directory must be E210:\n{}",
        r
    );
}

// R5-3: a hyphenated directory is likewise unspellable.
#[test]
fn hyphenated_directory_is_unspellable_e210() {
    let (_c, r) = check(
        "board",
        &[
            (
                "src/power-supply/x.cohdl",
                "pub device Thing { pins { A: 1 [passive] } }\n",
            ),
            ("src/main.cohdl", "design B { inst d: Thing  net N: d.A }\n"),
        ],
    );
    assert!(
        r.contains("E210") && r.contains("power-supply"),
        "hyphenated directory must be E210:\n{}",
        r
    );
}

// ---------------------------------------------------------------------------
// Sixth-review (2026-07-15) regressions.

// R6-3: an UNCALLED function body is semantically validated — wrong-kind
// instance/call targets, unresolved generic arguments, and net references to
// unknown locals all fail, not just missing bare targets.
#[test]
fn uncalled_fn_body_is_semantically_checked() {
    // Wrong-kind: a fn as an instance type, a device as a call target.
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Dev { pins { A: 1 [passive] } }\n\
             fn Helper() {}\n\
             fn dead() { inst x: Helper  Dev() }\n\
             design B { inst d: Dev  net N: d.A }\n",
        )],
    );
    assert!(
        r.contains("E205") && r.contains("`board::Helper` is a fn"),
        "{}",
        r
    );
    assert!(r.contains("is a device/part"), "{}", r);

    // Unresolved generic argument in an uncalled fn.
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Gen<V: Voltage> { pins { A: 1 [passive] } }\n\
             fn dead(p: Pin) { inst x: Gen<Missing>  net _: p, x.A }\n\
             design B {}\n",
        )],
    );
    assert!(
        r.contains("E202") && r.contains("cannot find `Missing`"),
        "{}",
        r
    );

    // Net reference to an unknown local in an uncalled fn.
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "fn dead(p: Pin) { net _: p, missing.A }\ndesign B {}\n",
        )],
    );
    assert!(
        r.contains("E202") && r.contains("unknown instance or parameter `missing`"),
        "{}",
        r
    );
}

// R6-3: a VALID uncalled fn body still passes (no false positives from the
// new declaration checks).
#[test]
fn valid_uncalled_fn_body_passes() {
    let (_c, r) = check(
        "board",
        &[(
            "src/main.cohdl",
            "pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }\n\
             fn helper(p: Pin, q: Pin) { inst d: Dev  net _: p, d.A  net _: q, d.B }\n\
             design B {}\n",
        )],
    );
    assert!(!r.contains("error"), "valid fn body must pass:\n{}", r);
}

// R6-4: a keyword PACKAGE ROOT is unspellable in a single-package project
// today (RFC-016 permits qualified intra-package references) — E210.
#[test]
fn keyword_package_root_is_e210() {
    let (_c, r) = check("device", &[("src/main.cohdl", "design B {}\n")]);
    assert!(
        r.contains("E210") && r.contains("package root `device`"),
        "{}",
        r
    );
}

// R6-4: a supplied std tree with a keyword/non-identifier segment is also
// caught (E210 no longer skips std paths).
#[test]
fn keyword_std_segment_is_e210() {
    let (_c, r) = check(
        "board",
        &[
            (
                "std/device/x.cohdl",
                "pub device Thing { pins { A: 1 [passive] } }\n",
            ),
            ("src/main.cohdl", "design B {}\n"),
        ],
    );
    assert!(r.contains("E210") && r.contains("device"), "{}", r);
}

// R7-2: uncalled fn bodies are checked for the deeper semantic classes, not
// just wrong-kind/missing names: concrete-device pin existence, unit-generic
// as an instance type, call arity, generic unit-type mismatch, and missing
// structural-variant selectors.
#[test]
fn uncalled_fn_body_deeper_semantics() {
    let cases: &[(&str, &str)] = &[
        // concrete pin that does not exist
        (
            "pub device D { pins { A: 1 [passive] } }\nfn dead(p: Pin) { inst d: D  net _: p, d.NOPE }\ndesign B {}\n",
            "E203",
        ),
        // a unit-typed generic used as an instance type
        (
            "fn dead<V: Voltage>() { inst x: V  net _: x }\ndesign B {}\n",
            "E205",
        ),
        // call with the wrong argument count
        (
            "fn sink(p: Pin) {}\nfn dead(q: Pin) { sink() }\ndesign B {}\n",
            "E502",
        ),
        // a generic unit-type mismatch on an instance
        (
            "pub device G<V: Voltage> { pins { A: 1 [passive] } }\nfn dead() { inst g: G<1kohm> }\ndesign B {}\n",
            "E112",
        ),
        // a variant device instantiated with no selector
        (
            "pub device Var { variants { A, B }  pins[A] { required S: 1 [passive] }  pins[B] { required S: 2 [passive] } }\nfn dead() { inst v: Var }\ndesign B {}\n",
            "E904",
        ),
    ];
    for (src, code) in cases {
        let (_c, r) = check("board", &[("src/main.cohdl", src)]);
        assert!(
            r.contains(code),
            "expected {} for:\n{}\ngot:\n{}",
            code,
            src,
            r
        );
    }
}

// R7-4: the package-root E210 is anchored to the PROJECT file, not a
// compiler-owned std file that loads first.
#[test]
fn keyword_package_root_anchors_to_project_file() {
    let (checked, r) = check(
        "device",
        &[
            (
                "std/prelude.cohdl",
                "pub device StdThing { pins { A: 1 [passive] } }\n",
            ),
            ("src/main.cohdl", "design B {}\n"),
        ],
    );
    assert!(r.contains("E210"), "{}", r);
    // The E210's primary span must be in src/main.cohdl, not std/.
    let e210 = checked
        .diags
        .iter()
        .find(|d| d.code == "E210")
        .expect("E210");
    let file = checked.sm.name(e210.primary.span.file);
    assert!(
        file.contains("src/main.cohdl"),
        "anchored to `{}`, not the project",
        file
    );
}

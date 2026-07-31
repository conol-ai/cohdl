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
fn every_shipped_component_library_has_consistent_part_footprints() {
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
    }
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

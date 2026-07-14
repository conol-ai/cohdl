//! RFC-018 pad/footprint-format conformance.
//!
//! The two mandatory checks (both type-system, per the RFC): pad internal
//! consistency (drill ⇔ plated_through_hole, size arity vs shape, Length
//! dimensions) at declaration time, and footprint-vs-device pad-count/
//! numbering at `cohdl build`. Plus: the Length unit itself, geometry
//! projection into `.kicad_mod` and IPC-2581 (schema-validated), fmt
//! round-trips, and determinism.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn check(files: &[(&str, &str)]) -> (cohdl::pipeline::Checked, String) {
    let files: Vec<(String, String)> = files
        .iter()
        .map(|(n, c)| (n.to_string(), c.to_string()))
        .collect();
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

/// A pad library + a real two-pad footprint bound to a two-pin device.
const REAL: &str = r#"
pub pad P_Rect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub pad P_Hole { shape: circle, size: (1.7mm), layer: through_all, plating: plated_through_hole, drill: 1.0mm }
pub footprint FP_R0402 {
    pad 1: P_Rect at (-0.5mm, 0mm)
    pad 2: P_Rect at (0.5mm, 0mm)
    courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1.0mm) }
    silkscreen_ref { at: (0mm, -1.2mm) }
}
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: FP_R0402 } }
design B {
    inst r1: R1
    inst r2: R1
    net N: r1.A, r2.A
    net M: r1.B, r2.B
}
"#;

fn build_real(
    src: &str,
) -> (
    cohdl::pipeline::Checked,
    Option<cohdl::pipeline::BuildArtifacts>,
) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    (checked, artifacts)
}

// ---------------------------------------------------------------------------
// The Length unit (the enabling RFC-001 extension).

#[test]
fn length_literals_lex_including_negatives() {
    let (checked, rendered) = check(&[("src/main.cohdl", REAL)]);
    assert!(!rendered.contains("error"), "{}", rendered);
    let fp = &checked.world.footprints["board::FP_R0402"];
    assert_eq!(fp.pads[0].x.text, "-0.5mm");
    assert_eq!(fp.pads[0].x.unit, cohdl::units::UnitType::Length);
    assert!(fp.pads[0].x.femto < 0, "negative offsets are signed");
}

#[test]
fn non_length_dimensions_are_rejected() {
    let (_c, rendered) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (5V), layer: top_copper, plating: smd }\n",
    )]);
    assert!(rendered.contains("E805"), "{}", rendered);
    assert!(rendered.contains("`Length`"), "{}", rendered);

    let (_c, rendered) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (1mm), layer: top_copper, plating: smd }\npub footprint F {\n    pad 1: P at (1ms, 0mm)\n}\n",
    )]);
    assert!(rendered.contains("E806"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Pad declaration checks (E805).

#[test]
fn pad_vocab_and_arity_are_checked() {
    // Unknown shape word.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: hexagon, size: (1mm), layer: top_copper, plating: smd }\n",
    )]);
    assert!(r.contains("E805") && r.contains("hexagon"), "{}", r);

    // Circle with two dimensions.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (1mm, 2mm), layer: top_copper, plating: smd }\n",
    )]);
    assert!(r.contains("E805") && r.contains("(d)"), "{}", r);

    // Rect with one dimension.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm), layer: top_copper, plating: smd }\n",
    )]);
    assert!(r.contains("E805") && r.contains("(w, h)"), "{}", r);

    // Missing required fields.
    let (_c, r) = check(&[("src/main.cohdl", "pub pad P { shape: rect }\n")]);
    assert!(r.contains("missing `size`"), "{}", r);
    assert!(r.contains("missing `layer`"), "{}", r);
    assert!(r.contains("missing `plating`"), "{}", r);
}

#[test]
fn drill_plating_biconditional() {
    // PTH without drill.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (1.7mm), layer: through_all, plating: plated_through_hole }\n",
    )]);
    assert!(r.contains("E805") && r.contains("no `drill:`"), "{}", r);

    // SMD with drill.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd, drill: 0.3mm }\n",
    )]);
    assert!(r.contains("E805") && r.contains("only valid with"), "{}", r);
}

// ---------------------------------------------------------------------------
// Footprint body checks (E806) + resolution.

#[test]
fn duplicate_pad_numbers_are_e806() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (1mm), layer: top_copper, plating: smd }\npub footprint F {\n    pad 1: P at (0mm, 0mm)\n    pad 1: P at (1mm, 0mm)\n}\n",
    )]);
    assert!(
        r.contains("E806") && r.contains("duplicate pad number"),
        "{}",
        r
    );
}

#[test]
fn pad_references_resolve_like_everything_else() {
    // Wrong kind.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub device D { pins { A: 1 [passive] } }\npub footprint F {\n    pad 1: D at (0mm, 0mm)\n}\n",
    )]);
    assert!(r.contains("E205") && r.contains("not a pad"), "{}", r);

    // Unknown, with suggestion.
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P_Rect { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub footprint F {\n    pad 1: P_Rekt at (0mm, 0mm)\n}\n",
    )]);
    assert!(r.contains("unknown pad"), "{}", r);
}

// ---------------------------------------------------------------------------
// The build-time pad-consistency check (E807) — RFC-017's deferred check.

#[test]
fn pad_set_must_match_the_bound_device() {
    // Exact match: builds clean.
    let (checked, artifacts) = build_real(REAL);
    assert!(
        artifacts.is_some() && !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );

    // Missing pad 2: E807 naming it.
    let missing = REAL.replace("    pad 2: P_Rect at (0.5mm, 0mm)\n", "");
    let (checked, artifacts) = build_real(&missing);
    assert!(artifacts.is_none(), "mismatch must fail the build");
    let r = checked.diags.render(&checked.sm);
    assert!(r.contains("E807") && r.contains("missing pad `2`"), "{}", r);

    // Extra pad 3: E807 naming it.
    let extra = REAL.replace(
        "    courtyard",
        "    pad 3: P_Rect at (1mm, 0mm)\n    courtyard",
    );
    let (checked, _) = build_real(&extra);
    let r = checked.diags.render(&checked.sm);
    assert!(r.contains("E807") && r.contains("extra pad `3`"), "{}", r);

    // An EMPTY footprint is RFC-017's stage-one placeholder — exempt.
    let placeholder = REAL
        .replace(
            "pub footprint FP_R0402 {\n    pad 1: P_Rect at (-0.5mm, 0mm)\n    pad 2: P_Rect at (0.5mm, 0mm)\n    courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1.0mm) }\n    silkscreen_ref { at: (0mm, -1.2mm) }\n}",
            "pub footprint FP_R0402 {}",
        );
    let (checked, artifacts) = build_real(&placeholder);
    assert!(
        artifacts.is_some() && !checked.diags.has_errors(),
        "placeholders stay buildable:\n{}",
        checked.diags.render(&checked.sm)
    );
}

// ---------------------------------------------------------------------------
// Geometry projection.

#[test]
fn kicad_mod_projection() {
    let (checked, _artifacts) = build_real(REAL);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert_eq!(mods.len(), 1, "one pad-bearing footprint used");
    let (fq, base, content) = &mods[0];
    assert_eq!(fq, "board::FP_R0402");
    // `-` is the path separator in file names: injective (identifiers may
    // legally contain `__`, so `::` → `__` could collide).
    assert_eq!(base, "board-FP_R0402");
    assert!(
        content.contains("(footprint \"board::FP_R0402\""),
        "{}",
        content
    );
    assert!(content.contains("(generator \"cohdl\")"), "{}", content);
    assert!(content.contains("(attr smd)"), "{}", content);
    assert!(
        content.contains("(pad \"1\" smd rect (at -0.5 0) (size 0.6 0.7) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\"))"),
        "{}",
        content
    );
    assert!(
        content.contains("(fp_rect (start -0.95 -0.5) (end 0.95 0.5) (layer \"F.CrtYd\")"),
        "courtyard corners computed exactly:\n{}",
        content
    );
    assert!(
        content.contains("(fp_text reference \"REF**\" (at 0 -1.2) (layer \"F.SilkS\"))"),
        "{}",
        content
    );
    // Determinism.
    let again = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert_eq!(mods, again);

    // A THT pad projects thru_hole with drill.
    let tht = REAL
        .replace(
            "pad 1: P_Rect at (-0.5mm, 0mm)",
            "pad 1: P_Hole at (-0.5mm, 0mm)",
        )
        .replace("mpn: \"n\"", "mpn: \"n2\"");
    let (checked, _) = build_real(&tht);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert!(
        mods[0].2.contains("(pad \"1\" thru_hole circle (at -0.5 0) (size 1.7 1.7) (drill 1) (layers \"*.Cu\" \"*.Mask\"))"),
        "{}",
        mods[0].2
    );
    assert!(mods[0].2.contains("(attr through_hole)"), "{}", mods[0].2);
}

#[test]
fn ipc2581_projects_real_pins_and_stays_schema_valid() {
    let (checked, _artifacts) = build_real(REAL);
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    // Real pins with shapes and locations.
    assert!(
        xml.contains("<Pin number=\"1\" type=\"SURFACE\">"),
        "{}",
        xml
    );
    assert!(xml.contains("<Location x=\"-0.5\" y=\"0\"/>"), "{}", xml);
    assert!(
        xml.contains("<RectCenter width=\"0.6\" height=\"0.7\"/>"),
        "{}",
        xml
    );
    // The courtyard becomes the package outline (4 corners + closing).
    assert!(
        xml.contains("<PolyBegin x=\"-0.95\" y=\"-0.5\"/>"),
        "{}",
        xml
    );
    // Schema validity with real geometry (xmllint; CI authoritative).
    let dir = std::env::temp_dir().join(format!("cohdl-fp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("real.xml");
    std::fs::write(&path, &xml).unwrap();
    match std::process::Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(manifest().join("tests/schema/IPC-2581B1.xsd"))
        .arg(&path)
        .output()
    {
        Ok(out) => assert!(
            out.status.success(),
            "pad-bearing document must validate:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("WARNING: xmllint not found — schema validity NOT checked locally");
        }
        Err(e) => panic!("xmllint failed to run: {}", e),
    }
}

// ---------------------------------------------------------------------------
// fmt round-trips the new grammar.

#[test]
fn fmt_round_trips_pads_and_footprint_bodies() {
    use cohdl::fmt::format_source;
    let src = "pub pad P { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }\npub footprint F {\n    pad 1: P at (-0.5mm, 0mm) // left\n    pad 2: P at (0.5mm, 0mm)\n    courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1mm) }\n    silkscreen_ref { at: (0mm, -1.2mm) }\n}\n";
    let once = format_source("fp.cohdl", src).unwrap();
    assert!(
        once.contains("pad 1: P at (-0.5mm, 0mm) // left"),
        "{}",
        once
    );
    assert!(
        once.contains("courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1mm) }"),
        "{}",
        once
    );
    let twice = format_source("fp.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);

    // Messy spacing canonicalizes; pad fields one per line.
    let messy = "pub pad P{shape:circle,size:(1mm),layer:through_all,plating:plated_through_hole,drill:0.3mm}";
    let once = format_source("pm.cohdl", messy).unwrap();
    assert!(once.contains("pad P {\n"), "{}", once);
    assert!(once.contains("    drill: 0.3mm\n"), "{}", once);
    let twice = format_source("pm.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

// ---------------------------------------------------------------------------
// Adversarial round regressions (2026-07-14): 15 confirmed findings.

// Finding: E807 silently skipped footprints referenced from `alt` AVL
// entries — a mismatched alt footprint is latent until a fab swaps sources.
#[test]
fn e807_checks_alt_entry_footprints_too() {
    let src = REAL.replace(
        "pub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_R0402 } }",
        "pub footprint FP_Bad {\n    pad 1: P_Rect at (0mm, 0mm)\n    pad 9: P_Rect at (1mm, 0mm)\n}\npub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_R0402 }\n    alt { mfr: \"m2\", mpn: \"n2\", footprint: FP_Bad } }",
    );
    let (checked, artifacts) = build_real(&src);
    assert!(artifacts.is_none(), "alt mismatch must fail the build");
    let r = checked.diags.render(&checked.sm);
    assert!(
        r.contains("E807") && r.contains("missing pad `2`") && r.contains("extra pad `9`"),
        "{}",
        r
    );

    // A MATCHING alt footprint builds clean and gets its own projection.
    let good = REAL.replace(
        "pub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_R0402 } }",
        "pub footprint FP_Alt {\n    pad 1: P_Rect at (0mm, 0mm)\n    pad 2: P_Rect at (1mm, 0mm)\n}\npub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP_R0402 }\n    alt { mfr: \"m2\", mpn: \"n2\", footprint: FP_Alt } }",
    );
    let (checked, artifacts) = build_real(&good);
    assert!(artifacts.is_some(), "{}", checked.diags.render(&checked.sm));
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    let names: Vec<&str> = mods.iter().map(|(fq, _, _)| fq.as_str()).collect();
    assert_eq!(
        names,
        ["board::FP_Alt", "board::FP_R0402"],
        "alt-referenced footprints project too"
    );
}

// Finding: `::` → `__` file naming was not injective (`a__b::c` vs
// `a::b__c` clobbered each other's artifact). `-` cannot appear in
// identifiers, so the mapping is injective by construction.
#[test]
fn kicad_mod_file_names_cannot_collide() {
    // Distinct pad names per module (an unqualified `P` declared in both
    // would be a legitimate E207 ambiguity — unrelated to file naming).
    let src = |pad: &str| {
        format!(
            "pub pad {pad} {{ shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }}\n\
             pub footprint FP {{\n    pad 1: {pad} at (0mm, 0mm)\n    pad 2: {pad} at (1mm, 0mm)\n}}\n"
        )
    };
    let files = vec![
        ("src/a__b/c.cohdl".to_string(), src("PadA")),
        ("src/a/b__c.cohdl".to_string(), src("PadB")),
        (
            "src/main.cohdl".to_string(),
            "pub device Res { pins { A: 1 [passive], B: 2 [passive] } }\n\
             pub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: board::a__b::c::FP } }\n\
             pub part R2: Res { primary { mfr: \"m\", mpn: \"n2\", footprint: board::a::b__c::FP } }\n\
             design B {\n    inst r1: R1\n    inst r2: R2\n    net N: r1.A, r2.A\n    net M: r1.B, r2.B\n}\n"
                .to_string(),
        ),
    ];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(artifacts.is_some(), "{}", checked.diags.render(&checked.sm));
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    let bases: std::collections::BTreeSet<&str> = mods.iter().map(|(_, b, _)| b.as_str()).collect();
    assert_eq!(
        mods.len(),
        bases.len(),
        "distinct footprints must get distinct file names: {:?}",
        mods.iter().map(|(fq, b, _)| (fq, b)).collect::<Vec<_>>()
    );
}

// Finding: nameless `pad {`/`footprint {` recovery left the `{` unconsumed
// for skip_braced_body (which expects it consumed) — depth started at 2 and
// every following declaration was swallowed plus a phantom unclosed-body
// error on a balanced body.
#[test]
fn nameless_pad_and_footprint_bodies_do_not_swallow_the_file() {
    let (checked, r) = check(&[(
        "src/main.cohdl",
        "pad { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub device Res { pins { A: 1 [passive] } }\n",
    )]);
    assert!(r.contains("needs a name"), "{}", r);
    assert!(
        !r.contains("unclosed body"),
        "balanced body must not report unclosed:\n{}",
        r
    );
    assert!(
        checked.world.devices.contains_key("board::Res"),
        "the following declaration must survive:\n{}",
        r
    );

    let (checked, r) = check(&[(
        "src/main.cohdl",
        "footprint {}\npub device Res { pins { A: 1 [passive] } }\n",
    )]);
    assert!(r.contains("needs a name"), "{}", r);
    assert!(checked.world.devices.contains_key("board::Res"), "{}", r);
}

// Finding: an unclosed `courtyard {` stole the footprint's closing brace,
// then recovery swallowed every following declaration and reported a
// phantom E202 for a symbol that IS declared.
#[test]
fn unclosed_courtyard_is_contained() {
    let (checked, r) = check(&[(
        "src/main.cohdl",
        "pub footprint FP {\n    pad 1: P_Rect at (0mm, 0mm)\n    courtyard { shape: rect, at: (0mm, 0mm), size: (1mm, 1mm)\n}\npub pad P_Rect { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub device Res { pins { A: 1 [passive] } }\n",
    )]);
    assert!(r.contains("E010"), "{}", r);
    assert!(
        !r.contains("unknown pad"),
        "P_Rect IS declared — no phantom E202:\n{}",
        r
    );
    assert!(checked.world.devices.contains_key("board::Res"), "{}", r);
    assert!(checked.world.pads.contains_key("board::P_Rect"), "{}", r);
}

// Finding: one typo inside a pad body produced 5-7 phantom errors (false
// "missing field" E805s for fields that are present, phantom E010s at the
// tuple comma) because sync_in_block was paren-blind and the field loop
// bailed out entirely.
#[test]
fn one_pad_field_typo_one_error_plus_true_missing() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, sizes: (1mm, 1mm), layer: top_copper, plating: smd }\n",
    )]);
    let errors = r.matches("error[").count();
    assert!(r.contains("unknown pad field `sizes`"), "{}", r);
    assert!(r.contains("missing `size`"), "{}", r);
    assert!(
        !r.contains("missing `layer`") && !r.contains("missing `plating`"),
        "layer/plating ARE present — no false missing reports:\n{}",
        r
    );
    assert_eq!(errors, 2, "exactly the typo + the true consequence:\n{}", r);
}

// Finding: a courtyard field typo spewed one phantom E806 per remaining
// token; a broken placement consumed the NEXT valid placement.
#[test]
fn courtyard_typo_and_broken_placement_are_contained() {
    let (checked, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub footprint FP {\n    pad 1: P at (0mm, 0mm)\n    courtyard { shape: rect, att: (0mm, 0mm), size: (1mm, 1mm) }\n}\ndesign B {}\n",
    )]);
    assert!(r.contains("unknown courtyard field `att`"), "{}", r);
    assert!(
        r.matches("error[").count() <= 2,
        "one typo must not cascade:\n{}",
        r
    );
    drop(checked);

    // A placement missing its coordinates must not eat the next placement.
    let (checked, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub footprint FP {\n    pad 1: P at\n    pad 2: P at (0.5mm, 0mm)\n}\ndesign B {}\n",
    )]);
    assert!(r.contains("error"), "{}", r);
    let fp = &checked.world.footprints["board::FP"];
    assert_eq!(
        fp.pads.len(),
        1,
        "pad 2 must survive the broken pad 1:\n{}",
        r
    );
    assert_eq!(fp.pads[0].number.text, "2", "{}", r);
}

// Finding: negative/zero sizes, drills, and courtyard dimensions passed
// every check and produced schema-invalid IPC-2581 (and inverted KiCad
// rects). Sizes are extents: must be > 0. Offsets stay signed.
#[test]
fn non_positive_extents_are_rejected() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (-1mm, 0mm), layer: top_copper, plating: smd }\n",
    )]);
    assert!(
        r.contains("E805") && r.contains("non-positive dimension `-1mm`"),
        "{}",
        r
    );
    assert!(r.contains("non-positive dimension `0mm`"), "{}", r);

    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: circle, size: (1mm), layer: through_all, plating: plated_through_hole, drill: -0.3mm }\n",
    )]);
    assert!(
        r.contains("E805") && r.contains("non-positive drill diameter"),
        "{}",
        r
    );

    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub footprint FP {\n    pad 1: P at (-1mm, 0mm)\n    courtyard { shape: rect, at: (0mm, 0mm), size: (-5mm, 1mm) }\n}\n",
    )]);
    assert!(
        r.contains("E806") && r.contains("non-positive dimension `-5mm`"),
        "{}",
        r
    );
    assert!(
        !r.contains("`-1mm`"),
        "placement offsets stay signed:\n{}",
        r
    );
}

// Finding: a courtyard-only footprint (real content, zero pads) was
// misclassified as an RFC-017 placeholder — exempt from E807 and its
// geometry silently dropped. Placeholder = fully empty body only.
#[test]
fn courtyard_only_footprint_is_real_content() {
    let src = REAL.replace(
        "pub footprint FP_R0402 {\n    pad 1: P_Rect at (-0.5mm, 0mm)\n    pad 2: P_Rect at (0.5mm, 0mm)\n    courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1.0mm) }\n    silkscreen_ref { at: (0mm, -1.2mm) }\n}",
        "pub footprint FP_R0402 {\n    courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1.0mm) }\n}",
    );
    let (checked, artifacts) = build_real(&src);
    assert!(artifacts.is_none(), "zero pads vs 2 device pins must fail");
    let r = checked.diags.render(&checked.sm);
    assert!(
        r.contains("E807") && r.contains("missing pads `1`, `2`"),
        "{}",
        r
    );
    // And its authored geometry projects (the check reports, but the
    // emitters must never silently drop authored content).
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert_eq!(mods.len(), 1, "courtyard-only footprints project");
    assert!(mods[0].2.contains("(fp_rect"), "{}", mods[0].2);
}

// Finding: both emitters re-parsed source text at 6-decimal precision
// (truncating legal 7..15-decimal literals to zero) and IPC-2581 computed
// corners in f64, so the two emitters could disagree. Both now share
// emit::geom's exact femto arithmetic.
#[test]
fn geometry_is_exact_beyond_six_decimals_and_canonical() {
    let src = REAL
        .replace("size: (1.9mm, 1.0mm)", "size: (0.0000019mm, 1.0mm)")
        .replace("silkscreen_ref { at: (0mm, -1.2mm) }\n", "");
    let (checked, _artifacts) = build_real(&src);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert!(
        mods[0].2.contains("(start -0.00000095 -0.5)"),
        "7-decimal literals halve exactly:\n{}",
        mods[0].2
    );
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    assert!(
        xml.contains("<PolyBegin x=\"-0.00000095\" y=\"-0.5\"/>"),
        "both emitters agree on the same corners:\n{}",
        xml
    );

    // Spelling canonicalization: `1.0mm` and `1mm` project identically.
    let a = build_real(&REAL.replace("drill: 1.0mm", "drill: 1mm"));
    let b = build_real(REAL);
    let ma = cohdl::emit::kicad_mod::emit_kicad_mods(&a.0.world, a.0.ir.as_ref().unwrap());
    let mb = cohdl::emit::kicad_mod::emit_kicad_mods(&b.0.world, b.0.ir.as_ref().unwrap());
    assert_eq!(ma, mb, "two spellings of one value are one geometry");
}

// Finding: a CIRCLE courtyard was silently dropped from the IPC-2581
// Package (zero-size placeholder outline) while .kicad_mod projected it —
// the emitters disagreed on whether geometry exists. It now projects as
// its bounding square (Outline requires a Polygon; disclosed).
#[test]
fn circle_courtyard_projects_in_both_emitters() {
    let src = REAL.replace(
        "courtyard { shape: rect, at: (0mm, 0mm), size: (1.9mm, 1.0mm) }",
        "courtyard { shape: circle, at: (0mm, 0mm), size: (2mm) }",
    );
    let (checked, _artifacts) = build_real(&src);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert!(mods[0].2.contains("(fp_circle"), "{}", mods[0].2);
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    assert!(
        xml.contains("<PolyBegin x=\"-1\" y=\"-1\"/>"),
        "circle courtyard becomes its bounding square, not a zero outline:\n{}",
        xml
    );
}

// Finding: `footprint:` naming a PAD symbol fell through to E202 "unknown
// footprint" — the resolver knows the symbol and its kind; wrong-kind is
// E205 naming both.
#[test]
fn footprint_field_naming_a_pad_is_a_kind_error() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\npub device Res { pins { A: 1 [passive] } }\npub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: P } }\n",
    )]);
    assert!(
        r.contains("E205") && r.contains("is a pad, not a footprint"),
        "{}",
        r
    );
    assert!(!r.contains("E202"), "not an unknown-name error:\n{}", r);
}

// Finding: E102/E105 still claimed Temperature is the only signed unit.
#[test]
fn negative_literal_messages_name_length() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub device D { pins { A: 1 [passive] } spec { v: -5V } }\n",
    )]);
    assert!(
        r.contains("`Temperature` and `Length`"),
        "the sign rule must name both signed unit types:\n{}",
        r
    );
}

// ---------------------------------------------------------------------------
// Fifth-review (2026-07-15) regressions.

// R5-4: the pad/device match is declaration-complete — a part that no design
// instantiates is still checked (a declaration-only library must be correct
// without a consumer exercising every export).
#[test]
fn unused_part_footprint_mismatch_is_caught() {
    let (checked, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub device One { pins { A: 1 [passive] } }\n\
         pub footprint FP2 {\n    pad 2: P at (0mm, 0mm)\n}\n\
         pub part Unused: One { primary { mfr: \"m\", mpn: \"n\", footprint: FP2 } }\n\
         pub device Real { pins { A: 1 [passive] } }\n\
         design B { inst x: Real  net N: x.A }\n",
    )]);
    assert!(
        r.contains("E807") && r.contains("missing pad `1`") && r.contains("extra pad `2`"),
        "an unused part's footprint mismatch must be caught:\n{}",
        r
    );
    assert!(checked.diags.has_errors());
}

// R5-5: a parser-accepted but astronomically large Length must be a clean
// diagnostic before any emit, never an i128 overflow panic in geometry.
#[test]
fn oversized_length_is_diagnosed_not_panicked() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub footprint FP {\n    pad 1: P at (0mm, 0mm)\n    courtyard { shape: rect, at: (17014118346046923173169mm, 0mm), size: (1mm, 1mm) }\n}\n",
    )]);
    assert!(
        r.contains("E806") && r.contains("too large to project"),
        "oversized geometry must be a clean diagnostic:\n{}",
        r
    );
}

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
    // Real pins with shapes and locations; plating rides @mountType (R5-8).
    // Pin geometry is a StandardPrimitiveRef into the shared dictionary —
    // the encoding consumers implement; an inline primitive under <Pin> is
    // schema-valid but was invisible to a real importer (Quilter).
    assert!(
        xml.contains(
            "<Pin number=\"1\" type=\"SURFACE\" electricalType=\"ELECTRICAL\" mountType=\"SURFACE_MOUNT_PAD\">"
        ),
        "{}",
        xml
    );
    assert!(xml.contains("<Location x=\"-0.5\" y=\"0\"/>"), "{}", xml);
    // The pin's shape lives in DictionaryStandard, referenced from the Pin.
    let entry = xml
        .lines()
        .find(|l| l.contains("<RectCenter width=\"0.6\" height=\"0.7\"/>"))
        .expect("pin shape present in DictionaryStandard");
    assert!(
        entry.contains("<EntryStandard id=\"PRIM_"),
        "pin shape must be a dictionary entry:\n{}",
        entry
    );
    let prim_id = entry
        .split("id=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let pin_block = xml
        .split("<Pin number=\"1\"")
        .nth(1)
        .unwrap()
        .split("</Pin>")
        .next()
        .unwrap();
    assert!(
        pin_block.contains(&format!("<StandardPrimitiveRef id=\"{}\"/>", prim_id)),
        "Pin must reference its dictionary shape:\n{}",
        pin_block
    );
    // The courtyard becomes the package outline (4 corners + closing). The
    // IPC-2581 frame is +y-up (matching KiCad's export), so the courtyard's
    // y is negated relative to the CoHDL/.kicad_mod +y-down authoring.
    assert!(
        xml.contains("<PolyBegin x=\"-0.95\" y=\"0.5\"/>"),
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
    // Both emitters compute the same EXACT femto corners; the IPC frame is
    // +y-up (KiCad's export convention), so its y is the negation of the
    // .kicad_mod's +y-down y (x, and both magnitudes, are identical).
    assert!(
        xml.contains("<PolyBegin x=\"-0.00000095\" y=\"0.5\"/>"),
        "IPC-2581 corner is the exact +y-up negation of the .kicad_mod corner:\n{}",
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
    // Bounding square of the circle, in IPC's +y-up frame (y negated).
    assert!(
        xml.contains("<PolyBegin x=\"-1\" y=\"1\"/>"),
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

// R5-4 / R6-5: the pad/device match is declaration-complete — a part that no
// design instantiates is still checked — but runs at BUILD (RFC-018's phase
// contract), walking every declared part rather than only instantiated IR.
#[test]
fn unused_part_footprint_mismatch_is_caught() {
    let (checked, artifacts) = build_real(
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub device One { pins { A: 1 [passive] } }\n\
         pub footprint FP2 {\n    pad 2: P at (0mm, 0mm)\n}\n\
         pub part Unused: One { primary { mfr: \"m\", mpn: \"n\", footprint: FP2 } }\n\
         pub device Real { pins { A: 1 [passive] } }\n\
         design B { inst x: Real  net N: x.A }\n",
    );
    let r = checked.diags.render(&checked.sm);
    assert!(artifacts.is_none(), "the mismatch must fail the build");
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

// R6-5: an invalid variant selector must not fabricate a false E807 "extra
// pad" on top of the real E903/E904.
#[test]
fn invalid_variant_selection_does_not_cascade_to_e807() {
    // Device with variants A/B; part selects nonexistent variant, footprint
    // matches variant A's pins.
    let (checked, _artifacts) = build_real(
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub device VarDev {\n    variants { A, B }\n    pins[A] { required S: 1 [passive] }\n    pins[B] { required S: 2 [passive] }\n}\n\
         pub footprint FP {\n    pad 1: P at (0mm, 0mm)\n}\n\
         pub part BadSel: VarDev[Z] { primary { mfr: \"m\", mpn: \"n\", footprint: FP } }\n\
         design B {}\n",
    );
    let r = checked.diags.render(&checked.sm);
    assert!(
        r.contains("E903"),
        "the real variant error must fire:\n{}",
        r
    );
    assert!(
        !r.contains("E807"),
        "no fabricated E807 from empty pins_for:\n{}",
        r
    );
}

// R6-9: an oversized silkscreen_ref coordinate is range-checked like every
// other geometry-bearing Length (it previously slipped the check).
#[test]
fn oversized_silkscreen_ref_is_rejected() {
    let (_c, r) = check(&[(
        "src/main.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub footprint FP {\n    pad 1: P at (0mm, 0mm)\n    silkscreen_ref { at: (17014118346046923173169mm, 0mm) }\n}\n",
    )]);
    assert!(
        r.contains("E806") && r.contains("too large to project"),
        "silkscreen coordinate must be range-checked:\n{}",
        r
    );
}

// R7-8: the pad-consistency checker is recovery-safe for an ill-formed variant
// selection (a selector on a non-variant device), even when invoked directly
// — no fabricated E807 on top of the real E905.
#[test]
fn selector_on_non_variant_device_does_not_cascade_e807() {
    use cohdl::check::footprints::check_pad_consistency;
    use cohdl::diag::Diagnostics;
    let files = vec![(
        "src/main.cohdl".to_string(),
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub device Plain { pins { A: 1 [passive] } }\n\
         pub footprint FP {\n    pad 1: P at (0mm, 0mm)\n}\n\
         pub part BadSel: Plain[Z] { primary { mfr: \"m\", mpn: \"n\", footprint: FP } }\n\
         design B {}\n"
            .to_string(),
    )];
    let checked = check_files_in("board", &files, None).expect("selection");
    // Call the checker directly (the branch the normal pipeline never reaches
    // because E905 already blocks the build).
    let mut diags = Diagnostics::new();
    check_pad_consistency(&checked.world, &mut diags);
    let r = diags.render(&checked.sm);
    assert!(
        !r.contains("E807"),
        "no fabricated E807 for a bad selector:\n{}",
        r
    );
}

// ---------------------------------------------------------------------------
// RFC-021 (rewritten): a footprint's OWN identifier IS its IPC-7351 name when
// the package prefix is in the closed family set — checked for grammar (E808)
// and pin-count/pitch geometry agreement (E809). A name outside the closed set
// is an ordinary RFC-016 identifier, unchecked.

/// A footprint NAMED `name` (the IPC-7351 name is the identifier itself) with
/// a pad symbol and the given pad body.
fn fp_named(name: &str, body: &str) -> String {
    format!(
        "pub pad P {{ shape: rect size: (0.5mm, 0.6mm) layer: top_copper plating: smd }}\n\
         pub footprint {} {{ {} }}\n\
         design D {{ }}",
        name, body
    )
}

#[test]
fn valid_chip_name_passes() {
    let src = fp_named(
        "CHIP_0402",
        "pad 1: P at (-0.5mm, 0mm) pad 2: P at (0.5mm, 0mm)",
    );
    let (checked, r) = check(&[("f.cohdl", &src)]);
    assert!(
        !checked.diags.has_errors(),
        "valid IPC-7351 name should pass:\n{}",
        r
    );
}

#[test]
fn qfn_name_with_ep_passes() {
    // 4 leads (0.4mm pitch) + 1 EP = 5 pads; the name declares 4 pins + _1EP.
    let body = "pad 1: P at (-1mm, -0.4mm) pad 2: P at (-1mm, 0mm) \
                pad 3: P at (1mm, 0mm) pad 4: P at (1mm, -0.4mm) pad 5: P at (0mm, 0mm)";
    let src = fp_named("QFN4N40P200X200_1EP50X50", body);
    let (checked, r) = check(&[("f.cohdl", &src)]);
    assert!(
        !checked.diags.has_errors(),
        "QFN with EP should pass:\n{}",
        r
    );
}

#[test]
fn malformed_name_is_e808() {
    // Closed family prefix (QFN) but missing the density suffix.
    let src = fp_named(
        "QFN10P300X300",
        "pad 1: P at (0mm, 0mm) pad 2: P at (0.4mm, 0mm)",
    );
    let (_c, r) = check(&[("f.cohdl", &src)]);
    assert!(r.contains("E808") && r.contains("density"), "{}", r);
}

#[test]
fn name_outside_closed_families_is_free_form() {
    // Prefix is not one of the closed families → ordinary identifier, unchecked
    // (no E808, no E809), even though the pads look nothing like any template.
    let src = fp_named(
        "FP_Widget_42",
        "pad 1: P at (0mm, 0mm) pad 2: P at (0.4mm, 0mm)",
    );
    let (checked, r) = check(&[("f.cohdl", &src)]);
    assert!(
        !checked.diags.has_errors(),
        "a non-IPC-7351 footprint name is free-form:\n{}",
        r
    );
}

#[test]
fn name_pin_count_mismatch_is_e809() {
    // Name says 5 pins, footprint places 2 pads.
    let src = fp_named(
        "SOT5P95X290X160N",
        "pad 1: P at (0mm, 0mm) pad 2: P at (0.95mm, 0mm)",
    );
    let (_c, r) = check(&[("f.cohdl", &src)]);
    assert!(r.contains("E809") && r.contains("pin"), "{}", r);
}

#[test]
fn name_pitch_mismatch_is_e809() {
    // Name says 0.95mm pitch, pads are 0.4mm apart.
    let src = fp_named(
        "SOT2P95X100X100N",
        "pad 1: P at (0mm, 0mm) pad 2: P at (0.4mm, 0mm)",
    );
    let (_c, r) = check(&[("f.cohdl", &src)]);
    assert!(r.contains("E809") && r.contains("pitch"), "{}", r);
}

#[test]
fn empty_footprint_name_grammar_only() {
    // No pads → grammar checked, geometry cross-check skipped (nothing to compare).
    let src = "pub footprint QFN10N40P300X300 { }\ndesign D { }";
    let (checked, r) = check(&[("f.cohdl", src)]);
    assert!(
        !checked.diags.has_errors(),
        "empty footprint + valid IPC-7351 name is ok:\n{}",
        r
    );
}

#[test]
fn fmt_preserves_ipc7351_identifier() {
    use cohdl::fmt::format_source;
    // The IPC-7351 name is the identifier — fmt must render it verbatim and be
    // idempotent (there is no separate metadata field to reorder anymore).
    let src = "pub footprint CHIP_0402{pad 1: P at (-0.5mm,0mm) pad 2: P at (0.5mm,0mm)}";
    let once = format_source("f.cohdl", src).unwrap();
    assert!(
        once.contains("footprint CHIP_0402"),
        "footprint identifier preserved:\n{}",
        once
    );
    let twice = format_source("f.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);
}

// ---------------------------------------------------------------------------
// RFC-022 — mechanical locating holes (mount_hole), disjoint from pad numbering.

/// A footprint with two electrical pads AND two mount_holes whose numbers (1, 2)
/// COLLIDE with the pad numbers — legal, because the two are separate namespaces.
const MH: &str = r#"
pub pad P_Rect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub footprint FP_MH {
    pad 1: P_Rect at (-0.5mm, 0mm)
    pad 2: P_Rect at (0.5mm, 0mm)
    mount_hole 1: non_plated at (0mm, 2mm) diameter 3mm
    mount_hole 2: plated at (0mm, -2mm) diameter 1.5mm
    courtyard { shape: rect, at: (0mm, 0mm), size: (4mm, 6mm) }
}
pub device Dev { pins { A: 1 [passive], B: 2 [passive] } }
pub part P1: Dev { primary { mfr: "m", mpn: "n", footprint: FP_MH } }
design B { inst u1: P1  inst u2: P1  net N: u1.A, u2.A  net M: u1.B, u2.B }
"#;

#[test]
fn mount_hole_parses_disjoint_from_pads() {
    let (checked, rendered) = check(&[("src/main.cohdl", MH)]);
    // mount_hole numbers 1,2 share values with pad numbers 1,2 — no conflict,
    // and the E807 pad-vs-device check (pins {1,2} vs pads {1,2}) still passes.
    assert!(!rendered.contains("error"), "{}", rendered);
    let fp = &checked.world.footprints["board::FP_MH"];
    assert_eq!(fp.pads.len(), 2);
    assert_eq!(fp.mount_holes.len(), 2);
    // RFC-023: no `shape:` written, so this stays a circle carrying `diameter`.
    assert_eq!(fp.mount_holes[0].shape, None);
    assert_eq!(
        fp.mount_holes[0].shape_or_default(),
        cohdl::ast::PadShape::Circle
    );
    match &fp.mount_holes[0].geom {
        cohdl::ast::MountHoleGeom::Diameter(d) => assert_eq!(d.text, "3mm"),
        other => panic!("expected a `diameter` geometry, found {:?}", other),
    }
    assert_eq!(
        fp.mount_holes[0].plating,
        cohdl::ast::MountHolePlating::NonPlated
    );
    assert_eq!(
        fp.mount_holes[1].plating,
        cohdl::ast::MountHolePlating::Plated
    );
}

#[test]
fn mount_hole_projects_np_and_plated_thru_hole() {
    let files = vec![("src/main.cohdl".to_string(), MH.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _ = build_artifacts(&mut checked, &LockState::default());
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    let content = &mods[0].2;
    // non_plated → KiCad's np_thru_hole with an empty pad number (no net).
    assert!(
        content.contains("(pad \"\" np_thru_hole circle (at 0 2) (size 3 3) (drill 3) (layers \"*.Cu\" \"*.Mask\"))"),
        "{}",
        content
    );
    // plated → an ordinary thru_hole, still an empty pad number (no net).
    assert!(
        content.contains("(pad \"\" thru_hole circle (at 0 -2) (size 1.5 1.5) (drill 1.5) (layers \"*.Cu\" \"*.Mask\"))"),
        "{}",
        content
    );
}

#[test]
fn mount_hole_ipc_is_nonplated_and_schema_valid() {
    let files = vec![("src/main.cohdl".to_string(), MH.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _ = build_artifacts(&mut checked, &LockState::default());
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    // The schema enum is NONPLATED (no underscore) — a non_plated mount_hole
    // must emit exactly that, both on its PadstackHoleDef and its board Hole.
    assert!(xml.contains("platingStatus=\"NONPLATED\""), "{}", xml);
    // Schema-validate (mirrors the other IPC tests' xmllint gate).
    let schema = manifest().join("tests/schema/IPC-2581B1.xsd");
    let tmp = std::env::temp_dir().join("cohdl_mh_ipc.xml");
    std::fs::write(&tmp, &xml).unwrap();
    let out = std::process::Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(&schema)
        .arg(&tmp)
        .output();
    if let Ok(o) = out {
        assert!(
            o.status.success(),
            "IPC-2581 with a mount_hole fails schema validation:\n{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

#[test]
fn mount_hole_duplicate_number_is_e810() {
    let src = MH.replace(
        "mount_hole 2: plated at (0mm, -2mm) diameter 1.5mm",
        "mount_hole 1: plated at (0mm, -2mm) diameter 1.5mm",
    );
    let (_checked, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("duplicate mount_hole number"),
        "{}",
        rendered
    );
}

#[test]
fn mount_hole_bad_diameter_is_e810() {
    // Non-positive diameter.
    let src = MH.replace("diameter 3mm", "diameter 0mm");
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(
        rendered.contains("E810"),
        "non-positive diameter:\n{}",
        rendered
    );
    // Wrong unit (a bare number is not a Length — RFC-001 zero coercion).
    let src2 = MH.replace("diameter 3mm", "diameter 3ohm");
    let (_c, rendered2) = check(&[("src/main.cohdl", &src2)]);
    assert!(
        rendered2.contains("E810"),
        "non-Length diameter:\n{}",
        rendered2
    );
}

#[test]
fn mount_hole_invalid_plating_is_e810() {
    let src = MH.replace("non_plated at (0mm, 2mm)", "smd at (0mm, 2mm)");
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("not a mount-hole plating"),
        "{}",
        rendered
    );
}

#[test]
fn mount_hole_round_trips_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "pub footprint F {\n    pad 1: P at (0mm, 0mm)\n    mount_hole 1: non_plated at (0mm, 2mm) diameter 3mm\n    mount_hole 2: plated at (0mm, -2mm) diameter 1.5mm\n}\n";
    let once = format_source("f.cohdl", src).unwrap();
    assert!(
        once.contains("mount_hole 1: non_plated at (0mm, 2mm) diameter 3mm"),
        "mount_hole preserved by fmt:\n{}",
        once
    );
    assert!(
        once.contains("mount_hole 2: plated at (0mm, -2mm) diameter 1.5mm"),
        "{}",
        once
    );
    let twice = format_source("f.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{}", once);
}

// ---------------------------------------------------------------------------
// RFC-023: non-circular locating holes — `mount_hole` gains an optional
// `shape:` (reusing RFC-018's PadShape set) and a shape-dependent geometry
// field: `diameter D` for a circle, `size: (w, h)` for a rect/oval.
// ---------------------------------------------------------------------------

/// A footprint carrying one hole of each RFC-023 shape, alongside an
/// RFC-022-era circular hole written with no `shape:` at all.
const MH23: &str = r#"
pub pad P_Rect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub footprint FP_MH23 {
    pad 1: P_Rect at (-0.5mm, 0mm)
    pad 2: P_Rect at (0.5mm, 0mm)
    mount_hole 1: non_plated at (0mm, 3mm) diameter 3mm
    mount_hole 2: non_plated shape: rect size: (2mm, 1.5mm) at (-4mm, 0mm)
    mount_hole 3: plated shape: oval size: (2.5mm, 1.2mm) at (4mm, 0mm)
    courtyard { shape: rect, at: (0mm, 0mm), size: (12mm, 10mm) }
}
pub device Dev23 { pins { A: 1 [passive], B: 2 [passive] } }
pub part P23: Dev23 { primary { mfr: "m", mpn: "n", footprint: FP_MH23 } }
design B23 { inst u1: P23  inst u2: P23  net N: u1.A, u2.A  net M: u1.B, u2.B }
"#;

#[test]
fn mount_hole_shape_and_size_parse() {
    use cohdl::ast::{MountHoleGeom, PadShape};
    let (checked, rendered) = check(&[("src/main.cohdl", MH23)]);
    assert!(!rendered.contains("error"), "{}", rendered);
    let fp = &checked.world.footprints["board::FP_MH23"];
    assert_eq!(fp.mount_holes.len(), 3);
    // 1: no `shape:` written -> defaults to circle, carries `diameter`.
    assert_eq!(fp.mount_holes[0].shape, None);
    assert_eq!(fp.mount_holes[0].shape_or_default(), PadShape::Circle);
    assert!(matches!(fp.mount_holes[0].geom, MountHoleGeom::Diameter(_)));
    // 2: rect + size:(w, h).
    assert_eq!(fp.mount_holes[1].shape_or_default(), PadShape::Rect);
    match &fp.mount_holes[1].geom {
        MountHoleGeom::Size(d, _) => {
            assert_eq!(d.len(), 2);
            assert_eq!(d[0].text, "2mm");
            assert_eq!(d[1].text, "1.5mm");
        }
        other => panic!("expected size:, found {:?}", other),
    }
    // 3: oval + size:, and plating is independent of shape.
    assert_eq!(fp.mount_holes[2].shape_or_default(), PadShape::Oval);
    assert_eq!(
        fp.mount_holes[2].plating,
        cohdl::ast::MountHolePlating::Plated
    );
}

#[test]
fn mount_hole_rect_oval_project_to_kicad_slots() {
    let files = vec![("src/main.cohdl".to_string(), MH23.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _ = build_artifacts(&mut checked, &LockState::default());
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    let content = &mods[0].2;
    // The circular hole is byte-for-byte what RFC-022 already emitted.
    assert!(
        content.contains(
            "(pad \"\" np_thru_hole circle (at 0 3) (size 3 3) (drill 3) (layers \"*.Cu\" \"*.Mask\"))"
        ),
        "{}",
        content
    );
    // A rect hole keeps its rect pad shape but gets an OVAL drill spanning
    // (w, h) — KiCad has no rectangular drill, so the slot is what's made.
    assert!(
        content.contains(
            "(pad \"\" np_thru_hole rect (at -4 0) (size 2 1.5) (drill oval 2 1.5) (layers \"*.Cu\" \"*.Mask\"))"
        ),
        "{}",
        content
    );
    // An oval, plated hole -> ordinary thru_hole, still no pad number/net.
    assert!(
        content.contains(
            "(pad \"\" thru_hole oval (at 4 0) (size 2.5 1.2) (drill oval 2.5 1.2) (layers \"*.Cu\" \"*.Mask\"))"
        ),
        "{}",
        content
    );
}

#[test]
fn mount_hole_shape_geometry_mismatch_is_e810() {
    // rect declared, but a scalar `diameter` written.
    let src = MH23.replace(
        "mount_hole 2: non_plated shape: rect size: (2mm, 1.5mm) at (-4mm, 0mm)",
        "mount_hole 2: non_plated shape: rect at (-4mm, 0mm) diameter 2mm",
    );
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("`shape: rect`, which takes `size: (w, h)`"),
        "must name expected vs actual:\n{}",
        rendered
    );

    // circle declared explicitly, but `size:` written.
    let src = MH23.replace(
        "mount_hole 1: non_plated at (0mm, 3mm) diameter 3mm",
        "mount_hole 1: non_plated shape: circle size: (3mm, 3mm) at (0mm, 3mm)",
    );
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("`shape: circle`, which takes `diameter D`"),
        "{}",
        rendered
    );
}

#[test]
fn mount_hole_defaulted_circle_rejects_size_and_says_so() {
    // No `shape:` at all -> defaults to circle, so `size:` is a mismatch. The
    // diagnostic must explain the DEFAULT, or it reads as a mystery.
    let src = MH23.replace(
        "mount_hole 1: non_plated at (0mm, 3mm) diameter 3mm",
        "mount_hole 1: non_plated size: (3mm, 2mm) at (0mm, 3mm)",
    );
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("no `shape:` written, so this hole defaults to `circle`"),
        "the default must be explained:\n{}",
        rendered
    );
}

#[test]
fn mount_hole_bad_shape_is_e810() {
    let src = MH23.replace("shape: rect size:", "shape: hexagon size:");
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("shapes are: rect, circle, oval"),
        "must list the closed set:\n{}",
        rendered
    );
}

#[test]
fn mount_hole_size_arity_is_e810() {
    let src = MH23.replace("size: (2mm, 1.5mm)", "size: (2mm, 1.5mm, 3mm)");
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(
        rendered.contains("takes `size: (w, h)` — found 3 dimensions"),
        "{}",
        rendered
    );
}

#[test]
fn mount_hole_non_positive_size_is_e810() {
    let src = MH23.replace("size: (2mm, 1.5mm)", "size: (2mm, 0mm)");
    let (_c, rendered) = check(&[("src/main.cohdl", &src)]);
    assert!(rendered.contains("E810"), "{}", rendered);
    assert!(rendered.contains("non-positive size"), "{}", rendered);
}

#[test]
fn mount_hole_shape_round_trips_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "pub footprint F {\n    pad 1: P at (0mm, 0mm)\n    mount_hole 1: non_plated at (0mm, 2mm) diameter 3mm\n    mount_hole 2: non_plated shape: rect size: (2mm, 1.5mm) at (-4mm, 0mm)\n    mount_hole 3: plated shape: oval size: (2.5mm, 1.2mm) at (4mm, 0mm)\n}\n";
    let once = format_source("f.cohdl", src).unwrap();
    // An RFC-022 circular hole must survive UNCHANGED — fmt never spells out
    // the `circle` default, so pre-RFC-023 sources stay byte-identical.
    assert!(
        once.contains("mount_hole 1: non_plated at (0mm, 2mm) diameter 3mm"),
        "circular form must not gain a `shape:`:\n{}",
        once
    );
    // Canonical order is the accepted grammar line's — `[shape:] at (x, y)
    // [geometry]` — so the example-style ordering above NORMALIZES to it.
    assert!(
        once.contains("mount_hole 2: non_plated shape: rect at (-4mm, 0mm) size: (2mm, 1.5mm)"),
        "{}",
        once
    );
    assert!(
        once.contains("mount_hole 3: plated shape: oval at (4mm, 0mm) size: (2.5mm, 1.2mm)"),
        "{}",
        once
    );
    let twice = format_source("f.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{}", once);
}

#[test]
fn mount_hole_rect_ipc_is_schema_valid() {
    let files = vec![("src/main.cohdl".to_string(), MH23.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _ = build_artifacts(&mut checked, &LockState::default());
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    // A non-circular hole still projects as hole geometry with no net.
    assert!(xml.contains("platingStatus=\"NONPLATED\""), "{}", xml);
    let schema = manifest().join("tests/schema/IPC-2581B1.xsd");
    let tmp = std::env::temp_dir().join("cohdl_mh23_ipc.xml");
    std::fs::write(&tmp, &xml).unwrap();
    let out = std::process::Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(&schema)
        .arg(&tmp)
        .output();
    if let Ok(o) = out {
        assert!(
            o.status.success(),
            "IPC-2581 with a rect/oval mount_hole fails schema validation:\n{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

#[test]
fn mount_hole_accepts_both_field_orderings() {
    use cohdl::ast::MountHoleGeom;
    // The accepted text's grammar line orders `[shape:] at (x, y) [geometry]`;
    // its own worked example writes `[shape:] [geometry] at (x, y)`. Both must
    // parse to the SAME hole, or one of the two spellings in the RFC is a lie.
    let grammar_order = MH23.replace(
        "mount_hole 2: non_plated shape: rect size: (2mm, 1.5mm) at (-4mm, 0mm)",
        "mount_hole 2: non_plated shape: rect at (-4mm, 0mm) size: (2mm, 1.5mm)",
    );
    let (a, ra) = check(&[("src/main.cohdl", MH23)]);
    let (b, rb) = check(&[("src/main.cohdl", &grammar_order)]);
    assert!(!ra.contains("error"), "example order:\n{}", ra);
    assert!(!rb.contains("error"), "grammar order:\n{}", rb);
    let (ha, hb) = (
        &a.world.footprints["board::FP_MH23"].mount_holes[1],
        &b.world.footprints["board::FP_MH23"].mount_holes[1],
    );
    assert_eq!(ha.shape_or_default(), hb.shape_or_default());
    assert_eq!(ha.x.text, hb.x.text);
    assert_eq!(ha.y.text, hb.y.text);
    match (&ha.geom, &hb.geom) {
        (MountHoleGeom::Size(x, _), MountHoleGeom::Size(y, _)) => {
            assert_eq!(x[0].text, y[0].text);
            assert_eq!(x[1].text, y[1].text);
        }
        other => panic!("both must be size: geometry, found {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Slot drills (provisional — no Accepted RFC yet; docs/provisional-syntax.md).
// `drill: (w, l)` is the pad counterpart of RFC-023's non-circular mount_hole:
// a connector shield leg needs a real slot, not a round hole under an oval pad.

/// The same pad library as REAL, with a slotted through-hole tab.
const SLOT: &str = r#"
pub pad P_Rect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub pad P_Tab { shape: oval, size: (1mm, 2.1mm), layer: through_all, plating: plated_through_hole, drill: (0.6mm, 1.7mm) }
pub footprint FP_R0402 {
    pad 1: P_Tab at (-0.5mm, 0mm)
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

#[test]
fn slot_drill_projects_kicad_oval_drill() {
    let (checked, _artifacts) = build_real(SLOT);
    let rendered = checked.diags.render(&checked.sm);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    // KiCad's own oval-drill form — the pad size is NOT the drill, and the
    // two dimensions keep their declared order (width, then length).
    assert!(
        mods[0].2.contains(
            "(pad \"1\" thru_hole oval (at -0.5 0) (size 1 2.1) (drill oval 0.6 1.7) (layers \"*.Cu\" \"*.Mask\"))"
        ),
        "{}",
        mods[0].2
    );
    // Determinism, same as every other geometry projection.
    let again = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    assert_eq!(mods, again);
}

#[test]
fn slot_drill_reports_its_minor_axis_to_ipc2581() {
    let (checked, _) = build_real(SLOT);
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    // IPC's <Hole> carries one scalar, so a slot reports the width it is
    // routed with (0.6), never its length — the same convention RFC-023's
    // oval mount holes already use. The full extent rides the primitive.
    assert!(xml.contains("diameter=\"0.6\""), "slot width as the hole");
    assert!(
        !xml.contains("diameter=\"1.7\""),
        "the slot LENGTH must never be reported as a drill diameter:\n{}",
        xml
    );
}

#[test]
fn a_slot_needs_an_elongated_pad() {
    // A slot inside a round pad would break out of its own annular ring.
    let (_, r) = check(&[(
        "p.cohdl",
        "pub pad P { shape: circle, size: (2mm), layer: through_all, plating: plated_through_hole, drill: (0.6mm, 1.7mm) }\n",
    )]);
    assert!(r.contains("E805"), "{}", r);
    assert!(r.contains("circle") && r.contains("slot"), "{}", r);
    // The help names the way out, not just the rule.
    assert!(r.contains("shape: oval"), "{}", r);
}

#[test]
fn a_slot_must_fit_inside_its_own_pad() {
    let (_, r) = check(&[(
        "p.cohdl",
        "pub pad P { shape: oval, size: (1mm, 2.1mm), layer: through_all, plating: plated_through_hole, drill: (0.6mm, 3mm) }\n",
    )]);
    assert!(r.contains("E805") && r.contains("annular ring"), "{}", r);
    // Names the offending axis, never a bare "too big".
    assert!(r.contains("length"), "{}", r);
}

#[test]
fn slot_dimensions_are_checked_like_any_other_length() {
    // Non-positive.
    let (_, r) = check(&[(
        "p.cohdl",
        "pub pad P { shape: oval, size: (1mm, 2mm), layer: through_all, plating: plated_through_hole, drill: (0mm, 1.7mm) }\n",
    )]);
    assert!(r.contains("E805") && r.contains("non-positive"), "{}", r);
    // Wrong unit — the diagnostic names expected vs actual, per the constitution.
    let (_, r) = check(&[(
        "p.cohdl",
        "pub pad P { shape: oval, size: (1mm, 2mm), layer: through_all, plating: plated_through_hole, drill: (0.6mm, 3ohm) }\n",
    )]);
    assert!(r.contains("E805") && r.contains("Length"), "{}", r);
    // Wrong arity: a slot is exactly (width, length).
    let (_, r) = check(&[(
        "p.cohdl",
        "pub pad P { shape: oval, size: (1mm, 2mm), layer: through_all, plating: plated_through_hole, drill: (0.6mm, 1mm, 2mm) }\n",
    )]);
    assert!(r.contains("E805") && r.contains("width, length"), "{}", r);
}

#[test]
fn fmt_round_trips_a_slot_drill() {
    use cohdl::fmt::format_source;
    // fmt silently dropping a construct it does not know is the classic
    // failure mode for a new grammar form, so assert the slot survives.
    let messy = "pub pad P{shape:oval,size:(1mm,2.1mm),layer:through_all,plating:plated_through_hole,drill:(0.6mm,1.7mm)}";
    let once = format_source("s.cohdl", messy).unwrap();
    assert!(once.contains("    drill: (0.6mm, 1.7mm)\n"), "{}", once);
    let twice = format_source("s.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);
}

// ---------------------------------------------------------------------------
// `window` — a board CUTOUT the part needs (provisional; no Accepted RFC).
// A reverse-mount LED's light leaves through a hole in the PCB, so a footprint
// that omits it describes a part shining into laminate.

const WINDOW_FP: &str = r#"
pub pad P_Rect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub footprint FP_R0402 {
    pad 1: P_Rect at (-2.725mm, 0mm)
    pad 2: P_Rect at (2.725mm, 0mm)
    window { shape: rect, at: (0mm, 0mm), size: (3.4mm, 3mm) }
    courtyard { shape: rect, at: (0mm, 0mm), size: (3.6mm, 3.2mm) }
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

#[test]
fn window_projects_onto_kicad_edge_cuts() {
    let (checked, _) = build_real(WINDOW_FP);
    let rendered = checked.diags.render(&checked.sm);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    // A cutout is board edge, NOT a courtyard and NOT a drilled hole.
    assert!(
        mods[0]
            .2
            .contains("(fp_rect (start -1.7 -1.5) (end 1.7 1.5) (layer \"Edge.Cuts\")"),
        "{}",
        mods[0].2
    );
    // The courtyard is still its own shape on its own layer.
    assert!(mods[0].2.contains("(layer \"F.CrtYd\")"), "{}", mods[0].2);
    // A window is not a hole: nothing gains a drill because of it.
    assert!(!mods[0].2.contains("drill"), "{}", mods[0].2);
}

#[test]
fn window_dimensions_are_checked_like_a_courtyard() {
    // Wrong arity for the shape.
    let (_, r) = check(&[(
        "w.cohdl",
        "pub footprint F { window { shape: rect, at: (0mm, 0mm), size: (3mm) } }\n",
    )]);
    assert!(r.contains("E806") && r.contains("window"), "{}", r);

    // Non-positive extent.
    let (_, r) = check(&[(
        "w.cohdl",
        "pub footprint F { window { shape: rect, at: (0mm, 0mm), size: (0mm, 3mm) } }\n",
    )]);
    assert!(r.contains("E806") && r.contains("non-positive"), "{}", r);

    // Wrong unit — named expected vs actual, per the constitution.
    let (_, r) = check(&[(
        "w.cohdl",
        "pub footprint F { window { shape: rect, at: (0mm, 0mm), size: (3mm, 2ohm) } }\n",
    )]);
    assert!(r.contains("E806") && r.contains("Length"), "{}", r);

    // At most one per footprint.
    let (_, r) = check(&[(
        "w.cohdl",
        "pub footprint F {\n  window { shape: rect, at: (0mm, 0mm), size: (3mm, 3mm) }\n  \
         window { shape: rect, at: (1mm, 0mm), size: (3mm, 3mm) }\n}\n",
    )]);
    assert!(r.contains("at most one `window`"), "{}", r);
}

#[test]
fn fmt_round_trips_a_window() {
    use cohdl::fmt::format_source;
    // A window-only footprint must not format as an empty one, and the
    // construct must survive — fmt silently dropping it is the failure mode.
    let messy = "pub footprint F{window{shape:rect,at:(0mm,0mm),size:(3.4mm,3mm)}}";
    let once = format_source("w.cohdl", messy).unwrap();
    assert!(
        once.contains("    window { shape: rect, at: (0mm, 0mm), size: (3.4mm, 3mm) }\n"),
        "{}",
        once
    );
    let twice = format_source("w.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);
}

// ---------------------------------------------------------------------------
// RFC-031: silkscreen graphics — four primitives plus two semantic markers.

const SILK: &str = r#"
pub pad P_T { shape: rect, size: (0.9mm, 1.2mm), layer: top_copper, plating: smd }
pub footprint FP_R0402 {
    pad 1: P_T at (-1.65mm, 0mm)
    pad 2: P_T at (1.65mm, 0mm)
    silkscreen {
        polarity_marker cathode_pin 1 shape band
        line from (-2.36mm, -1mm) to (1.65mm, -1mm) width 0.12mm
        circle at (0mm, 0mm) radius 0.3mm width 0.1mm
        arc at (0mm, 0mm) radius 1mm start_angle 0 end_angle 180 width 0.12mm
        polygon [(2mm, 0mm), (2.5mm, 0.3mm), (2.5mm, -0.3mm)]
    }
    courtyard { shape: rect, at: (0mm, 0mm), size: (4.2mm, 2mm) }
}
pub device Dv { pins { A: 1 [passive], B: 2 [passive] } }
pub part R1: Dv { primary { mfr: "m", mpn: "n", footprint: FP_R0402 } }
design B {
    inst r1: R1
    inst r2: R1
    net N: r1.A, r2.A
    net M: r1.B, r2.B
}
"#;

#[test]
fn silkscreen_primitives_project_onto_kicad_silk_layer() {
    let (checked, _) = build_real(SILK);
    let rendered = checked.diags.render(&checked.sm);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.as_ref().unwrap();
    let m = &cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir)[0].2;
    // Each primitive maps to KiCad's own native graphic item, on F.SilkS.
    assert!(
        m.contains("(fp_line (start -2.36 -1) (end 1.65 -1) (layer \"F.SilkS\")"),
        "{}",
        m
    );
    assert!(
        m.contains("(fp_circle (center 0 0) (end 0.3 0) (layer \"F.SilkS\")"),
        "{}",
        m
    );
    assert!(
        m.contains("(fp_arc (start 1 0) (mid 0 1) (end -1 0) (layer \"F.SilkS\")"),
        "{}",
        m
    );
    assert!(
        m.contains("(fp_poly (pts (xy 2 0) (xy 2.5 0.3) (xy 2.5 -0.3))"),
        "{}",
        m
    );
    // Determinism, like every other geometry projection.
    assert_eq!(
        m,
        &cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir)[0].2
    );
}

#[test]
fn markers_expand_to_real_geometry_clear_of_their_pad() {
    let (checked, _) = build_real(SILK);
    let ir = checked.ir.as_ref().unwrap();
    let m = &cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir)[0].2;
    // The cathode band: a 0.3mm stroke standing off the cathode land's EDGE
    // (pad 1 spans x -2.1..-1.2), perpendicular to the terminal axis, spanning
    // the pad's own height. A band ON the pad would defeat its purpose.
    assert!(
        m.contains(
            "(fp_line (start -2.4 -0.6) (end -2.4 0.6) (layer \"F.SilkS\") (stroke (width 0.3)"
        ),
        "{}",
        m
    );
}

#[test]
fn pin_1_marker_shapes_both_expand() {
    // Pad 1 spans x -2.1..-1.2, so the 0.3mm standoff point is -2.4: a dot's
    // CENTRE then sits a further radius out at -2.6, while a triangle's APEX
    // sits on the standoff point itself, pointing back at the pad.
    for (shape, needle) in [
        ("dot", "(fp_circle (center -2.6 0) (end -2.4 0)"),
        ("triangle", "(fp_poly (pts (xy -2.4 0)"),
    ] {
        let src = SILK.replace(
            "polarity_marker cathode_pin 1 shape band",
            &format!("pin_1_marker near pad 1 shape {}", shape),
        );
        let (checked, _) = build_real(&src);
        let ir = checked.ir.as_ref().unwrap();
        let m = &cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir)[0].2;
        assert!(m.contains(needle), "{} marker:\n{}", shape, m);
    }
}

#[test]
fn a_marker_must_name_a_pad_the_footprint_declares() {
    let (_, r) = check(&[(
        "s.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub footprint F {\n  pad 1: P at (0mm, 0mm)\n  pad 2: P at (2mm, 0mm)\n  \
         silkscreen { pin_1_marker near pad 7 shape dot }\n}\n",
    )]);
    assert!(r.contains("E812"), "{}", r);
    assert!(r.contains("does not declare"), "{}", r);
    // The help lists the real pads — never a bare "invalid pad".
    assert!(r.contains("its pads are: 1, 2"), "{}", r);
}

#[test]
fn silkscreen_closed_sets_and_shapes_are_checked() {
    // Unknown statement kind.
    let (_, r) = check(&[("s.cohdl", "pub footprint F { silkscreen { squiggle } }\n")]);
    assert!(
        r.contains("E812") && r.contains("unknown silkscreen statement"),
        "{}",
        r
    );
    // Unknown marker shape.
    let (_, r) = check(&[(
        "s.cohdl",
        "pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }\n\
         pub footprint F { pad 1: P at (0mm, 0mm)\n  silkscreen { pin_1_marker near pad 1 shape star } }\n",
    )]);
    assert!(r.contains("E812") && r.contains("dot, triangle"), "{}", r);
    // A polygon needs three vertices.
    let (_, r) = check(&[(
        "s.cohdl",
        "pub footprint F { silkscreen { polygon [(0mm, 0mm), (1mm, 1mm)] } }\n",
    )]);
    assert!(
        r.contains("E812") && r.contains("at least 3 vertices"),
        "{}",
        r
    );
    // A zero-width stroke draws nothing.
    let (_, r) = check(&[(
        "s.cohdl",
        "pub footprint F { silkscreen { line from (0mm, 0mm) to (1mm, 0mm) width 0mm } }\n",
    )]);
    assert!(r.contains("E812") && r.contains("draws nothing"), "{}", r);
    // Wrong unit, named expected vs actual.
    let (_, r) = check(&[(
        "s.cohdl",
        "pub footprint F { silkscreen { circle at (0mm, 0mm) radius 2ohm width 0.1mm } }\n",
    )]);
    assert!(r.contains("E812") && r.contains("Length"), "{}", r);
    // At most one block.
    let (_, r) = check(&[(
        "s.cohdl",
        "pub footprint F { silkscreen { } silkscreen { } }\n",
    )]);
    assert!(r.contains("at most one `silkscreen`"), "{}", r);
}

#[test]
fn fmt_round_trips_silkscreen_with_markers_first() {
    use cohdl::fmt::format_source;
    let messy = "pub footprint F{pad 1: P at (0mm,0mm)\nsilkscreen{line from (0mm,0mm) to (1mm,0mm) width 0.12mm\n\
                 pin_1_marker near pad 1 shape dot}}";
    let once = format_source("s.cohdl", messy).unwrap();
    // RFC-031's canonical order: semantic markers before raw primitives.
    let marker = once.find("pin_1_marker").expect("marker kept");
    let line = once.find("line from").expect("primitive kept");
    assert!(marker < line, "markers come first:\n{}", once);
    assert!(
        once.contains("        pin_1_marker near pad 1 shape dot\n"),
        "{}",
        once
    );
    let twice = format_source("s.cohdl", &once).unwrap();
    assert_eq!(once, twice, "not idempotent:\n{}", once);
}

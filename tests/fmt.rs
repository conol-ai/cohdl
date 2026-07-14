//! RFC-009 `cohdl fmt` conformance: the two mechanically-checkable correctness
//! properties (idempotence + semantic inertness), plus comment preservation
//! and the standing `fmt --check` gate that the committed repo is canonical.

use cohdl::fmt::format_source;
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files};
use std::path::{Path, PathBuf};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.cohdl` file shipped in the repo: std + the example design.
fn repo_cohdl_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "cohdl") {
                let name = p
                    .strip_prefix(manifest())
                    .unwrap_or(&p)
                    .display()
                    .to_string();
                out.push((name, std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    walk(&manifest().join("std"), &mut out);
    walk(&manifest().join("examples"), &mut out);
    out
}

// ---------------------------------------------------------------------------
// The `fmt --check` gate: the committed repo is already in canonical form, so
// a `git diff` on a fmt-clean repository only ever shows semantic changes.

#[test]
fn repo_is_in_canonical_form() {
    for (name, text) in repo_cohdl_files() {
        let formatted = format_source(&name, &text).expect("repo file parses");
        assert_eq!(
            formatted, text,
            "`{}` is not in canonical form — run `cargo run -- fmt std examples`",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Property 1: idempotence — fmt(fmt(x)) == fmt(x).

fn assert_idempotent(name: &str, src: &str) {
    let once = format_source(name, src).expect("first format parses");
    let twice = format_source(name, &once).expect("formatted output parses");
    assert_eq!(once, twice, "fmt is not idempotent for `{}`", name);
}

#[test]
fn idempotent_on_repo_files() {
    for (name, text) in repo_cohdl_files() {
        assert_idempotent(&name, &text);
    }
}

#[test]
fn idempotent_on_messy_input() {
    // Deliberately un-canonical spellings that must all normalize (and stay
    // normalized): inline pins, missing/extra spacing, single-line device.
    let fixtures = [
        "pub device M<C: Capacitance,V: Voltage=10V> {\n  variants { A , B }\n  pins[A]{required X: 1 [passive]}\n  pins[B]{required X: 1 [passive]}\n  spec{capacitance:C}\n}",
        "pub trait T:U+V{pins{required A: pin}}",
        "design B{inst c: M<100nF>[A]\nnet   N:c.X}",
    ];
    for (i, src) in fixtures.iter().enumerate() {
        assert_idempotent(&format!("messy{i}.cohdl"), src);
    }
}

// ---------------------------------------------------------------------------
// Property 2: semantic inertness — formatting never changes the emitted bytes.

fn build_netlist_bom(files: &[(String, String)], design: Option<&str>) -> (String, String) {
    let mut checked = check_files(files, design).expect("design selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build succeeds");
    assert!(!checked.diags.has_errors(), "clean build expected");
    (artifacts.netlist, artifacts.bom)
}

/// Every example project directory, so each is built with its own design.
fn example_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(manifest().join("examples"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

#[test]
fn formatting_is_semantically_inert() {
    let std_dir = manifest().join("std");
    for ex in example_dirs() {
        // std + this example's own sources — one design per project.
        let proj = cohdl::project::load_project(&ex, Some(&std_dir)).unwrap();
        let original = proj.files.clone();
        let formatted: Vec<(String, String)> = original
            .iter()
            .map(|(name, text)| (name.clone(), format_source(name, text).unwrap()))
            .collect();

        let (net_a, bom_a) = build_netlist_bom(&original, proj.top.as_deref());
        let (net_b, bom_b) = build_netlist_bom(&formatted, proj.top.as_deref());
        assert_eq!(
            net_a,
            net_b,
            "netlist bytes changed after fmt for {}",
            ex.display()
        );
        assert_eq!(
            bom_a,
            bom_b,
            "BOM bytes changed after fmt for {}",
            ex.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Comment preservation: every `//` comment survives formatting verbatim.

#[test]
fn comments_are_preserved() {
    for (name, text) in repo_cohdl_files() {
        let formatted = format_source(&name, &text).unwrap();
        for line in text.lines() {
            // Recover the comment text (only when `//` is outside a string).
            if let Some(idx) = comment_start(line) {
                let comment = line[idx..].trim_end();
                assert!(
                    formatted.contains(comment),
                    "comment `{}` from `{}` was dropped by fmt",
                    comment,
                    name
                );
            }
        }
    }
}

fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_str = !in_str,
            b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Adversarial edge cases surfaced by verification (all four must round-trip and
// never drop a comment) — the repo files don't exercise these paths.

/// Format twice; assert idempotent and that every listed comment survives.
fn assert_edge(name: &str, src: &str, comments: &[&str]) {
    let once = format_source(name, src).expect("parses");
    let twice = format_source(name, &once).expect("formatted output parses");
    assert_eq!(once, twice, "not idempotent for `{}`:\n{}", name, once);
    for c in comments {
        assert!(
            once.contains(c),
            "comment `{}` dropped by fmt for `{}`:\n{}",
            c,
            name,
            once
        );
    }
}

#[test]
fn backslash_string_round_trips() {
    // The grammar has no string escapes: a value with a backslash must survive
    // formatting unchanged and not grow (str_lit must not introduce escapes).
    let src = "pub device D {\n    pins { A: 1 [passive] }\n}\npub part P: D {\n    primary { mfr: \"A\\B\", mpn: \"M\", footprint: \"F\" }\n}";
    let once = format_source("bs.cohdl", src).unwrap();
    assert!(once.contains("\"A\\B\""), "backslash mangled:\n{}", once);
    let twice = format_source("bs.cohdl", &once).unwrap();
    assert_eq!(
        once, twice,
        "backslash string is not idempotent:\n{}",
        twice
    );
    // Count backslashes: must stay exactly one (no doubling).
    assert_eq!(
        twice.matches('\\').count(),
        1,
        "backslash count grew:\n{}",
        twice
    );
}

#[test]
fn trailing_comment_on_first_line_of_multiline_statement_survives() {
    // Trailing comment on a non-final line of a wrapped net member list.
    let src =
        "design B {\n    inst a: D\n    inst b: D\n    net N: a.A, // keep first\n        b.A\n}";
    assert_edge("t1.cohdl", src, &["// keep first"]);
}

#[test]
fn interior_full_line_comment_in_multiline_statement_survives() {
    let src = "design B {\n    inst a: D\n    inst b: D\n    net N: a.A,\n        // interior note\n        b.A\n    nc: a.A,\n        // dead pins\n        b.A\n}";
    assert_edge("t2.cohdl", src, &["// interior note", "// dead pins"]);
}

#[test]
fn trailing_comment_on_one_line_item_survives() {
    let src =
        "pub trait Foo { pins { required A: pin } } // trait note\nimpl Bar for D {} // impl note";
    assert_edge("t3.cohdl", src, &["// trait note", "// impl note"]);
}

// ---------------------------------------------------------------------------
// Review findings F6/F7/F11: comments around attributes, inside layout blocks,
// and inside trait/impl/part bodies; wrapping; blank-after-brace preservation.

#[test]
fn trailing_comment_on_attribute_survives() {
    // F6: `#[intent("why")] // keep this` — the trailing comment must survive,
    // on items and on statements, for intent, placement_hint, and designator.
    let src = "#[intent(\"why\")] // keep item note\npub device D {\n    pins { A: 1 [passive] }\n}\ndesign B {\n    #[intent(\"why2\")] // keep stmt note\n    #[placement_hint(\"corner\")] // keep hint note\n    #[designator(\"U7\")] // keep designator note\n    inst d: D\n    net N: d.A\n}";
    assert_edge(
        "attr-trail.cohdl",
        src,
        &[
            "// keep item note",
            "// keep stmt note",
            "// keep hint note",
            "// keep designator note",
        ],
    );
}

#[test]
fn comment_between_attribute_and_target_stays_between() {
    let src = "#[intent(\"why\")]\n// between attr and device\npub device D {\n    pins { A: 1 [passive] }\n}";
    let once = format_source("between.cohdl", src).unwrap();
    let attr_pos = once.find("#[intent").unwrap();
    let cmt_pos = once.find("// between attr and device").unwrap();
    let dev_pos = once.find("pub device D").unwrap();
    assert!(
        attr_pos < cmt_pos && cmt_pos < dev_pos,
        "comment must stay between attribute and declaration:\n{}",
        once
    );
    assert_idempotent("between.cohdl", src);
}

#[test]
fn comments_inside_layout_blocks_survive_in_place() {
    // F7: full-line and trailing comments inside layout {} stay inside.
    let src = "design B {\n    net A: x.p\n    net C: y.p\n    layout {\n        // class note\n        net_class K { A, C } // trailing class\n        diff_pair(A, C)\n    }\n}";
    let once = format_source("laycmt.cohdl", src).unwrap();
    let open = once.find("layout {").unwrap();
    let close = once.rfind('}').unwrap();
    for c in ["// class note", "// trailing class"] {
        let pos = once
            .find(c)
            .unwrap_or_else(|| panic!("{} dropped:\n{}", c, once));
        assert!(
            pos > open && pos < close,
            "{} moved out of the block:\n{}",
            c,
            once
        );
    }
    assert_idempotent("laycmt.cohdl", src);
}

#[test]
fn comments_inside_trait_impl_part_survive() {
    // F11: interior comments in trait/impl/part bodies were deleted.
    let src = "pub trait T {\n    // prefix note\n    designator_prefix: \"X\"\n    pins {\n        // pin note\n        required A: pin // trailing pin\n    }\n}\npub device D {\n    pins { A: 1 [passive], B: 2 [passive] }\n}\nimpl T for D {\n    pins {\n        // mapping note\n        A: B // trailing map\n    }\n}\npub part P: D {\n    // avl note\n    primary { mfr: \"m\", mpn: \"n\", footprint: \"f\" } // trailing avl\n}";
    assert_edge(
        "tip.cohdl",
        src,
        &[
            "// prefix note",
            "// pin note",
            "// trailing pin",
            "// mapping note",
            "// trailing map",
            "// avl note",
            "// trailing avl",
        ],
    );
}

#[test]
fn long_lines_wrap_at_100_columns() {
    // Pin buses, AVL entries, and layout constraints all wrap.
    let bus: Vec<String> = (1..=40).map(|i| i.to_string()).collect();
    let src = format!(
        "pub device D {{\n    pins {{ GND: {} [power_in] }}\n}}\npub part P: D {{\n    primary {{ mfr: \"SomeVeryLongManufacturerName\", mpn: \"EXTREMELY-LONG-PART-NUMBER-12345\", footprint: \"Library_Name:Some_Extremely_Long_Footprint_Name_3.5x2.65mm\" }}\n}}",
        bus.join(", ")
    );
    let once = format_source("wrap.cohdl", &src).unwrap();
    for line in once.lines() {
        assert!(
            line.len() <= 100 || !line.contains(','),
            "wrappable line exceeds 100 cols:\n{}",
            line
        );
    }
    // Role bracket rides the last line of the wrapped bus.
    assert!(once.contains("[power_in]"), "{}", once);
    assert_idempotent("wrap.cohdl", &src);
}

#[test]
fn trailing_comments_on_header_and_brace_lines_survive() {
    // Re-verification residual: trailing comments on a declaration's header
    // line (both `header { // note` and `header // note NEWLINE {` styles),
    // on block-opener lines, and on block-closer lines must survive.
    let src = "pub trait T // C-T\n{\n    pins { // C-PINS\n        required A: pin\n    } // C-CLOSE\n}\npub device D { // C-DEV\n    pins { A: 1 [passive] }\n    spec // C-SPEC\n    {\n        x: 1nF\n    }\n}\nimpl T for D // C-IMPL\n{\n    pins {\n        A: A\n    }\n}\npub part P: D // C-PART\n{\n    primary { mfr: \"m\", mpn: \"n\", footprint: \"f\" }\n}\npub fn h(p: Pin) // C-FN\n{\n    net _: p\n}\ndesign B // C-DSN\n{\n    inst d: P\n    net N: d.A\n    layout { // C-LAY\n        net_class K { N }\n    }\n}";
    assert_edge(
        "hdr.cohdl",
        src,
        &[
            "// C-T",
            "// C-PINS",
            "// C-CLOSE",
            "// C-DEV",
            "// C-SPEC",
            "// C-IMPL",
            "// C-PART",
            "// C-FN",
            "// C-DSN",
            "// C-LAY",
        ],
    );
    // The layout-header comment stays INSIDE the block (residual #4).
    let once = format_source("hdr.cohdl", src).unwrap();
    let lay = once
        .find("layout { // C-LAY")
        .expect("layout header keeps its comment");
    let close = once[lay..].find('}').unwrap();
    assert!(once[lay..lay + close].contains("net_class"), "{}", once);
}

#[test]
fn eof_backstop_never_drops_a_comment() {
    // A trailing comment on a brace-on-its-own-line — no emitter owns that
    // line's code, so the backstop must still preserve the comment.
    let src = "pub device D\n{ // stray on the brace line\n    pins { A: 1 [passive] }\n}";
    let once = format_source("stray.cohdl", src).unwrap();
    assert!(
        once.contains("// stray on the brace line"),
        "backstop failed:\n{}",
        once
    );
    let twice = format_source("stray.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn shared_line_trailing_comment_attaches_to_last_statement() {
    // Two statements on one source line: the line's trailing comment belongs
    // to the LAST construct on the line, not the first.
    let src = "design B {\n    inst d: P\n    net A: d.X  net C: d.Y // belongs to C\n}";
    let once = format_source("shared.cohdl", src).unwrap();
    assert!(
        once.contains("net C: d.Y // belongs to C"),
        "comment attached to the wrong statement:\n{}",
        once
    );
    let twice = format_source("shared.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn whole_construct_comment_stays_on_the_construct() {
    // Review-2 regression: a comment trailing a ONE-line construct describes
    // the whole construct — after expansion it must ride the construct's
    // LAST line (the closer), not the opening header.
    let src = "pub device D { pins { A: 1 [passive] } } // whole device comment\ndesign B { inst d: D net N: d.A } // whole design comment";
    let once = format_source("whole.cohdl", src).unwrap();
    assert!(
        once.contains("} // whole device comment"),
        "device comment must ride the closer:\n{}",
        once
    );
    assert!(
        once.contains("} // whole design comment"),
        "design comment must ride the closer:\n{}",
        once
    );
    assert!(
        !once.contains("{ // whole"),
        "comment must not move to the opener:\n{}",
        once
    );
    assert_idempotent("whole.cohdl", src);
}

#[test]
fn empty_body_comments_stay_inside_braces() {
    // Review-2 regression: comment-only trait/impl bodies must not collapse
    // to {} with the comments exiled to EOF.
    let src = "pub device D { pins { A: 1 [passive] } }\npub trait T {\n    // only trait body comment\n}\nimpl T for D {\n    // only impl body comment\n}";
    let once = format_source("emptyb.cohdl", src).unwrap();
    let t_open = once.find("pub trait T {").unwrap();
    let t_note = once.find("// only trait body comment").unwrap();
    let i_open = once.find("impl T for D {").unwrap();
    let i_note = once.find("// only impl body comment").unwrap();
    assert!(
        t_open < t_note && t_note < i_open && i_open < i_note,
        "body comments must stay inside their braces:\n{}",
        once
    );
    assert_idempotent("emptyb.cohdl", src);
}

#[test]
fn attributes_keep_source_order_and_comments() {
    // Review-2: attributes serialize in SOURCE order (designator before
    // intent as written), so comments between them cannot migrate.
    let src = "pub device D { pins { A: 1 [passive] } }\ndesign B {\n    #[designator(\"U7\")] // keep-d\n    // between attrs\n    #[intent(\"why\")] // keep-i\n    inst d: D\n    net N: d.A\n}";
    let once = format_source("attrord.cohdl", src).unwrap();
    let d_pos = once.find("#[designator").unwrap();
    let between = once.find("// between attrs").unwrap();
    let i_pos = once.find("#[intent").unwrap();
    assert!(
        d_pos < between && between < i_pos,
        "source order + comment position must hold:\n{}",
        once
    );
    assert!(
        once.contains("// keep-d") && once.contains("// keep-i"),
        "{}",
        once
    );
    assert_idempotent("attrord.cohdl", src);
}

#[test]
fn attr_sharing_line_with_decl_leaves_comment_to_the_decl() {
    // Review-2: `#[intent("x")] inst d: D // note` — the comment belongs to
    // the statement, not the attribute.
    let src = "pub device D { pins { A: 1 [passive] } }\ndesign B {\n    #[intent(\"x\")] inst d: D // decl note\n    net N: d.A\n}";
    let once = format_source("attrshare.cohdl", src).unwrap();
    assert!(
        once.contains("inst d: D // decl note"),
        "comment must follow the declaration:\n{}",
        once
    );
    assert!(!once.contains(")] // decl note"), "{}", once);
    assert_idempotent("attrshare.cohdl", src);
}

#[test]
fn blank_after_open_brace_is_preserved() {
    // RFC-009: an author-placed blank is never removed (only runs collapse).
    let src = "design B {\n\n    inst d: D\n    net N: d.A\n}";
    let once = format_source("blank.cohdl", src).unwrap();
    assert!(
        once.contains("design B {\n\n    inst"),
        "author blank after brace was removed:\n{}",
        once
    );
    assert_idempotent("blank.cohdl", src);
}

#[test]
fn tolerance_canonicalizes_unit_literal_unquoted() {
    // A tolerance that lexes as an RFC-001 unit literal canonicalizes to the
    // unquoted spelling; a length string keeps the quoted escape hatch.
    let src = "design B {\n    net A: x.p\n    net C: y.p\n    layout {\n        length_match(A, C) [tolerance: \"1ms\"]\n        length_match(A, C) [tolerance: \"0.15mm\"]\n    }\n}";
    let once = format_source("tol.cohdl", src).unwrap();
    assert!(once.contains("[tolerance: 1ms]"), "{}", once);
    assert!(once.contains("[tolerance: \"0.15mm\"]"), "{}", once);
    assert_idempotent("tol.cohdl", src);
}

// ---------------------------------------------------------------------------
// fmt is a serializer, not a repair tool: non-parsing source is an error.

#[test]
fn fmt_rejects_nonparsing_source() {
    // A device pin with no role bracket is a parse error (RFC-008/E901); fmt
    // must not silently complete it — it reports the parse error instead.
    let err = format_source("bad.cohdl", "pub device D {\n    pins { A: 1 }\n}")
        .expect_err("missing role must not format");
    assert!(err.contains("E901"), "{}", err);
}

// ---------------------------------------------------------------------------
// Review-3 regressions (R1/R2).

// R1: a QUOTED tolerance that happens to lex as a non-Time unit literal
// ("5V", "100nF", "1kohm") must stay quoted — unquoting it produces source
// the parser rejects (E110), breaking fmt validity and idempotence.
#[test]
fn tolerance_quoted_non_time_stays_quoted() {
    for lit in ["5V", "100nF", "1kohm", "50%"] {
        let src = format!(
            "design B {{\n    net A: x.p\n    net C: y.p\n    layout {{\n        length_match(A, C) [tolerance: \"{}\"]\n    }}\n}}",
            lit
        );
        let once = format_source("tolq.cohdl", &src).unwrap();
        assert!(
            once.contains(&format!("[tolerance: \"{}\"]", lit)),
            "non-Time tolerance `{}` must stay quoted:\n{}",
            lit,
            once
        );
        assert_idempotent("tolq.cohdl", &src);
    }
}

// R2a: attributes sharing ONE line serialize in written (span) order, never
// re-grouped by category.
#[test]
fn same_line_mixed_attributes_keep_written_order() {
    let src = "design B {\n    #[designator(\"U7\")] #[intent(\"why\")] #[placement_hint(\"near\")] inst d: D\n    net N: d.A\n}";
    let once = format_source("attrs.cohdl", src).unwrap();
    let d = once.find("#[designator").unwrap();
    let i = once.find("#[intent").unwrap();
    let p = once.find("#[placement_hint").unwrap();
    assert!(
        d < i && i < p,
        "written order (designator, intent, placement_hint) must survive:\n{}",
        once
    );
    assert_idempotent("attrs.cohdl", src);

    // And a different written order also survives.
    let src2 =
        "design B {\n    #[intent(\"why\")] #[designator(\"U7\")] inst d: D\n    net N: d.A\n}";
    let once2 = format_source("attrs2.cohdl", src2).unwrap();
    assert!(
        once2.find("#[intent").unwrap() < once2.find("#[designator").unwrap(),
        "reversed written order must survive too:\n{}",
        once2
    );
    assert_idempotent("attrs2.cohdl", src2);
}

// R2b: an EMPTY trait/impl whose opener line carries a trailing comment
// keeps the comment on that line (braces open up) — never exiled to EOF.
#[test]
fn empty_body_opener_comment_stays_attached() {
    let src = "pub trait T { // review 3 fixture\n}\npub device D { pins { A: 1 [passive] } }\nimpl T for D { // impl comment\n}\n";
    let once = format_source("empty.cohdl", src).unwrap();
    assert!(
        once.contains("pub trait T { // review 3 fixture"),
        "trait opener comment must stay attached:\n{}",
        once
    );
    assert!(
        once.contains("impl T for D { // impl comment"),
        "impl opener comment must stay attached:\n{}",
        once
    );
    assert!(
        !once.trim_end().ends_with("// impl comment"),
        "comment must not be exiled to EOF:\n{}",
        once
    );
    assert_idempotent("empty.cohdl", src);
}

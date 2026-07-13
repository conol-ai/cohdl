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

fn build_netlist_bom(files: &[(String, String)]) -> (String, String) {
    let mut checked = check_files(files, None).expect("design selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build succeeds");
    assert!(!checked.diags.has_errors(), "clean build expected");
    (artifacts.netlist, artifacts.bom)
}

#[test]
fn formatting_is_semantically_inert() {
    let original = repo_cohdl_files();
    let formatted: Vec<(String, String)> = original
        .iter()
        .map(|(name, text)| (name.clone(), format_source(name, text).unwrap()))
        .collect();

    let (net_a, bom_a) = build_netlist_bom(&original);
    let (net_b, bom_b) = build_netlist_bom(&formatted);
    assert_eq!(net_a, net_b, "netlist bytes changed after fmt");
    assert_eq!(bom_a, bom_b, "BOM bytes changed after fmt");
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
// fmt is a serializer, not a repair tool: non-parsing source is an error.

#[test]
fn fmt_rejects_nonparsing_source() {
    // A device pin with no role bracket is a parse error (RFC-008/E901); fmt
    // must not silently complete it — it reports the parse error instead.
    let err = format_source("bad.cohdl", "pub device D {\n    pins { A: 1 }\n}")
        .expect_err("missing role must not format");
    assert!(err.contains("E901"), "{}", err);
}

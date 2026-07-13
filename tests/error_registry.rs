//! RFC-011 error-code registry completeness check.
//!
//! The registry (`docs/error-codes.md`) and the compiler source must never
//! drift: this test runs the RFC-011 "both directions" completeness contract.
//!
//! 1. source → registry: every code literal (`"E###"` / `"D###"`) in `src/`
//!    has a row in the registry.
//! 2. registry → source: every registry row that is not marked `[DEPRECATED]`,
//!    `[RESERVED]`, or CLI-only ("not a source diagnostic") has at least one
//!    real call site in `src/`.
//!
//! This closes the "structurally present but not actually enforced" gap class
//! (DR-006) for the diagnostics registry — the same discipline the type system
//! and residual DRC already hold themselves to.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `src/`, recursively.
fn src_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }
    walk(&manifest().join("src"), &mut out);
    out
}

/// Is `s` a code string: an uppercase `E`/`D` followed by exactly three digits.
fn is_code(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 4 && (b[0] == b'E' || b[0] == b'D') && b[1..].iter().all(|c| c.is_ascii_digit())
}

/// All `"E###"` / `"D###"` string literals in the compiler source.
fn codes_in_source() -> BTreeSet<String> {
    let mut codes = BTreeSet::new();
    for path in src_files() {
        let text = std::fs::read_to_string(&path).unwrap();
        let bytes = text.as_bytes();
        let mut i = 0;
        while i + 5 < bytes.len() {
            // Look for a quote, then a 4-char code, then a quote.
            if bytes[i] == b'"' && bytes[i + 5] == b'"' {
                let inner = &text[i + 1..i + 5];
                if is_code(inner) {
                    codes.insert(inner.to_string());
                }
            }
            i += 1;
        }
    }
    codes
}

/// A parsed registry row: its code, and whether it is exempt from needing a
/// live call site (deprecated / reserved / CLI-only).
struct Row {
    code: String,
    exempt: bool,
}

fn registry_rows() -> Vec<Row> {
    let text = std::fs::read_to_string(manifest().join("docs/error-codes.md")).unwrap();
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix('|') else {
            continue;
        };
        // First cell is the code (e.g. "E107" or "E107 ").
        let first = rest.split('|').next().unwrap_or("").trim();
        if !is_code(first) {
            continue;
        }
        let exempt = line.contains("DEPRECATED")
            || line.contains("RESERVED")
            || line.contains("not a source diagnostic");
        rows.push(Row {
            code: first.to_string(),
            exempt,
        });
    }
    rows
}

#[test]
fn every_source_code_has_a_registry_row() {
    let registered: BTreeSet<String> = registry_rows().into_iter().map(|r| r.code).collect();
    let missing: Vec<String> = codes_in_source()
        .into_iter()
        .filter(|c| !registered.contains(c))
        .collect();
    assert!(
        missing.is_empty(),
        "codes used in src/ but absent from docs/error-codes.md: {:?}\n\
         (RFC-011: every diagnostic code must have a registry row)",
        missing
    );
}

#[test]
fn every_live_registry_row_has_a_call_site() {
    let in_source = codes_in_source();
    let dead: Vec<String> = registry_rows()
        .into_iter()
        .filter(|r| !r.exempt && !in_source.contains(&r.code))
        .map(|r| r.code)
        .collect();
    assert!(
        dead.is_empty(),
        "registry rows with no call site in src/: {:?}\n\
         (RFC-011: a documented-but-dead code must be marked [DEPRECATED] or \
         [RESERVED, not yet implemented])",
        dead
    );
}

#[test]
fn no_duplicate_registry_rows() {
    let mut seen = BTreeSet::new();
    let mut dups = Vec::new();
    for row in registry_rows() {
        if !seen.insert(row.code.clone()) {
            dups.push(row.code);
        }
    }
    assert!(
        dups.is_empty(),
        "a code is issued once and never repurposed — duplicate registry rows: {:?}",
        dups
    );
}

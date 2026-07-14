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

/// Is `s` a code string: an uppercase `E`/`D` followed by three digits (E402)
/// or four (E1001, the RFC-013 layout block).
fn is_code(s: &str) -> bool {
    let b = s.as_bytes();
    (b.len() == 4 || b.len() == 5)
        && (b[0] == b'E' || b[0] == b'D')
        && b[1..].iter().all(|c| c.is_ascii_digit())
}

/// All `"E###"` / `"E####"` / `"D###"` string literals in the compiler source
/// (the source → registry direction: even a stray code-shaped literal must be
/// documented).
fn codes_in_source() -> BTreeSet<String> {
    let mut codes = BTreeSet::new();
    for path in src_files() {
        let text = std::fs::read_to_string(&path).unwrap();
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'"' {
                // A quoted 4- or 5-char code closed by another quote.
                for len in [4usize, 5] {
                    let close = i + 1 + len;
                    if close < bytes.len() && bytes[close] == b'"' {
                        let inner = &text[i + 1..close];
                        if is_code(inner) {
                            codes.insert(inner.to_string());
                        }
                    }
                }
            }
            i += 1;
        }
    }
    codes
}

/// Strip Rust comments (line + nested block) from source while copying
/// string and char literals verbatim (so `"file://…"` is never misread as a
/// comment, and `'"'` never flips the string state). The registry → source
/// direction must not count commented-out constructors (review R4).
fn strip_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue; // the newline itself is copied on the next pass
        }
        if c == '/' && next == Some('*') {
            let mut depth = 1;
            i += 2;
            while i < chars.len() && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '\'' {
            // A char literal ('x', '\n', '"') is copied atomically so a
            // quote inside it cannot open a phantom string. A lifetime
            // tick ('a in generics) falls through as a lone quote.
            if next == Some('\\') && chars.get(i + 3) == Some(&'\'') {
                out.extend(&chars[i..i + 4]);
                i += 4;
                continue;
            }
            if chars.get(i + 2) == Some(&'\'') && next.is_some() {
                out.extend(&chars[i..i + 3]);
                i += 3;
                continue;
            }
        }
        // A Rust RAW string `r#*"…"#*` — its body is inert text, never code,
        // and may itself contain `"` and `Diagnostic::error("E999")` (review
        // F12.1). Recognize it only at a token boundary (the `r` in `error`
        // or `for` is not a prefix) and blank the whole thing so no phantom
        // call site is counted.
        if c == 'r'
            && !out
                .chars()
                .last()
                .is_some_and(|p| p.is_alphanumeric() || p == '_')
        {
            let mut j = i + 1;
            let mut hashes = 0;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                // Scan to the closing `"` followed by `hashes` `#`s.
                j += 1;
                let close: Vec<char> = std::iter::once('"')
                    .chain(std::iter::repeat_n('#', hashes))
                    .collect();
                while j < chars.len() {
                    if chars[j] == '"' && chars[j + 1..].starts_with(&close[1..]) {
                        j += close.len();
                        break;
                    }
                    j += 1;
                }
                out.push(' '); // the whole raw string becomes inert whitespace
                i = j;
                continue;
            }
        }
        if c == '"' {
            // Copy the string literal verbatim, honoring escapes.
            out.push('"');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(chars[i]);
                let closed = chars[i] == '"';
                i += 1;
                if closed {
                    break;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Codes that appear as the FIRST argument of a real
/// `Diagnostic::error(…)` / `Diagnostic::warning(…)` call in `text` — after
/// comment stripping, so a commented-out constructor never counts.
fn call_sites_in(text: &str) -> BTreeSet<String> {
    let text = strip_comments(text);
    let mut codes = BTreeSet::new();
    for kw in ["Diagnostic::error(", "Diagnostic::warning("] {
        let mut from = 0;
        while let Some(pos) = text[from..].find(kw) {
            let after = from + pos + kw.len();
            // Skip whitespace/newlines to the first argument.
            let rest = text[after..].trim_start();
            if let Some(stripped) = rest.strip_prefix('"') {
                for len in [4usize, 5] {
                    if stripped.len() > len && stripped.as_bytes()[len] == b'"' {
                        let inner = &stripped[..len];
                        if is_code(inner) {
                            codes.insert(inner.to_string());
                        }
                    }
                }
            }
            from = after;
        }
    }
    codes
}

/// The registry → source direction demands an actual call site, not a quoted
/// mention in a comment or an unused constant.
fn call_site_codes() -> BTreeSet<String> {
    let mut codes = BTreeSet::new();
    for path in src_files() {
        let text = std::fs::read_to_string(&path).unwrap();
        codes.extend(call_sites_in(&text));
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
    // The strict direction: a REAL `Diagnostic::error/warning` constructor
    // call, not any quoted mention of the code.
    let call_sites = call_site_codes();
    let dead: Vec<String> = registry_rows()
        .into_iter()
        .filter(|r| !r.exempt && !call_sites.contains(&r.code))
        .map(|r| r.code)
        .collect();
    assert!(
        dead.is_empty(),
        "registry rows with no Diagnostic::error/warning call site in src/: {:?}\n\
         (RFC-011: a documented-but-dead code must be marked [DEPRECATED] or \
         [RESERVED, not yet implemented])",
        dead
    );
}

#[test]
fn call_site_scanner_finds_known_sites() {
    // Self-check on the scanner: codes constructed in obviously different
    // styles (inline literal, multi-line construction) are all found.
    let sites = call_site_codes();
    for known in ["E010", "E101", "E701", "E1001", "D003"] {
        assert!(
            sites.contains(known),
            "scanner failed to find the known call site for {}",
            known
        );
    }
    // And it is strictly narrower than the any-literal scan.
    assert!(sites.is_subset(&codes_in_source()));
}

// Review R4: the scanner is comment-aware — a commented-out constructor (in
// either comment style) must NOT satisfy the registry → source direction,
// while string contents survive stripping (`//` inside a string is not a
// comment).
#[test]
fn scanner_ignores_commented_constructors() {
    let sample = r#"
// Diagnostic::error("E999", span, "commented out")
/* Diagnostic::warning("D999", span, "block comment") */
/* nested /* Diagnostic::error("E888", ...) */ still dead */
fn live() {
    let _u = "file://not-a-comment";
    let _c = '"';
    Diagnostic::error("E997", span, "a real call site");
}
"#;
    let sites = call_sites_in(sample);
    assert!(
        !sites.contains("E999") && !sites.contains("D999") && !sites.contains("E888"),
        "commented-out constructors must not count: {:?}",
        sites
    );
    assert!(
        sites.contains("E997"),
        "the real call site must still be found: {:?}",
        sites
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

// Review F12.1: a Rust RAW string is inert text, not code — a
// `Diagnostic::error("E999", …)` written inside `r#"…"#` must NOT be counted
// as a live call site (the comment stripper previously only understood
// ordinary `"…"` strings, so a raw string could inject a phantom code).
#[test]
fn raw_strings_are_not_call_sites() {
    let sample = r####"
        let normal = Diagnostic::error("E101", span, "real");
        let _doc = r#"see Diagnostic::error("E999", span, "not code")"#;
        let _nested = r##"a raw "quote" and Diagnostic::error("E998", ...)"##;
    "####;
    let sites = call_sites_in(sample);
    assert!(
        sites.contains("E101"),
        "the real call site is found: {:?}",
        sites
    );
    assert!(
        !sites.contains("E999") && !sites.contains("E998"),
        "raw-string bodies must not count as call sites: {:?}",
        sites
    );
}

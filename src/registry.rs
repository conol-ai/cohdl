//! RFC-030: the registry.cohdl.org client — three-tier namespace grammar,
//! HTTP transport, the package archive format, the local content cache, and
//! the credentials store.
//!
//! Transport is the system `curl` binary: the zero-dependency constitution
//! covers the compiler crate, and RFC-030 grants no dependency exception —
//! shelling out to the platform's own HTTP client keeps the crate clean
//! (documented in docs/compliance-report.md). The archive is uncompressed
//! POSIX tar (the RFC's ".tar.gz (or equivalent)" — DEFLATE is not worth
//! hand-rolling for kilobytes of source text).

use crate::deps::{PackageDiag, Version};
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_SEARCH_RESPONSE: usize = 1024 * 1024;
/// Application-side API-doc upload ceiling. Kept byte-for-byte aligned with
/// `registry/src/worker/apidocs.ts` and docs/apidocs.md.
pub const API_DOCS_MAX_BYTES: usize = 200_000_000;
/// Documents above this size use the Worker's fixed-length streaming path.
pub const API_DOCS_BUFFER_MAX_BYTES: usize = 16_000_000;

/// The one official registry (RFC-030); `COHDL_REGISTRY` overrides it for
/// development and tests.
pub fn registry_url() -> String {
    std::env::var("COHDL_REGISTRY").unwrap_or_else(|_| "https://registry.cohdl.org".to_string())
}

/// `$COHDL_HOME` (default `$HOME/.cohdl`): the content cache and credentials.
pub fn cohdl_home() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("COHDL_HOME") {
        return Some(PathBuf::from(p));
    }
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cohdl"))
}

/// The registry content cache: one package family dir per name (scoped names
/// nest naturally — `@sparkfun/power` → `registry/@sparkfun/power/`).
pub fn cache_root() -> Option<PathBuf> {
    cohdl_home().map(|h| h.join("registry"))
}

// ---------------------------------------------------------------------------
// Three-tier namespace grammar (structural — the name's shape IS its tier)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tier {
    /// Bare name — CoHDL official, never first-come-first-served.
    Official,
    /// `@brand/name` — verified manufacturer.
    Brand(String),
    /// `@contrib/name` — open community namespace.
    Contrib,
}

impl Tier {
    pub fn describe(&self) -> String {
        match self {
            Tier::Official => "official (bare name — reserved for CoHDL's own packages)".into(),
            Tier::Brand(b) => format!("manufacturer (`@{b}/…` — verified brand account)"),
            Tier::Contrib => "community (`@contrib/…` — open namespace)".into(),
        }
    }
}

/// Validate a registry package name against the closed three-tier grammar
/// (RFC-030): bare `name`, `@brand/name`, or `@contrib/name`, each segment
/// in RFC-016's package-name grammar.
pub fn name_tier(name: &str) -> Result<Tier, String> {
    if let Some(rest) = name.strip_prefix('@') {
        let Some((scope, pkg)) = rest.split_once('/') else {
            return Err(format!(
                "`{name}` is not a valid package name — a scoped name is `@scope/name`"
            ));
        };
        if crate::project::valid_package_name(scope).is_err()
            || crate::project::valid_package_name(pkg).is_err()
            || pkg.contains('/')
        {
            return Err(format!(
                "`{name}` is not a valid package name — each segment uses letters, digits, `_`, `-`"
            ));
        }
        if scope == "contrib" {
            Ok(Tier::Contrib)
        } else {
            Ok(Tier::Brand(scope.to_string()))
        }
    } else {
        crate::project::valid_package_name(name).map_err(|e| e.to_string())?;
        Ok(Tier::Official)
    }
}

/// Split a `name@X.Y.Z` argument (the `cohdl add` pin form). The `@` that
/// starts a scoped name is not a version separator.
pub fn split_name_version(arg: &str) -> (String, Option<String>) {
    let split_at = if let Some(rest) = arg.strip_prefix('@') {
        rest.find('@').map(|i| i + 1)
    } else {
        arg.find('@')
    };
    match split_at {
        Some(i) => (arg[..i].to_string(), Some(arg[i + 1..].to_string())),
        None => (arg.to_string(), None),
    }
}

// ---------------------------------------------------------------------------
// Credentials (~/.cohdl/credentials.toml — never committed anywhere)
// ---------------------------------------------------------------------------

pub fn read_token() -> Option<String> {
    let path = cohdl_home()?.join("credentials.toml");
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "token" {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

pub fn write_token(token: &str) -> Result<PathBuf, String> {
    let home = cohdl_home().ok_or("cannot determine $COHDL_HOME (set HOME or COHDL_HOME)")?;
    std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    let path = home.join("credentials.toml");
    std::fs::write(&path, format!("token = \"{}\"\n", token)).map_err(|e| e.to_string())?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// HTTP via the system curl
// ---------------------------------------------------------------------------

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

fn run_curl_with_timeout(args: &[String], max_time_seconds: u32) -> Result<HttpResponse, String> {
    let max_time = max_time_seconds.to_string();
    let out = Command::new("curl")
        .args([
            "-sS",
            "-w",
            "\n%{http_code}",
            "--max-time",
            max_time.as_str(),
        ])
        .args(args)
        .output()
        .map_err(|e| {
            format!("cannot run `curl`: {e} (the registry client uses the system curl)")
        })?;
    if !out.status.success() {
        let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            format!("curl failed with {}", out.status)
        } else {
            detail
        });
    }
    // The status code rides the last line (after our \n marker).
    let body = out.stdout;
    let split = body
        .iter()
        .rposition(|b| *b == b'\n')
        .ok_or("malformed curl output")?;
    let status: u16 = String::from_utf8_lossy(&body[split + 1..])
        .trim()
        .parse()
        .map_err(|_| "malformed curl status".to_string())?;
    Ok(HttpResponse {
        status,
        body: body[..split].to_vec(),
    })
}

fn run_curl(args: &[String]) -> Result<HttpResponse, String> {
    run_curl_with_timeout(args, 60)
}

pub fn http_get(url: &str) -> Result<HttpResponse, String> {
    run_curl(&[url.to_string()])
}

/// GET with query parameters encoded by curl itself. Values are passed as
/// argv entries, never through a shell; `--data-urlencode` is what keeps a
/// search for spaces, `&`, Unicode, or `#` from changing the request's shape.
fn http_get_query(url: &str, params: &[(&str, &str)]) -> Result<HttpResponse, String> {
    let mut args = vec![
        "-G".to_string(),
        "--max-filesize".to_string(),
        MAX_SEARCH_RESPONSE.to_string(),
    ];
    for (key, value) in params {
        args.push("--data-urlencode".to_string());
        args.push(format!("{key}={value}"));
    }
    args.push(url.to_string());
    run_curl(&args)
}

/// GET following redirects, with a longer deadline (a repeated `--max-time`
/// overrides run_curl's 60s — curl takes the last occurrence). GitHub
/// release downloads bounce through a CDN, which plain `http_get`
/// deliberately does not follow (`cohdl self-update` is the only caller).
pub fn http_get_follow(url: &str) -> Result<HttpResponse, String> {
    run_curl(&[
        "-L".to_string(),
        "--max-time".to_string(),
        "300".to_string(),
        url.to_string(),
    ])
}

pub fn http_post(
    url: &str,
    body_file: Option<&Path>,
    token: Option<&str>,
    content_type: &str,
) -> Result<HttpResponse, String> {
    let mut args = vec![
        "-X".to_string(),
        "POST".to_string(),
        "-H".to_string(),
        format!("Content-Type: {content_type}"),
    ];
    if let Some(t) = token {
        args.push("-H".to_string());
        args.push(format!("Authorization: Bearer {t}"));
    }
    if let Some(f) = body_file {
        args.push("--data-binary".to_string());
        args.push(format!("@{}", f.display()));
    }
    args.push(url.to_string());
    run_curl(&args)
}

/// PUT with a file body — the API-docs sidecar upload (docs/apidocs.md).
/// Same transport discipline as [`http_post`]: the system curl, a bearer
/// token, the body staged in a temp file.
pub fn http_put(
    url: &str,
    body_file: &Path,
    token: &str,
    content_type: &str,
    extra_headers: &[(&str, &str)],
) -> Result<HttpResponse, String> {
    let mut args = vec![
        "-X".to_string(),
        "PUT".to_string(),
        "-H".to_string(),
        format!("Content-Type: {content_type}"),
        "-H".to_string(),
        format!("Authorization: Bearer {token}"),
        "--data-binary".to_string(),
        format!("@{}", body_file.display()),
    ];
    for (name, value) in extra_headers {
        args.push("-H".to_string());
        args.push(format!("{name}: {value}"));
    }
    args.push(url.to_string());
    // API-doc sidecars may be hundreds of megabytes. Keep the ordinary
    // registry calls at 60 seconds, but allow a 200 MB upload to complete on
    // a slow link without changing any protocol semantics.
    run_curl_with_timeout(&args, 1800)
}

/// Remove insignificant JSON whitespace for the registry upload only.
///
/// `cohdl docs` remains human-readable on stdout and with `--out`; the
/// uploaded sidecar is the same JSON value in a substantially smaller byte
/// representation. The emitter has already produced valid UTF-8 JSON, so a
/// tiny string/escape state machine is sufficient and deterministic.
pub fn compact_json_for_upload(json: &str) -> Vec<u8> {
    // Count first so a heavily indented generated document does not reserve
    // its full pretty-printed size in addition to the compact body.
    let mut compact_len = 0usize;
    visit_compact_json_bytes(json, |_| compact_len += 1);
    let mut out = Vec::with_capacity(compact_len);
    visit_compact_json_bytes(json, |byte| out.push(byte));
    debug_assert_eq!(out.len(), compact_len);
    out
}

fn visit_compact_json_bytes(json: &str, mut keep: impl FnMut(u8)) {
    let mut in_string = false;
    let mut escaped = false;
    for byte in json.bytes() {
        if in_string {
            keep(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
            keep(byte);
        } else if !matches!(byte, b' ' | b'\n' | b'\r' | b'\t') {
            keep(byte);
        }
    }
    debug_assert!(
        !in_string,
        "the emitter always produces complete JSON strings"
    );
}

// ---------------------------------------------------------------------------
// Minimal JSON field extraction (the registry's responses are flat objects;
// the crate's hand-rolled-JSON discipline applies to parsing too)
// ---------------------------------------------------------------------------

/// The string value of a top-level `"key": "value"` pair.
pub fn json_str_field(body: &[u8], key: &str) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let pat = format!("\"{key}\"");
    let at = text.find(&pat)?;
    let rest = &text[at + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

/// Every string element of a top-level `"key": ["a", "b", …]` array.
pub fn json_str_array(body: &[u8], key: &str) -> Vec<String> {
    let Some(text) = std::str::from_utf8(body).ok() else {
        return Vec::new();
    };
    let pat = format!("\"{key}\"");
    let Some(at) = text.find(&pat) else {
        return Vec::new();
    };
    let rest = &text[at + pat.len()..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    rest[open + 1..open + close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Registry search JSON
// ---------------------------------------------------------------------------

/// One package result from the registry's stable `/search` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSearchHit {
    pub name: String,
    pub tier: String,
    pub latest: String,
    pub description: Option<String>,
    pub updated: String,
}

/// One public part/AVL result from the registry's stable `/search` endpoint.
/// The registry selects one matching purchasing identity per part; `primary`
/// tells the caller whether that flattened identity is the primary AVL row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSearchHit {
    pub package: String,
    pub tier: String,
    pub version: String,
    pub fq: String,
    pub name: String,
    pub device: String,
    pub intent: Option<String>,
    pub manufacturer: Option<String>,
    pub mpn: Option<String>,
    pub primary: bool,
}

/// The complete, grouped search response. Package and part ordering is the
/// registry's relevance ordering and is preserved byte-for-byte by the CLI's
/// deterministic JSON re-render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResults {
    pub query: String,
    pub packages: Vec<PackageSearchHit>,
    pub packages_has_more: bool,
    pub parts: Vec<PartSearchHit>,
    pub parts_has_more: bool,
}

/// A small RFC-8259 parser used for the public search response. This is more
/// deliberate than the legacy flat-field helpers above: search contains
/// nested arrays of publisher-controlled strings, so substring extraction is
/// not safe. It remains dependency-free per the compiler constitution.
#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number,
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

struct JsonParser<'a> {
    input: &'a [u8],
    pos: usize,
    nodes: usize,
}

impl<'a> JsonParser<'a> {
    const MAX_DEPTH: usize = 128;
    const MAX_NODES: usize = 4_096;
    const MAX_CONTAINER_ITEMS: usize = 256;
    const MAX_STRING_BYTES: usize = 64 * 1024;

    fn new(input: &'a [u8]) -> Result<Self, String> {
        std::str::from_utf8(input).map_err(|_| "response is not valid UTF-8".to_string())?;
        Ok(Self {
            input,
            pos: 0,
            nodes: 0,
        })
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        let value = self.value(0)?;
        self.ws();
        if self.pos != self.input.len() {
            return Err(format!("unexpected trailing data at byte {}", self.pos));
        }
        Ok(value)
    }

    fn ws(&mut self) {
        while self
            .input
            .get(self.pos)
            .is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.pos += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > Self::MAX_DEPTH {
            return Err("JSON nesting exceeds 128 levels".to_string());
        }
        self.nodes += 1;
        if self.nodes > Self::MAX_NODES {
            return Err("JSON response exceeds 4096 values".to_string());
        }
        self.ws();
        match self.input.get(self.pos).copied() {
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => self.string().map(JsonValue::String),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(b'-' | b'0'..=b'9') => {
                self.number()?;
                Ok(JsonValue::Number)
            }
            Some(other) => Err(format!(
                "unexpected byte 0x{other:02x} at byte {}",
                self.pos
            )),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.input.get(self.pos..self.pos + literal.len()) == Some(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(format!("invalid literal at byte {}", self.pos))
        }
    }

    fn array(&mut self, depth: usize) -> Result<JsonValue, String> {
        self.pos += 1; // [
        self.ws();
        let mut values = Vec::new();
        if self.input.get(self.pos) == Some(&b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(values));
        }
        loop {
            if values.len() >= Self::MAX_CONTAINER_ITEMS {
                return Err("JSON array exceeds 256 items".to_string());
            }
            values.push(self.value(depth + 1)?);
            self.ws();
            match self.input.get(self.pos) {
                Some(b',') => {
                    self.pos += 1;
                    self.ws();
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(values));
                }
                _ => return Err(format!("expected `,` or `]` at byte {}", self.pos)),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<JsonValue, String> {
        self.pos += 1; // {
        self.ws();
        let mut fields: Vec<(String, JsonValue)> = Vec::new();
        if self.input.get(self.pos) == Some(&b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(fields));
        }
        loop {
            if fields.len() >= Self::MAX_CONTAINER_ITEMS {
                return Err("JSON object exceeds 256 fields".to_string());
            }
            if self.input.get(self.pos) != Some(&b'"') {
                return Err(format!("expected an object key at byte {}", self.pos));
            }
            let key = self.string()?;
            if fields.iter().any(|(existing, _)| existing == &key) {
                return Err("duplicate object key".to_string());
            }
            self.ws();
            if self.input.get(self.pos) != Some(&b':') {
                return Err(format!("expected `:` at byte {}", self.pos));
            }
            self.pos += 1;
            let value = self.value(depth + 1)?;
            fields.push((key, value));
            self.ws();
            match self.input.get(self.pos) {
                Some(b',') => {
                    self.pos += 1;
                    self.ws();
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(fields));
                }
                _ => return Err(format!("expected `,` or `}}` at byte {}", self.pos)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        debug_assert_eq!(self.input.get(self.pos), Some(&b'"'));
        self.pos += 1;
        let mut out = String::new();
        loop {
            let Some(byte) = self.input.get(self.pos).copied() else {
                return Err("unterminated JSON string".to_string());
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return if out.len() <= Self::MAX_STRING_BYTES {
                        Ok(out)
                    } else {
                        Err("JSON string exceeds 65536 UTF-8 bytes".to_string())
                    };
                }
                b'\\' => {
                    self.pos += 1;
                    let Some(escape) = self.input.get(self.pos).copied() else {
                        return Err("unterminated JSON escape".to_string());
                    };
                    self.pos += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&first) {
                                if self.input.get(self.pos..self.pos + 2) != Some(b"\\u") {
                                    return Err(
                                        "high surrogate is not followed by a low surrogate".into(),
                                    );
                                }
                                self.pos += 2;
                                let second = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return Err(
                                        "high surrogate is not followed by a low surrogate".into(),
                                    );
                                }
                                0x10000
                                    + (((u32::from(first) - 0xd800) << 10)
                                        | (u32::from(second) - 0xdc00))
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err("unpaired low surrogate in JSON string".into());
                            } else {
                                u32::from(first)
                            };
                            out.push(
                                char::from_u32(scalar)
                                    .ok_or("invalid Unicode scalar in JSON string")?,
                            );
                        }
                        other => {
                            return Err(format!(
                                "invalid JSON escape byte 0x{other:02x} at byte {}",
                                self.pos - 1
                            ));
                        }
                    }
                }
                0x00..=0x1f => {
                    return Err(format!(
                        "unescaped control character in string at byte {}",
                        self.pos
                    ));
                }
                0x20..=0x7f => {
                    out.push(char::from(byte));
                    self.pos += 1;
                }
                _ => {
                    // The whole input was UTF-8-validated in `new`, so the
                    // next char is guaranteed to be complete.
                    let rest = std::str::from_utf8(&self.input[self.pos..]).unwrap();
                    let ch = rest.chars().next().unwrap();
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
            if out.len() > Self::MAX_STRING_BYTES {
                return Err("JSON string exceeds 65536 UTF-8 bytes".to_string());
            }
        }
    }

    fn hex4(&mut self) -> Result<u16, String> {
        let start = self.pos;
        let Some(bytes) = self.input.get(start..start + 4) else {
            return Err("truncated `\\u` escape".to_string());
        };
        let mut value = 0u16;
        for byte in bytes {
            let digit = match byte {
                b'0'..=b'9' => u16::from(*byte - b'0'),
                b'a'..=b'f' => u16::from(*byte - b'a' + 10),
                b'A'..=b'F' => u16::from(*byte - b'A' + 10),
                _ => return Err(format!("invalid `\\u` escape at byte {start}")),
            };
            value = value * 16 + digit;
        }
        self.pos += 4;
        Ok(value)
    }

    fn number(&mut self) -> Result<(), String> {
        let start = self.pos;
        if self.input.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }
        match self.input.get(self.pos) {
            Some(b'0') => {
                self.pos += 1;
                if self.input.get(self.pos).is_some_and(|b| b.is_ascii_digit()) {
                    return Err(format!("leading zero in number at byte {start}"));
                }
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while self.input.get(self.pos).is_some_and(|b| b.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            _ => return Err(format!("invalid number at byte {start}")),
        }
        if self.input.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            let fraction = self.pos;
            while self.input.get(self.pos).is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == fraction {
                return Err(format!("missing fraction digits at byte {start}"));
            }
        }
        if self
            .input
            .get(self.pos)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            self.pos += 1;
            if self
                .input
                .get(self.pos)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                self.pos += 1;
            }
            let exponent = self.pos;
            while self.input.get(self.pos).is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == exponent {
                return Err(format!("missing exponent digits at byte {start}"));
            }
        }
        Ok(())
    }
}

fn json_object(value: &JsonValue) -> Result<&[(String, JsonValue)], String> {
    match value {
        JsonValue::Object(fields) => Ok(fields),
        _ => Err("expected a JSON object".to_string()),
    }
}

fn json_field<'a>(fields: &'a [(String, JsonValue)], key: &str) -> Result<&'a JsonValue, String> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("missing `{key}`"))
}

fn required_string(fields: &[(String, JsonValue)], key: &str) -> Result<String, String> {
    match json_field(fields, key)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(format!("`{key}` must be a string")),
    }
}

fn required_tier(fields: &[(String, JsonValue)]) -> Result<String, String> {
    let tier = required_string(fields, "tier")?;
    match tier.as_str() {
        "official" | "brand" | "contrib" => Ok(tier),
        _ => Err("`tier` must be official, brand, or contrib".to_string()),
    }
}

fn optional_string(fields: &[(String, JsonValue)], key: &str) -> Result<Option<String>, String> {
    match json_field(fields, key)? {
        JsonValue::Null => Ok(None),
        JsonValue::String(value) => Ok(Some(value.clone())),
        _ => Err(format!("`{key}` must be a string or null")),
    }
}

fn required_bool(fields: &[(String, JsonValue)], key: &str) -> Result<bool, String> {
    match json_field(fields, key)? {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(format!("`{key}` must be a boolean")),
    }
}

fn required_array<'a>(
    fields: &'a [(String, JsonValue)],
    key: &str,
) -> Result<&'a [JsonValue], String> {
    match json_field(fields, key)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(format!("`{key}` must be an array")),
    }
}

fn parse_search_response(body: &[u8], expected_query: &str) -> Result<SearchResults, String> {
    // The endpoint itself is bounded, but refuse an unexpectedly huge body
    // before allocating a tree if a proxy or broken server violates that
    // contract.
    if body.len() > MAX_SEARCH_RESPONSE {
        return Err(format!(
            "response is {} bytes (maximum {})",
            body.len(),
            MAX_SEARCH_RESPONSE
        ));
    }
    let root = JsonParser::new(body)?.parse()?;
    let root = json_object(&root)?;
    let query = required_string(root, "query")?;
    if query != expected_query {
        return Err("response query does not match requested query".to_string());
    }

    let package_section = json_object(json_field(root, "packages")?)?;
    let mut packages = Vec::new();
    let package_values = required_array(package_section, "results")?;
    if package_values.len() > 20 {
        return Err("packages.results exceeds the default limit of 20".to_string());
    }
    for (index, value) in package_values.iter().enumerate() {
        let row = json_object(value).map_err(|e| format!("packages.results[{index}]: {e}"))?;
        packages.push(PackageSearchHit {
            name: required_string(row, "name")
                .map_err(|e| format!("packages.results[{index}]: {e}"))?,
            tier: required_tier(row).map_err(|e| format!("packages.results[{index}]: {e}"))?,
            latest: required_string(row, "latest")
                .map_err(|e| format!("packages.results[{index}]: {e}"))?,
            description: optional_string(row, "description")
                .map_err(|e| format!("packages.results[{index}]: {e}"))?,
            updated: required_string(row, "updated")
                .map_err(|e| format!("packages.results[{index}]: {e}"))?,
        });
    }
    let packages_has_more =
        required_bool(package_section, "has_more").map_err(|e| format!("packages: {e}"))?;

    let part_section = json_object(json_field(root, "parts")?)?;
    let mut parts = Vec::new();
    let part_values = required_array(part_section, "results")?;
    if part_values.len() > 20 {
        return Err("parts.results exceeds the default limit of 20".to_string());
    }
    for (index, value) in part_values.iter().enumerate() {
        let row = json_object(value).map_err(|e| format!("parts.results[{index}]: {e}"))?;
        parts.push(PartSearchHit {
            package: required_string(row, "package")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            tier: required_tier(row).map_err(|e| format!("parts.results[{index}]: {e}"))?,
            version: required_string(row, "version")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            fq: required_string(row, "fq").map_err(|e| format!("parts.results[{index}]: {e}"))?,
            name: required_string(row, "name")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            device: required_string(row, "device")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            intent: optional_string(row, "intent")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            manufacturer: optional_string(row, "manufacturer")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
            mpn: optional_string(row, "mpn").map_err(|e| format!("parts.results[{index}]: {e}"))?,
            primary: required_bool(row, "primary")
                .map_err(|e| format!("parts.results[{index}]: {e}"))?,
        });
    }
    let parts_has_more =
        required_bool(part_section, "has_more").map_err(|e| format!("parts: {e}"))?;

    Ok(SearchResults {
        query,
        packages,
        packages_has_more,
        parts,
        parts_has_more,
    })
}

// ---------------------------------------------------------------------------
// Package archive: uncompressed POSIX tar over the RFC-029 hash file set
// ---------------------------------------------------------------------------

fn tar_header(path: &str, size: u64) -> [u8; 512] {
    let mut h = [0u8; 512];
    let name = path.as_bytes();
    h[..name.len().min(100)].copy_from_slice(&name[..name.len().min(100)]);
    h[100..107].copy_from_slice(b"0000644"); // mode
    h[108..115].copy_from_slice(b"0000000"); // uid
    h[116..123].copy_from_slice(b"0000000"); // gid
    let size_oct = format!("{:011o}", size);
    h[124..124 + 11].copy_from_slice(size_oct.as_bytes());
    h[136..147].copy_from_slice(b"00000000000"); // mtime: epoch, deterministic
    h[156] = b'0'; // regular file
    h[257..262].copy_from_slice(b"ustar");
    h[263..265].copy_from_slice(b"00");
    // Checksum: spaces while summing, then written in octal.
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| *b as u32).sum();
    let chk = format!("{:06o}\0 ", sum);
    h[148..156].copy_from_slice(chk.as_bytes());
    h
}

/// Pack a package dir into a deterministic uncompressed tar: the RFC-029
/// hash file set (every regular file, dotfiles excluded), sorted by
/// `/`-normalized relative path, epoch mtimes.
pub fn pack_tar(dir: &Path) -> Result<Vec<u8>, String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::new();
    for (rel, path) in files {
        let content =
            std::fs::read(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        out.extend_from_slice(&tar_header(&rel, content.len() as u64));
        out.extend_from_slice(&content);
        let pad = (512 - content.len() % 512) % 512;
        out.extend(std::iter::repeat_n(0u8, pad));
    }
    out.extend(std::iter::repeat_n(0u8, 1024)); // end-of-archive
    Ok(out)
}

fn collect_files(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read `{}`: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in entries {
        let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') {
            continue;
        }
        if entry.is_dir() {
            collect_files(&entry, base, out)?;
        } else if entry.is_file() {
            let rel = entry
                .strip_prefix(base)
                .unwrap_or(&entry)
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel, entry));
        }
    }
    Ok(())
}

/// Unpack a plain tar into `dir`. Path-traversal-safe: every entry must stay
/// under the target (the same containment discipline the build artifacts
/// already enforce).
pub fn unpack_tar(data: &[u8], dir: &Path) -> Result<(), String> {
    let mut off = 0usize;
    while off + 512 <= data.len() {
        let h = &data[off..off + 512];
        if h.iter().all(|b| *b == 0) {
            break; // end-of-archive
        }
        let name_end = h[..100].iter().position(|b| *b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&h[..name_end]).map_err(|_| "bad tar entry name")?;
        let size_field = std::str::from_utf8(&h[124..136]).map_err(|_| "bad tar size")?;
        let size = usize::from_str_radix(size_field.trim_end_matches('\0').trim(), 8)
            .map_err(|_| "bad tar size")?;
        let kind = h[156];
        off += 512;
        if kind == b'0' || kind == 0 {
            if name.split('/').any(|seg| seg == ".." || seg.is_empty()) || name.starts_with('/') {
                return Err(format!("tar entry `{name}` escapes the target directory"));
            }
            let target = dir.join(name);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = data.get(off..off + size).ok_or("truncated tar entry")?;
            std::fs::write(&target, content).map_err(|e| e.to_string())?;
        }
        off += size.div_ceil(512) * 512;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// High-level registry operations
// ---------------------------------------------------------------------------

/// E1204 for callers outside this module (the CLI's `publish`/`login`
/// paths). A registry that was never reached — curl reports status 0 — is a
/// different mistake from a rejected publish (E1202) or a rejected token
/// (E1201), and must never be reported as one of those.
pub fn unreachable(detail: String) -> PackageDiag {
    e1204(detail)
}

fn e1204(detail: String) -> PackageDiag {
    PackageDiag::error("E1204", &registry_url(), 0, detail).with_help(
        "registry unreachable is a different failure from a hash mismatch (E1103) — check the network, COHDL_REGISTRY, or vendor the package under deps/".to_string(),
    )
}

/// Search never resolves project content, so the ordinary E1204 suggestion to
/// vendor a package is inapplicable. Keep the stable code/kind while giving
/// this read-only operation a relevant recovery step.
fn search_e1204(detail: String) -> PackageDiag {
    PackageDiag::error("E1204", &registry_url(), 0, detail).with_help(
        "check the network and COHDL_REGISTRY setting, then retry the search".to_string(),
    )
}

/// Search the hosted registry's package metadata and indexed public parts.
/// This is a read-only operation: it needs neither credentials nor a project.
pub fn search(query: &str) -> Result<SearchResults, PackageDiag> {
    let url = format!("{}/search", registry_url().trim_end_matches('/'));
    let resp = http_get_query(&url, &[("q", query)]).map_err(search_e1204)?;
    if resp.status != 200 {
        return Err(search_e1204(format!("GET {url} returned {}", resp.status)));
    }
    parse_search_response(&resp.body, query)
        .map_err(|error| search_e1204(format!("malformed response from GET {url}: {error}")))
}

/// Stable, byte-deterministic JSON for `cohdl search --json`. The registry is
/// decoded and then re-rendered rather than forwarded blindly, so a malformed
/// or duplicate-key response can never masquerade as valid CLI JSON.
pub fn render_search_json(search: &SearchResults) -> String {
    use std::fmt::Write as _;

    let string = crate::emit::json::json_str;
    let nullable = |value: &Option<String>| match value {
        Some(value) => string(value),
        None => "null".to_string(),
    };
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"query\": {},", string(&search.query));
    out.push_str("  \"packages\": {\n");
    if search.packages.is_empty() {
        out.push_str("    \"results\": [],\n");
    } else {
        out.push_str("    \"results\": [\n");
        for (index, package) in search.packages.iter().enumerate() {
            out.push_str("      {\n");
            let _ = writeln!(out, "        \"name\": {},", string(&package.name));
            let _ = writeln!(out, "        \"tier\": {},", string(&package.tier));
            let _ = writeln!(out, "        \"latest\": {},", string(&package.latest));
            let _ = writeln!(
                out,
                "        \"description\": {},",
                nullable(&package.description)
            );
            let _ = writeln!(out, "        \"updated\": {}", string(&package.updated));
            out.push_str(if index + 1 < search.packages.len() {
                "      },\n"
            } else {
                "      }\n"
            });
        }
        out.push_str("    ],\n");
    }
    let _ = writeln!(
        out,
        "    \"has_more\": {}",
        if search.packages_has_more {
            "true"
        } else {
            "false"
        }
    );
    out.push_str("  },\n");

    out.push_str("  \"parts\": {\n");
    if search.parts.is_empty() {
        out.push_str("    \"results\": [],\n");
    } else {
        out.push_str("    \"results\": [\n");
        for (index, part) in search.parts.iter().enumerate() {
            out.push_str("      {\n");
            let _ = writeln!(out, "        \"package\": {},", string(&part.package));
            let _ = writeln!(out, "        \"tier\": {},", string(&part.tier));
            let _ = writeln!(out, "        \"version\": {},", string(&part.version));
            let _ = writeln!(out, "        \"fq\": {},", string(&part.fq));
            let _ = writeln!(out, "        \"name\": {},", string(&part.name));
            let _ = writeln!(out, "        \"device\": {},", string(&part.device));
            let _ = writeln!(out, "        \"intent\": {},", nullable(&part.intent));
            let _ = writeln!(
                out,
                "        \"manufacturer\": {},",
                nullable(&part.manufacturer)
            );
            let _ = writeln!(out, "        \"mpn\": {},", nullable(&part.mpn));
            let _ = writeln!(
                out,
                "        \"primary\": {}",
                if part.primary { "true" } else { "false" }
            );
            out.push_str(if index + 1 < search.parts.len() {
                "      },\n"
            } else {
                "      }\n"
            });
        }
        out.push_str("    ],\n");
    }
    let _ = writeln!(
        out,
        "    \"has_more\": {}",
        if search.parts_has_more {
            "true"
        } else {
            "false"
        }
    );
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

/// All published versions of a package, newest-first.
pub fn published_versions(name: &str) -> Result<Vec<Version>, PackageDiag> {
    let url = format!("{}/packages/{}", registry_url(), name);
    let resp = http_get(&url).map_err(e1204)?;
    if resp.status == 404 {
        return Err(PackageDiag::error(
            "E1203",
            &registry_url(),
            0,
            format!("package `{name}` is not published on the registry"),
        ));
    }
    if resp.status != 200 {
        return Err(e1204(format!("GET {url} returned {}", resp.status)));
    }
    let mut versions: Vec<Version> = json_str_array(&resp.body, "versions")
        .iter()
        .filter_map(|v| crate::deps::parse_exact_version(v).ok())
        .collect();
    versions.sort();
    versions.reverse();
    Ok(versions)
}

/// Download one exact version into the cache; returns (package dir, the
/// registry's authoritative content hash). The unpacked content is
/// re-hashed locally and MUST match the server's hash before anything is
/// recorded (RFC-029's guarantee applies from the very first byte).
pub fn download_into_cache(name: &str, version: Version) -> Result<(PathBuf, String), PackageDiag> {
    let reg = registry_url();
    let meta_url = format!("{reg}/packages/{name}/{version}");
    let resp = http_get(&meta_url).map_err(e1204)?;
    if resp.status == 404 {
        return Err(PackageDiag::error(
            "E1203",
            &reg,
            0,
            format!("`{name} {version}` is not published on the registry"),
        ));
    }
    if resp.status != 200 {
        return Err(e1204(format!("GET {meta_url} returned {}", resp.status)));
    }
    let server_hash = json_str_field(&resp.body, "hash")
        .ok_or_else(|| e1204("registry response carries no `hash`".to_string()))?;

    let tar_url = format!("{reg}/packages/{name}/{version}.tar");
    let tar = http_get(&tar_url).map_err(e1204)?;
    if tar.status != 200 {
        return Err(e1204(format!("GET {tar_url} returned {}", tar.status)));
    }

    let cache = cache_root()
        .ok_or_else(|| e1204("cannot determine the cache dir (set HOME or COHDL_HOME)".into()))?;
    let dest = cache.join(name).join(version.to_string());
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&dest).map_err(|e| e1204(e.to_string()))?;
    unpack_tar(&tar.body, &dest).map_err(e1204)?;

    let local = crate::hash::package_content_hash(&dest).map_err(e1204)?;
    if local != server_hash {
        let _ = std::fs::remove_dir_all(&dest);
        return Err(PackageDiag::error(
            "E1206",
            &reg,
            0,
            format!(
                "downloaded `{name} {version}` re-hashes as {local}, but the registry declares {server_hash} — refusing to cache corrupted content"
            ),
        ));
    }
    Ok((dest, server_hash))
}

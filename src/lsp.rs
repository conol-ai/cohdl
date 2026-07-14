//! RFC-014: `cohdl lsp` — a thin Language Server Protocol frontend over the
//! existing pipeline.
//!
//! Architecture (DR-020): the JSON-RPC/stdio transport loop is hand-rolled
//! (Content-Length framing below, consistent with the project's style); the
//! protocol's message *shapes* come from the `lsp-types` crate — the project's
//! single scoped dependency exception, because the LSP spec's type surface is
//! large and externally versioned, unlike CoHDL's own small fixed formats.
//!
//! Every request re-runs the exact same `pipeline::check_files` the CLI uses —
//! zero new diagnostic logic, no parallel reimplementation (the RFC's
//! equivalence discipline: the LSP's diagnostics must match `cohdl check
//! --json` field-for-field; see tests/lsp.rs).
//!
//! Capabilities (exactly the four RFC-014 names, no more):
//! - `textDocument/publishDiagnostics` — re-projection of the RFC-010 output.
//! - `textDocument/hover` — resolved by-name mappings on an empty `impl`
//!   block; obligation/role on a pin declaration (both pre-named by DR-013).
//! - `textDocument/definition` — device/trait/fn/part name at a use site
//!   resolves to its declaration.
//! - `textDocument/references` — on a trait/device name: every `impl`
//!   statement involving it.

use crate::ast::*;
use crate::pipeline::{self, Checked};
use crate::span::{FileId, Span};
use lsp_types as lt;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Run the server on stdio until `exit`. The return distinguishes a clean
/// `shutdown`-then-`exit` (Ok) from an abrupt stream end.
pub fn run_stdio() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut server = Server {
        overlays: BTreeMap::new(),
        published: BTreeSet::new(),
        touched: None,
        shutdown_requested: false,
    };
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    loop {
        let Some(msg) = read_message(&mut reader)? else {
            // Stream closed without `exit` — treat as done.
            return Ok(());
        };
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        if method == "exit" {
            // LSP: exit after shutdown is clean; exit without it is not.
            return if server.shutdown_requested {
                Ok(())
            } else {
                Err("lsp: `exit` received before `shutdown`".to_string())
            };
        }
        if let Some(response) = server.handle(method, &msg) {
            write_message(&mut writer, &response)?;
        }
        // Diagnostics publishing rides after the triggering message.
        for note in server.take_pending_publishes() {
            write_message(&mut writer, &note)?;
        }
    }
}

// ---------------------------------------------------------------------------
// Transport: hand-rolled JSON-RPC framing (Content-Length header + payload).

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("lsp: read error: {}", e))?;
        if n == 0 {
            return Ok(None); // EOF
        }
        let line = line.trim_end();
        if line.is_empty() {
            break; // end of headers
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            content_length = v.trim().parse().ok();
        }
    }
    let len = content_length.ok_or("lsp: missing Content-Length header")?;
    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("lsp: payload read error: {}", e))?;
    let value: Value = serde_json::from_slice(&buf)
        .map_err(|e| format!("lsp: malformed JSON-RPC payload: {}", e))?;
    Ok(Some(value))
}

fn write_message(writer: &mut impl Write, value: &Value) -> Result<(), String> {
    let payload = serde_json::to_string(value).map_err(|e| e.to_string())?;
    write!(
        writer,
        "Content-Length: {}\r\n\r\n{}",
        payload.len(),
        payload
    )
    .and_then(|_| writer.flush())
    .map_err(|e| format!("lsp: write error: {}", e))
}

// ---------------------------------------------------------------------------
// The server.

struct Server {
    /// Unsaved buffer contents, keyed by canonical file path — these override
    /// on-disk contents when the project is loaded.
    overlays: BTreeMap<PathBuf, String>,
    /// URIs we last published diagnostics for (so stale ones self-clear).
    published: BTreeSet<String>,
    /// The document URI touched by the last did* notification — triggers a
    /// re-check + publish after the message is handled.
    touched: Option<String>,
    shutdown_requested: bool,
}

/// One analyzed project: the pipeline result plus FileId → absolute path.
struct Analysis {
    checked: Checked,
    abs_paths: Vec<PathBuf>,
}

impl Server {
    /// Handle one message; `Some(response)` for requests, `None` for
    /// notifications. Publishing is deferred to `take_pending_publishes`.
    fn handle(&mut self, method: &str, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match method {
            "initialize" => Some(respond(
                id?,
                json!({
                    "capabilities": {
                        "textDocumentSync": 1, // FULL — didChange carries the whole text
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "referencesProvider": true,
                    },
                    "serverInfo": { "name": "cohdl-lsp", "version": env!("CARGO_PKG_VERSION") },
                }),
            )),
            "initialized" => None,
            "shutdown" => {
                self.shutdown_requested = true;
                Some(respond(id?, Value::Null))
            }
            "textDocument/didOpen" => {
                let uri = params["textDocument"]["uri"].as_str()?.to_string();
                let text = params["textDocument"]["text"].as_str()?.to_string();
                if let Some(path) = uri_to_path(&uri) {
                    self.overlays.insert(path, text);
                }
                self.touched = Some(uri);
                None
            }
            "textDocument/didChange" => {
                let uri = params["textDocument"]["uri"].as_str()?.to_string();
                // FULL sync: the last contentChanges entry carries the text.
                let text = params["contentChanges"]
                    .as_array()?
                    .last()?
                    .get("text")?
                    .as_str()?
                    .to_string();
                if let Some(path) = uri_to_path(&uri) {
                    self.overlays.insert(path, text);
                }
                self.touched = Some(uri);
                None
            }
            "textDocument/didSave" => {
                let uri = params["textDocument"]["uri"].as_str()?.to_string();
                self.touched = Some(uri);
                None
            }
            "textDocument/didClose" => {
                let uri = params["textDocument"]["uri"].as_str()?.to_string();
                if let Some(path) = uri_to_path(&uri) {
                    self.overlays.remove(&path);
                }
                self.touched = Some(uri);
                None
            }
            "textDocument/hover" => {
                let result = self.hover(&params).map_or(Value::Null, |h| {
                    serde_json::to_value(h).unwrap_or(Value::Null)
                });
                Some(respond(id?, result))
            }
            "textDocument/definition" => {
                let result = self.definition(&params).map_or(Value::Null, |l| {
                    serde_json::to_value(l).unwrap_or(Value::Null)
                });
                Some(respond(id?, result))
            }
            "textDocument/references" => {
                let locs = self.references(&params);
                Some(respond(
                    id?,
                    serde_json::to_value(locs).unwrap_or(Value::Null),
                ))
            }
            _ => {
                // Unknown REQUESTS get MethodNotFound; notifications ignored.
                id.map(|id| {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": format!("method not found: {}", method) },
                    })
                })
            }
        }
    }
}

fn respond(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

// The `touched` field lives outside `handle`'s match for borrow simplicity.
impl Server {
    fn take_pending_publishes(&mut self) -> Vec<Value> {
        let Some(uri) = self.touched.take() else {
            return Vec::new();
        };
        let Some(path) = uri_to_path(&uri) else {
            return Vec::new();
        };
        let Some(analysis) = self.analyze(&path) else {
            return Vec::new();
        };
        // Group diagnostics per file, then publish: every file with
        // diagnostics + the touched file, and clear anything stale.
        let mut per_file: BTreeMap<u32, Vec<Value>> = BTreeMap::new();
        for d in analysis.checked.diags.iter() {
            let fid = d.primary.span.file;
            per_file
                .entry(fid.0)
                .or_default()
                .push(lsp_diagnostic(&analysis, d));
        }
        let mut notes = Vec::new();
        let mut now_published: BTreeSet<String> = BTreeSet::new();
        for (fid, diags) in &per_file {
            if let Some(u) = analysis.uri_for(FileId(*fid)) {
                let u = u.as_str().to_string();
                notes.push(publish_note(&u, diags.clone()));
                now_published.insert(u);
            }
        }
        // The touched file always gets a publish (possibly empty).
        if !now_published.contains(&uri) {
            notes.push(publish_note(&uri, Vec::new()));
            now_published.insert(uri);
        }
        // Clear diagnostics for files that had them before but not now.
        for stale in self.published.difference(&now_published) {
            notes.push(publish_note(stale, Vec::new()));
        }
        self.published = now_published;
        notes
    }

    /// Locate the project containing `path` (walk up for `cohdl.toml`, else
    /// single-file), load it with overlays applied, and run the exact same
    /// `pipeline::check_files` the CLI uses.
    fn analyze(&self, path: &Path) -> Option<Analysis> {
        let project_root = path
            .ancestors()
            .skip(1)
            .find(|a| a.join("cohdl.toml").is_file())
            .map(Path::to_path_buf);
        let std_dir = crate::project::find_std_dir(None);
        let target: &Path = project_root.as_deref().unwrap_or(path);
        let mut proj = crate::project::load_project(target, std_dir.as_deref()).ok()?;
        // Overlays: unsaved buffer contents win over the disk.
        for (i, abs) in proj.abs_paths.iter().enumerate() {
            if let Some(text) = self.overlays.get(abs) {
                proj.files[i].1 = text.clone();
            }
        }
        let checked = pipeline::check_files(&proj.files, proj.top.as_deref()).ok()?;
        Some(Analysis {
            checked,
            abs_paths: proj.abs_paths,
        })
    }

    // -- hover ---------------------------------------------------------------

    fn hover(&mut self, params: &Value) -> Option<lt::Hover> {
        let (analysis, fid, offset) = self.locate(params)?;
        let world = &analysis.checked.world;

        // Pin declaration hover: obligation + role (+ physical pads).
        for dev in world.devices.values() {
            for pb in &dev.pin_blocks {
                for pin in &pb.pins {
                    if contains(pin.name.span, fid, offset) {
                        let role = pin.role.map(|(r, _)| r.name()).unwrap_or("(missing role)");
                        let pads = pin
                            .numbers
                            .iter()
                            .map(|n| n.text.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let variant = pb
                            .variant
                            .as_ref()
                            .map(|v| format!(" (variant `{}`)", v.name))
                            .unwrap_or_default();
                        return Some(hover_markdown(
                            format!(
                                "**{} pin** `{}` on device `{}`{}\n\n- role: `{}`\n- pads: {}",
                                pin.obligation.keyword(),
                                pin.name.name,
                                dev.name.name,
                                variant,
                                role,
                                pads
                            ),
                            span_to_range(&analysis, pin.name.span),
                        ));
                    }
                }
            }
        }
        // Trait pin hover: obligation (abstract role — no pin role).
        for tr in world.traits.values() {
            for pin in &tr.pins {
                if contains(pin.name.span, fid, offset) {
                    return Some(hover_markdown(
                        format!(
                            "**{} pin role** `{}` on trait `{}` (abstract — mapped per `impl`)",
                            pin.obligation.keyword(),
                            pin.name.name,
                            tr.name.name
                        ),
                        span_to_range(&analysis, pin.name.span),
                    ));
                }
            }
        }
        // Empty-impl hover: the resolved by-name mapping DR-013 asked for.
        for imp in &world.impls {
            if contains(imp.span, fid, offset) {
                let key = (imp.trait_name.name.clone(), imp.device_name.name.clone());
                let mut text = format!(
                    "**impl** `{}` **for** `{}`",
                    imp.trait_name.name, imp.device_name.name
                );
                match world.resolved_impls.get(&key) {
                    Some(resolved) => {
                        if !resolved.pin_map.is_empty() {
                            text.push_str("\n\npins:");
                            for (role, pin) in &resolved.pin_map {
                                text.push_str(&format!("\n- `{}` ← `{}`", role, pin));
                            }
                        }
                        if !resolved.spec_map.is_empty() {
                            text.push_str("\n\nspec:");
                            for (field, target) in &resolved.spec_map {
                                text.push_str(&format!("\n- `{}` ← `{}`", field, target));
                            }
                        }
                        if resolved.pin_map.is_empty() && resolved.spec_map.is_empty() {
                            text.push_str("\n\n(no pins or spec fields required)");
                        }
                    }
                    None => text.push_str("\n\n(unresolved — see diagnostics)"),
                }
                return Some(hover_markdown(text, span_to_range(&analysis, imp.span)));
            }
        }
        None
    }

    // -- goto-definition -------------------------------------------------------

    fn definition(&mut self, params: &Value) -> Option<lt::Location> {
        let (analysis, fid, offset) = self.locate(params)?;
        let name = use_site_name(&analysis.checked.world, fid, offset)?;
        let world = &analysis.checked.world;
        let target = world
            .devices
            .get(&name)
            .map(|d| d.name.span)
            .or_else(|| world.traits.get(&name).map(|t| t.name.span))
            .or_else(|| world.fns.get(&name).map(|f| f.name.span))
            .or_else(|| world.parts.get(&name).map(|p| p.name.span))?;
        Some(lt::Location {
            uri: analysis.uri_for(target.file)?,
            range: span_to_range(&analysis, target),
        })
    }

    // -- references (find all impls) -------------------------------------------

    fn references(&mut self, params: &Value) -> Vec<lt::Location> {
        let Some((analysis, fid, offset)) = self.locate(params) else {
            return Vec::new();
        };
        let world = &analysis.checked.world;
        // What name is under the cursor — a trait or device, from either an
        // impl statement or the declaration itself?
        let mut trait_name = None;
        let mut device_name = None;
        for imp in &world.impls {
            if contains(imp.trait_name.span, fid, offset) {
                trait_name = Some(imp.trait_name.name.clone());
            }
            if contains(imp.device_name.span, fid, offset) {
                device_name = Some(imp.device_name.name.clone());
            }
        }
        for tr in world.traits.values() {
            if contains(tr.name.span, fid, offset) {
                trait_name = Some(tr.name.name.clone());
            }
        }
        for dev in world.devices.values() {
            if contains(dev.name.span, fid, offset) {
                device_name = Some(dev.name.name.clone());
            }
        }
        let mut locs = Vec::new();
        for imp in &world.impls {
            let matches = trait_name
                .as_deref()
                .is_some_and(|t| imp.trait_name.name == t)
                || device_name
                    .as_deref()
                    .is_some_and(|d| imp.device_name.name == d);
            if matches {
                if let Some(uri) = analysis.uri_for(imp.span.file) {
                    locs.push(lt::Location {
                        uri,
                        range: span_to_range(&analysis, imp.span),
                    });
                }
            }
        }
        locs
    }

    /// Shared request plumbing: analyze the document's project and convert the
    /// LSP position to (FileId, byte offset).
    fn locate(&mut self, params: &Value) -> Option<(Analysis, FileId, u32)> {
        let uri = params["textDocument"]["uri"].as_str()?;
        let path = uri_to_path(uri)?;
        let analysis = self.analyze(&path)?;
        let fid = analysis.fid_for(&path)?;
        let line = params["position"]["line"].as_u64()? as u32;
        let character = params["position"]["character"].as_u64()? as u32;
        let offset = position_to_offset(&analysis, fid, line, character)?;
        Some((analysis, fid, offset))
    }
}

impl Analysis {
    fn uri_for(&self, fid: FileId) -> Option<lt::Uri> {
        let path = self.abs_paths.get(fid.0 as usize)?;
        format!("file://{}", path.display()).parse().ok()
    }

    fn fid_for(&self, path: &Path) -> Option<FileId> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.abs_paths
            .iter()
            .position(|p| *p == canonical)
            .map(|i| FileId(i as u32))
    }
}

// ---------------------------------------------------------------------------
// Position/span conversions (LSP is 0-based lines + UTF-16 columns; the
// SourceMap is 1-based lines + Unicode-scalar columns).

fn span_to_range(analysis: &Analysis, span: Span) -> lt::Range {
    lt::Range {
        start: offset_to_position(analysis, span.file, span.start),
        end: offset_to_position(analysis, span.file, span.end),
    }
}

fn offset_to_position(analysis: &Analysis, fid: FileId, offset: u32) -> lt::Position {
    let lc = analysis.checked.sm.line_col(fid, offset);
    let line_text = analysis.checked.sm.line_text(fid, lc.line);
    // Scalar column → UTF-16 code units.
    let utf16: usize = line_text
        .chars()
        .take((lc.col - 1) as usize)
        .map(char::len_utf16)
        .sum();
    lt::Position {
        line: lc.line - 1,
        character: utf16 as u32,
    }
}

fn position_to_offset(analysis: &Analysis, fid: FileId, line: u32, character: u32) -> Option<u32> {
    let text = analysis.checked.sm.text(fid);
    let mut current = 0u32;
    for (i, l) in text.split('\n').enumerate() {
        if i as u32 == line {
            // UTF-16 column → byte offset within the line.
            let mut units = 0u32;
            let mut bytes = 0usize;
            for ch in l.chars() {
                if units >= character {
                    break;
                }
                units += ch.len_utf16() as u32;
                bytes += ch.len_utf8();
            }
            return Some(current + bytes as u32);
        }
        current += l.len() as u32 + 1;
    }
    None
}

fn contains(span: Span, fid: FileId, offset: u32) -> bool {
    span.file == fid && span.start <= offset && offset < span.end.max(span.start + 1)
}

fn hover_markdown(text: String, range: lt::Range) -> lt::Hover {
    lt::Hover {
        contents: lt::HoverContents::Markup(lt::MarkupContent {
            kind: lt::MarkupKind::Markdown,
            value: text,
        }),
        range: Some(range),
    }
}

// ---------------------------------------------------------------------------
// Diagnostics mapping (the RFC-010 equivalence surface).

fn lsp_diagnostic(analysis: &Analysis, d: &crate::diag::Diagnostic) -> Value {
    let severity = match d.severity {
        crate::diag::Severity::Error => 1,
        crate::diag::Severity::Warning => 2,
    };
    let mut related = Vec::new();
    for sec in &d.secondary {
        if let Some(uri) = analysis.uri_for(sec.span.file) {
            related.push(json!({
                "location": { "uri": uri.as_str(), "range": span_to_range(analysis, sec.span) },
                "message": sec.message,
            }));
        }
    }
    // Help lines ride relatedInformation too (anchored at the primary range) —
    // the four equivalence fields (range/severity/code/message) stay exactly
    // the RFC-010 values.
    for h in &d.help {
        if let Some(uri) = analysis.uri_for(d.primary.span.file) {
            related.push(json!({
                "location": { "uri": uri.as_str(), "range": span_to_range(analysis, d.primary.span) },
                "message": format!("help: {}", h),
            }));
        }
    }
    json!({
        "range": span_to_range(analysis, d.primary.span),
        "severity": severity,
        "code": d.code,
        "source": "cohdl",
        "message": d.message,
        "relatedInformation": related,
    })
}

fn publish_note(uri: &str, diagnostics: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics },
    })
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    // Minimal percent-decoding for the common cases (space).
    let decoded = raw.replace("%20", " ");
    let p = PathBuf::from(decoded);
    Some(p.canonicalize().unwrap_or(p))
}

// ---------------------------------------------------------------------------
// Use-site identification for goto-definition.

/// The device/trait/fn/part name whose USE SITE contains the offset.
fn use_site_name(world: &crate::resolve::World, fid: FileId, offset: u32) -> Option<String> {
    let hit = |id: &Ident| contains(id.span, fid, offset);

    // impl statements: both names are use sites.
    for imp in &world.impls {
        if hit(&imp.trait_name) {
            return Some(imp.trait_name.name.clone());
        }
        if hit(&imp.device_name) {
            return Some(imp.device_name.name.clone());
        }
    }
    // Trait super-trait bounds.
    for tr in world.traits.values() {
        for sup in &tr.super_traits {
            if hit(sup) {
                return Some(sup.name.clone());
            }
        }
    }
    // Generic bounds on devices and fns.
    let bound_hit = |generics: &[GenericParam]| -> Option<String> {
        for g in generics {
            if let GenericBound::Traits(ts) = &g.bound {
                for t in ts {
                    if hit(t) {
                        return Some(t.name.clone());
                    }
                }
            }
        }
        None
    };
    for dev in world.devices.values() {
        if let Some(n) = bound_hit(&dev.generics) {
            return Some(n);
        }
    }
    for f in world.fns.values() {
        if let Some(n) = bound_hit(&f.generics) {
            return Some(n);
        }
        for p in &f.params {
            if let FnParamTy::ImplTrait(ts, _) = &p.ty {
                for t in ts {
                    if hit(t) {
                        return Some(t.name.clone());
                    }
                }
            }
        }
        if let Some(n) = body_use_site(&f.body, &hit) {
            return Some(n);
        }
    }
    // Part device references.
    for part in world.parts.values() {
        if hit(&part.device.name) {
            return Some(part.device.name.name.clone());
        }
    }
    // Design bodies: inst types and fn calls.
    for design in world.designs.values() {
        if let Some(n) = body_use_site(&design.body, &hit) {
            return Some(n);
        }
    }
    None
}

fn body_use_site(body: &[Stmt], hit: &impl Fn(&Ident) -> bool) -> Option<String> {
    for stmt in body {
        match stmt {
            Stmt::Inst(s) => {
                if hit(&s.ty.name) {
                    return Some(s.ty.name.name.clone());
                }
                for arg in &s.ty.generic_args {
                    if let GenericArg::Name(id) = arg {
                        if hit(id) {
                            return Some(id.name.clone());
                        }
                    }
                }
            }
            Stmt::Call(s) if hit(&s.callee) => {
                return Some(s.callee.name.clone());
            }
            _ => {}
        }
    }
    None
}

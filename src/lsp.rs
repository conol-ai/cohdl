//! RFC-014: `cohdl lsp` — a thin Language Server Protocol frontend over the
//! existing pipeline.
//!
//! Architecture (DR-020): the JSON-RPC/stdio transport loop is hand-rolled
//! (Content-Length framing below, consistent with the project's style).
//! `lsp-types` — the project's single scoped dependency exception — supplies
//! the typed response shapes (`Hover`, `Location`, `Range`, `Position`,
//! `Uri`); the request envelopes, dispatch, and publishDiagnostics payloads
//! are raw `serde_json` values (an honest narrowing of DR-020's original
//! framing — see docs/compliance-report.md, review R10).
//!
//! Diagnostics come from the exact same `pipeline::check_files_in_with_deps`
//! the CLI uses — the same `Checked.diags` source, independently projected
//! into LSP shape here (`lsp_diagnostic`; the RFC's equivalence discipline is
//! that the four fields code/severity/message/range must match `cohdl check
//! --json`, enforced in tests/lsp.rs).
//!
//! Scope: POSIX hosts only — `file://` URIs with empty/`localhost`
//! authority; Windows drive/UNC forms are not supported.
//!
//! Capabilities (exactly the four RFC-014 names, no more):
//! - `textDocument/publishDiagnostics` — re-projection of the RFC-010 output.
//! - `textDocument/hover` — resolved by-name mappings on an empty `impl`
//!   block; obligation/role on a pin declaration or pin USE SITE (RFC-002);
//!   RFC-001's allowed-prefix row on a unit literal.
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

/// How the server session ended — the LSP spec assigns exit codes: 0 for
/// `exit` after `shutdown`, 1 for `exit` without it.
pub enum LspExit {
    Clean,
    WithoutShutdown,
}

/// Run the server on stdio until `exit`.
pub fn run_stdio() -> Result<LspExit, String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut server = Server {
        overlays: BTreeMap::new(),
        published: BTreeMap::new(),
        client_uris: BTreeMap::new(),
        touched: None,
        state: Lifecycle::PreInit,
        related_info: false,
    };
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    loop {
        let msg = match read_message(&mut reader) {
            Ok(Read::Message(m)) => m,
            Ok(Read::Eof) => {
                // Stream closed without `exit` — treat as done.
                return Ok(LspExit::Clean);
            }
            Ok(Read::Malformed(detail)) => {
                // JSON-RPC 2.0 §5.1: a parse error is an error RESPONSE
                // (id null), never process death — the editor session (and
                // its unsaved-buffer overlays) survives one corrupt frame.
                write_message(
                    &mut writer,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {}", detail) },
                    }),
                )?;
                continue;
            }
            Err(io) => return Err(io), // genuine I/O failure
        };
        // JSON-RPC 2.0 envelope validation (review F10): a frame must carry
        // `"jsonrpc": "2.0"` and a string `method`. A malformed request (has
        // an id) gets InvalidRequest; a malformed notification is dropped.
        // A JSON-RPC id must be a string, number, or null (JSON-RPC 2.0 §4)
        // — an object/array/bool id is a malformed request (review R5-11).
        let id_val = msg.get("id");
        let id_type_ok = match id_val {
            None => true,
            Some(v) => v.is_string() || v.is_number() || v.is_null(),
        };
        let version_ok = msg.get("jsonrpc").and_then(Value::as_str) == Some("2.0");
        let method_val = msg.get("method").and_then(Value::as_str);
        if !version_ok || method_val.is_none() || !id_type_ok {
            // Respond whenever an `id` FIELD is present — a request, even a
            // malformed one carrying `"id": null`, is not a notification
            // (which OMITS id entirely). Distinguishing presence from
            // non-null (review R6-7) stops a bad-envelope null-id request from
            // being silently dropped. Echo the id only when it is a valid
            // string/number — an invalid-type or null id cannot be a response
            // id, so null (JSON-RPC 2.0 §5).
            if id_val.is_some() {
                let resp_id = match id_val {
                    Some(v) if v.is_string() || v.is_number() => v.clone(),
                    _ => Value::Null,
                };
                write_message(
                    &mut writer,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": resp_id,
                        "error": {
                            "code": -32600,
                            "message": "Invalid Request: expected jsonrpc \"2.0\", a string method, and a string/number/null id",
                        },
                    }),
                )?;
            }
            continue;
        }
        let method = method_val.unwrap_or("");
        // Method-shape validation (review R7-6): the id FIELD's presence must
        // match the method's direction. A NOTIFICATION-only method presented
        // as a request (id field present) is InvalidRequest and must NOT
        // perform the notification action; a REQUEST-only method with no id is
        // likewise InvalidRequest. Unknown methods fall through to `handle`.
        let id_present = msg.get("id").is_some();
        let shape_error = (is_notification_method(method) && id_present)
            || (is_request_method(method) && !id_present);
        if shape_error {
            if id_present {
                let resp_id = match id_val {
                    Some(v) if v.is_string() || v.is_number() => v.clone(),
                    _ => Value::Null,
                };
                write_message(
                    &mut writer,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": resp_id,
                        "error": {
                            "code": -32600,
                            "message": format!("Invalid Request: `{}` is a notification and takes no id", method),
                        },
                    }),
                )?;
            }
            // A request-only method with no id has no id to answer under; drop.
            continue;
        }
        if method == "exit" {
            // LSP: exit after shutdown is clean; exit without it is not.
            return if matches!(server.state, Lifecycle::ShutDown) {
                Ok(LspExit::Clean)
            } else {
                Ok(LspExit::WithoutShutdown)
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

/// LSP methods this server treats as REQUESTS (expect an id + a response).
fn is_request_method(method: &str) -> bool {
    matches!(
        method,
        "initialize"
            | "shutdown"
            | "textDocument/hover"
            | "textDocument/definition"
            | "textDocument/references"
            | "textDocument/completion"
    )
}

/// LSP methods this server treats as NOTIFICATIONS (no id, no response).
fn is_notification_method(method: &str) -> bool {
    matches!(
        method,
        "initialized"
            | "exit"
            | "textDocument/didOpen"
            | "textDocument/didChange"
            | "textDocument/didSave"
            | "textDocument/didClose"
    )
}

// ---------------------------------------------------------------------------
// Transport: hand-rolled JSON-RPC framing (Content-Length header + payload).

/// One framed read: a message, EOF, or a recoverable framing/parse problem
/// (the frame boundary is known, so the session continues).
enum Read {
    Message(Value),
    Eof,
    Malformed(String),
}

fn read_message(reader: &mut impl BufRead) -> Result<Read, String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("lsp: read error: {}", e))?;
        if n == 0 {
            return Ok(Read::Eof);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break; // end of headers
        }
        // LSP headers follow HTTP field semantics: names are case-insensitive.
        if let Some((name, v)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = v.trim().parse().ok();
            }
        }
    }
    let Some(len) = content_length else {
        // The header block was cleanly delimited; resume at the next one.
        // Honest limitation: if a BODY followed the corrupt headers, its
        // bytes are misread as the next frame's headers — recovery is
        // best-effort at blank-line boundaries, not guaranteed resync.
        return Ok(Read::Malformed("missing Content-Length header".to_string()));
    };
    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("lsp: payload read error: {}", e))?;
    match serde_json::from_slice(&buf) {
        Ok(value) => Ok(Read::Message(value)),
        // The payload was fully consumed — recoverable.
        Err(e) => Ok(Read::Malformed(format!("invalid JSON payload: {}", e))),
    }
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

/// LSP 3.17 lifecycle: requests before `initialize` get -32002; a second
/// `initialize` and any request after `shutdown` get InvalidRequest.
enum Lifecycle {
    PreInit,
    Running,
    ShutDown,
}

struct Server {
    /// Unsaved buffer contents, keyed by canonical file path — these override
    /// on-disk contents when the project is loaded.
    overlays: BTreeMap<PathBuf, String>,
    /// Diagnostic ownership, keyed by analysis unit (project root, or the
    /// file itself for loose files): the URIs last published under that key.
    /// Re-analyzing one project must never clear another's diagnostics
    /// (review R7 — multi-root sessions).
    published: BTreeMap<String, BTreeSet<String>>,
    /// The client's own URI spelling per canonical path (didOpen/didChange).
    /// Publishes for an open document use the CLIENT's spelling — its
    /// percent-encoding or `localhost` authority may differ from ours, and
    /// the editor keys diagnostics by the URI it opened.
    client_uris: BTreeMap<PathBuf, String>,
    /// The document URI touched by the last did* notification — triggers a
    /// re-check + publish after the message is handled.
    touched: Option<String>,
    state: Lifecycle,
    /// Did the client advertise `publishDiagnostics.relatedInformation`?
    /// Secondary/help locations are only attached when it did.
    related_info: bool,
}

/// One analyzed project: the pipeline result plus FileId → absolute path.
struct Analysis {
    checked: Checked,
    abs_paths: Vec<PathBuf>,
    /// Diagnostic-ownership key (see `Server::published`).
    key: String,
}

/// Why a document could not be analyzed (review R6: these must not be
/// conflated — one is "nothing to check", the other is a real failure the
/// editor has to see instead of a false-clean empty publish).
enum AnalyzeError {
    /// The file neither exists on disk nor has an overlay (e.g. a phantom
    /// buffer after `didClose`) — clear its diagnostics and move on.
    Gone,
    /// A real project/std load or pipeline failure — surface it; never
    /// publish an empty list that claims the file is clean.
    Project(String),
}

/// Resolve one manifest project's direct dependency set with the CLI's
/// RFC-029 rules. The LSP has no `--no-std`: std must therefore be pinned
/// unless `COHDL_STD` supplies the std-only development override. Every other
/// dependency still resolves through project-local `deps/`, the shipped
/// library root, then the content cache, with cohdl.lock hashes verified.
///
/// Direct dependencies only: transitive manifest traversal is intentionally
/// outside this prerequisite.
fn resolve_manifest_deps(project_root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    use crate::deps;

    let (manifest_path, manifest) = crate::project::peek_manifest(project_root)?;
    let manifest_display = manifest_path.display().to_string();
    let override_std = crate::project::std_override(None);

    // Preserve the pre-RFC-029 development workflow used by editor tests and
    // local checkouts: an explicit std override needs no manifest pin.
    let Some(deps_raw) = &manifest.deps_raw else {
        if let Some(dir) = override_std {
            return Ok(vec![("std".to_string(), dir)]);
        }
        let newest = crate::project::newest_std()
            .map(|(v, _)| v.to_string())
            .unwrap_or_else(|| "X.Y.Z".to_string());
        let diag = deps::PackageDiag::error(
            "E1104",
            &manifest_display,
            0,
            "this project declares no `[dependencies]` — RFC-029 requires an exact std pin"
                .to_string(),
        )
        .with_help(format!(
            "add:\n           [dependencies]\n           std = \"{newest}\""
        ))
        .with_help("or run `cohdl update` to write it automatically".to_string());
        return Err(deps::render_human(&[diag]));
    };

    let mut entries =
        deps::validate_deps(&manifest_display, deps_raw).map_err(|d| deps::render_human(&d))?;
    if !entries.iter().any(|e| e.name == "std") && override_std.is_none() {
        let newest = crate::project::newest_std()
            .map(|(v, _)| v.to_string())
            .unwrap_or_else(|| "X.Y.Z".to_string());
        let diag = deps::PackageDiag::error(
            "E1104",
            &manifest_display,
            0,
            "`[dependencies]` has no `std` entry — every project implicitly depends on std and must pin its exact version"
                .to_string(),
        )
        .with_help(format!(
            "add `std = \"{newest}\"` under [dependencies]"
        ));
        return Err(deps::render_human(&[diag]));
    }

    // The override replaces std only. Non-std entries retain ordinary lock
    // verification and registry resolution.
    let mut resolved_deps = Vec::new();
    if let Some(dir) = &override_std {
        entries.retain(|e| e.name != "std");
        resolved_deps.push(("std".to_string(), dir.clone()));
    }

    let registry = deps::Registry {
        lib_root: crate::project::find_lib_root(),
        project_deps: project_root.join("deps"),
        cache_root: crate::registry::cache_root(),
    };
    let lock_path = project_root.join("cohdl.lock");
    let lock_display = lock_path.display().to_string();
    let prior_lock_text = std::fs::read_to_string(&lock_path).ok();
    let resolution = deps::resolve(
        &manifest_display,
        &lock_display,
        &entries,
        &registry,
        prior_lock_text.as_deref(),
        deps::Update::No,
    )
    .map_err(|d| deps::render_human(&d))?;

    // Match the CLI's first-resolution/version-change behavior, including its
    // refusal to replace a symlinked lockfile.
    if resolution.lock_changed {
        write_lock_file(&lock_path, &resolution.lock.render())?;
    }

    // Keep std first for the pipeline's established file order, followed by
    // every other direct dependency in stable name order.
    let mut rest = resolution.deps;
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    resolved_deps.extend(rest);
    if let Some(pos) = resolved_deps.iter().position(|(name, _)| name == "std") {
        let std_entry = resolved_deps.remove(pos);
        resolved_deps.insert(0, std_entry);
    }
    Ok(resolved_deps)
}

fn write_lock_file(path: &Path, content: &str) -> Result<(), String> {
    if std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(format!(
            "refusing to write `{}`: it is a symlink",
            path.display()
        ));
    }
    std::fs::write(path, content).map_err(|e| format!("cannot write `{}`: {}", path.display(), e))
}

impl Server {
    /// Handle one message; `Some(response)` for requests, `None` for
    /// notifications. Publishing is deferred to `take_pending_publishes`.
    fn handle(&mut self, method: &str, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        // Lifecycle gates (`exit` never reaches here — run_stdio owns it).
        match self.state {
            Lifecycle::PreInit if method != "initialize" => {
                // Requests: -32002 ServerNotInitialized. Notifications: dropped.
                return id.map(|id| error_response(id, -32002, "server not initialized"));
            }
            Lifecycle::ShutDown => {
                return id.map(|id| {
                    error_response(id, -32600, "invalid request: shutdown already received")
                });
            }
            _ => {}
        }
        match method {
            "initialize" => {
                if !matches!(self.state, Lifecycle::PreInit) {
                    return Some(error_response(
                        id?,
                        -32600,
                        "invalid request: initialize may only be sent once",
                    ));
                }
                // `initialize` is a REQUEST — bind its id BEFORE mutating
                // lifecycle state, so an `initialize` NOTIFICATION (no id)
                // cannot silently consume initialization (review F10).
                let id = id?;
                self.state = Lifecycle::Running;
                // Capability negotiation: only attach relatedInformation when
                // the client advertised support for it.
                self.related_info = params["capabilities"]["textDocument"]["publishDiagnostics"]
                    ["relatedInformation"]
                    .as_bool()
                    == Some(true);
                Some(respond(
                    id,
                    json!({
                        "capabilities": {
                            // FULL sync — didChange carries the whole text; save
                            // is advertised so conforming clients send didSave.
                            "textDocumentSync": { "openClose": true, "change": 1, "save": true },
                            "hoverProvider": true,
                            "definitionProvider": true,
                            "referencesProvider": true,
                            "completionProvider": { "triggerCharacters": [".", "::", "#", "[", "(", " "] },
                        },
                        "serverInfo": { "name": "cohdl-lsp", "version": env!("CARGO_PKG_VERSION") },
                    }),
                ))
            }
            "initialized" => None,
            "shutdown" => {
                // Also a REQUEST — a `shutdown` notification must not shut the
                // server down (review F10). Bind the id before mutating state.
                let id = id?;
                self.state = Lifecycle::ShutDown;
                Some(respond(id, Value::Null))
            }
            "textDocument/didOpen" => {
                let uri = params["textDocument"]["uri"].as_str()?.to_string();
                let text = params["textDocument"]["text"].as_str()?.to_string();
                if let Some(path) = uri_to_path(&uri) {
                    self.client_uris.insert(path.clone(), uri.clone());
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
                    self.client_uris.insert(path.clone(), uri.clone());
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
                    self.client_uris.remove(&path);
                }
                self.touched = Some(uri);
                None
            }
            "textDocument/hover" => {
                if position_params(&params).is_none() {
                    return Some(invalid_params(id?, method));
                }
                let result = self.hover(&params).map_or(Value::Null, |h| {
                    serde_json::to_value(h).unwrap_or(Value::Null)
                });
                Some(respond(id?, result))
            }
            "textDocument/definition" => {
                if position_params(&params).is_none() {
                    return Some(invalid_params(id?, method));
                }
                let result = self.definition(&params).map_or(Value::Null, |l| {
                    serde_json::to_value(l).unwrap_or(Value::Null)
                });
                Some(respond(id?, result))
            }
            "textDocument/references" => {
                if position_params(&params).is_none() {
                    return Some(invalid_params(id?, method));
                }
                let locs = self.references(&params);
                Some(respond(
                    id?,
                    serde_json::to_value(locs).unwrap_or(Value::Null),
                ))
            }
            "textDocument/completion" => {
                let result = self.completion(&params);
                Some(respond(id?, result))
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

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn invalid_params(id: Value, method: &str) -> Value {
    error_response(
        id,
        -32602,
        &format!(
            "invalid params for {}: need textDocument.uri and position",
            method
        ),
    )
}

/// The (uri, line, character) triple every positional request needs; `None`
/// means the params are malformed (InvalidParams, not a null result).
fn position_params(params: &Value) -> Option<(&str, u32, u32)> {
    // Checked conversion (review F10): a line/character above `u32::MAX`
    // must be InvalidParams (None), never a silent `as u32` wrap that lands
    // on a valid in-range position.
    Some((
        params["textDocument"]["uri"].as_str()?,
        u32::try_from(params["position"]["line"].as_u64()?).ok()?,
        u32::try_from(params["position"]["character"].as_u64()?).ok()?,
    ))
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
        match self.analyze(&path) {
            Ok(analysis) => self.publish_analysis(&uri, &path, analysis),
            // The document is gone (a phantom buffer after didClose): clear
            // its diagnostics explicitly and drop ownership (review R6).
            Err(AnalyzeError::Gone) => {
                for owned in self.published.values_mut() {
                    owned.remove(&uri);
                }
                vec![publish_note(&uri, Vec::new())]
            }
            // A real load failure: NEVER publish an empty list that claims
            // the file is clean — surface the failure instead, and leave
            // prior diagnostics owned (review R6). Matches the CLI, which
            // exits 2 with prose here.
            Err(AnalyzeError::Project(err)) => vec![show_message(&err)],
        }
    }

    /// Publish one analysis: every file with diagnostics + the touched file,
    /// then clear whatever THIS analysis unit owned before but not now.
    fn publish_analysis(&mut self, uri: &str, path: &Path, analysis: Analysis) -> Vec<Value> {
        let mut per_file: BTreeMap<u32, Vec<Value>> = BTreeMap::new();
        for d in analysis.checked.diags.iter() {
            let fid = d.primary.span.file;
            per_file.entry(fid.0).or_default().push(lsp_diagnostic(
                &analysis,
                d,
                self.related_info,
            ));
        }
        let mut notes = Vec::new();
        let mut now_published: BTreeSet<String> = BTreeSet::new();
        for (fid, diags) in &per_file {
            if let Some(u) = self.uri_spelling(&analysis, FileId(*fid)) {
                notes.push(publish_note(&u, diags.clone()));
                now_published.insert(u);
            }
        }
        // The touched file always gets a publish (possibly empty) — matched
        // by PATH identity, not URI spelling (the client's percent-encoding
        // may differ from ours).
        let touched_fid = analysis.fid_for(path);
        let touched_covered = touched_fid.is_some_and(|fid| per_file.contains_key(&fid.0));
        if !touched_covered {
            notes.push(publish_note(uri, Vec::new()));
            now_published.insert(uri.to_string());
        }
        // A design-selection failure has no source span — surface it as an
        // editor message so the file never silently LOOKS clean while the CLI
        // errors (review finding; parallels the CLI's exit-2 prose).
        if let Some(err) = &analysis.checked.selection_error {
            notes.push(show_message(err));
        }
        // Clear diagnostics that THIS analysis unit owned before but not now.
        // Ownership is keyed per project/loose file: re-checking one project
        // must not clear another's live diagnostics (review R7).
        let owned = self.published.entry(analysis.key).or_default();
        for stale in owned.difference(&now_published) {
            notes.push(publish_note(stale, Vec::new()));
        }
        *owned = now_published;
        notes
    }

    /// Locate the project containing `path` (walk up for `cohdl.toml`, else
    /// single-file), load it with overlays applied, and run the exact same
    /// dependency-aware pipeline entry point the CLI uses.
    fn analyze(&self, path: &Path) -> Result<Analysis, AnalyzeError> {
        let project_root = path
            .ancestors()
            .skip(1)
            .find(|a| a.join("cohdl.toml").is_file())
            .map(Path::to_path_buf);

        let (loaded, dep_names) = if let Some(root) = project_root.as_deref() {
            let deps = resolve_manifest_deps(root).map_err(AnalyzeError::Project)?;
            let names = deps.iter().map(|(name, _)| name.clone()).collect();
            (crate::project::load_project_with_deps(root, &deps), names)
        } else {
            // Loose files retain their unpinned-newest-std behavior. The LSP
            // has no --no-std, so a missing std remains a surfaced failure.
            let std_dir = crate::project::find_std_dir(None);
            let Some(std_dir) = std_dir else {
                return Err(AnalyzeError::Project(
                    "cannot locate the std library — set COHDL_STD".to_string(),
                ));
            };
            (
                crate::project::load_project(path, Some(&std_dir)),
                vec!["std".to_string()],
            )
        };
        let mut proj = match loaded {
            Ok(p) => p,
            Err(e) => {
                // Fall back to a synthetic std+overlay project ONLY for a
                // genuinely nonexistent unsaved loose file. A cohdl.toml that
                // fails to load, an unreadable source, or a broken std is a
                // real error the editor must see (review R6).
                if project_root.is_some() || path.exists() {
                    return Err(AnalyzeError::Project(e));
                }
                let Some(text) = self.overlays.get(path) else {
                    return Err(AnalyzeError::Gone);
                };
                let std_dir = crate::project::find_std_dir(None);
                let (mut files, mut abs) = crate::project::load_std_files(std_dir.as_deref())
                    .ok_or_else(|| {
                        AnalyzeError::Project("cannot load the std library".to_string())
                    })?;
                files.push((path.display().to_string(), text.clone()));
                abs.push(path.to_path_buf());
                crate::project::Project {
                    name: "buffer".to_string(),
                    dir: path.parent().map(Path::to_path_buf).unwrap_or_default(),
                    top: None,
                    files,
                    abs_paths: abs,
                }
            }
        };
        // Overlays: unsaved buffer contents win over the disk.
        for (i, abs) in proj.abs_paths.iter().enumerate() {
            if let Some(text) = self.overlays.get(abs) {
                proj.files[i].1 = text.clone();
            }
        }
        // For a MANIFEST project: every unsaved buffer inside the project
        // dir that isn't on disk yet joins the set (not just the touched
        // one — a use import in one file must see a device declared in
        // another, still-unsaved file), with its PROJECT-RELATIVE display so
        // RFC-016 module inference matches the CLI's for nested files (an
        // absolute display would land the buffer at the package root;
        // adversarial finding). Overlays iterate in BTreeMap order —
        // deterministic. Loose files stay single-file analysis units (two
        // loose files in one directory are SEPARATE units — review R7).
        if project_root.is_some() {
            for (opath, text) in &self.overlays {
                if proj.abs_paths.iter().any(|p| p == opath) {
                    continue;
                }
                let Ok(rel) = opath.strip_prefix(&proj.dir) else {
                    continue; // an unrelated project's buffer
                };
                proj.files.push((rel.display().to_string(), text.clone()));
                proj.abs_paths.push(opath.clone());
            }
        } else if !proj.abs_paths.iter().any(|p| p == path) {
            if let Some(text) = self.overlays.get(path) {
                proj.files.push((path.display().to_string(), text.clone()));
                proj.abs_paths.push(path.to_path_buf());
            }
        }
        // Diagnostic-ownership key: the project root for manifest projects,
        // the file itself for loose/phantom files (two loose files in one
        // directory are separate analysis units).
        let key = project_root
            .as_deref()
            .unwrap_or(path)
            .display()
            .to_string();
        let checked = pipeline::check_files_in_with_deps(
            &proj.name,
            &dep_names,
            &proj.files,
            proj.top.as_deref(),
        )
        .map_err(AnalyzeError::Project)?;
        Ok(Analysis {
            checked,
            abs_paths: proj.abs_paths,
            key,
        })
    }

    // -- hover ---------------------------------------------------------------

    fn hover(&mut self, params: &Value) -> Option<lt::Hover> {
        let (analysis, fid, offset) = self.locate(params)?;
        let world = &analysis.checked.world;

        let source = analysis.checked.sm.text(fid).as_bytes();
        let mut start = (offset as usize).min(source.len());
        while start > 0 && (source[start - 1].is_ascii_alphanumeric() || source[start - 1] == b'_')
        {
            start -= 1;
        }
        let mut end = (offset as usize).min(source.len());
        while end < source.len() && (source[end].is_ascii_alphanumeric() || source[end] == b'_') {
            end += 1;
        }
        let token = std::str::from_utf8(&source[start..end]).unwrap_or_default();
        let syntax_help = match token {
            "annulus" => Some("**annulus pad** — `size: (outer_diameter, inner_diameter)`; SMD copper only, with `outer > inner > 0`."),
            "segmented_annulus" => Some("**segmented annulus paste** — `segmented_annulus(outer_diameter, inner_diameter, gap)` emits four cardinal stencil sectors."),
            _ => None,
        };
        if let Some(help) = syntax_help {
            let span = Span::new(fid, start as u32, end as u32);
            return Some(hover_markdown(
                help.to_string(),
                span_to_range(&analysis, span),
            ));
        }

        // Pin declaration hover: obligation + role (+ physical pads).
        for dev in world.devices.values() {
            for pb in &dev.pin_blocks {
                for pin in &pb.pins {
                    if contains(pin.name.span, fid, offset) {
                        return Some(hover_markdown(
                            device_pin_text(dev, pb, pin),
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
                        trait_pin_text(tr, pin),
                        span_to_range(&analysis, pin.name.span),
                    ));
                }
            }
        }
        // Pad placement hover (RFC-018): the resolved pad's shape, size,
        // layer, and plating — the same "resolve and show" precedent as the
        // empty-impl hover.
        for fp in world.footprints.values() {
            for place in &fp.pads {
                if !contains(place.pad.span, fid, offset)
                    && !contains(place.number.span, fid, offset)
                {
                    continue;
                }
                let Some(pad) = world.pads.get(&place.pad.name) else {
                    continue;
                };
                let mut text = format!(
                    "**pad** `{}` (placed as pad `{}` at ({}, {}))",
                    crate::resolve::short(&place.pad.name),
                    place.number.text,
                    place.x.text,
                    place.y.text
                );
                if let Some((shape, _)) = &pad.shape {
                    text.push_str(&format!("\n\n- shape: `{}`", shape.name()));
                }
                if !pad.size.is_empty() {
                    let dims: Vec<&str> = pad.size.iter().map(|v| v.text.as_str()).collect();
                    text.push_str(&format!("\n- size: `({})`", dims.join(", ")));
                }
                if let Some((layer, _)) = &pad.layer {
                    text.push_str(&format!("\n- layer: `{}`", layer.name()));
                }
                if let Some((plating, _)) = &pad.plating {
                    text.push_str(&format!("\n- plating: `{}`", plating.name()));
                }
                if let Some((drill, _)) = &pad.drill {
                    text.push_str(&format!(
                        "\n- drill: `{}`",
                        crate::fmt::pad_drill_text(drill)
                    ));
                }
                return Some(hover_markdown(
                    text,
                    span_to_range(&analysis, place.pad.span),
                ));
            }
        }
        // Part declaration hover (RFC-017): MPN/MFR, the resolved footprint
        // symbol, and any #[doc] reference documents.
        for (fq, part) in &world.parts {
            if !contains(part.name.span, fid, offset) {
                continue;
            }
            let mut text = format!(
                "**part** `{}`: `{}`",
                part.name.name,
                crate::resolve::short(&part.device.name.name)
            );
            if let Some(mpn) = part.primary.field("mpn") {
                text.push_str(&format!("\n\n- mpn: `{}`", mpn.value));
            }
            if let Some(mfr) = part.primary.field("mfr") {
                text.push_str(&format!("\n- mfr: `{}`", mfr.value));
            }
            if let Some(fp) = &part.primary.footprint {
                text.push_str(&format!("\n- footprint: `{}`", fp.name));
            }
            if let Some(docs) = analysis.checked.world.docs.get(fq) {
                for d in docs {
                    text.push_str(&format!("\n- doc: `{}`", d));
                }
            }
            return Some(hover_markdown(
                text,
                span_to_range(&analysis, part.name.span),
            ));
        }
        // Pad DECLARATION hover (RFC-018): same facts as the placement hover,
        // minus the position.
        for pad in world.pads.values() {
            if !contains(pad.name.span, fid, offset) {
                continue;
            }
            let mut text = format!("**pad** `{}`", pad.name.name);
            if let Some((shape, _)) = &pad.shape {
                text.push_str(&format!("\n\n- shape: `{}`", shape.name()));
            }
            if !pad.size.is_empty() {
                let dims: Vec<&str> = pad.size.iter().map(|v| v.text.as_str()).collect();
                text.push_str(&format!("\n- size: `({})`", dims.join(", ")));
            }
            if let Some((layer, _)) = &pad.layer {
                text.push_str(&format!("\n- layer: `{}`", layer.name()));
            }
            if let Some((plating, _)) = &pad.plating {
                text.push_str(&format!("\n- plating: `{}`", plating.name()));
            }
            if let Some((drill, _)) = &pad.drill {
                text.push_str(&format!(
                    "\n- drill: `{}`",
                    crate::fmt::pad_drill_text(drill)
                ));
            }
            return Some(hover_markdown(
                text,
                span_to_range(&analysis, pad.name.span),
            ));
        }
        // Mount-hole hover (RFC-022/023): plating, shape, position, geometry.
        for fp in world.footprints.values() {
            for mh in &fp.mount_holes {
                if !contains(mh.span, fid, offset) {
                    continue;
                }
                let mut text = format!(
                    "**mount_hole** `{}` — `{}`, shape `{}` at ({}, {})",
                    mh.number.text,
                    mh.plating.name(),
                    mh.shape_or_default().name(),
                    mh.x.text,
                    mh.y.text
                );
                match &mh.geom {
                    MountHoleGeom::Diameter(d) => {
                        text.push_str(&format!("\n\n- diameter: `{}`", d.text));
                    }
                    MountHoleGeom::Size(dims, _) => {
                        let d: Vec<&str> = dims.iter().map(|v| v.text.as_str()).collect();
                        text.push_str(&format!("\n\n- size: `({})`", d.join(", ")));
                    }
                }
                return Some(hover_markdown(text, span_to_range(&analysis, mh.span)));
            }
        }
        // `use` import hover (RFC-016): the resolved target's kind.
        for u in &world.uses {
            if contains(u.span, fid, offset) {
                let text = match world.symbols.get(&u.fq) {
                    Some(sym) => format!("**use** `{}` — {}", u.fq, sym.kind),
                    None => format!("**use** `{}` — unresolved (see diagnostics)", u.fq),
                };
                return Some(hover_markdown(text, span_to_range(&analysis, u.span)));
            }
        }
        // Pin USE-SITE hover (RFC-002: any pin reference reveals its
        // obligation/role, not only the declaration): `d.A` in net/nc/call
        // statements resolves through the inst's type (part → device) or a
        // trait-typed fn parameter.
        if let Some(h) = pin_ref_hover(&analysis, fid, offset) {
            return Some(h);
        }
        // Unit-literal hover: RFC-001's (unit × allowed-prefix) table row.
        if let Some(h) = unit_literal_hover(&analysis, fid, offset) {
            return Some(h);
        }
        // `place` + physics-attribute hover: whole-construct summaries
        // (RFC-020/024/026 placements; RFC-027/028 attributes).
        let bodies: Vec<&[Stmt]> = world
            .designs
            .values()
            .map(|d| d.body.as_slice())
            .chain(world.fns.values().map(|f| f.body.as_slice()))
            .collect();
        for body in bodies {
            for stmt in body {
                let phys: &[PhysAttr] = match stmt {
                    Stmt::Inst(i) => &i.phys,
                    Stmt::Net(n) => &n.phys,
                    _ => &[],
                };
                for pa in phys {
                    if contains(pa.span(), fid, offset) {
                        return Some(hover_markdown(
                            phys_attr_text(pa),
                            span_to_range(&analysis, pa.span()),
                        ));
                    }
                }
                if let Stmt::Layout(lb) = stmt {
                    for pl in &lb.placements {
                        if contains(pl.span, fid, offset) {
                            return Some(hover_markdown(
                                placement_text(pl),
                                span_to_range(&analysis, pl.span),
                            ));
                        }
                    }
                }
            }
        }
        // Empty-impl hover: the resolved by-name mapping DR-013 asked for.
        for imp in &world.impls {
            if contains(imp.span, fid, offset) {
                let key = (imp.trait_name.name.clone(), imp.device_name.name.clone());
                // Display uses the short spelling; identity stays fq.
                let mut text = format!(
                    "**impl** `{}` **for** `{}`",
                    crate::resolve::short(&imp.trait_name.name),
                    crate::resolve::short(&imp.device_name.name)
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
        let world = &analysis.checked.world;
        // RFC-016 `use` imports (closes review R5-10): the imported path
        // resolves to its declaration via the symbol table.
        for u in &world.uses {
            if contains(u.span, fid, offset) {
                let sym = world.symbols.get(&u.fq)?;
                return Some(lt::Location {
                    uri: analysis.uri_for(sym.span.file)?,
                    range: span_to_range(&analysis, sym.span),
                });
            }
        }
        // Reference-level targets: instance refs (net/nc members, call args,
        // `place`, physics attributes), pin refs, fn parameters, and
        // layout-constraint net names.
        if let Some(target) = ref_definition(world, fid, offset) {
            return Some(lt::Location {
                uri: analysis.uri_for(target.file)?,
                range: span_to_range(&analysis, target),
            });
        }
        let name = use_site_name(world, fid, offset)?;
        let target = world
            .devices
            .get(&name)
            .map(|d| d.name.span)
            .or_else(|| world.traits.get(&name).map(|t| t.name.span))
            .or_else(|| world.fns.get(&name).map(|f| f.name.span))
            .or_else(|| world.parts.get(&name).map(|p| p.name.span))
            .or_else(|| world.footprints.get(&name).map(|f| f.name.span))
            .or_else(|| world.pads.get(&name).map(|p| p.name.span))?;
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
        // Declaration cursors: capture the fq KEY (impl names are fq after
        // RFC-016 resolution; the decl's own ident is bare).
        for (fq, tr) in &world.traits {
            if contains(tr.name.span, fid, offset) {
                trait_name = Some(fq.clone());
            }
        }
        for (fq, dev) in &world.devices {
            if contains(dev.name.span, fid, offset) {
                device_name = Some(fq.clone());
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

    fn completion(&self, params: &Value) -> Value {
        let Some((uri, _, _)) = position_params(params) else {
            return json!({ "isIncomplete": false, "items": [] });
        };
        let Some(path) = uri_to_path(uri) else {
            return json!({ "isIncomplete": false, "items": [] });
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return json!({ "isIncomplete": false, "items": [] });
        };
        let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
        let character = params["position"]["character"].as_u64().unwrap_or(0) as usize;
        let prefix = text
            .lines()
            .nth(line)
            .unwrap_or_default()
            .get(..character)
            .unwrap_or_default()
            .split_whitespace()
            .last()
            .unwrap_or_default()
            .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_')
            .to_string();

        let mut items = Vec::new();
        // The lexer's hard keywords plus the contextual (positional) grammar
        // words RFC-016..029 added — kept in sync with the parser, not just
        // `lex::is_keyword` (most newer constructs use contextual words).
        for keyword in [
            // hard keywords (lex::is_keyword)
            "pub",
            "trait",
            "device",
            "impl",
            "for",
            "fn",
            "design",
            "inst",
            "net",
            "nc",
            "part",
            "pins",
            "spec",
            "required",
            "optional",
            // modules + library registry (RFC-016/017)
            "use",
            "footprint",
            // parts
            "primary",
            "alt",
            "mpn",
            "mfr",
            // pads + footprint bodies (RFC-018/021/022/023/025)
            "pad",
            "mount_hole",
            "courtyard",
            "silkscreen_ref",
            "shape",
            "size",
            "layer",
            "plating",
            "drill",
            "paste",
            "mask_expansion",
            "annulus",
            "segmented_annulus",
            "diameter",
            "at",
            "rotate",
            // layout + placement (RFC-013/020/026)
            "layout",
            "place",
            "side",
            "top",
            "bottom",
            "net_class",
            "diff_pair",
            "length_match",
            "board_outline",
        ] {
            if keyword.starts_with(&prefix) || prefix.is_empty() {
                items.push(json!({
                    "label": keyword,
                    "kind": 14,
                    "detail": "keyword",
                }));
            }
        }
        // Attribute names complete after `#[` (the trigger set includes both).
        for attr in [
            "designator",
            "intent",
            "placement_hint",
            "doc",
            "ground",
            "high_current",
            "impedance",
            "bypass",
            "crystal_oscillator",
            "switching_converter",
            "bga_fanout",
        ] {
            if attr.starts_with(&prefix) || prefix.is_empty() {
                items.push(json!({
                    "label": attr,
                    "kind": 14,
                    "detail": "attribute",
                }));
            }
        }
        if prefix.is_empty() || "device".starts_with(&prefix) {
            items.push(json!({
                "label": "device",
                "kind": 7,
                "detail": "declaration",
            }));
        }
        if prefix.is_empty() || "trait".starts_with(&prefix) {
            items.push(json!({
                "label": "trait",
                "kind": 7,
                "detail": "declaration",
            }));
        }
        json!({ "isIncomplete": false, "items": items })
    }

    /// The URI to publish under for `fid`: the CLIENT's own spelling when
    /// the document is open (its percent-encoding or `localhost` authority
    /// may differ from ours), else our canonical `file://` encoding.
    fn uri_spelling(&self, analysis: &Analysis, fid: FileId) -> Option<String> {
        let path = analysis.abs_paths.get(fid.0 as usize)?;
        if let Some(u) = self.client_uris.get(path) {
            return Some(u.clone());
        }
        Some(encode_file_uri(path))
    }

    /// Shared request plumbing: analyze the document's project and convert the
    /// LSP position to (FileId, byte offset).
    fn locate(&mut self, params: &Value) -> Option<(Analysis, FileId, u32)> {
        let (uri, line, character) = position_params(params)?;
        let path = uri_to_path(uri)?;
        let analysis = self.analyze(&path).ok()?;
        let fid = analysis.fid_for(&path)?;
        let offset = position_to_offset(&analysis, fid, line, character)?;
        Some((analysis, fid, offset))
    }
}

impl Analysis {
    fn uri_for(&self, fid: FileId) -> Option<lt::Uri> {
        let path = self.abs_paths.get(fid.0 as usize)?;
        encode_file_uri(path).parse().ok()
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

/// The device-pin hover body — shared between the declaration hover and pin
/// USE-SITE hover (RFC-002 asks for the obligation at any pin reference).
fn device_pin_text(dev: &DeviceDef, pb: &PinBlock, pin: &DevicePin) -> String {
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
    format!(
        "**{} pin** `{}` on device `{}`{}\n\n- role: `{}`\n- pads: {}",
        pin.obligation.keyword(),
        pin.name.name,
        dev.name.name,
        variant,
        role,
        pads
    )
}

fn trait_pin_text(tr: &TraitDef, pin: &TraitPin) -> String {
    format!(
        "**{} pin role** `{}` on trait `{}` (abstract — mapped per `impl`)",
        pin.obligation.keyword(),
        pin.name.name,
        tr.name.name
    )
}

/// Every `PinRef` in a statement body (net members, nc members, call args).
fn body_pin_refs(body: &[Stmt]) -> Vec<&PinRef> {
    let mut out = Vec::new();
    for stmt in body {
        match stmt {
            Stmt::Net(s) => out.extend(s.members.iter()),
            Stmt::Nc(s) => out.extend(s.members.iter()),
            Stmt::Call(s) => out.extend(s.args.iter()),
            _ => {}
        }
    }
    out
}

/// Pin USE-SITE hover (review R10 / RFC-002): `d.A` resolves through the
/// inst's type (part → device) to the pin declaration; `target.A` on a
/// trait-typed fn parameter resolves to the trait pin role.
fn pin_ref_hover(analysis: &Analysis, fid: FileId, offset: u32) -> Option<lt::Hover> {
    let world = &analysis.checked.world;
    let bodies: Vec<(&[Stmt], Option<&FnDef>)> = world
        .designs
        .values()
        .map(|d| (d.body.as_slice(), None))
        .chain(world.fns.values().map(|f| (f.body.as_slice(), Some(f))))
        .collect();
    for (body, func) in bodies {
        for pr in body_pin_refs(body) {
            let Some(pin_id) = &pr.pin else { continue };
            if !contains(pin_id.span, fid, offset) {
                continue;
            }
            // An instance in this body: its type, through parts, to a device
            // — shared resolution with goto-definition (`inst_pin_block`).
            if body_inst(body, &pr.base.name).is_some() {
                if let Some((dev, pb)) = inst_pin_block(world, body, &pr.base.name) {
                    for pin in &pb.pins {
                        if pin.name.name == pin_id.name {
                            return Some(hover_markdown(
                                device_pin_text(dev, pb, pin),
                                span_to_range(analysis, pin_id.span),
                            ));
                        }
                    }
                }
                continue;
            }
            // A trait-typed fn parameter: the pin is a trait role.
            let f = func?;
            for tn in param_trait_names(f, &pr.base.name) {
                if let Some(tr) = world.traits.get(&tn) {
                    for pin in &tr.pins {
                        if pin.name.name == pin_id.name {
                            return Some(hover_markdown(
                                trait_pin_text(tr, pin),
                                span_to_range(analysis, pin_id.span),
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Unit-literal hover (review R10 / RFC-001): the literal's unit type plus
/// its row of the (unit × allowed-prefix) table.
fn unit_literal_hover(analysis: &Analysis, fid: FileId, offset: u32) -> Option<lt::Hover> {
    let world = &analysis.checked.world;
    let check = |v: &crate::units::UnitValue, s: Span| -> Option<(String, Span)> {
        if contains(s, fid, offset) {
            Some((unit_value_text(v), s))
        } else {
            None
        }
    };
    let arg_hits = |args: &[GenericArg]| -> Option<(String, Span)> {
        args.iter().find_map(|a| match a {
            GenericArg::Unit(v, s) => check(v, *s),
            _ => None,
        })
    };
    let body_hits = |body: &[Stmt]| -> Option<(String, Span)> {
        body.iter().find_map(|stmt| match stmt {
            Stmt::Inst(i) => arg_hits(&i.ty.generic_args),
            Stmt::Call(c) => arg_hits(&c.generic_args),
            Stmt::Net(n) => match &n.annotation {
                Some(NetAnnotation::Voltage(v, s)) => check(v, *s),
                _ => None,
            },
            _ => None,
        })
    };
    let (text, span) = world
        .devices
        .values()
        .find_map(|dev| {
            dev.spec_blocks
                .iter()
                .flat_map(|b| &b.fields)
                .find_map(|f| match &f.value {
                    SpecValue::Lit(v, s) => check(v, *s),
                    SpecValue::GenericRef(_) => None,
                })
                .or_else(|| {
                    dev.generics
                        .iter()
                        .find_map(|g| g.default.as_ref().and_then(|(v, s)| check(v, *s)))
                })
        })
        .or_else(|| {
            world
                .parts
                .values()
                .find_map(|p| arg_hits(&p.device.generic_args))
        })
        .or_else(|| world.designs.values().find_map(|d| body_hits(&d.body)))
        .or_else(|| world.fns.values().find_map(|f| body_hits(&f.body)))
        // A function's own generic defaults (`fn f<V: Voltage = 3.3V>`) are
        // unit literals too — the device path scanned `dev.generics` but the
        // fn path never scanned `f.generics` (review F11).
        .or_else(|| {
            world.fns.values().find_map(|f| {
                f.generics
                    .iter()
                    .find_map(|g| g.default.as_ref().and_then(|(v, s)| check(v, *s)))
            })
        })?;
    Some(hover_markdown(text, span_to_range(analysis, span)))
}

fn unit_value_text(v: &crate::units::UnitValue) -> String {
    format!(
        "**{} literal** `{}`\n\n{}",
        v.unit.type_name(),
        v.text,
        v.unit.prefix_table_help()
    )
}

// ---------------------------------------------------------------------------
// Diagnostics mapping (the RFC-010 equivalence surface).

fn lsp_diagnostic(analysis: &Analysis, d: &crate::diag::Diagnostic, related_info: bool) -> Value {
    let severity = match d.severity {
        crate::diag::Severity::Error => 1,
        crate::diag::Severity::Warning => 2,
    };
    let mut out = json!({
        "range": span_to_range(analysis, d.primary.span),
        "severity": severity,
        "code": d.code,
        "source": "cohdl",
        "message": d.message,
    });
    // relatedInformation only when the client advertised support for it
    // (capability negotiation — review R8).
    if !related_info {
        return out;
    }
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
    out["relatedInformation"] = Value::Array(related);
    out
}

fn publish_note(uri: &str, diagnostics: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics },
    })
}

fn show_message(message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "window/showMessage",
        "params": { "type": 1, "message": format!("cohdl: {}", message) },
    })
}

/// POSIX `file://` URI → path. The empty and `localhost` authorities are
/// local (RFC 8089); any other authority is rejected. Windows drive letters,
/// backslashes, and UNC forms are NOT supported — this server targets POSIX
/// hosts only (documented in docs/lsp.md).
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path = match rest.find('/') {
        Some(0) => rest,
        // URI hosts are case-insensitive (RFC 3986 §3.2.2) — `LOCALHOST`
        // and `Localhost` are the same local authority (review F10).
        Some(i) if rest[..i].eq_ignore_ascii_case("localhost") => &rest[i..],
        _ => return None,
    };
    let p = PathBuf::from(percent_decode(path));
    Some(p.canonicalize().unwrap_or(p))
}

/// RFC 3986 percent-encoding of a filesystem path into a `file://` URI —
/// everything outside unreserved + `/` is encoded, so `lsp_types::Uri::parse`
/// accepts paths with spaces (and any other byte) and the URI round-trips
/// through `uri_to_path`.
fn encode_file_uri(path: &Path) -> String {
    let mut out = String::from("file://");
    for &b in path.display().to_string().as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Full percent-decoding (any %XX escape), byte-accurate for UTF-8 paths.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------------------------------------------------------------------------
// Use-site identification for goto-definition.

/// The device/trait/fn/part name whose USE SITE contains the offset.
/// The `inst` statement declaring `name` in `body`, if any.
fn body_inst<'a>(body: &'a [Stmt], name: &str) -> Option<&'a InstStmt> {
    body.iter().find_map(|s| match s {
        Stmt::Inst(i) if i.name.name == name => Some(i),
        _ => None,
    })
}

/// The first named `net` statement in `body` declaring `name`.
fn body_net_span(body: &[Stmt], name: &str) -> Option<Span> {
    body.iter().find_map(|s| match s {
        Stmt::Net(n) => n
            .name
            .as_ref()
            .filter(|id| id.name == name)
            .map(|id| id.span),
        _ => None,
    })
}

/// Resolve an instance reference through its type (part → device) to the
/// SELECTED structural variant's pin block (review F9 discipline) — shared
/// by pin hover and reference-level goto-definition.
fn inst_pin_block<'w>(
    world: &'w crate::resolve::World,
    body: &[Stmt],
    base: &str,
) -> Option<(&'w DeviceDef, &'w PinBlock)> {
    let i = body_inst(body, base)?;
    let ty = i.ty.name.name.clone();
    let part = world.parts.get(&ty);
    let dev_name = part.map(|p| p.device.name.name.clone()).unwrap_or(ty);
    let variant: Option<String> = match part {
        Some(p) => p.device.variant.as_ref().map(|v| v.name.clone()),
        None => i.ty.variant.as_ref().map(|v| v.name.clone()),
    };
    let dev = world.devices.get(&dev_name)?;
    let pb = dev
        .pin_blocks
        .iter()
        .find(|b| b.variant.as_ref().map(|v| v.name.as_str()) == variant.as_deref())?;
    Some((dev, pb))
}

/// The trait bounds reachable from fn parameter `base` (generic bound or
/// `impl Trait` sugar).
fn param_trait_names(f: &FnDef, base: &str) -> Vec<String> {
    f.params
        .iter()
        .filter(|p| p.name.name == base)
        .filter_map(|p| match &p.ty {
            FnParamTy::Generic(g) => f
                .generics
                .iter()
                .find(|gp| gp.name.name == g.name)
                .and_then(|gp| match &gp.bound {
                    GenericBound::Traits(ts) => {
                        Some(ts.iter().map(|t| t.name.clone()).collect::<Vec<_>>())
                    }
                    GenericBound::Unit(_) => None,
                }),
            FnParamTy::ImplTrait(ts, _) => Some(ts.iter().map(|t| t.name.clone()).collect()),
            FnParamTy::Pin(_) => None,
        })
        .flatten()
        .collect()
}

/// Reference-level goto-definition (the constructs RFC-016..028 added after
/// RFC-014's original name-reference set): instance references in net/nc
/// members and call args (arrays included — the base name is the target),
/// `place` targets, physics-attribute instance/pin arguments, pin references
/// (to the device/trait pin declaration), fn parameters, and
/// layout-constraint net names (to the net statement).
fn ref_definition(world: &crate::resolve::World, fid: FileId, offset: u32) -> Option<Span> {
    let hit = |id: &Ident| contains(id.span, fid, offset);
    let bodies: Vec<(&[Stmt], Option<&FnDef>)> = world
        .designs
        .values()
        .map(|d| (d.body.as_slice(), None))
        .chain(world.fns.values().map(|f| (f.body.as_slice(), Some(f))))
        .collect();
    for (body, func) in bodies {
        // The instance a reference resolves to: an `inst` in this body, or a
        // fn parameter (RFC-028 lets physics attrs target Pin/Instance
        // params).
        let inst_target = |id: &Ident| -> Option<Span> {
            if let Some(i) = body_inst(body, &id.name) {
                return Some(i.name.span);
            }
            func.and_then(|f| {
                f.params
                    .iter()
                    .find(|p| p.name.name == id.name)
                    .map(|p| p.name.span)
            })
        };
        let pin_target = |base: &Ident, pin: &Ident| -> Option<Span> {
            if let Some((_, pb)) = inst_pin_block(world, body, &base.name) {
                return pb
                    .pins
                    .iter()
                    .find(|p| p.name.name == pin.name)
                    .map(|p| p.name.span);
            }
            let f = func?;
            for tn in param_trait_names(f, &base.name) {
                if let Some(tr) = world.traits.get(&tn) {
                    if let Some(tp) = tr.pins.iter().find(|p| p.name.name == pin.name) {
                        return Some(tp.name.span);
                    }
                }
            }
            None
        };

        for pr in body_pin_refs(body) {
            if hit(&pr.base) {
                if let Some(span) = inst_target(&pr.base) {
                    return Some(span);
                }
            }
            if let Some(pin) = &pr.pin {
                if hit(pin) {
                    if let Some(span) = pin_target(&pr.base, pin) {
                        return Some(span);
                    }
                }
            }
        }

        for stmt in body {
            let phys: &[PhysAttr] = match stmt {
                Stmt::Inst(i) => &i.phys,
                Stmt::Net(n) => &n.phys,
                _ => &[],
            };
            for pa in phys {
                match pa {
                    PhysAttr::Bypass { inst, pin, .. } => {
                        if hit(inst) {
                            if let Some(span) = inst_target(inst) {
                                return Some(span);
                            }
                        }
                        if let Some(pin) = pin {
                            if hit(pin) {
                                if let Some(span) = pin_target(inst, pin) {
                                    return Some(span);
                                }
                            }
                        }
                    }
                    PhysAttr::CrystalOscillator {
                        parent, pin1, pin2, ..
                    } => {
                        if hit(parent) {
                            if let Some(span) = inst_target(parent) {
                                return Some(span);
                            }
                        }
                        for pin in [pin1, pin2] {
                            if hit(pin) {
                                if let Some(span) = pin_target(parent, pin) {
                                    return Some(span);
                                }
                            }
                        }
                    }
                    PhysAttr::SwitchingConverter {
                        inductor,
                        input_capacitor,
                        output_capacitor,
                        ..
                    } => {
                        for id in [
                            Some(inductor),
                            input_capacitor.as_ref(),
                            output_capacitor.as_ref(),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            if hit(id) {
                                if let Some(span) = inst_target(id) {
                                    return Some(span);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            if let Stmt::Layout(lb) = stmt {
                for pl in &lb.placements {
                    if hit(&pl.inst) {
                        if let Some(i) = body_inst(body, &pl.inst.name) {
                            return Some(i.name.span);
                        }
                    }
                }
                for c in &lb.constraints {
                    let nets: &[Ident] = match c {
                        LayoutConstraint::NetClass { nets, .. } => nets,
                        LayoutConstraint::DiffPair { nets, .. } => nets,
                        LayoutConstraint::LengthMatch { nets, .. } => nets,
                    };
                    for n in nets {
                        if hit(n) {
                            if let Some(span) = body_net_span(body, &n.name) {
                                return Some(span);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// `place` hover body (RFC-020/024/026).
fn placement_text(pl: &Placement) -> String {
    let idx = pl
        .index
        .as_ref()
        .map(|(i, _)| format!("[{}]", i))
        .unwrap_or_default();
    format!(
        "**place** `{}{}` at ({}, {}) rotate {} side {}",
        pl.inst.name,
        idx,
        pl.at.0.text,
        pl.at.1.text,
        pl.rotate,
        pl.side.name()
    )
}

/// Physics-attribute hover body (RFC-027/028) — the parsed facts.
fn phys_attr_text(pa: &PhysAttr) -> String {
    let mut text = format!("**#[{}]** physics constraint (RFC-027)", pa.name());
    match pa {
        PhysAttr::Ground {
            primary,
            region_pour,
            ..
        } => {
            text.push_str(&format!(
                "\n\n- kind: `{}`",
                if *primary { "primary" } else { "secondary" }
            ));
            if *region_pour {
                text.push_str("\n- region_pour");
            }
        }
        PhysAttr::HighCurrent {
            current,
            power_pour,
            ..
        } => {
            text.push_str(&format!("\n\n- current: `{}`", current.text));
            if *power_pour {
                text.push_str("\n- power_pour");
            }
        }
        PhysAttr::Impedance {
            impedance,
            frequency,
            ..
        } => {
            text.push_str(&format!(
                "\n\n- impedance: `{}`\n- frequency: `{}`",
                impedance.text, frequency.text
            ));
        }
        PhysAttr::Bypass {
            inst,
            index,
            pin,
            capacitance,
            ..
        } => {
            let base = match index {
                Some((i, _)) => format!("{}[{}]", inst.name, i),
                None => inst.name.clone(),
            };
            let target = match pin {
                Some(p) => format!("{}.{}", base, p.name),
                None => base,
            };
            text.push_str(&format!(
                "\n\n- target: `{}`\n- capacitance: `{}`",
                target, capacitance.text
            ));
        }
        PhysAttr::CrystalOscillator {
            parent, pin1, pin2, ..
        } => {
            text.push_str(&format!(
                "\n\n- parent: `{}`\n- pins: `{}`, `{}`",
                parent.name, pin1.name, pin2.name
            ));
        }
        PhysAttr::SwitchingConverter {
            inductor,
            input_capacitor,
            output_capacitor,
            ..
        } => {
            text.push_str(&format!("\n\n- inductor: `{}`", inductor.name));
            if let Some(c) = input_capacitor {
                text.push_str(&format!("\n- input_capacitor: `{}`", c.name));
            }
            if let Some(c) = output_capacitor {
                text.push_str(&format!("\n- output_capacitor: `{}`", c.name));
            }
        }
        PhysAttr::BgaFanout { .. } => {}
    }
    text
}

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
    // Footprint pad placements reference pad symbols (RFC-018).
    for fp in world.footprints.values() {
        for place in &fp.pads {
            if hit(&place.pad) {
                return Some(place.pad.name.clone());
            }
        }
    }
    // Part device references + footprint symbol references (RFC-017).
    for part in world.parts.values() {
        if hit(&part.device.name) {
            return Some(part.device.name.name.clone());
        }
        for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
            if let Some(fp) = &entry.footprint {
                if hit(fp) {
                    return Some(fp.name.clone());
                }
            }
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
            Stmt::Call(s) => {
                if hit(&s.callee) {
                    return Some(s.callee.name.clone());
                }
                for arg in &s.generic_args {
                    if let GenericArg::Name(id) = arg {
                        if hit(id) {
                            return Some(id.name.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

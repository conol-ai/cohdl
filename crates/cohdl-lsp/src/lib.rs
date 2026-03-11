use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use cohdl_drc::{DiagnosticLevel, DrcDiagnostic, DrcRunner};
use cohdl_parser::{parse_source_file, ParseError};
use cohdl_sema::connectivity::build_connectivity;
use cohdl_sema::typeck::{type_check, TypeCheckResult};
use cohdl_sema::{resolve, ResolvedSourceFile, SemaError};
use cohdl_syntax::ast::{self, DeviceBodyItem, PinEntryKind, SourceFile, TopLevelItemKind};

// ── Embedded std library ─────────────────────────────────────────────────────

const STD_LIB_COHDL: &str = include_str!("../../../std/src/lib.cohdl");
const STD_TRAITS_COHDL: &str = include_str!("../../../std/src/traits.cohdl");
const STD_PASSIVE_COHDL: &str = include_str!("../../../std/src/passive.cohdl");

// ── Project resolution ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LspManifest {
    #[allow(dead_code)]
    design: LspDesignSection,
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
}

#[derive(Deserialize)]
struct LspDesignSection {
    #[allow(dead_code)]
    root: String,
    #[allow(dead_code)]
    top: Option<String>,
}

/// Walk up from `start_dir` looking for `cohdl.toml`.
fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        if dir.join("cohdl.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Resolve `module` declarations in source text by loading sibling `.cohdl` files.
fn resolve_modules_for_lsp(src_dir: &Path, root_src: &str) -> String {
    let source_file = match parse_source_file(root_src) {
        Ok(sf) => sf,
        Err(_) => return root_src.to_string(),
    };

    let mod_names: Vec<(&str, bool)> = source_file
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            TopLevelItemKind::Mod(m) => Some((m.name.name.as_str(), item.visibility.is_some())),
            _ => None,
        })
        .collect();

    if mod_names.is_empty() {
        return root_src.to_string();
    }

    let mut combined = String::new();
    for (name, is_pub) in &mod_names {
        let mod_path = src_dir.join(format!("{}.cohdl", name));
        if let Ok(content) = std::fs::read_to_string(&mod_path) {
            let vis = if *is_pub { "pub " } else { "" };
            combined.push_str(&format!("{}module {} {{\n{}\n}}\n", vis, name, content));
        }
    }
    combined.push_str(root_src);
    combined
}

/// Synthesize the bundled `std` dependency as a `module std { ... }` block.
fn synthesize_std() -> String {
    let source_file = match parse_source_file(STD_LIB_COHDL) {
        Ok(sf) => sf,
        Err(_) => return String::new(),
    };

    let embedded: HashMap<&str, &str> = [
        ("traits", STD_TRAITS_COHDL),
        ("passive", STD_PASSIVE_COHDL),
    ]
    .into_iter()
    .collect();

    let mut inner = String::new();
    for item in &source_file.items {
        if let TopLevelItemKind::Mod(m) = &item.kind {
            let name = &m.name.name;
            if let Some(content) = embedded.get(name.as_str()) {
                let vis = if item.visibility.is_some() { "pub " } else { "" };
                inner.push_str(&format!("{}module {} {{\n{}\n}}\n", vis, name, content));
            }
        }
    }

    format!("module std {{\n{}\n}}\n", inner)
}

/// Resolve all dependencies from a manifest. Currently only supports bundled `std`.
fn resolve_deps_for_lsp(deps: &HashMap<String, toml::Value>) -> String {
    let mut dep_src = String::new();
    for name in deps.keys() {
        if name == "std" {
            dep_src.push_str(&synthesize_std());
            dep_src.push('\n');
        }
    }
    dep_src
}

/// Build the full project source for analysis, returning `(combined_src, prefix_len)`.
/// `prefix_len` is the byte offset where the open file's content starts in the combined source.
fn build_project_source(file_path: &Path, file_src: &str) -> (String, usize) {
    let file_dir = file_path.parent().unwrap_or_else(|| Path::new("."));

    // Try to find the project root
    let project_root = find_project_root(file_dir);

    // Resolve dependencies
    let mut dep_src = String::new();
    if let Some(ref root) = project_root {
        let manifest_path = root.join("cohdl.toml");
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = toml::from_str::<LspManifest>(&content) {
                dep_src = resolve_deps_for_lsp(&manifest.dependencies);
            }
        }
    }

    // Resolve module declarations from the current file
    let user_src = resolve_modules_for_lsp(file_dir, file_src);

    // The prefix is dep_src + module content (everything before the original file)
    // user_src = [module wrappers] + file_src
    // The original file content starts at: dep_src.len() + (user_src.len() - file_src.len())
    let prefix_len = dep_src.len() + (user_src.len() - file_src.len());

    let combined = format!("{}{}", dep_src, user_src);
    (combined, prefix_len)
}

// ── Document store ──────────────────────────────────────────────────────────

/// In-memory store mapping document URIs to their latest source text.
#[derive(Debug, Default)]
struct DocumentStore {
    docs: HashMap<Url, String>,
}

// ── Cached analysis result ──────────────────────────────────────────────────

/// The result of running the full analysis pipeline on a document.
#[derive(Clone)]
struct AnalysisResult {
    source_file: Option<SourceFile>,
    resolved: Option<ResolvedSourceFile>,
    tc_result: Option<TypeCheckResult>,
}

// ── Server state ────────────────────────────────────────────────────────────

pub struct CohdlLanguageServer {
    client: Client,
    state: Arc<RwLock<ServerState>>,
}

struct ServerState {
    documents: DocumentStore,
    /// Cached analysis results keyed by URI.
    analyses: HashMap<Url, AnalysisResult>,
}

// ── Span → LSP Range conversion ─────────────────────────────────────────────

/// Convert a byte offset into (0-based line, 0-based UTF-16 character offset).
fn offset_to_position(src: &str, offset: usize) -> Position {
    let offset = offset.min(src.len());
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    Position::new(line, col)
}

/// Convert a `Span` (byte offsets) to an LSP `Range`.
fn span_to_range(src: &str, span: ast::Span) -> Range {
    Range::new(
        offset_to_position(src, span.start),
        offset_to_position(src, span.end),
    )
}

/// Convert an LSP `Position` (line/character) to a byte offset into the source.
fn position_to_offset(src: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in src.char_indices() {
        if line == pos.line && col == pos.character {
            return i;
        }
        if ch == '\n' {
            if line == pos.line {
                // Past end of line — clamp to newline position
                return i;
            }
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    src.len()
}

// ── Diagnostics pipeline ────────────────────────────────────────────────────

/// Run the full parse → sema → DRC pipeline and collect all diagnostics.
/// `prefix_len` is the byte offset where the user's file starts in `src`;
/// diagnostics outside that range are suppressed, and spans are adjusted.
fn run_diagnostics(
    src: &str,
    user_src: &str,
    prefix_len: usize,
) -> (Vec<Diagnostic>, AnalysisResult) {
    let mut diagnostics = Vec::new();
    let user_end = prefix_len + user_src.len();

    // ── Parse ────────────────────────────────────────────────────────────
    let source_file = match parse_source_file(src) {
        Ok(sf) => sf,
        Err(errors) => {
            for err in &errors {
                if err.span.start >= prefix_len && err.span.start < user_end {
                    let adjusted = ParseError {
                        message: err.message.clone(),
                        span: ast::Span {
                            start: err.span.start - prefix_len,
                            end: err.span.end - prefix_len,
                        },
                    };
                    diagnostics.push(parse_error_to_diagnostic(user_src, &adjusted));
                }
            }
            return (
                diagnostics,
                AnalysisResult {
                    source_file: None,
                    resolved: None,
                    tc_result: None,
                },
            );
        }
    };

    // ── Name resolution ──────────────────────────────────────────────────
    let resolved = resolve(&source_file);
    for err in &resolved.errors {
        if err.span.start >= prefix_len && err.span.start < user_end {
            let adjusted = SemaError::new(
                err.message.clone(),
                ast::Span {
                    start: err.span.start - prefix_len,
                    end: err.span.end - prefix_len,
                },
            );
            diagnostics.push(sema_error_to_diagnostic(user_src, &adjusted));
        }
    }

    // ── Type checking ────────────────────────────────────────────────────
    let tc_result = type_check(&source_file, &resolved);
    for err in &tc_result.errors {
        if err.span.start >= prefix_len && err.span.start < user_end {
            let adjusted = SemaError::new(
                err.message.clone(),
                ast::Span {
                    start: err.span.start - prefix_len,
                    end: err.span.end - prefix_len,
                },
            );
            diagnostics.push(sema_error_to_diagnostic(user_src, &adjusted));
        }
    }

    // ── Connectivity + DRC (for each design) ─────────────────────────────
    for design in &tc_result.designs {
        let conn_result = build_connectivity(design, &tc_result.device_pins);
        for err in &conn_result.errors {
            if err.span.start >= prefix_len && err.span.start < user_end {
                let adjusted = SemaError::new(
                    err.message.clone(),
                    ast::Span {
                        start: err.span.start - prefix_len,
                        end: err.span.end - prefix_len,
                    },
                );
                diagnostics.push(sema_error_to_diagnostic(user_src, &adjusted));
            }
        }
        let runner = DrcRunner::new();
        let drc_diags = runner.run(&conn_result.ir);
        for diag in &drc_diags {
            if diag.span.start >= prefix_len && diag.span.start < user_end {
                let adjusted = DrcDiagnostic {
                    rule_id: diag.rule_id.clone(),
                    level: diag.level,
                    span: ast::Span {
                        start: diag.span.start - prefix_len,
                        end: diag.span.end - prefix_len,
                    },
                    instance_path: diag.instance_path.clone(),
                    message: diag.message.clone(),
                };
                diagnostics.push(drc_diagnostic_to_diagnostic(user_src, &adjusted));
            }
        }
    }

    (
        diagnostics,
        AnalysisResult {
            source_file: Some(source_file),
            resolved: Some(resolved),
            tc_result: Some(tc_result),
        },
    )
}

fn parse_error_to_diagnostic(src: &str, err: &ParseError) -> Diagnostic {
    Diagnostic {
        range: span_to_range(src, err.span),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("cohdl".into()),
        code: Some(NumberOrString::String("PARSE".into())),
        message: err.message.clone(),
        ..Default::default()
    }
}

fn sema_error_to_diagnostic(src: &str, err: &SemaError) -> Diagnostic {
    Diagnostic {
        range: span_to_range(src, err.span),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("cohdl".into()),
        code: Some(NumberOrString::String("SEMA".into())),
        message: err.message.clone(),
        ..Default::default()
    }
}

fn drc_diagnostic_to_diagnostic(src: &str, diag: &DrcDiagnostic) -> Diagnostic {
    let severity = match diag.level {
        DiagnosticLevel::Error => DiagnosticSeverity::ERROR,
        DiagnosticLevel::Warning => DiagnosticSeverity::WARNING,
    };
    let message = if diag.instance_path.is_empty() {
        diag.message.clone()
    } else {
        format!("{}: {}", diag.instance_path, diag.message)
    };
    Diagnostic {
        range: span_to_range(src, diag.span),
        severity: Some(severity),
        source: Some("cohdl".into()),
        code: Some(NumberOrString::String(diag.rule_id.clone())),
        message,
        ..Default::default()
    }
}

// ── Hover helpers ───────────────────────────────────────────────────────────

/// Find the word at a byte offset in source text.
fn word_at_offset(src: &str, offset: usize) -> Option<&str> {
    if offset >= src.len() {
        return None;
    }
    let bytes = src.as_bytes();
    // Find start of identifier
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    // Find end of identifier
    let mut end = offset;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(&src[start..end])
}

/// Check if a dot-path pattern like `inst.pin` exists at the offset.
/// Returns (instance_name, pin_name) if found.
fn dot_path_at_offset(src: &str, offset: usize) -> Option<(String, String)> {
    if offset >= src.len() {
        return None;
    }
    let bytes = src.as_bytes();
    // Find the word at offset
    let mut end = offset;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let word = &src[start..end];

    // Check for dot before or after
    if start > 0 && bytes[start - 1] == b'.' {
        // This word is the pin part: find the instance name before the dot
        let dot_pos = start - 1;
        let mut inst_start = dot_pos;
        while inst_start > 0
            && (bytes[inst_start - 1].is_ascii_alphanumeric() || bytes[inst_start - 1] == b'_')
        {
            inst_start -= 1;
        }
        if inst_start < dot_pos {
            let inst = &src[inst_start..dot_pos];
            return Some((inst.to_string(), word.to_string()));
        }
    } else if end < bytes.len() && bytes[end] == b'.' {
        // This word is the instance part: find the pin name after the dot
        let pin_start = end + 1;
        let mut pin_end = pin_start;
        while pin_end < bytes.len()
            && (bytes[pin_end].is_ascii_alphanumeric() || bytes[pin_end] == b'_')
        {
            pin_end += 1;
        }
        if pin_start < pin_end {
            let pin = &src[pin_start..pin_end];
            return Some((word.to_string(), pin.to_string()));
        }
    }

    None
}

/// Build hover content for a device declaration.
fn hover_for_device(sf: &SourceFile, name: &str) -> Option<String> {
    for item in &sf.items {
        if let TopLevelItemKind::Device(d) = &item.kind {
            if d.name.name == name {
                return Some(format_device_hover(d));
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(h) = hover_for_device_in_items(&m.items, name) {
                return Some(h);
            }
        }
    }
    None
}

fn hover_for_device_in_items(items: &[ast::TopLevelItem], name: &str) -> Option<String> {
    for item in items {
        if let TopLevelItemKind::Device(d) = &item.kind {
            if d.name.name == name {
                return Some(format_device_hover(d));
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(h) = hover_for_device_in_items(&m.items, name) {
                return Some(h);
            }
        }
    }
    None
}

fn format_device_hover(d: &ast::DeviceDecl) -> String {
    let mut parts = vec![format!("device {}", d.name.name)];
    if let Some(gp) = &d.generic_params {
        let params: Vec<String> = gp
            .params
            .iter()
            .map(|p| {
                let kind = match &p.kind {
                    ast::GenericParamKind::Type(te) => te
                        .path
                        .segments
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::"),
                    ast::GenericParamKind::ImplConstraint(tb) => {
                        let bounds: Vec<String> = tb
                            .bounds
                            .iter()
                            .map(|b| {
                                b.path
                                    .segments
                                    .iter()
                                    .map(|s| s.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join("::")
                            })
                            .collect();
                        format!("impl {}", bounds.join(" + "))
                    }
                };
                format!("{}: {}", p.name.name, kind)
            })
            .collect();
        parts[0] = format!("device {}<{}>", d.name.name, params.join(", "));
    }
    if let Some(traits) = &d.impl_traits {
        let bounds: Vec<String> = traits
            .bounds
            .iter()
            .map(|b| {
                b.path
                    .segments
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::")
            })
            .collect();
        parts.push(format!("  impl {}", bounds.join(" + ")));
    }
    format!("```cohdl\n{}\n```", parts.join("\n"))
}

/// Build hover content for a part declaration.
fn hover_for_part(sf: &SourceFile, name: &str) -> Option<String> {
    for item in &sf.items {
        if let TopLevelItemKind::Part(p) = &item.kind {
            if p.name.name == name {
                return Some(format_part_hover(p));
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(h) = hover_for_part_in_items(&m.items, name) {
                return Some(h);
            }
        }
    }
    None
}

fn hover_for_part_in_items(items: &[ast::TopLevelItem], name: &str) -> Option<String> {
    for item in items {
        if let TopLevelItemKind::Part(p) = &item.kind {
            if p.name.name == name {
                return Some(format_part_hover(p));
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(h) = hover_for_part_in_items(&m.items, name) {
                return Some(h);
            }
        }
    }
    None
}

fn format_part_hover(p: &ast::PartDecl) -> String {
    let device_type = p
        .device_type
        .path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    let mut text = format!("part {}: {}", p.name.name, device_type);
    if let Some(ga) = &p.device_type.generic_args {
        let args: Vec<String> = ga.args.iter().map(|a| a.name.name.clone()).collect();
        text = format!("part {}: {}<{}>", p.name.name, device_type, args.join(", "));
    }
    format!("```cohdl\n{}\n```", text)
}

/// Build hover info for a pin reference (inst.pin).
fn hover_for_pin_ref(
    sf: &SourceFile,
    tc: &TypeCheckResult,
    inst_name: &str,
    pin_name: &str,
) -> Option<String> {
    // Find the device type for this instance by scanning designs
    for design in &tc.designs {
        for instance in &design.instances {
            if instance.name == inst_name {
                // Find the physical pin number from the device's pin block
                if let Some(pins) = tc.device_pins.get(&instance.device) {
                    let has_pin = pins.iter().any(|p| p == pin_name);
                    if has_pin {
                        // Try to find the physical pin number from the AST
                        let pin_num = find_pin_number(sf, &instance.device, pin_name);
                        // Find which net this pin is on
                        let net_name = find_net_for_pin(design, inst_name, pin_name);
                        let mut info = format!("{}.{}", inst_name, pin_name);
                        if let Some(num) = pin_num {
                            info.push_str(&format!("\nPhysical pin: {}", num));
                        }
                        if let Some(net) = net_name {
                            info.push_str(&format!("\nNet: {}", net));
                        }
                        return Some(format!("```\n{}\n```", info));
                    }
                }
            }
        }
    }
    None
}

/// Find the physical pin number for a pin name in a device.
fn find_pin_number(sf: &SourceFile, device_name: &str, pin_name: &str) -> Option<String> {
    for item in &sf.items {
        if let TopLevelItemKind::Device(d) = &item.kind {
            if d.name.name == device_name {
                return find_pin_in_device_body(&d.body, pin_name);
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(n) = find_pin_number_in_items(&m.items, device_name, pin_name) {
                return Some(n);
            }
        }
    }
    None
}

fn find_pin_number_in_items(
    items: &[ast::TopLevelItem],
    device_name: &str,
    pin_name: &str,
) -> Option<String> {
    for item in items {
        if let TopLevelItemKind::Device(d) = &item.kind {
            if d.name.name == device_name {
                return find_pin_in_device_body(&d.body, pin_name);
            }
        }
        if let TopLevelItemKind::Module(m) = &item.kind {
            if let Some(n) = find_pin_number_in_items(&m.items, device_name, pin_name) {
                return Some(n);
            }
        }
    }
    None
}

fn find_pin_in_device_body(body: &[DeviceBodyItem], pin_name: &str) -> Option<String> {
    for item in body {
        if let DeviceBodyItem::Pins(pins) = item {
            for entry in &pins.entries {
                match &entry.kind {
                    PinEntryKind::Single { name, number } if name.name == pin_name => {
                        return Some(number.to_string());
                    }
                    PinEntryKind::List { name, numbers } if name.name == pin_name => {
                        let nums: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();
                        return Some(format!("[{}]", nums.join(", ")));
                    }
                    PinEntryKind::Range {
                        name, start, end, ..
                    } if name.name == pin_name => {
                        return Some(format!("[{}..{}]", start, end));
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Find which net a given instance.pin is connected to.
fn find_net_for_pin(
    design: &cohdl_sema::typeck::TypedDesign,
    inst_name: &str,
    pin_name: &str,
) -> Option<String> {
    // Find instance ID
    let inst_id = design.instances.iter().find(|i| i.name == inst_name)?.id;
    // Find net containing this pin
    for net in &design.nets {
        for (id, pname) in &net.endpoints {
            if *id == inst_id && pname == pin_name {
                return Some(net.name.clone());
            }
        }
    }
    None
}

// ── Completion helpers ──────────────────────────────────────────────────────

/// Determine completion context by examining text before the cursor.
enum CompletionContext {
    /// After `inst <name>:` — suggest device/part names
    InstType,
    /// After `net <name>:` or after `,` in a net statement — suggest instance.pin combos
    NetEndpoint,
    /// No special context
    None,
}

fn determine_completion_context(src: &str, offset: usize) -> CompletionContext {
    let before = &src[..offset.min(src.len())];
    let trimmed = before.trim_end();

    // Check for `inst <name>:` pattern — cursor is after colon
    // Look backwards for the inst keyword
    if let Some(colon_pos) = trimmed.rfind(':') {
        let before_colon = trimmed[..colon_pos].trim();
        // Check if this looks like `inst <name>`
        let words: Vec<&str> = before_colon.split_whitespace().collect();
        if words.len() >= 2 && words[words.len() - 2] == "inst" {
            return CompletionContext::InstType;
        }
        // Check if this looks like `net <name>:` or is a comma in a net statement
        if words.len() >= 2 && words[words.len() - 2] == "net" {
            return CompletionContext::NetEndpoint;
        }
    }

    // Check for comma after a net endpoint (net name: endpoint, <cursor>)
    if trimmed.ends_with(',') {
        // Walk back to find if this is a net statement
        let line_start = trimmed.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line = trimmed[line_start..].trim_start();
        if line.starts_with("net ") {
            return CompletionContext::NetEndpoint;
        }
    }

    CompletionContext::None
}

/// Collect all device and part names from the source file.
fn collect_device_part_names(sf: &SourceFile) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    collect_device_part_names_from_items(&sf.items, &mut items);
    items
}

fn collect_device_part_names_from_items(
    items: &[ast::TopLevelItem],
    out: &mut Vec<CompletionItem>,
) {
    for item in items {
        match &item.kind {
            TopLevelItemKind::Device(d) => {
                out.push(CompletionItem {
                    label: d.name.name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some("device".into()),
                    ..Default::default()
                });
            }
            TopLevelItemKind::Part(p) => {
                out.push(CompletionItem {
                    label: p.name.name.clone(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some("part".into()),
                    ..Default::default()
                });
            }
            TopLevelItemKind::Module(m) => {
                collect_device_part_names_from_items(&m.items, out);
            }
            _ => {}
        }
    }
}

/// Collect instance.pin completion items from all designs.
fn collect_instance_pin_completions(_sf: &SourceFile, tc: &TypeCheckResult) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for design in &tc.designs {
        for instance in &design.instances {
            if let Some(pins) = tc.device_pins.get(&instance.device) {
                for pin in pins {
                    items.push(CompletionItem {
                        label: format!("{}.{}", instance.name, pin),
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some(format!("pin on {} ({})", instance.name, instance.device)),
                        ..Default::default()
                    });
                }
            }
        }
    }
    items
}

// ── Go to definition helpers ────────────────────────────────────────────────

/// Find the definition span of a symbol by name.
fn find_definition_span(sf: &SourceFile, name: &str) -> Option<ast::Span> {
    find_definition_in_items(&sf.items, name)
}

fn find_definition_in_items(items: &[ast::TopLevelItem], name: &str) -> Option<ast::Span> {
    for item in items {
        let item_name = match &item.kind {
            TopLevelItemKind::Trait(t) => Some(&t.name),
            TopLevelItemKind::Device(d) => Some(&d.name),
            TopLevelItemKind::Part(p) => Some(&p.name),
            TopLevelItemKind::TypeAlias(t) => Some(&t.name),
            TopLevelItemKind::Fn(f) => Some(&f.name),
            TopLevelItemKind::Design(d) => Some(&d.name),
            TopLevelItemKind::Module(m) => {
                if m.name.name == name {
                    return Some(m.name.span);
                }
                if let Some(span) = find_definition_in_items(&m.items, name) {
                    return Some(span);
                }
                None
            }
            _ => None,
        };
        if let Some(ident) = item_name {
            if ident.name == name {
                return Some(ident.span);
            }
        }
    }
    None
}

// ── LanguageServer implementation ───────────────────────────────────────────

impl CohdlLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(ServerState {
                documents: DocumentStore::default(),
                analyses: HashMap::new(),
            })),
        }
    }

    async fn on_change(&self, uri: Url, text: String) {
        // Try to resolve the full project source (modules + dependencies)
        let (combined, prefix_len) = if let Ok(file_path) = uri.to_file_path() {
            build_project_source(&file_path, &text)
        } else {
            (text.clone(), 0)
        };

        let (diagnostics, analysis) = run_diagnostics(&combined, &text, prefix_len);

        {
            let mut state = self.state.write().await;
            state.documents.docs.insert(uri.clone(), text);
            state.analyses.insert(uri.clone(), analysis);
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CohdlLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".into(), ".".into(), ",".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "cohdl-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // We use FULL sync so the entire text is in the first content change.
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut state = self.state.write().await;
            state.documents.docs.remove(&uri);
            state.analyses.remove(&uri);
        }
        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;
        let src = match state.documents.docs.get(uri) {
            Some(s) => s,
            None => return Ok(None),
        };
        let analysis = match state.analyses.get(uri) {
            Some(a) => a.clone(),
            None => return Ok(None),
        };

        let offset = position_to_offset(src, pos);

        // Check for dot-path (instance.pin) hover
        if let Some((inst_name, pin_name)) = dot_path_at_offset(src, offset) {
            if let (Some(sf), Some(tc)) = (&analysis.source_file, &analysis.tc_result) {
                if let Some(content) = hover_for_pin_ref(sf, tc, &inst_name, &pin_name) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: None,
                    }));
                }
            }
        }

        // Check for device/part name hover
        if let Some(word) = word_at_offset(src, offset) {
            let word = word.to_string();
            if let Some(sf) = &analysis.source_file {
                if let Some(content) = hover_for_device(sf, &word) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: None,
                    }));
                }
                if let Some(content) = hover_for_part(sf, &word) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;
        let src = match state.documents.docs.get(uri) {
            Some(s) => s,
            None => return Ok(None),
        };
        let analysis = match state.analyses.get(uri) {
            Some(a) => a.clone(),
            None => return Ok(None),
        };

        let offset = position_to_offset(src, pos);
        let word = match word_at_offset(src, offset) {
            Some(w) => w.to_string(),
            None => return Ok(None),
        };

        // Try resolved names first (most accurate)
        if let Some(resolved) = &analysis.resolved {
            for rn in &resolved.resolved_names {
                if rn.span.start <= offset && offset <= rn.span.end {
                    let sym = resolved.symbols.get(rn.symbol_id);
                    let range = span_to_range(src, sym.span);
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                        uri.clone(),
                        range,
                    ))));
                }
            }
        }

        // Fallback: find definition by name in the AST
        if let Some(sf) = &analysis.source_file {
            if let Some(span) = find_definition_span(sf, &word) {
                let range = span_to_range(src, span);
                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    uri.clone(),
                    range,
                ))));
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let state = self.state.read().await;
        let src = match state.documents.docs.get(uri) {
            Some(s) => s,
            None => return Ok(None),
        };
        let analysis = match state.analyses.get(uri) {
            Some(a) => a.clone(),
            None => return Ok(None),
        };

        let offset = position_to_offset(src, pos);
        let context = determine_completion_context(src, offset);

        match context {
            CompletionContext::InstType => {
                if let Some(sf) = &analysis.source_file {
                    let items = collect_device_part_names(sf);
                    return Ok(Some(CompletionResponse::Array(items)));
                }
            }
            CompletionContext::NetEndpoint => {
                if let (Some(sf), Some(tc)) = (&analysis.source_file, &analysis.tc_result) {
                    let items = collect_instance_pin_completions(sf, tc);
                    return Ok(Some(CompletionResponse::Array(items)));
                }
            }
            CompletionContext::None => {}
        }

        Ok(None)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Span conversion tests ───────────────────────────────────────────

    #[test]
    fn offset_to_position_basic() {
        let src = "hello\nworld\n";
        assert_eq!(offset_to_position(src, 0), Position::new(0, 0));
        assert_eq!(offset_to_position(src, 5), Position::new(0, 5));
        assert_eq!(offset_to_position(src, 6), Position::new(1, 0));
        assert_eq!(offset_to_position(src, 11), Position::new(1, 5));
    }

    #[test]
    fn offset_to_position_empty() {
        let src = "";
        assert_eq!(offset_to_position(src, 0), Position::new(0, 0));
        assert_eq!(offset_to_position(src, 100), Position::new(0, 0));
    }

    #[test]
    fn position_to_offset_basic() {
        let src = "hello\nworld\n";
        assert_eq!(position_to_offset(src, Position::new(0, 0)), 0);
        assert_eq!(position_to_offset(src, Position::new(0, 5)), 5);
        assert_eq!(position_to_offset(src, Position::new(1, 0)), 6);
        assert_eq!(position_to_offset(src, Position::new(1, 5)), 11);
    }

    #[test]
    fn span_to_range_converts() {
        let src = "line1\nline2\nline3\n";
        let span = ast::Span { start: 6, end: 11 };
        let range = span_to_range(src, span);
        assert_eq!(range.start, Position::new(1, 0));
        assert_eq!(range.end, Position::new(1, 5));
    }

    // ── Diagnostics pipeline tests ──────────────────────────────────────

    #[test]
    fn diagnostics_for_parse_error() {
        let src = "device {"; // Invalid syntax
        let (diagnostics, analysis) = run_diagnostics(src, src, 0);
        assert!(!diagnostics.is_empty());
        assert!(analysis.source_file.is_none());
        assert!(diagnostics
            .iter()
            .any(|d| d.code == Some(NumberOrString::String("PARSE".into()))));
    }

    #[test]
    fn diagnostics_for_valid_source() {
        let src = r#"
            trait TwoTerminal {
                pins { A: Pin, B: Pin }
            }
            device MLCC: impl TwoTerminal {
                pins { A: 1, B: 2 }
            }
        "#;
        let (diagnostics, analysis) = run_diagnostics(src, src, 0);
        // Should parse and analyze successfully (no diagnostics expected for
        // well-formed source without a design)
        assert!(analysis.source_file.is_some());
        assert!(analysis.resolved.is_some());
        assert!(analysis.tc_result.is_some());
        // No parse or sema errors expected for this basic snippet
        let error_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                d.code == Some(NumberOrString::String("PARSE".into()))
                    || d.code == Some(NumberOrString::String("SEMA".into()))
            })
            .collect();
        // This source has no errors
        assert!(
            error_diags.is_empty(),
            "unexpected errors: {:?}",
            error_diags
        );
    }

    #[test]
    fn diagnostics_for_undefined_symbol() {
        let src = r#"
            design Board {
                inst c: NonExistent
            }
        "#;
        let (diagnostics, _) = run_diagnostics(src, src, 0);
        assert!(diagnostics.iter().any(|d| d.message.contains("undefined")));
    }

    #[test]
    fn diagnostics_carry_correct_range() {
        // A small file with an undefined reference — the diagnostic span should
        // map to the correct line/col.
        let src = "design X {\n    inst c: Missing\n}\n";
        let (diagnostics, _) = run_diagnostics(src, src, 0);
        let undef = diagnostics.iter().find(|d| d.message.contains("undefined"));
        assert!(undef.is_some());
        let range = undef.unwrap().range;
        // "Missing" is on line 1 (0-based), column 12
        assert_eq!(range.start.line, 1);
    }

    #[test]
    fn diagnostics_sema_error_conversion() {
        let err = SemaError {
            message: "test error".into(),
            span: ast::Span { start: 6, end: 11 },
        };
        let src = "line1\nline2\nline3\n";
        let diag = sema_error_to_diagnostic(src, &err);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.range.start.line, 1);
    }

    #[test]
    fn diagnostics_drc_warning_severity() {
        let diag = DrcDiagnostic {
            rule_id: "W001".into(),
            level: DiagnosticLevel::Warning,
            span: ast::Span { start: 0, end: 5 },
            instance_path: "Board::c1".into(),
            message: "test warning".into(),
        };
        let src = "hello\nworld\n";
        let lsp_diag = drc_diagnostic_to_diagnostic(src, &diag);
        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::WARNING));
        assert!(lsp_diag.message.contains("Board::c1"));
        assert!(lsp_diag.message.contains("test warning"));
    }

    #[test]
    fn diagnostics_drc_error_severity() {
        let diag = DrcDiagnostic {
            rule_id: "E001".into(),
            level: DiagnosticLevel::Error,
            span: ast::Span { start: 0, end: 3 },
            instance_path: "".into(),
            message: "voltage exceeded".into(),
        };
        let src = "abc";
        let lsp_diag = drc_diagnostic_to_diagnostic(src, &diag);
        assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.message, "voltage exceeded");
    }

    // ── Word-at-offset tests ────────────────────────────────────────────

    #[test]
    fn word_at_offset_finds_identifier() {
        let src = "inst mcu: STM32";
        assert_eq!(word_at_offset(src, 5), Some("mcu"));
        assert_eq!(word_at_offset(src, 10), Some("STM32"));
    }

    #[test]
    fn word_at_offset_at_boundary() {
        let src = "hello world";
        assert_eq!(word_at_offset(src, 0), Some("hello"));
        assert_eq!(word_at_offset(src, 4), Some("hello"));
        assert_eq!(word_at_offset(src, 6), Some("world"));
    }

    // ── Dot-path detection tests ────────────────────────────────────────

    #[test]
    fn dot_path_at_offset_detects_pin_ref() {
        let src = "net vdd: mcu.VDD_IO";
        // Cursor on "mcu" part
        assert_eq!(
            dot_path_at_offset(src, 9),
            Some(("mcu".into(), "VDD_IO".into()))
        );
        // Cursor on "VDD_IO" part
        assert_eq!(
            dot_path_at_offset(src, 13),
            Some(("mcu".into(), "VDD_IO".into()))
        );
    }

    // ── Completion context tests ────────────────────────────────────────

    #[test]
    fn completion_context_inst_type() {
        let src = "inst mcu: ";
        match determine_completion_context(src, src.len()) {
            CompletionContext::InstType => {}
            _ => panic!("expected InstType context"),
        }
    }

    #[test]
    fn completion_context_net_endpoint() {
        let src = "net vdd: ";
        match determine_completion_context(src, src.len()) {
            CompletionContext::NetEndpoint => {}
            _ => panic!("expected NetEndpoint context"),
        }
    }

    #[test]
    fn completion_context_net_comma() {
        let src = "net vdd: mcu.VDD,";
        match determine_completion_context(src, src.len()) {
            CompletionContext::NetEndpoint => {}
            _ => panic!("expected NetEndpoint context after comma"),
        }
    }

    #[test]
    fn completion_context_none() {
        let src = "hello world";
        match determine_completion_context(src, src.len()) {
            CompletionContext::None => {}
            _ => panic!("expected None context"),
        }
    }

    // ── Device/part name collection ─────────────────────────────────────

    #[test]
    fn collects_device_and_part_names() {
        let src = r#"
            trait TwoTerminal {
                pins { A: Pin, B: Pin }
            }
            device MLCC: impl TwoTerminal {
                pins { A: 1, B: 2 }
            }
            part mlcc_100nF: MLCC {
                primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            }
        "#;
        let sf = parse_source_file(src).unwrap();
        let items = collect_device_part_names(&sf);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"MLCC"));
        assert!(labels.contains(&"mlcc_100nF"));
    }

    // ── Definition lookup ───────────────────────────────────────────────

    #[test]
    fn find_definition_span_works() {
        let src = r#"
            trait TwoTerminal {
                pins { A: Pin, B: Pin }
            }
            device MLCC: impl TwoTerminal {
                pins { A: 1, B: 2 }
            }
        "#;
        let sf = parse_source_file(src).unwrap();
        let span = find_definition_span(&sf, "MLCC");
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(&src[span.start..span.end], "MLCC");
    }

    #[test]
    fn find_definition_span_in_module() {
        let src = r#"
            module power {
                pub fn decoupling(vdd: Net) {}
            }
        "#;
        let sf = parse_source_file(src).unwrap();
        let span = find_definition_span(&sf, "decoupling");
        assert!(span.is_some());
    }
}

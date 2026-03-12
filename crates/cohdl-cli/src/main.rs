use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use cohdl_codegen_kicad::{emit_avl_bom, emit_kicad_netlist, emit_simple_bom};
use cohdl_codegen_lceda::emit_lceda_netlist;
use cohdl_drc::{DiagnosticLevel, DrcDiagnostic, DrcRunner};
use cohdl_parser::{parse_source_file, ParseError};
use cohdl_sema::connectivity::{build_connectivity, ConnectivityResult};
use cohdl_sema::designator::{instance_infos_from_typed_design, DesignatorDb};
use cohdl_sema::typeck::{type_check, TypeCheckResult};
use cohdl_sema::{resolve, ResolvedSourceFile, SemaError};

// ── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "cohdl",
    version,
    about = "The cohdl hardware-description language compiler"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Control colored output
    #[arg(long, global = true, default_value = "auto")]
    color: ColorOption,
}

#[derive(Clone, ValueEnum)]
enum ColorOption {
    Auto,
    Always,
    Never,
}

impl ColorOption {
    fn to_color_choice(&self) -> ColorChoice {
        match self {
            ColorOption::Auto => ColorChoice::Auto,
            ColorOption::Always => ColorChoice::Always,
            ColorOption::Never => ColorChoice::Never,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Full pipeline: parse → sema → DRC → codegen
    Build {
        /// Name of the design to compile (defaults to `top` from cohdl.toml)
        #[arg(long)]
        design: Option<String>,

        /// What to emit
        #[arg(long, value_delimiter = ',', default_value = "all")]
        emit: Vec<EmitTarget>,

        /// Output directory
        #[arg(long, default_value = "out")]
        out_dir: PathBuf,
    },
    /// Parse + sema + DRC only (no codegen)
    Check,
    /// Format source files (placeholder)
    Fmt,
    /// Initialize a new cohdl project
    Init {
        /// Project name (defaults to directory name)
        name: Option<String>,
    },
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
enum EmitTarget {
    /// LCEDA Pro netlist (.enet) — default
    Netlist,
    /// KiCad legacy netlist (.net)
    NetlistKicad,
    BomSimple,
    BomAvl,
    All,
}

// ── cohdl.toml ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CohdlManifest {
    package: PackageSection,
    design: DesignSection,
    #[serde(default)]
    dependencies: std::collections::HashMap<String, DependencySpec>,
}

#[derive(Deserialize)]
struct PackageSection {
    name: String,
    #[allow(dead_code)]
    version: String,
}

#[derive(Deserialize)]
struct DesignSection {
    root: String,
    top: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DependencySpec {
    Simple(String),
    Table {
        #[allow(dead_code)]
        path: Option<String>,
    },
}

fn load_manifest() -> Result<CohdlManifest, String> {
    let content = fs::read_to_string("cohdl.toml")
        .map_err(|e| format!("could not read cohdl.toml: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("invalid cohdl.toml: {}", e))
}

// ── Source map ──────────────────────────────────────────────────────────────

/// Tracks which regions of the combined source string belong to which file.
struct SourceMap {
    entries: Vec<SourceMapEntry>,
}

struct SourceMapEntry {
    /// Start byte offset of the file's content in the combined source.
    content_start: usize,
    /// Length of the file's content.
    content_len: usize,
    /// File path to display in diagnostics.
    file: String,
}

impl SourceMap {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Look up which file a byte offset belongs to.
    /// Returns `(file_path, content_slice, local_offset)`.
    fn lookup<'a>(&self, combined_src: &'a str, offset: usize) -> (&str, &'a str, usize) {
        for entry in &self.entries {
            if offset >= entry.content_start && offset < entry.content_start + entry.content_len {
                let slice =
                    &combined_src[entry.content_start..entry.content_start + entry.content_len];
                return (&entry.file, slice, offset - entry.content_start);
            }
        }
        // Fallback: treat the whole source as a single file.
        ("unknown", combined_src, offset)
    }

    /// Shift all entry offsets by `delta` (used when prepending dependency source).
    fn shift(&mut self, delta: usize) {
        for entry in &mut self.entries {
            entry.content_start += delta;
        }
    }
}

/// Resolve `module <name>` declarations in the root source file.
///
/// Parses the root file to find `ModDecl` nodes, loads each referenced
/// `<name>.cohdl` from the same directory, and returns the combined source
/// with module files prepended before the root source (like Rust's `mod`).
fn resolve_modules(root_path: &str, root_src: &str) -> Result<(String, SourceMap), String> {
    use cohdl_syntax::ast::TopLevelItemKind;

    let source_file = match parse_source_file(root_src) {
        Ok(sf) => sf,
        // Parse errors will be reported with full diagnostics by run_pipeline;
        // just return the root source as-is so the pipeline can handle it.
        Err(_) => {
            let mut sm = SourceMap::new();
            sm.entries.push(SourceMapEntry {
                content_start: 0,
                content_len: root_src.len(),
                file: root_path.to_string(),
            });
            return Ok((root_src.to_string(), sm));
        }
    };

    let mod_items: Vec<(&str, bool)> = source_file
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            TopLevelItemKind::Mod(m) => Some((m.name.name.as_str(), item.visibility.is_some())),
            _ => None,
        })
        .collect();

    if mod_items.is_empty() {
        let mut sm = SourceMap::new();
        sm.entries.push(SourceMapEntry {
            content_start: 0,
            content_len: root_src.len(),
            file: root_path.to_string(),
        });
        return Ok((root_src.to_string(), sm));
    }

    let src_dir = Path::new(root_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let mut combined = String::new();
    let mut sm = SourceMap::new();
    for (name, is_pub) in &mod_items {
        let mod_path = src_dir.join(format!("{}.cohdl", name));
        let content = fs::read_to_string(&mod_path).map_err(|e| {
            format!(
                "could not read module `{}` ({}): {}",
                name,
                mod_path.display(),
                e
            )
        })?;
        let vis = if *is_pub { "pub " } else { "" };
        let prefix = format!("{}module {} {{\n", vis, name);
        let content_start = combined.len() + prefix.len();
        combined.push_str(&prefix);
        combined.push_str(&content);
        sm.entries.push(SourceMapEntry {
            content_start,
            content_len: content.len(),
            file: mod_path.display().to_string(),
        });
        combined.push_str("\n}\n");
    }
    let root_start = combined.len();
    combined.push_str(root_src);
    sm.entries.push(SourceMapEntry {
        content_start: root_start,
        content_len: root_src.len(),
        file: root_path.to_string(),
    });
    Ok((combined, sm))
}

// ── Dependency resolution ───────────────────────────────────────────────────

/// Resolve a single dependency's source into a `module <name> { ... }` block.
fn resolve_dependency_source(name: &str, spec: &DependencySpec) -> Result<String, String> {
    match spec {
        DependencySpec::Simple(_version) => {
            if name == "std" {
                resolve_bundled_std()
            } else {
                Err(format!(
                    "unknown bundled dependency `{}`; only `std` is supported",
                    name
                ))
            }
        }
        DependencySpec::Table { path } => {
            if let Some(dep_path) = path {
                resolve_path_dependency(name, dep_path)
            } else {
                Err(format!(
                    "dependency `{}` has no `path`; only path and bundled dependencies are supported",
                    name
                ))
            }
        }
    }
}

/// Build the synthesized source for the bundled `std` dependency.
///
/// Reads the std library from `~/.cohdl/lib/std` (installed by `scripts/install.sh`).
fn resolve_bundled_std() -> Result<String, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let std_path = PathBuf::from(home).join(".cohdl").join("lib").join("std");
    if !std_path.exists() {
        return Err(format!(
            "std library not found at {}; run `scripts/install.sh` to install it",
            std_path.display()
        ));
    }
    resolve_path_dependency("std", std_path.to_str().unwrap())
}

/// Build the synthesized source for a path-based dependency.
fn resolve_path_dependency(name: &str, dep_path: &str) -> Result<String, String> {
    use cohdl_syntax::ast::TopLevelItemKind;

    let dep_dir = Path::new(dep_path);
    let manifest_path = dep_dir.join("cohdl.toml");
    let manifest_content = fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "could not read dependency `{}` manifest ({}): {}",
            name,
            manifest_path.display(),
            e
        )
    })?;
    let manifest: CohdlManifest = toml::from_str(&manifest_content)
        .map_err(|e| format!("invalid cohdl.toml for dependency `{}`: {}", name, e))?;

    let root_path = dep_dir.join(&manifest.design.root);
    let root_src = fs::read_to_string(&root_path).map_err(|e| {
        format!(
            "could not read dependency `{}` root ({}): {}",
            name,
            root_path.display(),
            e
        )
    })?;

    let source_file = match parse_source_file(&root_src) {
        Ok(sf) => sf,
        Err(_) => return Ok(format!("module {} {{\n{}\n}}\n", name, root_src)),
    };

    let src_dir = root_path.parent().unwrap_or_else(|| Path::new("."));
    let mut inner = String::new();

    for item in &source_file.items {
        if let TopLevelItemKind::Mod(m) = &item.kind {
            let mod_name = &m.name.name;
            let mod_path = src_dir.join(format!("{}.cohdl", mod_name));
            let content = fs::read_to_string(&mod_path).map_err(|e| {
                format!(
                    "could not read module `{}` of dependency `{}` ({}): {}",
                    mod_name,
                    name,
                    mod_path.display(),
                    e
                )
            })?;
            let vis = if item.visibility.is_some() {
                "pub "
            } else {
                ""
            };
            inner.push_str(&format!("{}module {} {{\n{}\n}}\n", vis, mod_name, content));
        }
    }

    // Note: we skip appending root_src's bare `mod` declarations since the
    // inline module blocks above already define those modules.
    Ok(format!("module {} {{\n{}\n}}\n", name, inner))
}

/// Resolve all dependencies and return concatenated source to prepend.
fn resolve_dependencies(manifest: &CohdlManifest) -> Result<(String, SourceMap), String> {
    let mut dep_src = String::new();
    let sm = SourceMap::new();
    // Sort dependency names for deterministic output
    let mut dep_names: Vec<&String> = manifest.dependencies.keys().collect();
    dep_names.sort();
    for name in dep_names {
        let spec = &manifest.dependencies[name];
        let src = resolve_dependency_source(name, spec)?;
        dep_src.push_str(&src);
        dep_src.push('\n');
    }
    Ok((dep_src, sm))
}

// ── Diagnostic rendering ────────────────────────────────────────────────────

/// Compute (line, col) from a byte offset into source text. Both are 1-based.
fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Get the source line containing the given byte offset (0-indexed line text).
fn source_line_at(src: &str, offset: usize) -> (usize, &str) {
    let mut line_num = 1;
    let mut line_start = 0;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line_num += 1;
            line_start = i + 1;
        }
    }
    let line_end = src[line_start..]
        .find('\n')
        .map(|p| line_start + p)
        .unwrap_or(src.len());
    (line_num, &src[line_start..line_end])
}

fn render_parse_errors(
    stderr: &mut StandardStream,
    source_map: &SourceMap,
    src: &str,
    errors: &[ParseError],
) {
    for err in errors {
        let (file_path, file_src, local_offset) = source_map.lookup(src, err.span.start);
        let (line, col) = line_col(file_src, local_offset);
        let (line_num, line_text) = source_line_at(file_src, local_offset);
        let span_len = (err.span.end - err.span.start).max(1);

        // Error[PARSE]: message
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error[PARSE]").ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": parse error").ok();
        stderr.reset().ok();

        // --> file:line:col
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        // source line
        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        // underline
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        write!(stderr, "{} | ", padding).ok();
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();

        // message
        let msg_padding = " ".repeat(gutter.len());
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        write!(stderr, "{} | ", msg_padding).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", err.message).ok();
        writeln!(stderr).ok();
    }
}

fn render_sema_errors(
    stderr: &mut StandardStream,
    source_map: &SourceMap,
    src: &str,
    errors: &[SemaError],
) {
    for err in errors {
        let (file_path, file_src, local_offset) = source_map.lookup(src, err.span.start);
        let (line, col) = line_col(file_src, local_offset);
        let (line_num, line_text) = source_line_at(file_src, local_offset);
        let span_len = (err.span.end - err.span.start).max(1);

        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error[SEMA]").ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": {}", err.message).ok();
        stderr.reset().ok();

        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        write!(stderr, "{} | ", padding).ok();
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();
        writeln!(stderr).ok();
    }
}

fn render_drc_diagnostics(
    stderr: &mut StandardStream,
    source_map: &SourceMap,
    src: &str,
    diagnostics: &[DrcDiagnostic],
) {
    for diag in diagnostics {
        let (file_path, file_src, local_offset) = source_map.lookup(src, diag.span.start);
        let (line, col) = line_col(file_src, local_offset);
        let (line_num, line_text) = source_line_at(file_src, local_offset);
        let span_len = (diag.span.end - diag.span.start).max(1);

        let (color, label) = match diag.level {
            DiagnosticLevel::Error => (Color::Red, "Error"),
            DiagnosticLevel::Warning => (Color::Yellow, "Warning"),
        };

        // Error[E001]: message
        stderr
            .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
            .ok();
        write!(stderr, "{}[{}]", label, diag.rule_id).ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": {}", diag.instance_path).ok();
        stderr.reset().ok();

        // --> file:line:col
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        // source line
        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        // underline
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        write!(stderr, "{} | ", padding).ok();
        stderr
            .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
            .ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();

        // message
        let msg_padding = " ".repeat(gutter.len());
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        write!(stderr, "{} | ", msg_padding).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", diag.message).ok();
        writeln!(stderr).ok();
    }
}

// ── Pipeline ────────────────────────────────────────────────────────────────

struct PipelineResult {
    connectivity: Option<ConnectivityResult>,
    tc_result: Option<TypeCheckResult>,
    design_name: Option<String>,
    has_errors: bool,
}

fn run_pipeline(
    stderr: &mut StandardStream,
    source_map: &SourceMap,
    src: &str,
    top_design: &str,
) -> PipelineResult {
    let mut has_errors = false;

    // ── Parse ────────────────────────────────────────────────────────────
    let source_file = match parse_source_file(src) {
        Ok(sf) => sf,
        Err(errors) => {
            render_parse_errors(stderr, source_map, src, &errors);
            return PipelineResult {
                connectivity: None,
                tc_result: None,
                design_name: None,
                has_errors: true,
            };
        }
    };

    // ── Sema: name resolution ────────────────────────────────────────────
    let resolved: ResolvedSourceFile = resolve(&source_file);
    if !resolved.errors.is_empty() {
        render_sema_errors(stderr, source_map, src, &resolved.errors);
        has_errors = true;
    }

    // ── Sema: type checking ──────────────────────────────────────────────
    let tc_result: TypeCheckResult = type_check(&source_file, &resolved);
    if !tc_result.errors.is_empty() {
        render_sema_errors(stderr, source_map, src, &tc_result.errors);
        has_errors = true;
    }

    // Find the requested design
    let design = tc_result.designs.iter().find(|d| d.name == top_design);
    let design = match design {
        Some(d) => d,
        None => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": design `{}` not found", top_design).ok();
            return PipelineResult {
                connectivity: None,
                tc_result: Some(tc_result),
                design_name: None,
                has_errors: true,
            };
        }
    };

    let design_name = design.name.clone();

    // ── Connectivity ─────────────────────────────────────────────────────
    let conn_result = build_connectivity(design, &tc_result.device_pins);
    if !conn_result.errors.is_empty() {
        render_sema_errors(stderr, source_map, src, &conn_result.errors);
        has_errors = true;
    }

    // ── DRC ──────────────────────────────────────────────────────────────
    let runner = DrcRunner::new();
    let drc_diags = runner.run(&conn_result.ir);
    if !drc_diags.is_empty() {
        render_drc_diagnostics(stderr, source_map, src, &drc_diags);
        if drc_diags.iter().any(|d| d.level == DiagnosticLevel::Error) {
            has_errors = true;
        }
    }

    PipelineResult {
        connectivity: Some(conn_result),
        tc_result: Some(tc_result),
        design_name: Some(design_name),
        has_errors,
    }
}

// ── Subcommand handlers ─────────────────────────────────────────────────────

fn cmd_build(
    stderr: &mut StandardStream,
    design_override: Option<String>,
    emit: Vec<EmitTarget>,
    out_dir: PathBuf,
) -> i32 {
    let manifest = match load_manifest() {
        Ok(m) => m,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let top_design = match design_override
        .as_deref()
        .or(manifest.design.top.as_deref())
    {
        Some(t) => t,
        None => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(
                stderr,
                ": no top design specified (use --design or set design.top in cohdl.toml)"
            )
            .ok();
            return 1;
        }
    };
    let file_path = &manifest.design.root;

    let root_src = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": could not read {}: {}", file_path, e).ok();
            return 1;
        }
    };

    let (user_src, mut user_sm) = match resolve_modules(file_path, &root_src) {
        Ok(pair) => pair,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let (dep_src, mut dep_sm) = match resolve_dependencies(&manifest) {
        Ok(pair) => pair,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let src = format!("{}{}", dep_src, user_src);
    // Shift user source map entries by the length of the dependency source prefix.
    user_sm.shift(dep_src.len());
    let mut source_map = SourceMap::new();
    source_map.entries.append(&mut dep_sm.entries);
    source_map.entries.append(&mut user_sm.entries);

    let result = run_pipeline(stderr, &source_map, &src, top_design);

    if result.has_errors {
        return 1;
    }

    let mut conn = match result.connectivity {
        Some(c) => c,
        None => return 1,
    };

    // ── Designator assignment ────────────────────────────────────────────
    if let (Some(tc_result), Some(design_name)) = (&result.tc_result, &result.design_name) {
        if let Some(design) = tc_result.designs.iter().find(|d| &d.name == design_name) {
            let lock_path = Path::new("design.lock");
            let mut db = match DesignatorDb::load(lock_path) {
                Ok(db) => db,
                Err(e) => {
                    stderr
                        .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))
                        .ok();
                    write!(stderr, "Warning").ok();
                    stderr.reset().ok();
                    writeln!(stderr, ": {}", e).ok();
                    DesignatorDb::new()
                }
            };

            // Collect old paths before assignment for tombstoning.
            let old_paths: Vec<String> = db.designators().keys().cloned().collect();

            let infos =
                instance_infos_from_typed_design(design, &conn.ir, &tc_result.trait_prefixes);
            let (assignments, desig_errors) = db.assign(&infos);

            if !desig_errors.is_empty() {
                render_sema_errors(stderr, &source_map, &src, &desig_errors);
                return 1;
            }

            // Tombstone removed instances (paths in old_paths that are no longer live).
            let live_paths: std::collections::HashSet<&str> =
                infos.iter().map(|i| i.hierarchical_path.as_str()).collect();
            let removed: Vec<String> = old_paths
                .into_iter()
                .filter(|p| !live_paths.contains(p.as_str()))
                .collect();
            if !removed.is_empty() {
                db.tombstone_removed(&removed);
            }

            // Apply designators to the connectivity IR.
            conn.ir.apply_designators(&assignments);

            // Save the lock file.
            if let Err(e) = db.save(lock_path) {
                stderr
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                    .ok();
                write!(stderr, "Error").ok();
                stderr.reset().ok();
                writeln!(stderr, ": {}", e).ok();
                return 1;
            }
        }
    }

    // ── Codegen ──────────────────────────────────────────────────────────
    let emit_all = emit.contains(&EmitTarget::All);

    if let Err(e) = fs::create_dir_all(&out_dir) {
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error").ok();
        stderr.reset().ok();
        writeln!(stderr, ": could not create output directory: {}", e).ok();
        return 1;
    }

    if emit_all || emit.contains(&EmitTarget::Netlist) {
        let netlist = emit_lceda_netlist(&conn.ir);
        let path = out_dir.join(format!("{}.enet", manifest.package.name));
        if let Err(e) = fs::write(&path, &netlist) {
            writeln!(stderr, "Error: could not write {}: {}", path.display(), e).ok();
            return 1;
        }
        writeln!(stderr, "  Wrote {}", path.display()).ok();
    }

    if emit.contains(&EmitTarget::NetlistKicad) {
        let netlist = emit_kicad_netlist(&conn.ir);
        let path = out_dir.join(format!("{}.net", manifest.package.name));
        if let Err(e) = fs::write(&path, &netlist) {
            writeln!(stderr, "Error: could not write {}: {}", path.display(), e).ok();
            return 1;
        }
        writeln!(stderr, "  Wrote {}", path.display()).ok();
    }

    if emit_all || emit.contains(&EmitTarget::BomSimple) {
        let bom = emit_simple_bom(&conn.ir);
        let path = out_dir.join(format!("{}-bom.csv", manifest.package.name));
        if let Err(e) = fs::write(&path, &bom) {
            writeln!(stderr, "Error: could not write {}: {}", path.display(), e).ok();
            return 1;
        }
        writeln!(stderr, "  Wrote {}", path.display()).ok();
    }

    if emit_all || emit.contains(&EmitTarget::BomAvl) {
        let bom = emit_avl_bom(&conn.ir);
        let path = out_dir.join(format!("{}-bom-avl.csv", manifest.package.name));
        if let Err(e) = fs::write(&path, &bom) {
            writeln!(stderr, "Error: could not write {}: {}", path.display(), e).ok();
            return 1;
        }
        writeln!(stderr, "  Wrote {}", path.display()).ok();
    }

    0
}

fn cmd_check(stderr: &mut StandardStream) -> i32 {
    let manifest = match load_manifest() {
        Ok(m) => m,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let top_design = match manifest.design.top.as_deref() {
        Some(t) => t,
        None => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(
                stderr,
                ": no top design specified (set design.top in cohdl.toml)"
            )
            .ok();
            return 1;
        }
    };

    let file_path = &manifest.design.root;
    let root_src = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": could not read {}: {}", file_path, e).ok();
            return 1;
        }
    };

    let (user_src, mut user_sm) = match resolve_modules(file_path, &root_src) {
        Ok(pair) => pair,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let (dep_src, mut dep_sm) = match resolve_dependencies(&manifest) {
        Ok(pair) => pair,
        Err(e) => {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let src = format!("{}{}", dep_src, user_src);
    user_sm.shift(dep_src.len());
    let mut source_map = SourceMap::new();
    source_map.entries.append(&mut dep_sm.entries);
    source_map.entries.append(&mut user_sm.entries);

    let result = run_pipeline(stderr, &source_map, &src, top_design);

    if result.has_errors {
        1
    } else {
        writeln!(stderr, "  No errors found.").ok();
        0
    }
}

fn cmd_init(stderr: &mut StandardStream, name: Option<String>) -> i32 {
    // Determine project name: explicit arg > current directory name > fallback
    let project_name = match name {
        Some(n) => n,
        None => std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "my-project".to_string()),
    };

    // Refuse to overwrite an existing cohdl.toml
    if std::path::Path::new("cohdl.toml").exists() {
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error").ok();
        stderr.reset().ok();
        writeln!(stderr, ": cohdl.toml already exists").ok();
        return 1;
    }

    let manifest = format!(
        r#"[package]
name    = "{}"
version = "0.1.0"

[design]
root = "src/main.cohdl"
top  = "MainBoard"

[dependencies]
std = "0.1.0"
"#,
        project_name
    );

    let starter_source = r#"design MainBoard {
}
"#;

    if let Err(e) = fs::create_dir_all("src") {
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error").ok();
        stderr.reset().ok();
        writeln!(stderr, ": could not create src directory: {}", e).ok();
        return 1;
    }

    if let Err(e) = fs::write("cohdl.toml", &manifest) {
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .ok();
        write!(stderr, "Error").ok();
        stderr.reset().ok();
        writeln!(stderr, ": could not write cohdl.toml: {}", e).ok();
        return 1;
    }

    let source_path = std::path::Path::new("src/main.cohdl");
    if !source_path.exists() {
        if let Err(e) = fs::write(source_path, starter_source) {
            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": could not write src/main.cohdl: {}", e).ok();
            return 1;
        }
    }

    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .ok();
    write!(stderr, "  Created").ok();
    stderr.reset().ok();
    writeln!(stderr, " project `{}`", project_name).ok();

    0
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let mut stderr = StandardStream::stderr(cli.color.to_color_choice());

    let exit_code = match cli.command {
        Command::Build {
            design,
            emit,
            out_dir,
        } => cmd_build(&mut stderr, design, emit, out_dir),
        Command::Check => cmd_check(&mut stderr),
        Command::Fmt => {
            println!("formatter not yet implemented");
            0
        }
        Command::Init { name } => cmd_init(&mut stderr, name),
    };

    process::exit(exit_code);
}

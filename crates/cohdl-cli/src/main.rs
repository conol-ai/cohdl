use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use cohdl_codegen_kicad::{emit_avl_bom, emit_kicad_netlist, emit_simple_bom};
use cohdl_drc::{DiagnosticLevel, DrcDiagnostic, DrcRunner};
use cohdl_parser::{parse_source_file, ParseError};
use cohdl_sema::connectivity::{build_connectivity, ConnectivityResult};
use cohdl_sema::typeck::{type_check, TypeCheckResult};
use cohdl_sema::{resolve, ResolvedSourceFile, SemaError};

// ── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "cohdl", version, about = "The cohdl hardware-description language compiler")]
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
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
enum EmitTarget {
    Netlist,
    BomSimple,
    BomAvl,
    All,
}

// ── cohdl.toml ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CohdlManifest {
    package: PackageSection,
    design: DesignSection,
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
    top: String,
}

fn load_manifest() -> Result<CohdlManifest, String> {
    let content = fs::read_to_string("cohdl.toml")
        .map_err(|e| format!("could not read cohdl.toml: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("invalid cohdl.toml: {}", e))
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
    file_path: &str,
    src: &str,
    errors: &[ParseError],
) {
    for err in errors {
        let (line, col) = line_col(src, err.span.start);
        let (line_num, line_text) = source_line_at(src, err.span.start);
        let span_len = (err.span.end - err.span.start).max(1);

        // Error[PARSE]: message
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
        write!(stderr, "Error[PARSE]").ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": parse error").ok();
        stderr.reset().ok();

        // --> file:line:col
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        // source line
        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        // underline
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        write!(stderr, "{} | ", padding).ok();
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();

        // message
        let msg_padding = " ".repeat(gutter.len());
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        write!(stderr, "{} | ", msg_padding).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", err.message).ok();
        writeln!(stderr).ok();
    }
}

fn render_sema_errors(
    stderr: &mut StandardStream,
    file_path: &str,
    src: &str,
    errors: &[SemaError],
) {
    for err in errors {
        let (line, col) = line_col(src, err.span.start);
        let (line_num, line_text) = source_line_at(src, err.span.start);
        let span_len = (err.span.end - err.span.start).max(1);

        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
        write!(stderr, "Error[SEMA]").ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": {}", err.message).ok();
        stderr.reset().ok();

        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        write!(stderr, "{} | ", padding).ok();
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();
        writeln!(stderr).ok();
    }
}

fn render_drc_diagnostics(
    stderr: &mut StandardStream,
    file_path: &str,
    src: &str,
    diagnostics: &[DrcDiagnostic],
) {
    for diag in diagnostics {
        let (line, col) = line_col(src, diag.span.start);
        let (line_num, line_text) = source_line_at(src, diag.span.start);
        let span_len = (diag.span.end - diag.span.start).max(1);

        let (color, label) = match diag.level {
            DiagnosticLevel::Error => (Color::Red, "Error"),
            DiagnosticLevel::Warning => (Color::Yellow, "Warning"),
        };

        // Error[E001]: message
        stderr.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true)).ok();
        write!(stderr, "{}[{}]", label, diag.rule_id).ok();
        stderr.set_color(ColorSpec::new().set_bold(true)).ok();
        writeln!(stderr, ": {}", diag.instance_path).ok();
        stderr.reset().ok();

        // --> file:line:col
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "  --> {}:{}:{}", file_path, line, col).ok();
        stderr.reset().ok();

        // source line
        let gutter = format!("{}", line_num);
        let padding = " ".repeat(gutter.len());
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        writeln!(stderr, "{} |", padding).ok();
        write!(stderr, "{} | ", gutter).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", line_text).ok();

        // underline
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        write!(stderr, "{} | ", padding).ok();
        stderr.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true)).ok();
        write!(stderr, "{}{}", " ".repeat(col - 1), "^".repeat(span_len)).ok();
        stderr.reset().ok();
        writeln!(stderr).ok();

        // message
        let msg_padding = " ".repeat(gutter.len());
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
        write!(stderr, "{} | ", msg_padding).ok();
        stderr.reset().ok();
        writeln!(stderr, "{}", diag.message).ok();
        writeln!(stderr).ok();
    }
}

// ── Pipeline ────────────────────────────────────────────────────────────────

struct PipelineResult {
    connectivity: Option<ConnectivityResult>,
    has_errors: bool,
}

fn run_pipeline(
    stderr: &mut StandardStream,
    file_path: &str,
    src: &str,
    top_design: &str,
) -> PipelineResult {
    let mut has_errors = false;

    // ── Parse ────────────────────────────────────────────────────────────
    let source_file = match parse_source_file(src) {
        Ok(sf) => sf,
        Err(errors) => {
            render_parse_errors(stderr, file_path, src, &errors);
            return PipelineResult {
                connectivity: None,
                has_errors: true,
            };
        }
    };

    // ── Sema: name resolution ────────────────────────────────────────────
    let resolved: ResolvedSourceFile = resolve(&source_file);
    if !resolved.errors.is_empty() {
        render_sema_errors(stderr, file_path, src, &resolved.errors);
        has_errors = true;
    }

    // ── Sema: type checking ──────────────────────────────────────────────
    let tc_result: TypeCheckResult = type_check(&source_file, &resolved);
    if !tc_result.errors.is_empty() {
        render_sema_errors(stderr, file_path, src, &tc_result.errors);
        has_errors = true;
    }

    // Find the requested design
    let design = tc_result.designs.iter().find(|d| d.name == top_design);
    let design = match design {
        Some(d) => d,
        None => {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": design `{}` not found", top_design).ok();
            return PipelineResult {
                connectivity: None,
                has_errors: true,
            };
        }
    };

    // ── Connectivity ─────────────────────────────────────────────────────
    let conn_result = build_connectivity(design, &tc_result.device_pins);
    if !conn_result.errors.is_empty() {
        render_sema_errors(stderr, file_path, src, &conn_result.errors);
        has_errors = true;
    }

    // ── DRC ──────────────────────────────────────────────────────────────
    let runner = DrcRunner::new();
    let drc_diags = runner.run(&conn_result.ir);
    if !drc_diags.is_empty() {
        render_drc_diagnostics(stderr, file_path, src, &drc_diags);
        if drc_diags
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error)
        {
            has_errors = true;
        }
    }

    PipelineResult {
        connectivity: Some(conn_result),
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
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let top_design = design_override
        .as_deref()
        .unwrap_or(&manifest.design.top);
    let file_path = &manifest.design.root;

    let src = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": could not read {}: {}", file_path, e).ok();
            return 1;
        }
    };

    let result = run_pipeline(stderr, file_path, &src, top_design);

    if result.has_errors {
        return 1;
    }

    let conn = match result.connectivity {
        Some(c) => c,
        None => return 1,
    };

    // ── Codegen ──────────────────────────────────────────────────────────
    let emit_all = emit.contains(&EmitTarget::All);

    if let Err(e) = fs::create_dir_all(&out_dir) {
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
        write!(stderr, "Error").ok();
        stderr.reset().ok();
        writeln!(stderr, ": could not create output directory: {}", e).ok();
        return 1;
    }

    if emit_all || emit.contains(&EmitTarget::Netlist) {
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
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": {}", e).ok();
            return 1;
        }
    };

    let file_path = &manifest.design.root;
    let src = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
            write!(stderr, "Error").ok();
            stderr.reset().ok();
            writeln!(stderr, ": could not read {}: {}", file_path, e).ok();
            return 1;
        }
    };

    let result = run_pipeline(stderr, file_path, &src, &manifest.design.top);

    if result.has_errors {
        1
    } else {
        writeln!(stderr, "  No errors found.").ok();
        0
    }
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
    };

    process::exit(exit_code);
}

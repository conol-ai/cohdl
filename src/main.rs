//! `cohdl` CLI: `check` and `build`, each with an optional `--json`
//! (RFC-010: structured diagnostics). `fmt` (RFC-009) is a separate command.

use cohdl::emit;
use cohdl::lock::LockState;
use cohdl::pipeline;
use cohdl::project;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
cohdl — the CoHDL v2 compiler

USAGE:
    cohdl check [PATH] [--design NAME] [--std DIR | --no-std] [--json]
    cohdl build [PATH] [--design NAME] [--std DIR | --no-std] [--out-dir DIR]
                [--emit ipc2581] [--json]
    cohdl fmt   [PATH] [--check]
    cohdl lsp

PATH is a project directory (with cohdl.toml + src/) or a single .cohdl file;
defaults to the current directory.

    check    parse, resolve, type-check, and run residual DRC
    build    check + assign designators + bind parts + emit KiCad .net + BOM CSV
    fmt      rewrite every .cohdl file into canonical form (RFC-009)
    lsp      start the Language Server Protocol server on stdio (RFC-014)

    --json   emit one JSON document to stdout instead of human-readable text
             (RFC-010; check/build only)
    --emit   build: emit an additional output format. The only value today is
             `ipc2581` — a partially-specified IPC-2581B1 document
             (<name>.xml, logical-complete/physical-minimal; RFC-015)
    --check  fmt: report drift without rewriting; exit non-zero if any file is
             not already in canonical form
";

struct Args {
    command: String,
    path: PathBuf,
    /// Whether a positional PATH was explicitly given (lsp takes none).
    path_given: bool,
    design: Option<String>,
    std_flag: Option<PathBuf>,
    no_std: bool,
    out_dir: PathBuf,
    out_dir_given: bool,
    json: bool,
    fmt_check: bool,
    /// RFC-015: `--emit <FORMAT>` on `build`. The raw value is kept so
    /// validate() can check command compatibility BEFORE the value — `fmt
    /// --emit bogus` must say "--emit is not valid with fmt", not suggest a
    /// format that would then be rejected anyway.
    emit: Option<String>,
}

impl Args {
    /// The E000-class option matrix: every flag is validated against the
    /// command (review finding: global parsing silently ignored invalid
    /// combinations, e.g. `lsp --design`, `fmt --out-dir`).
    fn validate(&self) -> Result<(), String> {
        let bad = |what: &str| {
            Err(format!(
                "`{}` is not valid with `{}`\n\n{}",
                what, self.command, USAGE
            ))
        };
        if self.std_flag.is_some() && self.no_std {
            return Err(format!(
                "`--std` and `--no-std` are mutually exclusive\n\n{}",
                USAGE
            ));
        }
        match self.command.as_str() {
            "check" => {
                if self.fmt_check {
                    return bad("--check");
                }
                if self.out_dir_given {
                    return bad("--out-dir");
                }
                if self.emit.is_some() {
                    return bad("--emit");
                }
            }
            "build" => {
                if self.fmt_check {
                    return bad("--check");
                }
                // Command compatibility above value validity: only here,
                // where --emit is legal at all, is the value checked.
                if let Some(format) = &self.emit {
                    if format != "ipc2581" {
                        return Err(format!(
                            "unknown `--emit` format `{}` (valid: ipc2581)\n\n{}",
                            format, USAGE
                        ));
                    }
                }
            }
            "fmt" => {
                if self.json {
                    return bad("--json");
                }
                if self.design.is_some() {
                    return bad("--design");
                }
                if self.std_flag.is_some() {
                    return bad("--std");
                }
                if self.no_std {
                    return bad("--no-std");
                }
                if self.out_dir_given {
                    return bad("--out-dir");
                }
                if self.emit.is_some() {
                    return bad("--emit");
                }
            }
            "lsp" => {
                if self.json
                    || self.fmt_check
                    || self.design.is_some()
                    || self.std_flag.is_some()
                    || self.no_std
                    || self.out_dir_given
                    || self.path_given
                    || self.emit.is_some()
                {
                    return Err(format!("`lsp` takes no flags or arguments\n\n{}", USAGE));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// RFC-015: was `--emit ipc2581` requested (call after validate()).
    fn emit_ipc2581(&self) -> bool {
        self.emit.as_deref() == Some("ipc2581")
    }
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().ok_or_else(|| USAGE.to_string())?;
    let mut args = Args {
        command,
        path: PathBuf::from("."),
        path_given: false,
        design: None,
        std_flag: None,
        no_std: false,
        out_dir: PathBuf::from("out"),
        out_dir_given: false,
        json: false,
        fmt_check: false,
        emit: None,
    };
    let mut positional = Vec::new();
    while let Some(a) = argv.next() {
        match a.as_str() {
            "--design" => {
                args.design = Some(argv.next().ok_or("--design needs a value")?);
            }
            "--std" => {
                args.std_flag = Some(PathBuf::from(argv.next().ok_or("--std needs a value")?));
            }
            "--no-std" => args.no_std = true,
            "--json" => args.json = true,
            "--check" => args.fmt_check = true,
            "--out-dir" => {
                args.out_dir = PathBuf::from(argv.next().ok_or("--out-dir needs a value")?);
                args.out_dir_given = true;
            }
            "--emit" => {
                args.emit = Some(argv.next().ok_or("--emit needs a value")?);
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => {
                return Err(format!("unknown flag `{}`\n\n{}", other, USAGE));
            }
            other => positional.push(other.to_string()),
        }
    }
    if positional.len() > 1 {
        return Err(format!("too many arguments\n\n{}", USAGE));
    }
    if let Some(p) = positional.pop() {
        args.path = PathBuf::from(p);
        args.path_given = true;
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(clean) => {
            if clean {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(msg) => {
            eprintln!("error: {}", msg);
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<bool, String> {
    // Per-command flag validation (E000-class invocation errors).
    args.validate()?;
    match args.command.as_str() {
        "check" | "build" => {}
        "fmt" => return fmt_command(args),
        "lsp" => {
            // LSP exit codes: 0 after shutdown+exit; 1 for exit without
            // shutdown (per the spec); 2 for transport/I/O failures.
            return match cohdl::lsp::run_stdio()? {
                cohdl::lsp::LspExit::Clean => Ok(true),
                cohdl::lsp::LspExit::WithoutShutdown => {
                    eprintln!("lsp: `exit` received before `shutdown`");
                    Ok(false)
                }
            };
        }
        other => return Err(format!("unknown command `{}`\n\n{}", other, USAGE)),
    }

    let std_dir = if args.no_std {
        None
    } else {
        let found = project::find_std_dir(args.std_flag.clone());
        if found.is_none() {
            return Err(
                "cannot locate the std library — pass --std <dir>, set COHDL_STD, or use --no-std"
                    .to_string(),
            );
        }
        found
    };

    let proj = project::load_project(&args.path, std_dir.as_deref())?;
    let mut checked =
        pipeline::check_files(&proj.files, args.design.as_deref().or(proj.top.as_deref()))?;

    // A design-selection failure is an invocation-level error (exit 2), but
    // the source diagnostics collected before it are NEVER discarded — they
    // render to stderr first, in both plain and --json modes.
    if let Some(sel_err) = &checked.selection_error {
        eprint!("{}", checked.diags.render(&checked.sm));
        return Err(sel_err.clone());
    }

    if args.command == "check" {
        if args.json {
            print!("{}", emit::json::render(&checked, None));
            return Ok(!checked.diags.has_errors());
        }
        eprint!("{}", checked.diags.render(&checked.sm));
        if checked.diags.has_errors() {
            return Ok(false);
        }
        if checked.design_name.is_none() {
            eprintln!("note: no `design` in this project — declarations checked only");
        }
        eprintln!("  No errors found.");
        return Ok(true);
    }

    // ---- build ----
    if checked.diags.has_errors() {
        if args.json {
            print!("{}", emit::json::render(&checked, None));
        } else {
            eprint!("{}", checked.diags.render(&checked.sm));
        }
        return Ok(false);
    }
    if checked.ir.is_none() {
        // Same rule: never discard collected diagnostics (warnings) on an
        // invocation-level failure.
        eprint!("{}", checked.diags.render(&checked.sm));
        return Err("nothing to build: the project declares no `design`".to_string());
    }

    // R3: an invocation-level failure after the check phase (lock parse,
    // directory creation, artifact writes) must never hide the diagnostics
    // already collected — warnings render to stderr first, then the error.
    // Same rule as the selection-error path above.
    fn diags_then(checked: &pipeline::Checked, e: String) -> String {
        eprint!("{}", checked.diags.render(&checked.sm));
        e
    }

    let lock_path = proj.dir.join("design.lock");
    let prior_lock = match std::fs::read_to_string(&lock_path) {
        Ok(text) => LockState::parse(&text).map_err(|e| diags_then(&checked, e.to_string()))?,
        Err(_) => LockState::default(),
    };

    let artifacts = pipeline::build_artifacts(&mut checked, &prior_lock);
    let Some(artifacts) = artifacts else {
        if args.json {
            print!("{}", emit::json::render(&checked, None));
        } else {
            eprint!("{}", checked.diags.render(&checked.sm));
        }
        return Ok(false);
    };
    if checked.diags.has_errors() {
        if args.json {
            print!("{}", emit::json::render(&checked, None));
        } else {
            eprint!("{}", checked.diags.render(&checked.sm));
            for note in &artifacts.notes {
                eprintln!("note: {}", note);
            }
        }
        return Ok(false);
    }

    let out_dir = proj.dir.join(&args.out_dir);
    std::fs::create_dir_all(&out_dir).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot create `{}`: {}", out_dir.display(), e),
        )
    })?;
    let net_path = out_dir.join(format!("{}.net", proj.name));
    let bom_path = out_dir.join(format!("{}-bom.csv", proj.name));
    std::fs::write(&net_path, &artifacts.netlist).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot write `{}`: {}", net_path.display(), e),
        )
    })?;
    std::fs::write(&bom_path, &artifacts.bom).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot write `{}`: {}", bom_path.display(), e),
        )
    })?;
    std::fs::write(&lock_path, artifacts.lock.render()).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot write `{}`: {}", lock_path.display(), e),
        )
    })?;

    // RFC-013: the layout-constraint artifact, only when there is layout data.
    // A design that no longer carries layout metadata must not leave a stale
    // constraints file behind for a partner tool to consume.
    let layout_path = out_dir.join(format!("{}-layout.json", proj.name));
    match &artifacts.layout {
        Some(layout) => {
            std::fs::write(&layout_path, layout).map_err(|e| {
                diags_then(
                    &checked,
                    format!("cannot write `{}`: {}", layout_path.display(), e),
                )
            })?;
        }
        None => {
            if layout_path.exists() {
                std::fs::remove_file(&layout_path).map_err(|e| {
                    diags_then(
                        &checked,
                        format!("cannot remove stale `{}`: {}", layout_path.display(), e),
                    )
                })?;
                if !args.json {
                    eprintln!("  removed stale {}", layout_path.display());
                }
            }
        }
    }

    // RFC-015: the IPC-2581 handoff artifact, only when `--emit ipc2581` was
    // requested. Same stale-file rule as layout.json: a partner-consumed
    // document that no longer matches the netlist must not linger.
    let ipc_path = out_dir.join(format!("{}.xml", proj.name));
    if args.emit_ipc2581() {
        let ir = checked.ir.as_ref().unwrap();
        let doc = emit::ipc2581::emit_ipc2581(&checked.world, ir, &proj.name);
        std::fs::write(&ipc_path, doc).map_err(|e| {
            diags_then(
                &checked,
                format!("cannot write `{}`: {}", ipc_path.display(), e),
            )
        })?;
    } else if ipc_path.exists() {
        std::fs::remove_file(&ipc_path).map_err(|e| {
            diags_then(
                &checked,
                format!("cannot remove stale `{}`: {}", ipc_path.display(), e),
            )
        })?;
        if !args.json {
            eprintln!("  removed stale {}", ipc_path.display());
        }
    }

    if args.json {
        let build = emit::json::BuildArtifacts {
            netlist: net_path.display().to_string(),
            bom: bom_path.display().to_string(),
            layout: artifacts
                .layout
                .as_ref()
                .map(|_| layout_path.display().to_string()),
            ipc2581: args.emit_ipc2581().then(|| ipc_path.display().to_string()),
        };
        print!("{}", emit::json::render(&checked, Some(&build)));
        return Ok(true);
    }

    // Non-JSON: a successful build still renders its diagnostics (warnings —
    // e.g. D003 — must reach the human exactly as they reach `check` and
    // `--json`; RFC-010 equivalence), then any build notes as prose.
    eprint!("{}", checked.diags.render(&checked.sm));
    for note in &artifacts.notes {
        eprintln!("note: {}", note);
    }
    let ir = checked.ir.as_ref().unwrap();
    eprintln!(
        "  Built design `{}`: {} instances, {} nets",
        checked.design_name.as_deref().unwrap_or("?"),
        ir.instances.len(),
        ir.nets.len()
    );
    eprintln!("  wrote {}", net_path.display());
    eprintln!("  wrote {}", bom_path.display());
    if artifacts.layout.is_some() {
        eprintln!("  wrote {}", layout_path.display());
    }
    if args.emit_ipc2581() {
        eprintln!("  wrote {}", ipc_path.display());
    }
    eprintln!("  wrote {}", lock_path.display());
    Ok(true)
}

/// `cohdl fmt` (RFC-009): rewrite every `.cohdl` file at PATH into canonical
/// form, or with `--check` report drift without touching anything.
fn fmt_command(args: &Args) -> Result<bool, String> {
    let files = collect_cohdl_files(&args.path)?;
    if files.is_empty() {
        return Err(format!(
            "no .cohdl files found at `{}`",
            args.path.display()
        ));
    }
    let mut ok = true;
    for path in files {
        let name = path.display().to_string();
        let original =
            std::fs::read_to_string(&path).map_err(|e| format!("cannot read `{}`: {}", name, e))?;
        match cohdl::fmt::format_source(&name, &original) {
            Err(diags) => {
                // fmt is not a repair tool — non-parsing source is a parse error
                // from the existing pipeline, surfaced verbatim.
                eprint!("{}", diags);
                eprintln!(
                    "error: `{}` does not parse — fmt only formats valid source",
                    name
                );
                ok = false;
            }
            Ok(formatted) if formatted == original => {}
            Ok(_) if args.fmt_check => {
                eprintln!("would reformat {}", name);
                ok = false;
            }
            Ok(formatted) => {
                std::fs::write(&path, formatted)
                    .map_err(|e| format!("cannot write `{}`: {}", name, e))?;
                eprintln!("formatted {}", name);
            }
        }
    }
    if args.fmt_check && ok {
        eprintln!("  All files are in canonical form.");
    }
    Ok(ok)
}

/// Every `.cohdl` file at `path`: the file itself if it is one, else every
/// `.cohdl` under the directory (recursively), sorted for determinism.
fn collect_cohdl_files(path: &std::path::Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(format!("`{}` is not a file or directory", path.display()));
    }
    let mut out = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("cannot read `{}`: {}", dir.display(), e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out)?;
            } else if p.extension().is_some_and(|e| e == "cohdl") {
                out.push(p);
            }
        }
        Ok(())
    }
    walk(path, &mut out)?;
    Ok(out)
}

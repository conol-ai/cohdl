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
    cohdl build [PATH] [--design NAME] [--std DIR | --no-std] [--out-dir DIR] [--json]
    cohdl fmt   [PATH] [--check]

PATH is a project directory (with cohdl.toml + src/) or a single .cohdl file;
defaults to the current directory.

    check    parse, resolve, type-check, and run residual DRC
    build    check + assign designators + bind parts + emit KiCad .net + BOM CSV
    fmt      rewrite every .cohdl file into canonical form (RFC-009)

    --json   emit one JSON document to stdout instead of human-readable text
             (RFC-010; check/build only)
    --check  fmt: report drift without rewriting; exit non-zero if any file is
             not already in canonical form
";

struct Args {
    command: String,
    path: PathBuf,
    design: Option<String>,
    std_flag: Option<PathBuf>,
    no_std: bool,
    out_dir: PathBuf,
    json: bool,
    fmt_check: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().ok_or_else(|| USAGE.to_string())?;
    let mut args = Args {
        command,
        path: PathBuf::from("."),
        design: None,
        std_flag: None,
        no_std: false,
        out_dir: PathBuf::from("out"),
        json: false,
        fmt_check: false,
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
    match args.command.as_str() {
        "check" | "build" => {}
        "fmt" => return fmt_command(args),
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
        return Err("nothing to build: the project declares no `design`".to_string());
    }

    let lock_path = proj.dir.join("design.lock");
    let prior_lock = match std::fs::read_to_string(&lock_path) {
        Ok(text) => LockState::parse(&text).map_err(|e| e.to_string())?,
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
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("cannot create `{}`: {}", out_dir.display(), e))?;
    let net_path = out_dir.join(format!("{}.net", proj.name));
    let bom_path = out_dir.join(format!("{}-bom.csv", proj.name));
    std::fs::write(&net_path, &artifacts.netlist)
        .map_err(|e| format!("cannot write `{}`: {}", net_path.display(), e))?;
    std::fs::write(&bom_path, &artifacts.bom)
        .map_err(|e| format!("cannot write `{}`: {}", bom_path.display(), e))?;
    std::fs::write(&lock_path, artifacts.lock.render())
        .map_err(|e| format!("cannot write `{}`: {}", lock_path.display(), e))?;

    if args.json {
        let build = emit::json::BuildArtifacts {
            netlist: net_path.display().to_string(),
            bom: bom_path.display().to_string(),
        };
        print!("{}", emit::json::render(&checked, Some(&build)));
        return Ok(true);
    }

    // Non-JSON: surface any build notes (e.g. part-binding ambiguity) as prose.
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

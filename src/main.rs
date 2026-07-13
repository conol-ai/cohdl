//! `cohdl` CLI: `check` and `build` (the MVP surface — no fmt, no LSP,
//! no --json; see docs/design/09-mvp-definition.md's cut list).

use cohdl::lock::LockState;
use cohdl::pipeline;
use cohdl::project;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
cohdl — the CoHDL v2 compiler

USAGE:
    cohdl check [PATH] [--design NAME] [--std DIR | --no-std]
    cohdl build [PATH] [--design NAME] [--std DIR | --no-std] [--out-dir DIR]

PATH is a project directory (with cohdl.toml + src/) or a single .cohdl file;
defaults to the current directory.

    check    parse, resolve, type-check, and run residual DRC
    build    check + assign designators + bind parts + emit KiCad .net + BOM CSV
";

struct Args {
    command: String,
    path: PathBuf,
    design: Option<String>,
    std_flag: Option<PathBuf>,
    no_std: bool,
    out_dir: PathBuf,
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
    if args.command != "check" && args.command != "build" {
        return Err(format!("unknown command `{}`\n\n{}", args.command, USAGE));
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
        eprint!("{}", checked.diags.render(&checked.sm));
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
    eprint!("{}", checked.diags.render(&checked.sm));
    let Some(artifacts) = artifacts else {
        return Ok(false);
    };
    if checked.diags.has_errors() {
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

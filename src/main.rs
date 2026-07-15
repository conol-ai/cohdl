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
                if args.emit.is_some() {
                    return Err(format!("`--emit` given more than once\n\n{}", USAGE));
                }
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

/// Refuse to descend into an output directory reachable through a symlink
/// (review R7-1): a build must be contained under the project root, so no
/// symlinked ancestor between `root` and `dir` (inclusive) may be followed —
/// otherwise a planted `out -> ../victim` lets a successful build write
/// outside the project entirely. Checked before any `create_dir_all`.
fn ensure_contained(root: &std::path::Path, dir: &std::path::Path) -> Result<(), String> {
    // Walk the ancestors of `dir` that lie strictly below `root`, nearest
    // first, and refuse any that exists as a symlink.
    let mut chain: Vec<&std::path::Path> = Vec::new();
    let mut cur = dir;
    loop {
        if cur == root {
            break;
        }
        chain.push(cur);
        match cur.parent() {
            Some(p) => cur = p,
            None => break, // dir is not under root; the caller joins under it
        }
    }
    for c in chain {
        if let Ok(md) = std::fs::symlink_metadata(c) {
            if md.file_type().is_symlink() {
                return Err(format!(
                    "refusing to build into `{}`: `{}` is a symlink (a build must stay within the project directory)",
                    dir.display(),
                    c.display()
                ));
            }
        }
    }
    Ok(())
}

/// Write a generated artifact safely (review R6-1/R7-1). Containment: never
/// follow a symlink at the destination (an existing symlink — live or
/// dangling — is refused, and the write uses `create_new`/O_EXCL so a
/// symlink racing into the path cannot be followed). Ownership: refuse to
/// overwrite an existing regular file CoHDL did not write last time —
/// `owned` is the set of paths from the prior build manifest, so every
/// artifact kind (not just marker-bearing ones) is protected uniformly.
fn write_artifact(
    path: &std::path::Path,
    content: &str,
    owned: &std::collections::BTreeSet<std::path::PathBuf>,
) -> Result<(), String> {
    use std::io::Write as _;
    match std::fs::symlink_metadata(path) {
        Ok(md) if md.file_type().is_symlink() => {
            return Err(format!(
                "refusing to write `{}`: it is a symlink (a build must not follow a symlink out of the output directory)",
                path.display()
            ));
        }
        Ok(md) if md.is_dir() => {
            return Err(format!(
                "refusing to write `{}`: a directory exists at that path",
                path.display()
            ));
        }
        Ok(_) => {
            if !owned.contains(path) {
                return Err(format!(
                    "refusing to overwrite `{}`: it was not written by cohdl (not in the build manifest)",
                    path.display()
                ));
            }
            std::fs::remove_file(path)
                .map_err(|e| format!("cannot replace `{}`: {}", path.display(), e))?;
        }
        Err(_) => {} // does not exist
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot write `{}`: {}", path.display(), e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| format!("cannot write `{}`: {}", path.display(), e))?;
    Ok(())
}

/// Remove a file CoHDL wrote last time (in the prior manifest), safely:
/// `symlink_metadata` so a symlink is unlinked (never followed to its
/// target) and a directory is left alone (review R7-1).
fn remove_owned(path: &std::path::Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(md) if md.is_dir() => Ok(()), // never rmdir here
        Ok(_) => std::fs::remove_file(path)
            .map_err(|e| format!("cannot remove stale `{}`: {}", path.display(), e)),
        Err(_) => Ok(()), // already gone
    }
}

/// The prior build's manifest: the set of paths CoHDL wrote last time,
/// one per line, relative to the manifest's directory. Absent/unreadable →
/// empty (a first build owns nothing, so any pre-existing file is foreign).
fn read_manifest(
    manifest_path: &std::path::Path,
    base: &std::path::Path,
) -> std::collections::BTreeSet<std::path::PathBuf> {
    let mut owned = std::collections::BTreeSet::new();
    if let Ok(text) = std::fs::read_to_string(manifest_path) {
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                owned.insert(base.join(line));
            }
        }
    }
    owned
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
    let mut checked = pipeline::check_files_in(
        &proj.name,
        &proj.files,
        args.design.as_deref().or(proj.top.as_deref()),
    )?;

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

    // Defence in depth (review F4): the artifact basename is `proj.name`,
    // which for a manifest project is already an identifier, but a
    // single-file target derives it from the filename stem. Refuse anything
    // that is not a safe basename so no artifact write — or the stale-file
    // cleanup that deletes — can ever escape the output directory.
    if proj.name.is_empty()
        || proj.name.contains(['/', '\\'])
        || proj.name == "."
        || proj.name == ".."
    {
        return Err(diags_then(
            &checked,
            format!(
                "`{}` is not a safe output basename (path separators or `.`/`..`); rename the file or set `[package] name`",
                proj.name
            ),
        ));
    }

    let out_dir = proj.dir.join(&args.out_dir);
    let mods_dir = out_dir.join("footprints");
    // Containment (R7-1): refuse to build into an output dir reachable through
    // a symlinked ancestor — a planted `out -> ../victim` must not let the
    // build write outside the project.
    ensure_contained(&proj.dir, &out_dir).map_err(|e| diags_then(&checked, e))?;
    std::fs::create_dir_all(&out_dir).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot create `{}`: {}", out_dir.display(), e),
        )
    })?;

    // Ownership manifest (R7-1): the set of files CoHDL wrote LAST build,
    // stored project-relative so every artifact kind — netlist, BOM, lock,
    // layout, `.kicad_mod`, IPC — shares one owner set. `write_artifact`
    // refuses to overwrite an existing file NOT in this set (foreign), and
    // stale removal only touches files that ARE in it.
    let manifest_path = out_dir.join(".cohdl-manifest");
    let mut owned = read_manifest(&manifest_path, &proj.dir);
    // `design.lock` is always CoHDL's: the build already read and format-
    // validated it as `prior_lock` (an unparseable lock errored earlier), and
    // it is committed for designator stability — so it is owned regardless of
    // the manifest (which a fresh checkout won't carry).
    owned.insert(lock_path.clone());
    let mut written: Vec<std::path::PathBuf> = Vec::new();

    let net_path = out_dir.join(format!("{}.net", proj.name));
    let bom_path = out_dir.join(format!("{}-bom.csv", proj.name));
    let layout_path = out_dir.join(format!("{}-layout.json", proj.name));
    let ipc_path = out_dir.join(format!("{}.xml", proj.name));

    write_artifact(&net_path, &artifacts.netlist, &owned).map_err(|e| diags_then(&checked, e))?;
    written.push(net_path.clone());
    write_artifact(&bom_path, &artifacts.bom, &owned).map_err(|e| diags_then(&checked, e))?;
    written.push(bom_path.clone());
    write_artifact(&lock_path, &artifacts.lock.render(), &owned)
        .map_err(|e| diags_then(&checked, e))?;
    written.push(lock_path.clone());

    // RFC-013: the layout-constraint artifact, only when there is layout data.
    // A design that no longer carries layout metadata leaves it out of the new
    // manifest, so the stale-file sweep below removes it.
    if let Some(layout) = &artifacts.layout {
        write_artifact(&layout_path, layout, &owned).map_err(|e| diags_then(&checked, e))?;
        written.push(layout_path.clone());
    }

    // RFC-018: `.kicad_mod` projections for pad-bearing footprints.
    let mods = {
        let ir = checked.ir.as_ref().unwrap();
        emit::kicad_mod::emit_kicad_mods(&checked.world, ir)
    };
    let mut mod_paths: Vec<String> = Vec::new();
    if !mods.is_empty() {
        ensure_contained(&proj.dir, &mods_dir).map_err(|e| diags_then(&checked, e))?;
        std::fs::create_dir_all(&mods_dir).map_err(|e| {
            diags_then(
                &checked,
                format!("cannot create `{}`: {}", mods_dir.display(), e),
            )
        })?;
        for (_fq, base, content) in &mods {
            let p = mods_dir.join(format!("{}.kicad_mod", base));
            write_artifact(&p, content, &owned).map_err(|e| diags_then(&checked, e))?;
            written.push(p.clone());
            mod_paths.push(p.display().to_string());
        }
    }

    // RFC-015: the IPC-2581 handoff artifact, only when `--emit ipc2581`.
    if args.emit_ipc2581() {
        let ir = checked.ir.as_ref().unwrap();
        let doc = emit::ipc2581::emit_ipc2581(&checked.world, ir, &proj.name);
        write_artifact(&ipc_path, &doc, &owned).map_err(|e| diags_then(&checked, e))?;
        written.push(ipc_path.clone());
    }

    // Stale sweep (R7-1): every prior-owned file we did NOT rewrite this build
    // is removed, safely (a symlink is unlinked, never followed to its target).
    let written_set: std::collections::BTreeSet<&std::path::PathBuf> = written.iter().collect();
    for old in &owned {
        if !written_set.contains(old) {
            remove_owned(old).map_err(|e| diags_then(&checked, e))?;
            if !args.json {
                eprintln!("  removed stale {}", old.display());
            }
        }
    }

    // Persist the new manifest (project-relative, sorted — byte-stable).
    let mut rels: Vec<String> = written
        .iter()
        .filter_map(|p| p.strip_prefix(&proj.dir).ok())
        .map(|rel| rel.display().to_string())
        .collect();
    rels.sort();
    let manifest_body = format!("{}\n", rels.join("\n"));
    // The manifest is CoHDL's own metadata: symlink-safe overwrite.
    if std::fs::symlink_metadata(&manifest_path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(diags_then(
            &checked,
            format!(
                "refusing to write `{}`: it is a symlink",
                manifest_path.display()
            ),
        ));
    }
    std::fs::write(&manifest_path, &manifest_body).map_err(|e| {
        diags_then(
            &checked,
            format!("cannot write `{}`: {}", manifest_path.display(), e),
        )
    })?;

    if args.json {
        let build = emit::json::BuildArtifacts {
            netlist: net_path.display().to_string(),
            bom: bom_path.display().to_string(),
            layout: artifacts
                .layout
                .as_ref()
                .map(|_| layout_path.display().to_string()),
            ipc2581: args.emit_ipc2581().then(|| ipc_path.display().to_string()),
            kicad_mod: mod_paths.clone(),
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
    for p in &mod_paths {
        eprintln!("  wrote {}", p);
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

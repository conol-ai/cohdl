//! `cohdl` CLI: `check` and `build`, each with an optional `--json`
//! (RFC-010: structured diagnostics). `fmt` (RFC-009) is a separate command.

use cohdl::emit;
use cohdl::lock::LockState;
use cohdl::pipeline;
use cohdl::project;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
cohdl — the CoHDL v2 compiler

USAGE:
    cohdl check  [PATH] [--design NAME] [--std DIR | --no-std] [--json]
    cohdl build  [PATH] [--design NAME] [--std DIR | --no-std] [--out-dir DIR]
                 [--emit ipc2581] [--json]
    cohdl update [NAME] [PATH] [--dep NAME]
    cohdl add    NAME[@X.Y.Z] [PATH]
    cohdl remove NAME [PATH]
    cohdl install [PATH]
    cohdl login
    cohdl publish [PATH]
    cohdl fmt    [PATH] [--check]
    cohdl lsp

PATH is a project directory (with cohdl.toml + src/) or a single .cohdl file;
defaults to the current directory.

    check    parse, resolve, type-check, and run residual DRC
    build    check + assign designators + bind parts + emit KiCad .net + BOM CSV
    update   re-resolve dependencies to the latest published exact version
             (registry first, local fallback) and rewrite [dependencies] +
             cohdl.lock (RFC-029/030); migrates a pre-RFC-029 manifest
    add      add a dependency: resolve latest (or the given @X.Y.Z), fetch
             into the cache, write [dependencies] + cohdl.lock (RFC-030)
    remove   remove a dependency from [dependencies] and cohdl.lock (RFC-030)
    install  resolve every dependency per cohdl.lock, fetching anything
             missing from registry.cohdl.org into ~/.cohdl/registry (RFC-030)
    login    store a registry token (opens the account page for you to copy
             one; paste it at the prompt)
    publish  package the current project and publish it to the registry
             (three-tier namespace pre-flight, then POST; RFC-030)
    fmt      rewrite every .cohdl file into canonical form (RFC-009); in a
             project directory, also canonicalizes cohdl.toml's [dependencies]
    lsp      start the Language Server Protocol server on stdio (RFC-014)

    --json   emit one JSON document to stdout instead of human-readable text
             (RFC-010; check/build only)
    --emit   build: emit an additional output format. The only value today is
             `ipc2581` — a partially-specified IPC-2581B1 document
             (<name>.xml, logical-complete/physical-minimal; RFC-015)
    --check  fmt: report drift without rewriting; exit non-zero if any file is
             not already in canonical form
    --dep    update: re-resolve only the named dependency
    --std    development override: use DIR verbatim as the std package —
             emits the mandatory E1105 warning; the result is not reproducible
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
    /// RFC-029: `update --dep NAME` — re-resolve one dependency only.
    dep: Option<String>,
    /// RFC-030: the NAME positional of add/remove/update (`name` or
    /// `name@X.Y.Z` for add).
    name: Option<String>,
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
        if self.dep.is_some() && self.command != "update" {
            return bad("--dep");
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
            "add" | "remove" | "install" | "login" | "publish" | "update" => {
                if self.json {
                    return bad("--json");
                }
                if self.fmt_check {
                    return bad("--check");
                }
                if self.design.is_some() {
                    return bad("--design");
                }
                // `update` writes lock rows from resolved registry content —
                // combining it with an override or std-less mode would record
                // hashes of content no locked build will ever see.
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
        dep: None,
        name: None,
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
            "--dep" => {
                args.dep = Some(argv.next().ok_or("--dep needs a value")?);
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => {
                return Err(format!("unknown flag `{}`\n\n{}", other, USAGE));
            }
            other => positional.push(other.to_string()),
        }
    }
    // RFC-030: add/remove take NAME [PATH]; update takes [NAME] [PATH]
    // (a NAME is recognized by the registry-name grammar; when a name
    // collides with a directory name, use --dep / a ./path spelling).
    let takes_name = matches!(args.command.as_str(), "add" | "remove" | "update");
    let max = if takes_name { 2 } else { 1 };
    if positional.len() > max {
        return Err(format!("too many arguments\n\n{}", USAGE));
    }
    for p in positional {
        let base = p.split('@').next().unwrap_or(&p);
        let path_like = p.contains(std::path::MAIN_SEPARATOR)
            || p.starts_with('.')
            || cohdl::registry::name_tier(base).is_err()
            || (args.command == "update" && Path::new(&p).is_dir());
        let plain_name = !path_like;
        let is_name = takes_name && args.name.is_none() && (p.starts_with('@') || plain_name);
        if is_name {
            args.name = Some(p);
        } else if !args.path_given {
            args.path = PathBuf::from(p);
            args.path_given = true;
        } else {
            return Err(format!("too many arguments\n\n{}", USAGE));
        }
    }
    if matches!(args.command.as_str(), "add" | "remove") && args.name.is_none() {
        return Err(format!(
            "`{}` needs a package name\n\n{}",
            args.command, USAGE
        ));
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
        "update" => return update_command(args),
        "add" => return add_command(args),
        "remove" => return remove_command(args),
        "install" => return install_command(args),
        "login" => return login_command(),
        "publish" => return publish_command(args),
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

    // RFC-029: dependency resolution precedes everything — no `.cohdl` file
    // is opened against an unverified dependency set. Manifest projects go
    // through `[dependencies]` + cohdl.lock; single-file targets resolve the
    // newest available std (nothing to pin against).
    let (proj, dep_names) = if args.path.is_dir() && args.path.join("cohdl.toml").is_file() {
        let deps_list = match resolve_manifest_deps(args) {
            Ok(deps) => deps,
            Err(DepFailure::Prose(e)) => return Err(e),
            Err(DepFailure::Diags(diags)) => {
                if args.json {
                    print!("{}", emit::json::render_package_failure(&diags));
                } else {
                    eprint!("{}", cohdl::deps::render_human(&diags));
                }
                return Ok(false);
            }
        };
        let names: Vec<String> = deps_list.iter().map(|(n, _)| n.clone()).collect();
        (
            project::load_project_with_deps(&args.path, &deps_list)?,
            names,
        )
    } else {
        let std_dir = if args.no_std {
            None
        } else {
            warn_if_std_override(args);
            let found = project::find_std_dir(args.std_flag.clone());
            if found.is_none() {
                return Err(
                    "cannot locate the std library — pass --std <dir>, set COHDL_STD, or use --no-std"
                        .to_string(),
                );
            }
            found
        };
        let names = if std_dir.is_some() {
            vec!["std".to_string()]
        } else {
            Vec::new()
        };
        (
            project::load_project(&args.path, std_dir.as_deref())?,
            names,
        )
    };
    let mut checked = pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
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

    // RFC-020: resolve a `board_outline: "…dxf"` reference into real geometry
    // before emitting. The path is project-relative (validated at check — no
    // absolute/`..`/URL), so it joins under the project directory.
    let proj_dir = proj.dir.clone();
    pipeline::resolve_board_outline(&mut checked, |path| {
        std::fs::read_to_string(proj_dir.join(path)).map_err(|e| e.to_string())
    });

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

    // RFC-027: the Quilter physics-constraint CSV set, only when the design
    // carries physics facts. Absent facts leave the files out of the new
    // manifest, so the stale-file sweep removes them.
    let mut quilter_paths: Vec<std::path::PathBuf> = Vec::new();
    if let Some(csvs) = &artifacts.quilter {
        for (name, content) in csvs {
            let path = out_dir.join(name);
            write_artifact(&path, content, &owned).map_err(|e| diags_then(&checked, e))?;
            written.push(path.clone());
            quilter_paths.push(path);
        }
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
            quilter: artifacts.quilter.as_ref().map(|_| {
                quilter_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect()
            }),
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

/// How RFC-029 dependency resolution fails: structured E11xx diagnostics
/// (rendered human or `--json`, exit 1) or an invocation-level prose error
/// (exit 2).
enum DepFailure {
    Diags(Vec<cohdl::deps::PackageDiag>),
    Prose(String),
}

/// The mandatory, unsuppressable E1105 warning when a `--std`/`COHDL_STD`
/// override is active: the build is not using a locked std and must not be
/// treated as reproducible. stderr in every mode, by design.
fn warn_if_std_override(args: &Args) {
    if let Some(dir) = project::std_override(args.std_flag.clone()) {
        let warn = cohdl::deps::PackageDiag::warning(
            "E1105",
            &dir.display().to_string(),
            0,
            "std override active — this build does not use a locked std and is not reproducible"
                .to_string(),
        );
        eprint!("{}", cohdl::deps::render_human(&[warn]));
    }
}

/// RFC-029: resolve the manifest's `[dependencies]` against the on-disk
/// registry and cohdl.lock. Returns the (name, dir) set for
/// `load_project_with_deps` — std first, then the rest in name order — and
/// writes first-resolution lock rows.
fn resolve_manifest_deps(args: &Args) -> Result<Vec<(String, PathBuf)>, DepFailure> {
    use cohdl::deps;

    let (manifest_path, manifest) =
        project::peek_manifest(&args.path).map_err(DepFailure::Prose)?;
    let manifest_display = manifest_path.display().to_string();
    let override_std = project::std_override(args.std_flag.clone());

    // Pre-RFC-029 manifest: no `[dependencies]` at all (tolerated only for
    // --no-std builds, which have opted out of std entirely).
    let Some(deps_raw) = &manifest.deps_raw else {
        if args.no_std {
            return Ok(Vec::new());
        }
        if let Some(dir) = override_std {
            // A dev override needs no pin — it bypasses the registry.
            warn_if_std_override(args);
            return Ok(vec![("std".to_string(), dir)]);
        }
        let newest = project::find_std_root()
            .and_then(|root| deps::newest_available(&root, "std"))
            .map(|(v, _)| v.to_string())
            .unwrap_or_else(|| "X.Y.Z".to_string());
        return Err(DepFailure::Diags(vec![cohdl::deps::PackageDiag::error(
            "E1104",
            &manifest_display,
            0,
            "this project declares no `[dependencies]` — RFC-029 requires an exact std pin"
                .to_string(),
        )
        .with_help(format!(
            "add:\n           [dependencies]\n           std = \"{newest}\""
        ))
        .with_help(
            "or run `cohdl update` to write it automatically".to_string(),
        )]));
    };

    let mut entries =
        deps::validate_deps(&manifest_display, deps_raw).map_err(DepFailure::Diags)?;
    // Implicit-std rule: a manifest project that has not opted out (--no-std)
    // must pin std like any other dependency.
    if args.no_std {
        entries.retain(|e| e.name != "std");
    } else if !entries.iter().any(|e| e.name == "std") && override_std.is_none() {
        let newest = project::find_std_root()
            .and_then(|root| deps::newest_available(&root, "std"))
            .map(|(v, _)| v.to_string())
            .unwrap_or_else(|| "X.Y.Z".to_string());
        return Err(DepFailure::Diags(vec![cohdl::deps::PackageDiag::error(
            "E1104",
            &manifest_display,
            0,
            "`[dependencies]` has no `std` entry — every project implicitly depends on std and must pin its exact version"
                .to_string(),
        )
        .with_help(format!(
            "add `std = \"{newest}\"` under [dependencies], or build with --no-std"
        ))]));
    }

    // Dev override: std comes verbatim from the override dir; it is neither
    // verified against nor recorded in cohdl.lock.
    let mut resolved_deps: Vec<(String, PathBuf)> = Vec::new();
    if let Some(dir) = &override_std {
        warn_if_std_override(args);
        entries.retain(|e| e.name != "std");
        resolved_deps.push(("std".to_string(), dir.clone()));
    }

    let registry = deps::Registry {
        std_root: project::find_std_root(),
        project_deps: args.path.join("deps"),
        cache_root: cohdl::registry::cache_root(),
    };
    let lock_path = args.path.join("cohdl.lock");
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
    .map_err(DepFailure::Diags)?;

    // First-resolution rows (and manifest-version changes) are recorded now;
    // an unchanged lock is left byte-identical. Never through a symlink.
    if resolution.lock_changed {
        write_lock_file(&lock_path, &resolution.lock.render()).map_err(DepFailure::Prose)?;
    }

    // std first (the pipeline's established file order), then name order.
    let mut rest = resolution.deps;
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    resolved_deps.extend(rest);
    if let Some(pos) = resolved_deps.iter().position(|(n, _)| n == "std") {
        let std_entry = resolved_deps.remove(pos);
        resolved_deps.insert(0, std_entry);
    }
    Ok(resolved_deps)
}

/// cohdl.lock writes: symlink-refusing plain overwrite (the lock is CoHDL's
/// own metadata, same rule as the build manifest).
fn write_lock_file(path: &std::path::Path, content: &str) -> Result<(), String> {
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

/// `cohdl update [PATH] [--dep NAME]` (RFC-029): the only sanctioned way a
/// locked hash changes. Re-resolves every pinned dependency (or one, with
/// --dep) and rewrites cohdl.lock. On a pre-RFC-029 manifest, performs the
/// migration: appends `[dependencies]` pinning the newest available std.
fn update_command(args: &Args) -> Result<bool, String> {
    use cohdl::deps;

    if !args.path.is_dir() || !args.path.join("cohdl.toml").is_file() {
        return Err(format!(
            "`{}` is not a project directory (update needs a cohdl.toml manifest)",
            args.path.display()
        ));
    }
    let (manifest_path, mut manifest) = project::peek_manifest(&args.path)?;
    let manifest_display = manifest_path.display().to_string();

    // Migration: no [dependencies] section → write one pinning newest std.
    if manifest.deps_raw.is_none() {
        let Some((newest, _)) =
            project::find_std_root().and_then(|r| deps::newest_available(&r, "std"))
        else {
            return Err(
                "cannot locate a versioned std library to pin (no std root found)".to_string(),
            );
        };
        let text = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("cannot read `{}`: {}", manifest_display, e))?;
        let mut new_text = text.clone();
        if !new_text.ends_with('\n') {
            new_text.push('\n');
        }
        new_text.push_str(&format!("\n[dependencies]\nstd = \"{}\"\n", newest));
        std::fs::write(&manifest_path, &new_text)
            .map_err(|e| format!("cannot write `{}`: {}", manifest_display, e))?;
        eprintln!(
            "  migrated {}: added [dependencies] std = \"{}\"",
            manifest_display, newest
        );
        let (_, m) = project::peek_manifest(&args.path)?;
        manifest = m;
    }

    let deps_raw = manifest.deps_raw.as_ref().expect("migrated above");
    let mut entries = match deps::validate_deps(&manifest_display, deps_raw) {
        Ok(e) => e,
        Err(diags) => {
            eprint!("{}", deps::render_human(&diags));
            return Ok(false);
        }
    };
    // RFC-030: `cohdl update NAME` — positional or --dep.
    let target = args.dep.clone().or_else(|| {
        args.name
            .as_ref()
            .map(|n| cohdl::registry::split_name_version(n).0)
    });
    if let Some(name) = &target {
        if !entries.iter().any(|e| &e.name == name) {
            return Err(format!(
                "`{}` is not a dependency of this project (see [dependencies] in {})",
                name, manifest_display
            ));
        }
    }

    let registry = deps::Registry {
        std_root: project::find_std_root(),
        project_deps: args.path.join("deps"),
        cache_root: cohdl::registry::cache_root(),
    };

    // RFC-030: update means "re-resolve to the latest published exact
    // version" — the registry first, local families as the fallback (std
    // and vendored packages live only on disk). The manifest is rewritten
    // only on a real bump; content missing locally is fetched.
    for e in entries.iter_mut() {
        if let Some(name) = &target {
            if &e.name != name {
                continue;
            }
        }
        let registry_latest = cohdl::registry::published_versions(&e.name)
            .ok()
            .and_then(|v| v.first().copied());
        let local_latest = registry
            .families(&e.name)
            .iter()
            .filter_map(|f| deps::newest_available(f, &e.name))
            .map(|(v, _)| v)
            .max();
        let latest = registry_latest.into_iter().chain(local_latest).max();
        if let Some(latest) = latest {
            if latest > e.version {
                manifest_set_dep(&manifest_path, &e.name, &latest.to_string())?;
                eprintln!("  {}: {} -> {}", e.name, e.version, latest);
                e.version = latest;
            }
        }
        let on_disk = registry.families(&e.name).iter().any(|f| {
            deps::available_versions(f, &e.name)
                .map(|v| v.iter().any(|(ver, _)| *ver == e.version))
                .unwrap_or(false)
        });
        if !on_disk {
            match cohdl::registry::download_into_cache(&e.name, e.version) {
                Ok(_) => {}
                Err(d) => {
                    eprint!("{}", render_diag(&d));
                    return Ok(false);
                }
            }
        }
    }

    let lock_path = args.path.join("cohdl.lock");
    let lock_display = lock_path.display().to_string();
    let prior_lock_text = std::fs::read_to_string(&lock_path).ok();
    let update = match &target {
        Some(name) => deps::Update::One(name.clone()),
        None => deps::Update::All,
    };

    match deps::resolve(
        &manifest_display,
        &lock_display,
        &entries,
        &registry,
        prior_lock_text.as_deref(),
        update,
    ) {
        Ok(resolution) => {
            if resolution.lock_changed || prior_lock_text.is_none() {
                write_lock_file(&lock_path, &resolution.lock.render())?;
            }
            for (name, entry) in &resolution.lock.entries {
                eprintln!("  locked {} {} ({})", name, entry.version, entry.hash);
            }
            eprintln!("  wrote {}", lock_display);
            Ok(true)
        }
        Err(diags) => {
            eprint!("{}", deps::render_human(&diags));
            Ok(false)
        }
    }
}

/// Surgical `[dependencies]` edit: insert or replace `name = "version"`
/// (quoted key for scoped names), creating the section if missing.
fn manifest_set_dep(manifest_path: &Path, name: &str, version: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
    let key = if name.starts_with('@') {
        format!("\"{name}\"")
    } else {
        name.to_string()
    };
    let entry = format!("{key} = \"{version}\"");
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut replaced = false;
    if let Some(start) = lines.iter().position(|l| l.trim() == "[dependencies]") {
        let end = lines[start + 1..]
            .iter()
            .position(|l| l.trim().starts_with('['))
            .map(|off| start + 1 + off)
            .unwrap_or(lines.len());
        for l in lines[start + 1..end].iter_mut() {
            let k = l
                .trim()
                .split('=')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            if k == name {
                *l = entry.clone();
                replaced = true;
                break;
            }
        }
        if !replaced {
            lines.insert(end, entry);
        }
    } else {
        if !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push("[dependencies]".to_string());
        lines.push(entry);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    std::fs::write(manifest_path, out).map_err(|e| e.to_string())
}

/// Remove `name` from `[dependencies]`; Ok(false) when it was not present.
fn manifest_remove_dep(manifest_path: &Path, name: &str) -> Result<bool, String> {
    let text = std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
    let mut removed = false;
    let mut in_deps = false;
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| {
            let t = l.trim();
            if t.starts_with('[') {
                in_deps = t == "[dependencies]";
                return true;
            }
            if in_deps {
                let k = t.split('=').next().unwrap_or("").trim().trim_matches('"');
                if k == name {
                    removed = true;
                    return false;
                }
            }
            true
        })
        .collect();
    if removed {
        let mut out = lines.join("\n");
        out.push('\n');
        std::fs::write(manifest_path, out).map_err(|e| e.to_string())?;
    }
    Ok(removed)
}

/// Upsert one cohdl.lock row (the registry's server-computed hash is what a
/// fresh install verifies against — RFC-030).
fn lock_upsert(
    project: &Path,
    name: &str,
    version: cohdl::deps::Version,
    hash: String,
) -> Result<(), String> {
    let lock_path = project.join("cohdl.lock");
    let mut lock = match std::fs::read_to_string(&lock_path) {
        Ok(text) => cohdl::deps::LockFile::parse(&text)?,
        Err(_) => cohdl::deps::LockFile::default(),
    };
    lock.entries
        .insert(name.to_string(), cohdl::deps::LockEntry { version, hash });
    write_lock_file(&lock_path, &lock.render())
}

fn render_diag(d: &cohdl::deps::PackageDiag) -> String {
    cohdl::deps::render_human(std::slice::from_ref(d))
}

/// `cohdl add NAME[@X.Y.Z]` (RFC-030): resolve, fetch into the cache, write
/// `[dependencies]` + the cohdl.lock row in one step.
fn add_command(args: &Args) -> Result<bool, String> {
    use cohdl::registry;
    let arg = args.name.as_deref().expect("validated");
    let (name, ver) = registry::split_name_version(arg);
    let tier = match registry::name_tier(&name) {
        Ok(t) => t,
        Err(e) => {
            eprint!(
                "{}",
                render_diag(&cohdl::deps::PackageDiag::error("E1202", "cohdl add", 0, e))
            );
            return Ok(false);
        }
    };
    if !args.path.join("cohdl.toml").is_file() {
        return Err(format!(
            "`{}` is not a project directory (add needs a cohdl.toml manifest)",
            args.path.display()
        ));
    }
    let version = match &ver {
        Some(v) => cohdl::deps::parse_exact_version(v).map_err(|e| e.to_string())?,
        None => match registry::published_versions(&name) {
            Ok(versions) => *versions
                .first()
                .ok_or_else(|| format!("`{name}` has no published versions on the registry"))?,
            Err(d) => {
                eprint!("{}", render_diag(&d));
                return Ok(false);
            }
        },
    };
    let (_, server_hash) = match registry::download_into_cache(&name, version) {
        Ok(r) => r,
        Err(d) => {
            eprint!("{}", render_diag(&d));
            return Ok(false);
        }
    };
    manifest_set_dep(&args.path.join("cohdl.toml"), &name, &version.to_string())?;
    lock_upsert(&args.path, &name, version, server_hash)?;
    eprintln!("  added {} {} — {}", name, version, tier.describe());
    Ok(true)
}

/// `cohdl remove NAME` (RFC-030): the symmetric inverse of add.
fn remove_command(args: &Args) -> Result<bool, String> {
    let name = args.name.as_deref().expect("validated");
    let (manifest_path, manifest) = project::peek_manifest(&args.path)?;
    let current: Vec<String> = manifest
        .deps_raw
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|(n, _, _)| n.clone())
        .collect();
    if !current.iter().any(|n| n == name) {
        eprint!(
            "{}",
            render_diag(
                &cohdl::deps::PackageDiag::error(
                    "E1205",
                    &manifest_path.display().to_string(),
                    0,
                    format!("`{name}` is not a dependency of this project"),
                )
                .with_help(if current.is_empty() {
                    "the project has no [dependencies]".to_string()
                } else {
                    format!("current dependencies: {}", current.join(", "))
                })
            )
        );
        return Ok(false);
    }
    manifest_remove_dep(&manifest_path, name)?;
    let lock_path = args.path.join("cohdl.lock");
    if let Ok(text) = std::fs::read_to_string(&lock_path) {
        if let Ok(mut lock) = cohdl::deps::LockFile::parse(&text) {
            if lock.entries.remove(name).is_some() {
                write_lock_file(&lock_path, &lock.render())?;
            }
        }
    }
    eprintln!("  removed {}", name);
    Ok(true)
}

/// `cohdl install` (RFC-030): RFC-029's resolution with the registry as the
/// content source — anything not on disk is fetched into the cache first.
fn install_command(args: &Args) -> Result<bool, String> {
    use cohdl::{deps, registry};
    let (manifest_path, manifest) = project::peek_manifest(&args.path)?;
    let manifest_display = manifest_path.display().to_string();
    let Some(deps_raw) = &manifest.deps_raw else {
        return Err(
            "this project declares no `[dependencies]` — run `cohdl update` to migrate".into(),
        );
    };
    let entries = match deps::validate_deps(&manifest_display, deps_raw) {
        Ok(e) => e,
        Err(diags) => {
            eprint!("{}", deps::render_human(&diags));
            return Ok(false);
        }
    };
    let reg = deps::Registry {
        std_root: project::find_std_root(),
        project_deps: args.path.join("deps"),
        cache_root: registry::cache_root(),
    };
    let mut fetched = 0usize;
    for dep in &entries {
        let on_disk = reg.families(&dep.name).iter().any(|f| {
            deps::available_versions(f, &dep.name)
                .map(|v| v.iter().any(|(ver, _)| *ver == dep.version))
                .unwrap_or(false)
        });
        if on_disk {
            continue;
        }
        match registry::download_into_cache(&dep.name, dep.version) {
            Ok((dir, _)) => {
                fetched += 1;
                eprintln!(
                    "  fetched {} {} -> {}",
                    dep.name,
                    dep.version,
                    dir.display()
                );
            }
            Err(d) => {
                eprint!("{}", render_diag(&d));
                return Ok(false);
            }
        }
    }
    let lock_path = args.path.join("cohdl.lock");
    let prior = std::fs::read_to_string(&lock_path).ok();
    match deps::resolve(
        &manifest_display,
        &lock_path.display().to_string(),
        &entries,
        &reg,
        prior.as_deref(),
        deps::Update::No,
    ) {
        Ok(res) => {
            if res.lock_changed || prior.is_none() {
                write_lock_file(&lock_path, &res.lock.render())?;
            }
            eprintln!(
                "  installed {} dependencies ({} fetched from the registry)",
                res.deps.len(),
                fetched
            );
            Ok(true)
        }
        Err(diags) => {
            eprint!("{}", deps::render_human(&diags));
            Ok(false)
        }
    }
}

/// `cohdl login` (RFC-030): browser-page + paste-a-token flow (the cargo
/// shape); the token is verified against POST /login and stored with the
/// account's publish grants in ~/.cohdl/credentials.toml.
fn login_command() -> Result<bool, String> {
    use cohdl::registry;
    let reg = registry::registry_url();
    eprintln!("Open {}/me and create a token, then paste it below.", reg);
    eprint!("token: ");
    let mut token = String::new();
    std::io::stdin()
        .read_line(&mut token)
        .map_err(|e| e.to_string())?;
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("no token given".to_string());
    }
    let tmp = std::env::temp_dir().join(format!("cohdl-login-{}", std::process::id()));
    std::fs::write(&tmp, format!("{{\"token\":\"{}\"}}", token)).map_err(|e| e.to_string())?;
    let resp = registry::http_post(
        &format!("{reg}/login"),
        Some(&tmp),
        Some(&token),
        "application/json",
    );
    let _ = std::fs::remove_file(&tmp);
    let resp = resp.map_err(|e| format!("cannot reach the registry: {e}"))?;
    if resp.status != 200 {
        return Err(format!(
            "the registry rejected the token (HTTP {})",
            resp.status
        ));
    }
    let account =
        registry::json_str_field(&resp.body, "account").unwrap_or_else(|| "?".to_string());
    let path = registry::write_token(&token)?;
    eprintln!(
        "  logged in as {} (token stored in {})",
        account,
        path.display()
    );
    Ok(true)
}

/// `cohdl publish` (RFC-030): three-tier pre-flight, deterministic tar of
/// the RFC-029 hash file set, POST; the server's own recomputed hash is
/// authoritative (a local/server disagreement is surfaced, E1206).
fn publish_command(args: &Args) -> Result<bool, String> {
    use cohdl::registry;
    let (manifest_path, manifest) = project::peek_manifest(&args.path)?;
    let Some(version) = &manifest.version else {
        return Err(format!(
            "{}: publishing needs `[package] version`",
            manifest_path.display()
        ));
    };
    cohdl::deps::parse_exact_version(version)
        .map_err(|e| format!("{}: {}", manifest_path.display(), e))?;
    let tier = registry::name_tier(&manifest.name)
        .map_err(|e| format!("{}: {}", manifest_path.display(), e))?;
    let Some(token) = registry::read_token() else {
        eprint!(
            "{}",
            render_diag(
                &cohdl::deps::PackageDiag::error(
                    "E1201",
                    "cohdl publish",
                    0,
                    "publishing needs authentication".to_string(),
                )
                .with_help("run `cohdl login` first".to_string())
            )
        );
        return Ok(false);
    };
    eprintln!(
        "  publishing {} {} — {}",
        manifest.name,
        version,
        tier.describe()
    );

    let tar = registry::pack_tar(&args.path)?;
    let local_hash = cohdl::hash::package_content_hash(&args.path)?;
    let tmp = std::env::temp_dir().join(format!("cohdl-publish-{}.tar", std::process::id()));
    std::fs::write(&tmp, &tar).map_err(|e| e.to_string())?;
    let url = format!(
        "{}/packages/{}/{}",
        registry::registry_url(),
        manifest.name,
        version
    );
    let resp = registry::http_post(&url, Some(&tmp), Some(&token), "application/x-tar");
    let _ = std::fs::remove_file(&tmp);
    let resp = resp.map_err(|e| format!("cannot reach the registry: {e}"))?;
    match resp.status {
        200 | 201 => {
            let server_hash = registry::json_str_field(&resp.body, "hash").unwrap_or_default();
            if server_hash != local_hash {
                eprint!(
                    "{}",
                    render_diag(&cohdl::deps::PackageDiag::warning(
                        "E1206",
                        &url,
                        0,
                        format!(
                            "the registry computed {server_hash} but this client computed {local_hash} — the server's hash is authoritative for cohdl.lock"
                        ),
                    ))
                );
            }
            eprintln!(
                "  published {} {} ({})",
                manifest.name, version, server_hash
            );
            Ok(true)
        }
        401 => {
            eprint!(
                "{}",
                render_diag(
                    &cohdl::deps::PackageDiag::error(
                        "E1201",
                        "cohdl publish",
                        0,
                        "the registry rejected the stored token".to_string(),
                    )
                    .with_help("run `cohdl login` again".to_string())
                )
            );
            Ok(false)
        }
        403 | 409 => {
            let msg = registry::json_str_field(&resp.body, "error")
                .unwrap_or_else(|| format!("publish rejected (HTTP {})", resp.status));
            eprint!(
                "{}",
                render_diag(&cohdl::deps::PackageDiag::error("E1202", &url, 0, msg))
            );
            Ok(false)
        }
        other => Err(format!("publish failed: HTTP {other}")),
    }
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

    // RFC-029: canonicalize the manifest's [dependencies] section (entries
    // sorted by name, comments kept ahead of them) when formatting a project
    // directory. cohdl.lock is machine-generated and never touched.
    let manifest_path = args.path.join("cohdl.toml");
    if args.path.is_dir() && manifest_path.is_file() {
        let name = manifest_path.display().to_string();
        let original = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("cannot read `{}`: {}", name, e))?;
        if let Some(formatted) = fmt_manifest_deps(&original) {
            if args.fmt_check {
                eprintln!("would reformat {}", name);
                ok = false;
            } else {
                std::fs::write(&manifest_path, formatted)
                    .map_err(|e| format!("cannot write `{}`: {}", name, e))?;
                eprintln!("formatted {}", name);
            }
        }
    }
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

/// RFC-029 canonical form for the manifest's `[dependencies]` section:
/// comment/blank lines first (original order), then entries sorted by name.
/// Returns `Some(new_text)` only when the canonical form differs.
fn fmt_manifest_deps(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.trim() == "[dependencies]")?
        .checked_add(1)?;
    let end = lines[start..]
        .iter()
        .position(|l| l.trim().starts_with('['))
        .map(|off| start + off)
        .unwrap_or(lines.len());

    let body = &lines[start..end];
    let mut head: Vec<&str> = Vec::new(); // comments, in original order
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut trailing_blanks = 0usize;
    for l in body {
        let t = l.trim();
        if t.is_empty() {
            trailing_blanks += 1;
            continue;
        }
        trailing_blanks = 0;
        if t.starts_with('#') {
            head.push(l);
        } else if let Some((k, v)) = t.split_once('=') {
            entries.push((k.trim().to_string(), v.trim().to_string()));
        } else {
            return None; // not a shape fmt understands — leave the file alone
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut canonical: Vec<String> = Vec::new();
    for c in &head {
        canonical.push(c.to_string());
    }
    for (k, v) in &entries {
        canonical.push(format!("{} = {}", k, v));
    }
    for _ in 0..trailing_blanks {
        canonical.push(String::new());
    }

    let current: Vec<String> = body.iter().map(|l| l.to_string()).collect();
    if canonical == current {
        return None;
    }
    let mut out: Vec<String> = lines[..start].iter().map(|l| l.to_string()).collect();
    out.extend(canonical);
    out.extend(lines[end..].iter().map(|l| l.to_string()));
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    Some(joined)
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

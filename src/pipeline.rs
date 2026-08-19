//! The full compilation pipeline, shared by the CLI and the fixture tests.
//!
//! Verdict ladder: parses ⊂ resolves ⊂ type-checks ⊂ connects ⊂ passes
//! residual DRC ⊂ emits netlist. `check` runs everything up to and including
//! residual DRC; `build` additionally assigns designators, binds parts, and
//! emits the netlist + BOM.

use crate::diag::Diagnostics;
use crate::ir::DesignIr;
use crate::lock::LockState;
use crate::resolve::World;
use crate::span::SourceMap;

pub struct Checked {
    pub sm: SourceMap,
    pub diags: Diagnostics,
    pub world: World,
    pub ir: Option<DesignIr>,
    /// The design that was compiled (None if selection failed).
    pub design_name: Option<String>,
    /// A design-selection failure (no such design / ambiguous designs). The
    /// pipeline still returns everything collected up to that point — source
    /// diagnostics are NEVER discarded because selection failed.
    pub selection_error: Option<String>,
}

/// Parse + resolve + type-check + expand + residual DRC.
///
/// `design` selection: explicit override > manifest top > the only design in
/// the project. A selection failure is reported via `Checked.selection_error`
/// (project-level, no span) alongside all collected diagnostics; `Err` is
/// reserved for conditions where nothing could be compiled at all.
pub fn check_files(files: &[(String, String)], design: Option<&str>) -> Result<Checked, String> {
    check_files_in("main", files, design)
}

/// RFC-016: sanitize a package name into its path-root segment — path
/// segments must lex as identifiers, so `-` (common in package names, e.g.
/// `rpi-pico2`) becomes `_`, as do any other non-identifier characters.
pub fn package_root(name: &str) -> String {
    // RFC-030: scoped registry names (`@sparkfun/power`) root their modules
    // at the sanitized `scope_name` spelling — the `@` carries no identifier
    // value and a leading `_` would read like a private convention.
    let name = name.strip_prefix('@').unwrap_or(name);
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() || out.chars().next().unwrap().is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

/// The first directory/file-stem segment of a project file's path that is
/// NOT a spellable non-keyword identifier, with a human reason — or `None`
/// if every segment that becomes a module-path component is spellable. Only
/// files nested in a subdirectory contribute segments (a file directly under
/// `src/` lives at the package root, so its own name is never a segment).
fn unspellable_module_segment(display: &str, deps: &[String]) -> Option<(String, &'static str)> {
    let d = display.replace('\\', "/");
    // A project (`src/…`) and every dependency package tree (`<name>/…`,
    // std included — RFC-029 makes std an ordinary package) contribute
    // module segments; a keyword/non-identifier segment in any of them is
    // unspellable (review R6-4 extends this to std). Loose files (no known
    // prefix) live at the package root with no segments.
    let rel = d.strip_prefix("src/").or_else(|| {
        deps.iter()
            .find_map(|dep| d.strip_prefix(&format!("{dep}/")))
    })?;
    let parts: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 1 {
        return None; // directly under src/: no module segment
    }
    // Directory segments + the file stem all become path segments.
    let mut segs: Vec<&str> = parts[..parts.len() - 1].to_vec();
    let last = parts[parts.len() - 1];
    segs.push(last.strip_suffix(".cohdl").unwrap_or(last));
    for seg in segs {
        if crate::lex::is_keyword(seg) {
            return Some((seg.to_string(), "it is a reserved keyword"));
        }
        if !crate::lex::is_identifier(seg) {
            return Some((seg.to_string(), "it is not a valid identifier"));
        }
    }
    None
}

/// Derive a file's RFC-016 module identity from its display name: `std/…`
/// displays are the std package; `src/dir/file.cohdl` displays nest under
/// the project package (directories + file stem become segments — files
/// directly under `src/` live at the package root); anything else (loose
/// files, test fixtures) is the package root.
pub(crate) fn infer_module(
    package: &str,
    deps: &[String],
    display: &str,
) -> crate::resolve::ModuleInfo {
    let d = display.replace('\\', "/");
    let dep_hit = deps.iter().find_map(|dep| {
        if d == *dep {
            Some((dep, ""))
        } else {
            d.strip_prefix(&format!("{dep}/")).map(|rel| (dep, rel))
        }
    });
    let (root, rel) = if let Some((dep, rel)) = dep_hit {
        (package_root(dep), rel.to_string())
    } else if let Some(rel) = d.strip_prefix("src/") {
        (package.to_string(), rel.to_string())
    } else {
        (package.to_string(), String::new())
    };
    let mut segs = vec![root.clone()];
    let parts: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() > 1 {
        for dir in &parts[..parts.len() - 1] {
            segs.push(package_root(dir));
        }
        let stem = parts
            .last()
            .unwrap()
            .strip_suffix(".cohdl")
            .unwrap_or(parts.last().unwrap());
        segs.push(package_root(stem));
    }
    crate::resolve::ModuleInfo {
        package: root,
        module: segs.join("::"),
    }
}

/// `check_files` with an explicit project package name (RFC-016). Files
/// displayed under `std/` form the `std` package; everything else belongs
/// to `package` with modules mirroring the `src/` file tree.
pub fn check_files_in(
    package: &str,
    files: &[(String, String)],
    design: Option<&str>,
) -> Result<Checked, String> {
    check_files_in_with_deps(package, &["std".to_string()], files, design)
}

/// RFC-029: the general form — the compile's package set is explicit:
/// `package` plus every resolved dependency name in `deps`. Files displayed
/// under `<dep>/…` form that dependency's package (std is just one of them).
pub fn check_files_in_with_deps(
    package: &str,
    deps: &[String],
    files: &[(String, String)],
    design: Option<&str>,
) -> Result<Checked, String> {
    let root = package_root(package);
    // The projected package root is itself a qualified-path segment (RFC-016
    // permits fully-qualified intra-package references), so it must be a
    // spellable non-keyword identifier NOW — a manifest `name = "device"`
    // otherwise indexes the whole package at a root no `device::…::Name`
    // path can spell (review R6-4). This is reachable in a single-package
    // project, independent of dependency loading.
    let root_unspellable: Option<&'static str> = if crate::lex::is_keyword(&root) {
        Some("it is a reserved keyword")
    } else if !crate::lex::is_identifier(&root) {
        Some("it is not a valid identifier")
    } else {
        None
    };
    let mut sm = SourceMap::new();
    let mut diags = Diagnostics::new();
    let mut parsed = Vec::new();
    let mut modules = Vec::new();
    let mut root_error_emitted = false;
    for (name, content) in files.iter() {
        let file_id = sm.add_file(name.clone(), content.clone());
        // The package-root error belongs to the PROJECT, not to a compiler-
        // owned std file that happens to load first (review R7-4). Anchor it
        // to the first non-`std/` source so the CLI/LSP surface it on the file
        // the author can actually fix.
        let is_dep = deps
            .iter()
            .any(|dep| name == dep || name.starts_with(&format!("{dep}/")));
        if !root_error_emitted && !is_dep {
            root_error_emitted = true;
            if let Some(why) = root_unspellable {
                diags.push(crate::diag::Diagnostic::error(
                    "E210",
                    crate::span::Span::new(file_id, 0, if content.is_empty() { 0 } else { 1 }),
                    format!(
                        "package root `{}` is not a valid module-path segment ({}) — a qualified `{}::…` reference could never be written",
                        root, why, root
                    ),
                ).with_help("set `[package] name` to a non-keyword identifier (letters, digits, `_`)"));
            }
        }
        // RFC-016 (review R5-3): a subdirectory or nested-file name becomes a
        // qualified-path SEGMENT, so it must be a spellable non-keyword
        // identifier — otherwise the declarations under it are indexed at an
        // identity no source can reference (`src/device/x.cohdl`,
        // `src/power-supply/x.cohdl`). Diagnose it against the file's start.
        if let Some((seg, why)) = unspellable_module_segment(name, deps) {
            diags.push(crate::diag::Diagnostic::error(
                "E210",
                crate::span::Span::new(file_id, 0, if content.is_empty() { 0 } else { 1 }),
                format!(
                    "`{}` is not a valid module-path segment ({}) — a qualified path could never reference the declarations in `{}`",
                    seg, why, name
                ),
            ).with_help("rename the directory/file to a non-keyword identifier (letters, digits, `_`; no `-`)"));
        }
        let tokens = crate::lex::lex(file_id, sm.text(file_id), &mut diags);
        parsed.push(crate::parse::parse(tokens, &mut diags));
        modules.push(infer_module(&root, deps, name));
    }
    let world = crate::check::check_declarations_in(parsed, &modules, &mut diags);

    let mut selection_error = None;
    let design_name = match design {
        Some(d) => {
            if !world.designs.contains_key(d) {
                let available: Vec<&String> = world.designs.keys().collect();
                selection_error = Some(format!(
                    "no design named `{}` (available: {})",
                    d,
                    if available.is_empty() {
                        "none".to_string()
                    } else {
                        available
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ));
                None
            } else {
                Some(d.to_string())
            }
        }
        None => match world.designs.len() {
            0 => None, // declaration-only project: still checkable
            1 => Some(world.designs.keys().next().unwrap().clone()),
            _ => {
                selection_error = Some(format!(
                    "project has {} designs ({}) — pass --design or set `[design] top` in cohdl.toml",
                    world.designs.len(),
                    world.designs.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
                None
            }
        },
    };

    let ir = design_name
        .as_deref()
        .and_then(|d| crate::check::check_design(&world, d, &mut diags));

    if let Some(ir) = &ir {
        crate::drc::run_drc(&world, ir, &mut diags);
    }

    diags.sort(&sm);
    Ok(Checked {
        sm,
        diags,
        world,
        ir,
        design_name,
        selection_error,
    })
}

pub struct BuildArtifacts {
    pub netlist: String,
    pub bom: String,
    pub lock: LockState,
    /// RFC-013 `layout.json`, present only when the design carries layout
    /// metadata (constraints or `#[placement_hint]`). Never affects the
    /// netlist/BOM bytes.
    pub layout: Option<String>,
    /// RFC-027: the Quilter physics-constraint CSV set (file name, content),
    /// present only when the design carries at least one physics attribute or
    /// an annotated `diff_pair`. Never affects the netlist/BOM bytes.
    pub quilter: Option<Vec<(String, String)>>,
    /// Informational notes for the build output (e.g. ambiguous part
    /// bindings resolved deterministically — provisional §2).
    pub notes: Vec<String>,
}

/// The `build` half: designators (RFC-005), part binding, emitters.
/// Only call when `checked.diags` has no errors and `checked.ir` is Some.
pub fn build_artifacts(checked: &mut Checked, prior_lock: &LockState) -> Option<BuildArtifacts> {
    // A design that failed the check phase produces no artifacts: do not
    // assign designators, bind parts, or emit against a design known invalid.
    if checked.diags.has_errors() {
        return None;
    }
    let mut diags = Diagnostics::new();
    // RFC-018 pad/device consistency runs at BUILD (the RFC pins it here, and
    // the compliance ledger/error registry all say build-only — review R6-5),
    // but declaration-complete: it walks `world.parts`, not the instantiated
    // IR, so an unused part's mismatched footprint is still caught (R5-4).
    crate::check::footprints::check_pad_consistency(&checked.world, &mut diags);
    let ir = checked.ir.as_mut()?;
    let mut notes = Vec::new();
    let lock = crate::lock::assign_designators(&checked.world, ir, prior_lock, &mut diags);
    crate::emit::bind_parts(&checked.world, ir, &mut diags, &mut notes);
    let failed = diags.has_errors();
    diags.sort(&checked.sm);
    checked.diags.extend(diags);
    if failed {
        return None;
    }
    let ir = checked.ir.as_ref().unwrap();
    Some(BuildArtifacts {
        netlist: crate::emit::kicad::emit_kicad_net(&checked.world, ir),
        bom: crate::emit::bom::emit_bom_csv(&checked.world, ir),
        lock,
        layout: crate::emit::layout::emit_layout_json(ir),
        quilter: crate::emit::quilter::emit_quilter_csvs(ir),
        notes,
    })
}

/// RFC-020: resolve a design's `board_outline` DXF reference into real
/// geometry, at `cohdl build`. The bytes come from `load` (the CLI reads the
/// FS, relative to the project root; tests pass a literal), keeping this
/// FS-free and testable. A missing/unreadable file or an outline that doesn't
/// parse / isn't closed is an E1006 sub-case pushed onto `checked.diags`, which
/// makes the subsequent `build_artifacts` return `None`. A no-op when the
/// design declares no outline (or one already resolved). Call BEFORE
/// `build_artifacts` so `layout.json` and the IPC `Profile` see the geometry.
pub fn resolve_board_outline(checked: &mut Checked, load: impl Fn(&str) -> Result<String, String>) {
    use crate::diag::Diagnostic;
    let (path, span) = match checked
        .ir
        .as_ref()
        .and_then(|ir| ir.layout.board_outline.as_ref())
    {
        Some(bo) if bo.geom.is_none() => (bo.path.clone(), bo.span),
        _ => return,
    };
    let layer = crate::dxf::OUTLINE_LAYER;
    match load(&path) {
        Ok(text) => match crate::dxf::extract_outline(&text, layer) {
            Ok(geom) => {
                if let Some(bo) = checked
                    .ir
                    .as_mut()
                    .and_then(|ir| ir.layout.board_outline.as_mut())
                {
                    bo.geom = Some(geom);
                }
            }
            Err(e) => checked
                .diags
                .push(Diagnostic::error("E1006", span, e.message(layer))),
        },
        Err(e) => checked.diags.push(Diagnostic::error(
            "E1006",
            span,
            format!("cannot read board-outline DXF `{}`: {}", path, e),
        )),
    }
}

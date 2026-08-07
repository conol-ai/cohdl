//! Name resolution (RFC-016): per-package module trees with explicit `use`
//! imports, replacing the original flat global scope.
//!
//! Architecture: after declarations are indexed (fully-qualified paths,
//! `package::module::Name`), every reference identifier in the AST is
//! REWRITTEN in place to its resolved fully-qualified path. Downstream
//! stages (check/expand/emit/LSP) keep doing exact-key lookups against the
//! fq-keyed maps below — no per-site resolution logic anywhere else.
//! Spans never change, so diagnostics stay precise; unresolved references
//! keep their as-written text and fail downstream lookups exactly as the
//! flat model did.
//!
//! Scope per the accepted text: trait/device/part/fn paths only. Designs
//! are NOT importable/qualifiable — they stay bare-named and project-global.

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics};
use crate::span::FileId;
use crate::units::{UnitType, UnitValue};
use std::collections::{BTreeMap, BTreeSet};

/// Per-file module identity (RFC-016): the owning package's root segment
/// (sanitized to an identifier) and the file's full module path — which
/// INCLUDES the root, e.g. `std` or `sparkfun::power::buck`.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub package: String,
    pub module: String,
}

impl ModuleInfo {
    pub fn root(package: &str) -> ModuleInfo {
        ModuleInfo {
            package: package.to_string(),
            module: package.to_string(),
        }
    }
}

/// One entry in the union namespace: everything reference resolution needs
/// to know about a declared trait/device/fn/part.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub kind: &'static str,
    pub is_pub: bool,
    pub span: crate::span::Span,
}

/// The last `::` segment of a (possibly fully-qualified) name — the display
/// spelling for humans; emitters and message text use this so a resolved
/// `passive::MLCC` still reads `MLCC`.
pub fn short(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

/// Everything declared across the compilation. `traits`/`devices`/`fns`/
/// `parts` are keyed by FULLY-QUALIFIED path; `designs` by bare name.
#[derive(Debug, Default)]
pub struct World {
    pub traits: BTreeMap<String, TraitDef>,
    pub devices: BTreeMap<String, DeviceDef>,
    pub fns: BTreeMap<String, FnDef>,
    pub parts: BTreeMap<String, PartDef>,
    /// RFC-018 reusable pad definitions.
    pub pads: BTreeMap<String, PadDef>,
    /// RFC-017/018 footprints (empty body = stage-one placeholder).
    pub footprints: BTreeMap<String, FootprintDef>,
    pub designs: BTreeMap<String, DesignDef>,
    pub impls: Vec<ImplDef>,
    /// (trait fq path, device fq path) → index into `impls`. Populated only
    /// for non-duplicate impls.
    pub impl_index: BTreeMap<(String, String), usize>,
    /// Resolved role/field maps per impl, filled by `check::impls`.
    pub resolved_impls: BTreeMap<(String, String), ResolvedImpl>,
    /// fq path → symbol facts, for suggestions and the LSP.
    pub symbols: BTreeMap<String, Symbol>,
    /// RFC-017 `#[doc("path")]` reference documents, fq path → paths in
    /// source order. Opaque to compilation; surfaced by the LSP.
    pub docs: BTreeMap<String, Vec<String>>,
    /// RFC-016 `use` imports, retained for the LSP (closes review R5-10:
    /// definition/hover on the imported path). Source order; `fq` is the
    /// imported path as written (`::`-joined), resolved or not.
    pub uses: Vec<UseRef>,
}

/// One `use a::b::Name;` statement: the path and the whole statement's span.
#[derive(Debug, Clone)]
pub struct UseRef {
    pub fq: String,
    pub span: crate::span::Span,
}

/// The outcome of checking one `impl Trait for Device`: every trait-required
/// role/field resolved to a concrete device pin/spec name.
#[derive(Debug, Clone, Default)]
pub struct ResolvedImpl {
    /// trait pin role → device pin name.
    pub pin_map: BTreeMap<String, String>,
    /// trait spec field → device spec field name.
    pub spec_map: BTreeMap<String, String>,
}

impl World {
    /// All traits transitively required by `trait_name` (excluding itself),
    /// in deterministic order. Cycles are pre-rejected by `validate`.
    pub fn super_traits_transitive(&self, trait_name: &str) -> Vec<String> {
        let mut out = BTreeSet::new();
        let mut stack = vec![trait_name.to_string()];
        while let Some(t) = stack.pop() {
            if let Some(def) = self.traits.get(&t) {
                for sup in &def.super_traits {
                    if out.insert(sup.name.clone()) {
                        stack.push(sup.name.clone());
                    }
                }
            }
        }
        out.into_iter().collect()
    }

    /// Does `device` have a (checked) impl for `trait_name`?
    pub fn has_impl(&self, trait_name: &str, device: &str) -> bool {
        self.impl_index
            .contains_key(&(trait_name.to_string(), device.to_string()))
    }

    /// The designator prefix for a device: the prefix of the
    /// lexicographically-smallest implemented trait that declares one;
    /// `"U"` when none does (provisional §6). RFC-016: "smallest" compares
    /// the trait's SHORT name (the pre-module flat order, so moving a trait
    /// between modules/packages never changes designators — adversarial
    /// finding), with the fq path as a deterministic tiebreaker.
    pub fn designator_prefix(&self, device: &str) -> String {
        self.impl_index
            .keys()
            .filter(|(_, d)| d == device)
            .filter_map(|(trait_name, _)| {
                let t = self.traits.get(trait_name)?;
                let (prefix, _) = t.designator_prefix.as_ref()?;
                Some(((short(trait_name), trait_name), prefix.as_str()))
            })
            .min_by(|a, b| a.0.cmp(&b.0))
            .map_or_else(|| "U".to_string(), |(_, p)| p.to_string())
    }

    /// All trait names `device` implements (checked impls only).
    pub fn implemented_traits(&self, device: &str) -> BTreeSet<String> {
        self.impl_index
            .keys()
            .filter(|(_, d)| d == device)
            .map(|(t, _)| t.clone())
            .collect()
    }

    /// RFC-016's "suggest the closest match": a declared fq path whose last
    /// segment equals `path`'s last segment (smallest fq wins,
    /// deterministically). Used by unknown-name diagnostics.
    pub fn suggest(&self, path: &str) -> Option<&str> {
        let want = short(path);
        self.symbols
            .keys()
            .find(|fq| short(fq) == want && *fq != path)
            .map(String::as_str)
    }
}

/// Compat entry: every file in one package rooted at `main` (plus the `std`
/// package inferred from `std/` display prefixes) — the exact single-flat-
/// scope ergonomics older callers/tests expect.
pub fn build_world(files: Vec<SourceFile>, diags: &mut Diagnostics) -> World {
    let modules: Vec<ModuleInfo> = (0..files.len()).map(|_| ModuleInfo::root("main")).collect();
    build_world_in(files, &modules, diags)
}

pub fn build_world_in(
    mut files: Vec<SourceFile>,
    modules: &[ModuleInfo],
    diags: &mut Diagnostics,
) -> World {
    debug_assert_eq!(files.len(), modules.len());

    // ---- pass 1: index declarations (fq) + designs (bare, global) ----------
    let mut symbols: BTreeMap<String, Symbol> = BTreeMap::new();
    let mut seen_designs: BTreeMap<String, crate::span::Span> = BTreeMap::new();
    for (i, file) in files.iter().enumerate() {
        let module = &modules[i].module;
        for item in &file.items {
            let kind_str = item.kind.kind_str();
            match &item.kind {
                ItemKind::Design(d) => {
                    // Designs are project-global and never importable
                    // (RFC-016 scope) — bare-name duplicate check.
                    if let Some(prev) = seen_designs.insert(d.name.name.clone(), d.name.span) {
                        diags.push(
                            Diagnostic::error(
                                "E201",
                                d.name.span,
                                format!(
                                    "duplicate declaration of `{}` (design names are project-global)",
                                    d.name.name
                                ),
                            )
                            .with_secondary(prev, "earlier declared here as a design".to_string()),
                        );
                    }
                }
                ItemKind::Impl(_) | ItemKind::Use(_) => {}
                _ => {
                    if let Some(name) = item.kind.name() {
                        let fq = format!("{}::{}", module, name.name);
                        if let Some(prev) = symbols.get(&fq) {
                            diags.push(
                                Diagnostic::error(
                                    "E201",
                                    name.span,
                                    format!(
                                        "duplicate declaration of `{}` in module `{}` (top-level names share one scope per module)",
                                        name.name, module
                                    ),
                                )
                                .with_secondary(
                                    prev.span,
                                    format!("earlier declared here as a {}", prev.kind),
                                ),
                            );
                            continue;
                        }
                        symbols.insert(
                            fq,
                            Symbol {
                                kind: kind_str,
                                is_pub: item.is_pub,
                                span: name.span,
                            },
                        );
                    }
                }
            }
        }
    }

    // A design sharing a bare name with a same-package declaration was an
    // error in the flat model and stays one — the shadowing latitude covers
    // std only, not confusion inside one project (adversarial finding).
    for (i, file) in files.iter().enumerate() {
        let package = &modules[i].package;
        for item in &file.items {
            let ItemKind::Design(d) = &item.kind else {
                continue;
            };
            let decl = symbols
                .iter()
                .find(|(fq, _)| root_of(fq) == package && short(fq) == d.name.name);
            if let Some((fq, sym)) = decl {
                diags.push(
                    Diagnostic::error(
                        "E201",
                        d.name.span,
                        format!(
                            "duplicate declaration of `{}` — a design and a {} share the name in package `{}`",
                            d.name.name, sym.kind, package
                        ),
                    )
                    .with_secondary(sym.span, format!("`{}` is declared here", fq)),
                );
            }
        }
    }

    // ---- pass 2: per-file `use` imports (validated at the use site) --------
    // FileId → local name → (fq path, use span).
    let mut imports: BTreeMap<u32, BTreeMap<String, (String, crate::span::Span)>> = BTreeMap::new();
    let mut use_refs: Vec<UseRef> = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let package = &modules[i].package;
        for item in &file.items {
            let ItemKind::Use(u) = &item.kind else {
                continue;
            };
            let fq = u.path_text();
            let local = u.local().name.clone();
            let fid = u.span.file.0;
            use_refs.push(UseRef {
                fq: fq.clone(),
                span: u.span,
            });
            match symbols.get(&fq) {
                None => {
                    // A design at that path is a real declaration — say so
                    // precisely instead of "nothing is declared there".
                    let local_name = &u.local().name;
                    let mut d = if seen_designs.contains_key(local_name) {
                        Diagnostic::error(
                            "E202",
                            u.span,
                            format!(
                                "`{}` is a design — designs are project-global and cannot be imported",
                                local_name
                            ),
                        )
                    } else {
                        Diagnostic::error(
                            "E202",
                            u.span,
                            format!("unresolved `use` path `{}` — nothing is declared there", fq),
                        )
                    };
                    if let Some(sugg) = suggest_in(&symbols, &fq) {
                        d = d.with_help(format!("did you mean `use {};`?", sugg));
                    }
                    diags.push(d);
                    continue;
                }
                Some(sym) => {
                    if root_of(&fq) != package && !sym.is_pub {
                        diags.push(
                            Diagnostic::error(
                                "E209",
                                u.span,
                                format!(
                                    "`{}` is not `pub` — it is only visible inside package `{}`",
                                    fq,
                                    root_of(&fq)
                                ),
                            )
                            .with_secondary(sym.span, "declared here without `pub`".to_string())
                            .with_help(format!(
                                "mark the {} `pub` in its own package to export it",
                                sym.kind
                            )),
                        );
                        // Recovery: import anyway so downstream stays quiet.
                    }
                }
            }
            let file_imports = imports.entry(fid).or_default();
            if let Some((prev_fq, prev_span)) = file_imports.get(&local) {
                if prev_fq != &fq {
                    diags.push(
                        Diagnostic::error(
                            "E208",
                            u.span,
                            format!(
                                "`{}` is already imported from `{}` — one local name, one import",
                                local, prev_fq
                            ),
                        )
                        .with_secondary(
                            *prev_span,
                            format!("earlier imported here from `{}`", prev_fq),
                        ),
                    );
                }
                continue;
            }
            file_imports.insert(local, (fq, u.span));
        }
    }

    // ---- pass 3: resolution indexes ----------------------------------------
    // (package, bare name) → sorted fq candidates; std prelude (pub only).
    let mut unqualified: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut prelude: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for fq in symbols.keys() {
        let pkg = root_of(fq).to_string();
        let bare = short(fq).to_string();
        unqualified
            .entry((pkg.clone(), bare.clone()))
            .or_default()
            .push(fq.clone());
        if pkg == "std" && symbols[fq].is_pub {
            prelude.entry(bare).or_default().push(fq.clone());
        }
    }

    // ---- pass 4: rewrite every reference ident to its resolved fq path -----
    let resolver = Resolver {
        symbols: &symbols,
        unqualified: &unqualified,
        prelude: &prelude,
        imports: &imports,
    };
    for (i, file) in files.iter_mut().enumerate() {
        resolver.rewrite_file(file, &modules[i], diags);
    }

    // ---- pass 5: move declarations into the fq-keyed maps ------------------
    let mut world = World {
        symbols,
        uses: use_refs,
        ..World::default()
    };
    for (i, file) in files.into_iter().enumerate() {
        let module = &modules[i].module;
        for item in file.items {
            if !item.docs.is_empty() {
                if let Some(name) = item.kind.name() {
                    world.docs.insert(
                        format!("{}::{}", module, name.name),
                        item.docs.iter().map(|(p, _)| p.clone()).collect(),
                    );
                }
            }
            match item.kind {
                ItemKind::Trait(t) => {
                    world
                        .traits
                        .insert(format!("{}::{}", module, t.name.name), t);
                }
                ItemKind::Device(d) => {
                    world
                        .devices
                        .insert(format!("{}::{}", module, d.name.name), d);
                }
                ItemKind::Fn(f) => {
                    world.fns.insert(format!("{}::{}", module, f.name.name), f);
                }
                ItemKind::Part(p) => {
                    world
                        .parts
                        .insert(format!("{}::{}", module, p.name.name), p);
                }
                ItemKind::Pad(p) => {
                    world.pads.insert(format!("{}::{}", module, p.name.name), p);
                }
                ItemKind::Footprint(f) => {
                    world
                        .footprints
                        .insert(format!("{}::{}", module, f.name.name), f);
                }
                ItemKind::Design(d) => {
                    world.designs.insert(d.name.name.clone(), d);
                }
                ItemKind::Impl(i) => {
                    world.impls.push(i);
                }
                ItemKind::Use(_) => {}
            }
        }
    }

    validate(&mut world, diags);
    world
}

/// The package root segment of a qualified path.
fn root_of(path: &str) -> &str {
    path.split("::").next().unwrap_or(path)
}

fn suggest_in<'a>(symbols: &'a BTreeMap<String, Symbol>, path: &str) -> Option<&'a str> {
    let want = short(path);
    symbols
        .keys()
        .find(|fq| short(fq) == want && *fq != path)
        .map(String::as_str)
}

/// The reference-rewriting pass (see the module header): every trait/device/
/// part/fn reference ident becomes its resolved fq path, in place.
struct Resolver<'a> {
    symbols: &'a BTreeMap<String, Symbol>,
    unqualified: &'a BTreeMap<(String, String), Vec<String>>,
    prelude: &'a BTreeMap<String, Vec<String>>,
    imports: &'a BTreeMap<u32, BTreeMap<String, (String, crate::span::Span)>>,
}

impl Resolver<'_> {
    fn rewrite_file(&self, file: &mut SourceFile, module: &ModuleInfo, diags: &mut Diagnostics) {
        let no_shadow: BTreeSet<String> = BTreeSet::new();
        for item in &mut file.items {
            match &mut item.kind {
                ItemKind::Trait(t) => {
                    for sup in &mut t.super_traits {
                        self.resolve(sup, module, &no_shadow, diags);
                    }
                }
                ItemKind::Device(d) => {
                    for g in &mut d.generics {
                        if let GenericBound::Traits(ts) = &mut g.bound {
                            for t in ts {
                                self.resolve(t, module, &no_shadow, diags);
                            }
                        }
                    }
                }
                ItemKind::Fn(f) => {
                    // The fn's own generic parameter names shadow globals
                    // (unchanged precedence from the flat model).
                    let shadow: BTreeSet<String> =
                        f.generics.iter().map(|g| g.name.name.clone()).collect();
                    for g in &mut f.generics {
                        if let GenericBound::Traits(ts) = &mut g.bound {
                            for t in ts {
                                self.resolve(t, module, &no_shadow, diags);
                            }
                        }
                    }
                    for p in &mut f.params {
                        if let FnParamTy::ImplTrait(ts, _) = &mut p.ty {
                            for t in ts {
                                self.resolve(t, module, &no_shadow, diags);
                            }
                        }
                    }
                    self.rewrite_body(&mut f.body, module, &shadow, diags);
                }
                ItemKind::Part(p) => {
                    self.resolve(&mut p.device.name, module, &no_shadow, diags);
                    for arg in &mut p.device.generic_args {
                        if let GenericArg::Name(id) = arg {
                            self.resolve(id, module, &no_shadow, diags);
                        }
                    }
                    // RFC-017: footprint symbol references resolve exactly
                    // like every other cross-library name.
                    for entry in std::iter::once(&mut p.primary).chain(p.alts.iter_mut()) {
                        if let Some(fp) = &mut entry.footprint {
                            self.resolve(fp, module, &no_shadow, diags);
                        }
                    }
                }
                ItemKind::Pad(_) => {}
                ItemKind::Footprint(f) => {
                    // RFC-018: pad symbol references resolve like every
                    // other cross-library name.
                    for place in &mut f.pads {
                        self.resolve(&mut place.pad, module, &no_shadow, diags);
                    }
                }
                ItemKind::Design(d) => {
                    let body = &mut d.body;
                    self.rewrite_body(body, module, &no_shadow, diags);
                }
                ItemKind::Impl(im) => {
                    self.resolve(&mut im.trait_name, module, &no_shadow, diags);
                    self.resolve(&mut im.device_name, module, &no_shadow, diags);
                }
                ItemKind::Use(_) => {}
            }
        }
    }

    fn rewrite_body(
        &self,
        body: &mut [Stmt],
        module: &ModuleInfo,
        shadow: &BTreeSet<String>,
        diags: &mut Diagnostics,
    ) {
        for stmt in body {
            match stmt {
                Stmt::Inst(s) => {
                    if !shadow.contains(&s.ty.name.name) {
                        self.resolve(&mut s.ty.name, module, shadow, diags);
                        // R5-2: an instance type that resolves to NOTHING is
                        // unknown NOW — at the rewrite pass, over EVERY body
                        // (including functions no design ever calls). The
                        // old behavior left it for expansion, which never
                        // runs on an uncalled fn, so a bad qualified path
                        // there produced a false-clean verdict. A resolved
                        // reference has been rewritten to an fq path present
                        // in `symbols`; anything else is genuinely absent.
                        // The message matches expansion's exactly, so the
                        // design-body case (reported by both) is deduped.
                        if !self.symbols.contains_key(&s.ty.name.name) {
                            let mut d = Diagnostic::error(
                                "E202",
                                s.ty.name.span,
                                format!("unknown device or part `{}`", s.ty.name.name),
                            );
                            if let Some(sugg) = suggest_in(self.symbols, &s.ty.name.name) {
                                d = d.with_help(format!("did you mean `{}`?", sugg));
                            }
                            diags.push(d);
                        }
                    }
                    for arg in &mut s.ty.generic_args {
                        if let GenericArg::Name(id) = arg {
                            if !shadow.contains(&id.name) {
                                self.resolve(id, module, shadow, diags);
                            }
                        }
                    }
                }
                Stmt::Call(s) => {
                    self.resolve(&mut s.callee, module, shadow, diags);
                    // R5-2: an unresolved call target, same discipline.
                    if !shadow.contains(&s.callee.name)
                        && !self.symbols.contains_key(&s.callee.name)
                    {
                        let mut d = Diagnostic::error(
                            "E504",
                            s.callee.span,
                            format!("unknown fn `{}`", s.callee.name),
                        );
                        if let Some(sugg) = suggest_in(self.symbols, &s.callee.name) {
                            d = d.with_help(format!("did you mean `{}`?", sugg));
                        }
                        diags.push(d);
                    }
                    for arg in &mut s.generic_args {
                        if let GenericArg::Name(id) = arg {
                            if !shadow.contains(&id.name) {
                                self.resolve(id, module, shadow, diags);
                            }
                        }
                    }
                }
                Stmt::Net(_) | Stmt::Nc(_) | Stmt::Layout(_) => {}
            }
        }
    }

    /// Resolve one reference ident in place. Resolution order for bare
    /// names: the file's `use` imports, then the own package's modules,
    /// then the std prelude (pub items). Unresolved names keep their text
    /// (downstream unknown-name diagnostics fire, exactly as before).
    fn resolve(
        &self,
        id: &mut Ident,
        module: &ModuleInfo,
        _shadow: &BTreeSet<String>,
        diags: &mut Diagnostics,
    ) {
        if id.name.contains("::") {
            // Qualified path: exact symbol, with cross-package pub check.
            if let Some(sym) = self.symbols.get(&id.name) {
                if root_of(&id.name) != module.package && !sym.is_pub {
                    diags.push(
                        Diagnostic::error(
                            "E209",
                            id.span,
                            format!(
                                "`{}` is not `pub` — it is only visible inside package `{}`",
                                id.name,
                                root_of(&id.name)
                            ),
                        )
                        .with_secondary(sym.span, "declared here without `pub`".to_string())
                        .with_help(format!(
                            "mark the {} `pub` in its own package to export it",
                            sym.kind
                        )),
                    );
                }
            }
            // Found or not, the text already IS the path — nothing to do.
            return;
        }
        let fid = FileId(id.span.file.0);
        // 1. Explicit imports.
        if let Some(file_imports) = self.imports.get(&fid.0) {
            if let Some((fq, _)) = file_imports.get(&id.name) {
                id.name = fq.clone();
                return;
            }
        }
        // 2. The own package's modules (all of them — intra-package names
        //    stay visible unqualified everywhere in the package).
        if let Some(cands) = self
            .unqualified
            .get(&(module.package.clone(), id.name.clone()))
        {
            if cands.len() > 1 {
                diags.push(
                    Diagnostic::error(
                        "E207",
                        id.span,
                        format!(
                            "`{}` is ambiguous — it is declared at {}",
                            id.name,
                            cands
                                .iter()
                                .map(|c| format!("`{}`", c))
                                .collect::<Vec<_>>()
                                .join(" and ")
                        ),
                    )
                    .with_help("qualify the path, or import one with `use`"),
                );
            }
            id.name = cands[0].clone();
            return;
        }
        // 3. The std prelude (pub std items, implicitly in scope — the
        //    standard library is the one package whose exports need no
        //    `use`; documented in docs/compliance-report.md).
        if let Some(cands) = self.prelude.get(&id.name) {
            if cands.len() > 1 {
                diags.push(
                    Diagnostic::error(
                        "E207",
                        id.span,
                        format!(
                            "`{}` is ambiguous — it is declared at {}",
                            id.name,
                            cands
                                .iter()
                                .map(|c| format!("`{}`", c))
                                .collect::<Vec<_>>()
                                .join(" and ")
                        ),
                    )
                    .with_help("qualify the path, or import one with `use`"),
                );
            }
            id.name = cands[0].clone();
        }
        // Unresolved: leave as written.
    }
}

fn validate(world: &mut World, diags: &mut Diagnostics) {
    validate_traits(world, diags);
    validate_devices(world, diags);
    validate_fns(world, diags);
    validate_pads(world, diags);
    validate_footprints(world, diags);
    index_impls(world, diags);
    // Parts are validated in check::generics (they need generic-arg checking).
}

const MM: i128 = 1_000_000_000_000_000;
const MAX_ANNULUS_DIAMETER: i128 = 100 * MM;
const MAX_FULL_CIRCLE_SEGMENTS: usize = 512;
const MAX_PASTE_VERTICES: usize = 520;

#[derive(Clone, Copy)]
pub(crate) struct SegmentedAnnulusSectorPlan {
    pub(crate) start_angle: f64,
    pub(crate) step_angle: f64,
    pub(crate) segments: usize,
    pub(crate) vertices: usize,
}

pub(crate) struct SegmentedAnnulusPlan {
    pub(crate) sectors: [SegmentedAnnulusSectorPlan; 4],
}

fn circle_segments(diameter: i128) -> Option<usize> {
    if diameter <= 0 {
        return None;
    }
    let radius_mm = diameter as f64 / 2.0e15;
    let half_angle = (1.0 - 0.001 / radius_mm).acos();
    let segments = (std::f64::consts::PI / half_angle).ceil();
    if !segments.is_finite() || segments < 3.0 || segments > usize::MAX as f64 {
        return None;
    }
    Some(segments as usize)
}

fn segmented_annulus_sector_plan(
    full_segments: usize,
    half_gap_angle: f64,
    quadrant: usize,
) -> Option<SegmentedAnnulusSectorPlan> {
    let start_angle = quadrant as f64 * std::f64::consts::FRAC_PI_2 + half_gap_angle;
    let end_angle = (quadrant + 1) as f64 * std::f64::consts::FRAC_PI_2 - half_gap_angle;
    let segments = (((end_angle - start_angle) / std::f64::consts::TAU) * full_segments as f64)
        .ceil()
        .max(1.0);
    if !segments.is_finite() || segments > usize::MAX as f64 {
        return None;
    }
    let segments = segments as usize;
    let vertices = segments.checked_add(1)?.checked_mul(2)?;
    Some(SegmentedAnnulusSectorPlan {
        start_angle,
        step_angle: (end_angle - start_angle) / segments as f64,
        segments,
        vertices,
    })
}

pub(crate) fn segmented_annulus_plan(
    outer: i128,
    inner: i128,
    gap: i128,
) -> Option<SegmentedAnnulusPlan> {
    if outer <= inner || inner <= gap || gap <= 0 {
        return None;
    }
    let full_segments = circle_segments(outer)?;
    if full_segments > MAX_FULL_CIRCLE_SEGMENTS {
        return None;
    }

    let half_gap_angle = (gap as f64 / inner as f64).asin();
    if !half_gap_angle.is_finite() {
        return None;
    }
    let sector = |quadrant| segmented_annulus_sector_plan(full_segments, half_gap_angle, quadrant);
    let sectors = [sector(0)?, sector(1)?, sector(2)?, sector(3)?];
    let total_vertices = sectors
        .iter()
        .try_fold(0usize, |total, sector| total.checked_add(sector.vertices))?;
    if total_vertices > MAX_PASTE_VERTICES {
        return None;
    }
    Some(SegmentedAnnulusPlan { sectors })
}

/// RFC-018 pad declaration checks — all local, at declaration time:
/// required fields, Length-typed dimensions, size arity vs shape, and the
/// drill ⇔ plated_through_hole biconditional.
fn validate_pads(world: &World, diags: &mut Diagnostics) {
    use crate::units::UnitType;
    for pad in world.pads.values() {
        let mut require = |present: bool, what: &str| {
            if !present {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.name.span,
                    format!("pad `{}` is missing `{}`", pad.name.name, what),
                ));
            }
        };
        require(pad.shape.is_some(), "shape");
        require(pad.size_span.is_some(), "size");
        require(pad.layer.is_some(), "layer");
        require(pad.plating.is_some(), "plating");

        if let (Some((shape, _)), Some(size_span)) = (&pad.shape, &pad.size_span) {
            if pad.size.len() != shape.size_arity() {
                diags.push(Diagnostic::error(
                    "E805",
                    *size_span,
                    format!(
                        "`{}` pads take {} — `{}` has {} dimension{}",
                        shape.name(),
                        match shape.size_arity() {
                            1 => "`size: (d)`",
                            _ => "`size: (w, h)`",
                        },
                        pad.name.name,
                        pad.size.len(),
                        if pad.size.len() == 1 { "" } else { "s" }
                    ),
                ));
            }
        }
        for v in &pad.size {
            if v.unit != UnitType::Length {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.size_span.unwrap_or(pad.name.span),
                    format!(
                        "pad dimensions are `Length` (`mm`) literals — `{}` is a `{}`",
                        v.text,
                        v.unit.type_name()
                    ),
                ));
            } else if v.femto <= 0 {
                // Length is signed for placement OFFSETS; an extent must be
                // a positive distance or the projected geometry is invalid.
                diags.push(Diagnostic::error(
                    "E805",
                    pad.size_span.unwrap_or(pad.name.span),
                    format!(
                        "pad `{}` has a non-positive dimension `{}` — a size is an extent and must be > 0mm",
                        pad.name.name, v.text
                    ),
                ));
            } else if !v.length_in_geom_range() {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.size_span.unwrap_or(pad.name.span),
                    format!(
                        "pad `{}` dimension `{}` is too large to project (review R5-5) — keep footprint geometry under 10^15 mm",
                        pad.name.name, v.text
                    ),
                ));
            }
        }

        if matches!(pad.shape, Some((crate::ast::PadShape::Annulus, _))) {
            if !matches!(pad.plating, Some((crate::ast::PadPlating::Smd, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.name.span,
                    format!("annulus pad `{}` must use `plating: smd`", pad.name.name),
                ));
            }
            if !matches!(
                pad.layer,
                Some((
                    crate::ast::PadLayer::TopCopper | crate::ast::PadLayer::BottomCopper,
                    _
                ))
            ) {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.name.span,
                    format!("annulus pad `{}` must be on a copper face", pad.name.name),
                ));
            }
            if pad.drill.is_some() {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.name.span,
                    format!("annulus pad `{}` cannot have a `drill`", pad.name.name),
                ));
            }
            if pad.chamfer.is_some() || pad.corner_radius.is_some() {
                diags.push(Diagnostic::error(
                    "E805",
                    pad.name.span,
                    format!(
                        "annulus pad `{}` cannot have a chamfer or corner radius",
                        pad.name.name
                    ),
                ));
            }
            if let [outer, inner] = pad.size.as_slice() {
                if outer.unit == UnitType::Length && inner.unit == UnitType::Length {
                    if outer.femto <= inner.femto {
                        diags.push(Diagnostic::error(
                            "E805",
                            pad.size_span.unwrap_or(pad.name.span),
                            format!(
                                "annulus pad `{}` requires outer diameter > inner diameter > 0",
                                pad.name.name
                            ),
                        ));
                    }
                    if outer.femto > MAX_ANNULUS_DIAMETER {
                        diags.push(Diagnostic::error(
                            "E805",
                            pad.size_span.unwrap_or(pad.name.span),
                            format!(
                                "annulus pad `{}` outer diameter exceeds the 100mm limit",
                                pad.name.name
                            ),
                        ));
                    }
                    match circle_segments(outer.femto) {
                        Some(n) if n <= MAX_FULL_CIRCLE_SEGMENTS => {}
                        _ => diags.push(Diagnostic::error(
                            "E805",
                            pad.size_span.unwrap_or(pad.name.span),
                            format!("annulus pad `{}` needs more than 512 full-circle segments for 1um accuracy", pad.name.name),
                        )),
                    }
                    if let Some((margin, span)) = &pad.mask_expansion {
                        let twice = margin.femto.checked_mul(2);
                        let expanded_outer = twice.and_then(|v| outer.femto.checked_add(v));
                        let expanded_inner = twice.and_then(|v| inner.femto.checked_sub(v));
                        if expanded_outer.is_none() || expanded_inner.is_none() {
                            diags.push(Diagnostic::error(
                                "E805",
                                *span,
                                format!(
                                    "annulus pad `{}` mask expansion overflows geometry",
                                    pad.name.name
                                ),
                            ));
                        } else if expanded_inner.is_some_and(|v| v <= 0) {
                            diags.push(Diagnostic::error(
                                "E805",
                                *span,
                                format!(
                                    "annulus pad `{}` mask expansion collapses its inner opening",
                                    pad.name.name
                                ),
                            ));
                        }
                    }
                }
            }
        }

        if let Some((corner, cut, span)) = &pad.chamfer {
            if !matches!(pad.shape, Some((crate::ast::PadShape::Rect, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a `{}` chamfer, but chamfers require `shape: rect`",
                        pad.name.name,
                        corner.name()
                    ),
                ));
            }
            if !matches!(pad.plating, Some((crate::ast::PadPlating::Smd, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a chamfer, but chamfers are only supported on `plating: smd` pads",
                        pad.name.name
                    ),
                ));
            }
            if cut.unit != UnitType::Length {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "a pad chamfer is a `Length` (`mm`) literal — `{}` is a `{}`",
                        cut.text,
                        cut.unit.type_name()
                    ),
                ));
            } else if cut.femto <= 0 {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a non-positive chamfer `{}` — the cut length must be > 0mm",
                        pad.name.name, cut.text
                    ),
                ));
            } else if !cut.length_in_geom_range() {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` chamfer `{}` is too large to project (review R5-5)",
                        pad.name.name, cut.text
                    ),
                ));
            } else if let [w, h] = pad.size.as_slice() {
                if cut.femto >= w.femto.min(h.femto) {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` chamfer `{}` must be smaller than both pad dimensions",
                            pad.name.name, cut.text
                        ),
                    ));
                }
            }
        }

        if let Some((radius, span)) = &pad.corner_radius {
            if !matches!(pad.shape, Some((crate::ast::PadShape::Rect, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a `corner_radius`, but rounded corners require `shape: rect`",
                        pad.name.name
                    ),
                ));
            }
            if !matches!(pad.plating, Some((crate::ast::PadPlating::Smd, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a `corner_radius`, but rounded corners are only supported on `plating: smd` pads",
                        pad.name.name
                    ),
                ));
            }
            if pad.chamfer.is_some() {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` cannot combine `corner_radius` with `chamfer`",
                        pad.name.name
                    ),
                ));
            }
            if radius.unit != UnitType::Length {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "a pad `corner_radius` is a `Length` (`mm`) literal — `{}` is a `{}`",
                        radius.text,
                        radius.unit.type_name()
                    ),
                ));
            } else if radius.femto <= 0 {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a non-positive `corner_radius` `{}` — the radius must be > 0mm",
                        pad.name.name, radius.text
                    ),
                ));
            } else if !radius.length_in_geom_range() {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` corner radius `{}` is too large to project (review R5-5)",
                        pad.name.name, radius.text
                    ),
                ));
            } else if let [w, h] = pad.size.as_slice() {
                if radius.femto > w.femto.min(h.femto) / 2 {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` corner radius `{}` exceeds half its smaller dimension",
                            pad.name.name, radius.text
                        ),
                    ));
                }
            }
        }

        if let Some((margin, span)) = &pad.mask_expansion {
            if margin.unit != UnitType::Length {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "a solder-mask expansion is a `Length` (`mm`) literal — `{}` is a `{}`",
                        margin.text,
                        margin.unit.type_name()
                    ),
                ));
            } else if margin.femto < 0 {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a negative `mask_expansion` `{}` — use a nonnegative expansion",
                        pad.name.name, margin.text
                    ),
                ));
            } else if !margin.length_in_geom_range() {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` mask expansion `{}` is too large to project (review R5-5)",
                        pad.name.name, margin.text
                    ),
                ));
            }
        }

        if let Some((paste, span)) = &pad.paste {
            if !matches!(pad.plating, Some((crate::ast::PadPlating::Smd, _))) {
                diags.push(Diagnostic::error(
                    "E805",
                    *span,
                    format!(
                        "pad `{}` has a paste override, but paste is only valid on `plating: smd` pads",
                        pad.name.name
                    ),
                ));
            }
            if let crate::ast::PadPaste::Rect(w, h) = paste {
                for v in [w, h] {
                    if v.unit != UnitType::Length {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "paste aperture dimensions are `Length` (`mm`) literals — `{}` is a `{}`",
                                v.text,
                                v.unit.type_name()
                            ),
                        ));
                    } else if v.femto <= 0 {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "pad `{}` has a non-positive paste aperture dimension `{}`",
                                pad.name.name, v.text
                            ),
                        ));
                    } else if !v.length_in_geom_range() {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "pad `{}` paste aperture dimension `{}` is too large to project (review R5-5)",
                                pad.name.name, v.text
                            ),
                        ));
                    }
                }
                let (copper_w, copper_h) = match pad.size.as_slice() {
                    [d] => (Some(d), Some(d)),
                    [cw, ch] => (Some(cw), Some(ch)),
                    _ => (None, None),
                };
                if let (Some(cw), Some(ch)) = (copper_w, copper_h) {
                    if w.unit == UnitType::Length
                        && h.unit == UnitType::Length
                        && (w.femto > cw.femto || h.femto > ch.femto)
                    {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "pad `{}` paste aperture ({}, {}) exceeds its copper envelope ({}, {})",
                                pad.name.name, w.text, h.text, cw.text, ch.text
                            ),
                        ));
                    }
                }
            } else if let crate::ast::PadPaste::SegmentedAnnulus(values) = paste {
                let [outer, inner, gap] = values.as_ref();
                if !matches!(pad.shape, Some((crate::ast::PadShape::Annulus, _))) {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!("pad `{}` uses `segmented_annulus` paste, but its copper is not annular", pad.name.name),
                    ));
                }
                for value in [outer, inner, gap] {
                    if value.unit != UnitType::Length {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!("segmented-annulus arguments are `Length` (`mm`) literals — `{}` is a `{}`", value.text, value.unit.type_name()),
                        ));
                    } else if value.femto <= 0 {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "pad `{}` has a non-positive segmented-annulus argument `{}`",
                                pad.name.name, value.text
                            ),
                        ));
                    }
                }
                if let [copper_outer, copper_inner] = pad.size.as_slice() {
                    if outer.femto > copper_outer.femto || inner.femto < copper_inner.femto {
                        diags.push(Diagnostic::error(
                            "E805",
                            *span,
                            format!("pad `{}` segmented paste must stay within its annular copper envelope", pad.name.name),
                        ));
                    }
                }
                if outer.femto <= inner.femto || gap.femto >= inner.femto {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` segmented paste requires outer > inner > gap > 0",
                            pad.name.name
                        ),
                    ));
                }
                if segmented_annulus_plan(outer.femto, inner.femto, gap.femto).is_none() {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` segmented paste exceeds the 520-vertex limit",
                            pad.name.name
                        ),
                    ));
                }
            }
        }

        match (&pad.plating, &pad.drill) {
            (Some((crate::ast::PadPlating::PlatedThroughHole, span)), None) => {
                diags.push(
                    Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` is `plated_through_hole` but has no `drill:`",
                            pad.name.name
                        ),
                    )
                    .with_help("add `drill: <diameter>` (a `mm` literal)"),
                );
            }
            (Some((crate::ast::PadPlating::Smd, _)), Some((_, drill_span))) => {
                diags.push(Diagnostic::error(
                    "E805",
                    *drill_span,
                    format!(
                        "pad `{}` is `smd` — `drill:` is only valid with `plating: plated_through_hole`",
                        pad.name.name
                    ),
                ));
            }
            _ => {}
        }
        if let Some((drill, span)) = &pad.drill {
            let noun = match drill {
                crate::ast::PadDrill::Round(_) => "drill diameter",
                crate::ast::PadDrill::Slot(..) => "slot dimension",
            };
            for v in drill.values() {
                if v.unit != UnitType::Length {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "the {} is a `Length` (`mm`) literal — `{}` is a `{}`",
                            noun,
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                } else if v.femto <= 0 {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` has a non-positive {} `{}` — a drill must be > 0mm",
                            pad.name.name, noun, v.text
                        ),
                    ));
                } else if !v.length_in_geom_range() {
                    diags.push(Diagnostic::error(
                        "E805",
                        *span,
                        format!(
                            "pad `{}` drill `{}` is too large to project (review R5-5)",
                            pad.name.name, v.text
                        ),
                    ));
                }
            }
            // A slot inside a round pad is not a manufacturable padstack: the
            // hole would break out of its own annular ring on the long axis.
            // Same shape/geometry pairing rule RFC-023 applies to mount_hole.
            if let crate::ast::PadDrill::Slot(w, l) = drill {
                // A pad's shape is explicit (a missing one is its own error),
                // so match it directly rather than assuming a default.
                if matches!(pad.shape, Some((crate::ast::PadShape::Circle, _))) {
                    diags.push(
                        Diagnostic::error(
                            "E805",
                            *span,
                            format!(
                                "pad `{}` is a `circle` but declares a slot drill `({}, {})` — a slot needs an elongated pad",
                                pad.name.name, w.text, l.text
                            ),
                        )
                        .with_help(
                            "use `shape: oval` (or `rect`), or a round `drill: <diameter>`"
                                .to_string(),
                        ),
                    );
                }
                // The hole must stay inside the copper it is drilled through,
                // per axis — otherwise the pad has no annular ring at all.
                if let [pw, ph] = pad.size.as_slice() {
                    for (hole, pad_dim, axis) in [(w, pw, "width"), (l, ph, "length")] {
                        if hole.unit == UnitType::Length
                            && pad_dim.unit == UnitType::Length
                            && hole.femto > pad_dim.femto
                        {
                            diags.push(Diagnostic::error(
                                "E805",
                                *span,
                                format!(
                                    "pad `{}` slot {} `{}` exceeds the pad's own {} `{}` — the hole would leave no annular ring",
                                    pad.name.name, axis, hole.text, axis, pad_dim.text
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// RFC-018 footprint body checks: pad references resolve to PAD symbols and
/// coordinates/courtyard dimensions are Length. An electrical pad number may
/// have multiple physical placements (exposed-pad copper, paste lands, vias).
fn validate_footprints(world: &World, diags: &mut Diagnostics) {
    use crate::units::UnitType;
    for fp in world.footprints.values() {
        for place in &fp.pads {
            if !world.pads.contains_key(&place.pad.name) {
                let d = if world.symbols.contains_key(&place.pad.name) {
                    let kind = world
                        .symbols
                        .get(&place.pad.name)
                        .map(|s| s.kind)
                        .unwrap_or("name");
                    Diagnostic::error(
                        "E205",
                        place.pad.span,
                        format!(
                            "`{}` is a {}, not a pad — pad placements reference `pad` declarations",
                            place.pad.name, kind
                        ),
                    )
                } else {
                    with_suggestion(
                        world,
                        &place.pad.name,
                        Diagnostic::error(
                            "E202",
                            place.pad.span,
                            format!("unknown pad `{}`", place.pad.name),
                        ),
                    )
                };
                diags.push(d);
            }
            for v in [&place.x, &place.y] {
                if v.unit != UnitType::Length {
                    diags.push(Diagnostic::error(
                        "E806",
                        place.span,
                        format!(
                            "pad offsets are `Length` (`mm`) literals — `{}` is a `{}`",
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                } else if !v.length_in_geom_range() {
                    diags.push(Diagnostic::error(
                        "E806",
                        place.span,
                        format!(
                            "pad offset `{}` is too large to project (review R5-5)",
                            v.text
                        ),
                    ));
                }
            }
        }
        for (kw, c) in [
            ("courtyard", fp.courtyard.as_ref()),
            ("window", fp.window.as_deref()),
        ] {
            let Some(c) = c else { continue };
            if c.size.len() != c.shape.0.size_arity() {
                diags.push(Diagnostic::error(
                    "E806",
                    c.size_span,
                    format!(
                        "a `{}` {} takes {} dimension{}",
                        c.shape.0.name(),
                        kw,
                        c.shape.0.size_arity(),
                        if c.shape.0.size_arity() == 1 { "" } else { "s" }
                    ),
                ));
            }
            for v in c.size.iter().chain([&c.at.0, &c.at.1]) {
                if v.unit != UnitType::Length {
                    diags.push(Diagnostic::error(
                        "E806",
                        c.span,
                        format!(
                            "{} dimensions are `Length` (`mm`) literals — `{}` is a `{}`",
                            kw,
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                } else if !v.length_in_geom_range() {
                    diags.push(Diagnostic::error(
                        "E806",
                        c.span,
                        format!(
                            "{} dimension `{}` is too large to project (review R5-5)",
                            kw, v.text
                        ),
                    ));
                }
            }
            for v in &c.size {
                if v.unit == UnitType::Length && v.femto <= 0 {
                    diags.push(Diagnostic::error(
                        "E806",
                        c.size_span,
                        format!(
                            "{} of footprint `{}` has a non-positive dimension `{}` — a size is an extent and must be > 0mm",
                            kw, fp.name.name, v.text
                        ),
                    ));
                }
            }
        }
        // RFC-031: silkscreen primitives are Length-typed and well-formed, and
        // every marker names a pad THIS footprint actually declares.
        if let Some(block) = &fp.silkscreen {
            use crate::ast::{SilkGraphic, SilkItem};
            let declared: Vec<&str> = fp.pads.iter().map(|p| p.number.text.as_str()).collect();
            for item in &block.items {
                let span = item.span();
                let lengths: Vec<&UnitValue> = match item {
                    SilkItem::Graphic(SilkGraphic::Line { from, to, width }, _) => {
                        vec![&from.0, &from.1, &to.0, &to.1, width]
                    }
                    SilkItem::Graphic(
                        SilkGraphic::Circle {
                            at, radius, width, ..
                        },
                        _,
                    ) => {
                        vec![&at.0, &at.1, radius, width]
                    }
                    SilkItem::Graphic(
                        SilkGraphic::Arc {
                            at, radius, width, ..
                        },
                        _,
                    ) => {
                        vec![&at.0, &at.1, radius, width]
                    }
                    SilkItem::Graphic(SilkGraphic::Polygon { points, .. }, _) => {
                        points.iter().flat_map(|(x, y)| [x, y]).collect()
                    }
                    _ => Vec::new(),
                };
                for v in lengths {
                    if v.unit != UnitType::Length {
                        diags.push(Diagnostic::error(
                            "E812",
                            span,
                            format!(
                                "silkscreen dimensions are `Length` (`mm`) literals — `{}` is a `{}`",
                                v.text,
                                v.unit.type_name()
                            ),
                        ));
                    } else if !v.length_in_geom_range() {
                        diags.push(Diagnostic::error(
                            "E812",
                            span,
                            format!(
                                "silkscreen dimension `{}` is too large to project (review R5-5)",
                                v.text
                            ),
                        ));
                    }
                }
                // A stroke or radius of zero draws nothing; say so rather than
                // emitting an invisible artefact.
                let positive: Vec<(&str, &UnitValue)> = match item {
                    SilkItem::Graphic(SilkGraphic::Line { width, .. }, _) => vec![("width", width)],
                    SilkItem::Graphic(SilkGraphic::Circle { radius, width, .. }, _)
                    | SilkItem::Graphic(SilkGraphic::Arc { radius, width, .. }, _) => {
                        vec![("radius", radius), ("width", width)]
                    }
                    _ => Vec::new(),
                };
                for (what, v) in positive {
                    if v.unit == UnitType::Length && v.femto <= 0 {
                        diags.push(Diagnostic::error(
                            "E812",
                            span,
                            format!(
                                "silkscreen {} `{}` must be > 0mm — a zero-width stroke draws nothing",
                                what, v.text
                            ),
                        ));
                    }
                }
                // Marker targets: a local, single-declaration lookup, per the
                // RFC's type-system-first test.
                let target = match item {
                    SilkItem::Pin1Marker { pad, .. } => Some(("pin_1_marker", pad)),
                    SilkItem::PolarityMarker { cathode_pad, .. } => {
                        Some(("polarity_marker", cathode_pad))
                    }
                    SilkItem::Graphic(..) => None,
                };
                if let Some((kind, pad)) = target {
                    if !declared.contains(&pad.text.as_str()) {
                        diags.push(
                            Diagnostic::error(
                                "E812",
                                pad.span,
                                format!(
                                    "`{}` names pad `{}`, which footprint `{}` does not declare",
                                    kind, pad.text, fp.name.name
                                ),
                            )
                            .with_help(if declared.is_empty() {
                                "this footprint declares no pads at all".to_string()
                            } else {
                                format!("its pads are: {}", declared.join(", "))
                            }),
                        );
                    }
                }
                // A polarity marker needs a second ELECTRICAL terminal to
                // orient against. Repeated physical placements of one number
                // (EP copper/paste/vias) are still one terminal.
                let electrical_pad_count = fp
                    .pads
                    .iter()
                    .map(|p| p.number.text.as_str())
                    .collect::<BTreeSet<_>>()
                    .len();
                if matches!(item, SilkItem::PolarityMarker { .. }) && electrical_pad_count < 2 {
                    diags.push(Diagnostic::error(
                        "E812",
                        span,
                        format!(
                            "`polarity_marker` needs at least two electrical pad numbers to orient against — footprint `{}` declares {}",
                            fp.name.name,
                            electrical_pad_count
                        ),
                    ));
                }
            }
        }
        if let Some((x, y, span)) = &fp.silkscreen_ref {
            for v in [x, y] {
                if v.unit != UnitType::Length {
                    diags.push(Diagnostic::error(
                        "E806",
                        *span,
                        format!(
                            "`silkscreen_ref` coordinates are `Length` (`mm`) literals — `{}` is a `{}`",
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                } else if !v.length_in_geom_range() {
                    diags.push(Diagnostic::error(
                        "E806",
                        *span,
                        format!(
                            "`silkscreen_ref` coordinate `{}` is too large to project (review R5-5/R6-9)",
                            v.text
                        ),
                    ));
                }
            }
        }
        // RFC-025: a pad placement's rotation tracks `place`'s own set, checked
        // at declaration (E811) exactly like `place`'s E1007 — now any whole
        // degree in 0..=359 (see that deviation note).
        for pp in &fp.pads {
            if pp.rotate > 359 {
                let shown = if pp.rotate == u16::MAX {
                    "that value".to_string()
                } else {
                    pp.rotate.to_string()
                };
                diags.push(Diagnostic::error(
                    "E811",
                    pp.span,
                    format!(
                        "pad `{}` in footprint `{}`: `rotate {}` is not a rotation — give a whole number of degrees in 0..=359 (counter-clockwise)",
                        pp.number.text, fp.name.name, shown
                    ),
                ));
            }
        }
        // RFC-022 mount_hole checks — structural, local, numbered in their OWN
        // namespace (never compared with pad numbers or the bound device's pins).
        let mut mh_seen: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
        for mh in &fp.mount_holes {
            if let Some(prev) = mh_seen.insert(mh.number.text.as_str(), mh.number.span) {
                diags.push(
                    Diagnostic::error(
                        "E810",
                        mh.number.span,
                        format!(
                            "duplicate mount_hole number `{}` in footprint `{}`",
                            mh.number.text, fp.name.name
                        ),
                    )
                    .with_secondary(prev, "first placed here".to_string()),
                );
            }
            // RFC-023: the geometry form must match the shape — explicit or
            // defaulted. `circle` takes `diameter`, `rect`/`oval` take `size:`.
            let shape = mh.shape_or_default();
            let wants_size = matches!(shape, PadShape::Rect | PadShape::Oval);
            let has_size = matches!(mh.geom, crate::ast::MountHoleGeom::Size(..));
            if wants_size != has_size {
                let (expected, found) = if wants_size {
                    ("size: (w, h)", "diameter")
                } else {
                    ("diameter D", "size:")
                };
                let d = Diagnostic::error(
                    "E810",
                    mh.geom.span(mh.span),
                    format!(
                        "mount_hole `{}` in footprint `{}` is `shape: {}`, which takes `{}` — found `{}`",
                        mh.number.text,
                        fp.name.name,
                        shape.name(),
                        expected,
                        found
                    ),
                );
                diags.push(match mh.shape {
                    Some((_, s)) => d.with_secondary(s, "shape declared here".to_string()),
                    // No explicit shape: say so, so the default isn't a mystery.
                    None => d.with_secondary(
                        mh.number.span,
                        "no `shape:` written, so this hole defaults to `circle`".to_string(),
                    ),
                });
            }
            // Dimension count: `size:` is exactly (w, h), like `pad`'s own.
            if let crate::ast::MountHoleGeom::Size(dims, span) = &mh.geom {
                if dims.len() != 2 {
                    diags.push(Diagnostic::error(
                        "E810",
                        *span,
                        format!(
                            "mount_hole `{}` in footprint `{}` takes `size: (w, h)` — found {} dimension{}",
                            mh.number.text,
                            fp.name.name,
                            dims.len(),
                            if dims.len() == 1 { "" } else { "s" }
                        ),
                    ));
                }
            }
            let geom_vals: Vec<(&UnitValue, &str)> = match &mh.geom {
                crate::ast::MountHoleGeom::Diameter(d) => vec![(d, "diameter")],
                crate::ast::MountHoleGeom::Size(dims, _) => {
                    dims.iter().map(|d| (d, "size")).collect()
                }
            };
            for (v, what) in [(&mh.x, "offset"), (&mh.y, "offset")]
                .into_iter()
                .chain(geom_vals.iter().copied())
            {
                if v.unit != UnitType::Length {
                    diags.push(Diagnostic::error(
                        "E810",
                        mh.span,
                        format!(
                            "mount_hole {} is a `Length` (`mm`) literal — `{}` is a `{}`",
                            what,
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                } else if !v.length_in_geom_range() {
                    diags.push(Diagnostic::error(
                        "E810",
                        mh.span,
                        format!("mount_hole {} `{}` is too large to project", what, v.text),
                    ));
                }
            }
            // A hole with a zero or negative extent is not a hole.
            for (v, what) in geom_vals {
                if v.unit == UnitType::Length && v.femto <= 0 {
                    diags.push(Diagnostic::error(
                        "E810",
                        mh.span,
                        format!(
                            "mount_hole `{}` in footprint `{}` has a non-positive {} `{}` — a hole must be > 0mm",
                            mh.number.text, fp.name.name, what, v.text
                        ),
                    ));
                }
            }
        }
        validate_footprint_name(fp, diags);
    }
}

/// RFC-021 (rewritten): a footprint's OWN identifier is its IPC-7351 name when
/// its package prefix falls in the closed six-template family set. Validate the
/// name grammar (E808) and, for the geometry-regular families, agreement between
/// the name's declared pin count / pitch and the footprint's own pad placements
/// (E809). A name whose prefix is outside the closed set is an ordinary RFC-016
/// identifier — `parse` returns `UnknownFamily` and we leave it unchecked. Both
/// checks are declaration-local (the footprint's own content), never DRC.
fn validate_footprint_name(fp: &crate::ast::FootprintDef, diags: &mut Diagnostics) {
    use crate::check::ipc7351;
    let name = &fp.name.name;
    let span = fp.name.span;
    let parsed = match ipc7351::parse(name) {
        Ok(p) => p,
        // Outside the closed six-template set → free-form name, unchecked.
        Err(ipc7351::ParseErr::UnknownFamily) => return,
        Err(e) => {
            diags.push(Diagnostic::error(
                "E808",
                span,
                format!(
                    "footprint `{name}` is named for a closed IPC-7351 family but is not a valid IPC-7351 name: {}",
                    e.message()
                ),
            ));
            return;
        }
    };
    // Geometry cross-check needs real pad content; an empty (placeholder)
    // footprint has nothing to check the name against.
    if fp.pads.is_empty() {
        return;
    }
    // One electrical pin may have several physical placements (a large top
    // exposed pad, paste-segmentation lands, thermal vias, and a back land).
    // IPC-7351's pin count names electrical pad numbers, not those placements.
    let pad_count = fp
        .pads
        .iter()
        .map(|p| p.number.text.as_str())
        .collect::<BTreeSet<_>>()
        .len() as u32;
    // Pins the geometry implies: distinct pad numbers minus the one exposed
    // pad a `-1EP` name declares (QFN family).
    let geom_pins = pad_count.saturating_sub(u32::from(parsed.has_ep));
    if let Some(name_pins) = parsed.pins {
        if geom_pins != name_pins {
            diags.push(Diagnostic::error(
                "E809",
                span,
                format!(
                    "footprint `{}` names {} pin{}, but declares {} electrical pad number{}{}",
                    fp.name.name,
                    name_pins,
                    plural(name_pins),
                    pad_count,
                    plural(pad_count),
                    if parsed.has_ep {
                        " (one of which is the `_1EP` exposed pad)"
                    } else {
                        ""
                    }
                ),
            ));
            return; // a wrong pin count makes the pitch report noise
        }
    }
    // Pitch check (families whose template encodes a pitch): the closest
    // regular-terminal center spacing must equal the declared pitch. The
    // `_1EP` terminal is excluded: its copper, paste windows, and thermal vias
    // are not perimeter leads and may legally lie closer to one. Squared
    // distances use a tiny fixed-width 256-bit helper because every individual
    // geometry coordinate fits i128 but its square need not.
    if let Some(name_pitch) = parsed.pitch_hundredths {
        if let Some(min) = min_pair_sq_femto(fp, parsed.pins, parsed.has_ep) {
            let pitch_femto = u128::from(name_pitch) * 10_000_000_000_000; // 0.01mm in femto
            if min.square != square_u128(pitch_femto) {
                let actual_h = isqrt_wide(min.square, min.dx, min.dy) / 10_000_000_000_000;
                diags.push(Diagnostic::error(
                    "E809",
                    span,
                    format!(
                        "footprint `{}` names {} ({}mm) pitch, but its closest pad spacing is {} ({}mm)",
                        fp.name.name,
                        name_pitch,
                        hundredths_mm(u128::from(name_pitch)),
                        actual_h,
                        hundredths_mm(actual_h)
                    ),
                ));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WideDistance {
    /// Little-endian base-2^32 limbs of dx^2 + dy^2.
    square: [u32; 8],
    dx: u128,
    dy: u128,
}

/// Smallest squared center-to-center distance between placements of distinct
/// regular electrical pad numbers. For `_1EP` QFNs the exposed-pad number is
/// excluded, including every repeated placement carrying that number.
fn min_pair_sq_femto(
    fp: &crate::ast::FootprintDef,
    name_pins: Option<u32>,
    has_ep: bool,
) -> Option<WideDistance> {
    let excluded_ep = if has_ep {
        name_pins.and_then(|pins| exposed_pad_number(fp, pins))
    } else {
        None
    };
    let mut min: Option<WideDistance> = None;
    for i in 0..fp.pads.len() {
        for j in (i + 1)..fp.pads.len() {
            if fp.pads[i].number.text == fp.pads[j].number.text {
                continue;
            }
            if excluded_ep
                .is_some_and(|ep| fp.pads[i].number.text == ep || fp.pads[j].number.text == ep)
            {
                continue;
            }
            // Saturation is defense-in-depth for already-invalid, out-of-range
            // literals. Every accepted geometry value is far inside this bound.
            let dx = fp.pads[i]
                .x
                .femto
                .saturating_sub(fp.pads[j].x.femto)
                .unsigned_abs();
            let dy = fp.pads[i]
                .y
                .femto
                .saturating_sub(fp.pads[j].y.femto)
                .unsigned_abs();
            let d = WideDistance {
                square: add_wide_saturating(square_u128(dx), square_u128(dy)),
                dx,
                dy,
            };
            if min.is_none_or(|m| cmp_wide(&d.square, &m.square).is_lt()) {
                min = Some(d);
            }
        }
    }
    min
}

/// Identify the one exposed-pad electrical number in a name-validated
/// `_1EP` footprint. Prefer the conventional regular lead sequence 1..=N;
/// if a device uses symbolic lead numbers, the central-most electrical number
/// is the exposed pad (the defining geometry of the closed QFN family).
fn exposed_pad_number(fp: &crate::ast::FootprintDef, name_pins: u32) -> Option<&str> {
    let numbers: BTreeSet<&str> = fp.pads.iter().map(|p| p.number.text.as_str()).collect();
    let conventional: BTreeSet<String> = (1..=name_pins).map(|n| n.to_string()).collect();
    if conventional.iter().all(|n| numbers.contains(n.as_str())) {
        return numbers.iter().copied().find(|n| !conventional.contains(*n));
    }

    let mut candidates: BTreeMap<&str, (u128, usize)> = BTreeMap::new();
    for p in &fp.pads {
        let radial =
            p.x.femto
                .unsigned_abs()
                .saturating_add(p.y.femto.unsigned_abs());
        let entry = candidates
            .entry(p.number.text.as_str())
            .or_insert((radial, 0));
        entry.0 = entry.0.min(radial);
        entry.1 += 1;
    }
    candidates
        .into_iter()
        .min_by(|(an, (ar, ac)), (bn, (br, bc))| {
            ar.cmp(br).then_with(|| bc.cmp(ac)).then_with(|| an.cmp(bn))
        })
        .map(|(number, _)| number)
}

/// Exact u128 multiplication into 256 bits, using base-2^32 limbs so every
/// intermediate product plus carry fits u64.
fn square_u128(n: u128) -> [u32; 8] {
    let limbs = [
        n as u32,
        (n >> 32) as u32,
        (n >> 64) as u32,
        (n >> 96) as u32,
    ];
    let mut out = [0u32; 8];
    for i in 0..4 {
        let mut carry = 0u64;
        for j in 0..4 {
            let k = i + j;
            let total = u64::from(out[k]) + u64::from(limbs[i]) * u64::from(limbs[j]) + carry;
            out[k] = total as u32;
            carry = total >> 32;
        }
        let mut k = i + 4;
        while carry != 0 && k < out.len() {
            let total = u64::from(out[k]) + carry;
            out[k] = total as u32;
            carry = total >> 32;
            k += 1;
        }
    }
    out
}

fn add_wide_saturating(a: [u32; 8], b: [u32; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    let mut carry = 0u64;
    for i in 0..8 {
        let total = u64::from(a[i]) + u64::from(b[i]) + carry;
        out[i] = total as u32;
        carry = total >> 32;
    }
    if carry == 0 {
        out
    } else {
        [u32::MAX; 8]
    }
}

fn cmp_wide(a: &[u32; 8], b: &[u32; 8]) -> std::cmp::Ordering {
    for i in (0..8).rev() {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
    }
    std::cmp::Ordering::Equal
}

/// Integer square root for the mismatch diagnostic. Equality above compares
/// the full 256-bit squares directly; this conversion never affects validity.
fn isqrt_wide(square: [u32; 8], dx: u128, dy: u128) -> u128 {
    let mut lo = 0u128;
    let mut hi = dx.saturating_add(dy);
    while lo < hi {
        let delta = hi - lo;
        let mid = lo + delta / 2 + delta % 2;
        if !cmp_wide(&square_u128(mid), &square).is_gt() {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

fn plural(n: u32) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Render hundredths-of-a-mm as a minimal decimal mm string (`40` → `0.4`).
fn hundredths_mm(h: u128) -> String {
    let (whole, rem) = (h / 100, h % 100);
    if rem == 0 {
        whole.to_string()
    } else {
        let mut f = format!("{:02}", rem);
        while f.ends_with('0') {
            f.pop();
        }
        format!("{}.{}", whole, f)
    }
}

fn validate_traits(world: &World, diags: &mut Diagnostics) {
    for tr in world.traits.values() {
        for sup in &tr.super_traits {
            if !world.traits.contains_key(&sup.name) {
                let d = if world.devices.contains_key(&sup.name)
                    || world.parts.contains_key(&sup.name)
                    || world.fns.contains_key(&sup.name)
                {
                    Diagnostic::error(
                        "E205",
                        sup.span,
                        format!(
                            "`{}` is not a trait (sub-trait bounds must name traits)",
                            sup.name
                        ),
                    )
                } else {
                    with_suggestion(
                        world,
                        &sup.name,
                        Diagnostic::error(
                            "E202",
                            sup.span,
                            format!("unknown trait `{}`", sup.name),
                        ),
                    )
                };
                diags.push(d);
            }
        }
        // Duplicate pin roles / spec fields within the trait.
        check_dup_names(
            tr.pins.iter().map(|p| &p.name),
            "pin role",
            &tr.name.name,
            diags,
        );
        check_dup_names(
            tr.specs.iter().map(|s| &s.name),
            "spec field",
            &tr.name.name,
            diags,
        );
    }

    // Cyclic sub-trait bounds (E306): DFS from each trait.
    for start in world.traits.keys() {
        let mut path: Vec<String> = Vec::new();
        if let Some(cycle) = find_trait_cycle(world, start, &mut path, &mut BTreeSet::new()) {
            // Report only when `start` is the lexicographically-smallest
            // member so each cycle is reported exactly once.
            if cycle.iter().min() == Some(start) {
                let tr = &world.traits[start];
                diags.push(Diagnostic::error(
                    "E306",
                    tr.name.span,
                    format!("cyclic sub-trait bounds: {}", cycle.join(" → ")),
                ));
            }
        }
    }
}

fn find_trait_cycle(
    world: &World,
    current: &str,
    path: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
) -> Option<Vec<String>> {
    if let Some(pos) = path.iter().position(|p| p == current) {
        let mut cycle = path[pos..].to_vec();
        cycle.push(current.to_string());
        return Some(cycle);
    }
    if !visited.insert(current.to_string()) {
        return None;
    }
    path.push(current.to_string());
    if let Some(def) = world.traits.get(current) {
        for sup in &def.super_traits {
            if let Some(c) = find_trait_cycle(world, &sup.name, path, visited) {
                path.pop();
                return Some(c);
            }
        }
    }
    path.pop();
    None
}

fn validate_devices(world: &World, diags: &mut Diagnostics) {
    for dev in world.devices.values() {
        check_dup_names(
            dev.generics.iter().map(|g| &g.name),
            "generic parameter",
            &dev.name.name,
            diags,
        );
        validate_generic_params(world, &dev.generics, diags);
        validate_device_variants(dev, diags);

        // Per-block pin checks: duplicate names/numbers within one variant's
        // layout (two variants legitimately reuse the same numbers).
        for block in &dev.pin_blocks {
            check_dup_names(
                block.pins.iter().map(|p| &p.name),
                "pin",
                &dev.name.name,
                diags,
            );
            let mut nums: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
            for pin in &block.pins {
                for n in &pin.numbers {
                    if let Some(prev) = nums.insert(n.text.as_str(), n.span) {
                        diags.push(
                            Diagnostic::error(
                                "E201",
                                n.span,
                                format!(
                                    "physical pin number `{}` is used twice on device `{}`",
                                    n.text, dev.name.name
                                ),
                            )
                            .with_secondary(prev, "first used here"),
                        );
                    }
                }
            }
        }

        for block in &dev.spec_blocks {
            check_dup_names(
                block.fields.iter().map(|s| &s.name),
                "spec field",
                &dev.name.name,
                diags,
            );
            // Spec fields referencing generic params must name a unit-bound
            // param, in base and variant blocks alike.
            for field in &block.fields {
                if let SpecValue::GenericRef(r) = &field.value {
                    match dev.generics.iter().find(|g| g.name.name == r.name) {
                        None => diags.push(
                            Diagnostic::error(
                                "E202",
                                r.span,
                                format!(
                                    "`{}` is not a generic parameter of device `{}`",
                                    r.name, dev.name.name
                                ),
                            )
                            .with_help(
                                "device spec values are unit literals or the device's own generic parameters",
                            ),
                        ),
                        Some(g) => {
                            if let GenericBound::Traits(_) = &g.bound {
                                diags.push(Diagnostic::error(
                                    "E205",
                                    r.span,
                                    format!(
                                        "generic parameter `{}` is trait-bound and cannot be used as a spec value (spec fields are unit-typed)",
                                        r.name
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// RFC-008 structural checks on a device's variant declarations:
/// exhaustiveness of `pins[VARIANT]` coverage (E902), undeclared qualifiers
/// (E907), unqualified `pins {}` on a variant device (E908), and duplicate
/// blocks for the same variant.
fn validate_device_variants(dev: &DeviceDef, diags: &mut Diagnostics) {
    let variant_names: Vec<&str> = dev.variants.iter().map(|v| v.name.as_str()).collect();

    // Qualifiers must name declared variants; blocks per variant are unique.
    let mut seen_pin_blocks: BTreeMap<Option<&str>, crate::span::Span> = BTreeMap::new();
    for block in &dev.pin_blocks {
        let v = block.variant.as_ref();
        if let Some(v) = v {
            if !variant_names.contains(&v.name.as_str()) {
                diags.push(undeclared_variant_diag(dev, v, "pins"));
                continue;
            }
        } else if dev.has_variants() {
            diags.push(
                Diagnostic::error(
                    "E908",
                    block.span,
                    format!(
                        "device `{}` declares variants — every `pins` block must be qualified (`pins[VARIANT] {{ … }}`)",
                        dev.name.name
                    ),
                )
                .with_help(format!("variants are: {}", variant_names.join(", "))),
            );
            continue;
        }
        let key = v.map(|x| x.name.as_str());
        if let Some(prev) = seen_pin_blocks.insert(key, block.span) {
            diags.push(
                Diagnostic::error(
                    "E201",
                    block.span,
                    match key {
                        Some(k) => format!(
                            "duplicate `pins[{}]` block on device `{}`",
                            k, dev.name.name
                        ),
                        None => format!("duplicate `pins` block on device `{}`", dev.name.name),
                    },
                )
                .with_secondary(prev, "the earlier block is here"),
            );
        }
    }
    let mut seen_spec_blocks: BTreeMap<Option<&str>, crate::span::Span> = BTreeMap::new();
    for block in &dev.spec_blocks {
        if let Some(v) = &block.variant {
            if !variant_names.contains(&v.name.as_str()) {
                diags.push(undeclared_variant_diag(dev, v, "spec"));
                continue;
            }
        }
        let key = block.variant.as_ref().map(|x| x.name.as_str());
        if let Some(prev) = seen_spec_blocks.insert(key, block.span) {
            diags.push(
                Diagnostic::error(
                    "E201",
                    block.span,
                    match key {
                        Some(k) => format!(
                            "duplicate `spec[{}]` block on device `{}`",
                            k, dev.name.name
                        ),
                        None => format!("duplicate `spec` block on device `{}`", dev.name.name),
                    },
                )
                .with_secondary(prev, "the earlier block is here"),
            );
        }
    }

    // Exhaustiveness (E902): every declared variant has a pins[VARIANT]
    // block — checked at the device's own declaration, naming each missing
    // variant (RFC-008 Tooling & operations).
    for v in &dev.variants {
        let covered = dev
            .pin_blocks
            .iter()
            .any(|b| b.variant.as_ref().is_some_and(|x| x.name == v.name));
        if !covered {
            diags.push(
                Diagnostic::error(
                    "E902",
                    v.span,
                    format!(
                        "variant `{}` of device `{}` has no `pins[{}]` block — every declared variant needs a pin layout",
                        v.name, dev.name.name, v.name
                    ),
                )
                .with_help("add the missing block, or remove the variant from `variants { }`"),
            );
        }
    }
}

fn undeclared_variant_diag(dev: &DeviceDef, v: &Ident, block_kind: &str) -> Diagnostic {
    let d = Diagnostic::error(
        "E907",
        v.span,
        format!(
            "`{}[{}]`: device `{}` declares no variant named `{}`",
            block_kind, v.name, dev.name.name, v.name
        ),
    );
    if dev.has_variants() {
        d.with_help(format!(
            "declared variants are: {}",
            dev.variants
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    } else {
        d.with_help(format!(
            "device `{}` has no `variants {{ }}` block at all",
            dev.name.name
        ))
    }
}

fn validate_fns(world: &World, diags: &mut Diagnostics) {
    for f in world.fns.values() {
        check_dup_names(
            f.generics.iter().map(|g| &g.name),
            "generic parameter",
            &f.name.name,
            diags,
        );
        check_dup_names(
            f.params.iter().map(|p| &p.name),
            "parameter",
            &f.name.name,
            diags,
        );
        validate_generic_params(world, &f.generics, diags);
        for p in &f.params {
            match &p.ty {
                FnParamTy::Pin(_) => {}
                FnParamTy::Generic(g) => {
                    match f.generics.iter().find(|gp| gp.name.name == g.name) {
                        None => diags.push(Diagnostic::error(
                            "E202",
                            g.span,
                            format!(
                            "`{}` is not `Pin`, `impl Trait`, or a generic parameter of fn `{}`",
                            g.name, f.name.name
                        ),
                        )),
                        Some(gp) => {
                            if let GenericBound::Unit(u) = &gp.bound {
                                diags.push(Diagnostic::error(
                                    "E205",
                                    g.span,
                                    format!(
                                        "generic parameter `{}` is unit-bound (`{}`) — only trait-bound parameters can type a value parameter",
                                        g.name,
                                        u.unit.type_name()
                                    ),
                                ));
                            }
                        }
                    }
                }
                FnParamTy::ImplTrait(traits, _) => {
                    for t in traits {
                        check_trait_ref(world, t, diags);
                    }
                }
            }
        }
    }
}

fn validate_generic_params(world: &World, generics: &[GenericParam], diags: &mut Diagnostics) {
    for g in generics {
        match &g.bound {
            GenericBound::Unit(u) => {
                if let Some((val, span)) = &g.default {
                    if val.unit != u.unit {
                        diags.push(
                            Diagnostic::error(
                                "E110",
                                *span,
                                format!(
                                    "default for `{}` has the wrong unit type: expected `{}`, found `{}`",
                                    g.name.name,
                                    u.unit.type_name(),
                                    val.unit.type_name()
                                ),
                            )
                            .with_primary_label(format!("this literal is `{}`", val.unit.type_name())),
                        );
                    }
                }
            }
            GenericBound::Traits(traits) => {
                for t in traits {
                    check_trait_ref(world, t, diags);
                }
                if let Some((_, span)) = &g.default {
                    diags.push(Diagnostic::error(
                        "E406",
                        *span,
                        format!(
                            "generic parameter `{}` is trait-bound — defaults are only valid on unit-type parameters (RFC-007)",
                            g.name.name
                        ),
                    ));
                }
            }
        }
    }
}

fn check_trait_ref(world: &World, t: &Ident, diags: &mut Diagnostics) {
    if world.traits.contains_key(&t.name) {
        return;
    }
    // A unit type name in trait-bound position parses as a unit bound, so
    // reaching here with a unit name is impossible; distinguish device/part.
    let diag = if world.devices.contains_key(&t.name) || world.parts.contains_key(&t.name) {
        Diagnostic::error(
            "E205",
            t.span,
            format!(
                "`{}` is not a trait (trait bounds must name traits)",
                t.name
            ),
        )
    } else if UnitType::from_type_name(&t.name).is_some() {
        // e.g. `impl Voltage + Capacitor` — nonsensical mix.
        Diagnostic::error(
            "E205",
            t.span,
            format!("`{}` is a unit type, not a trait", t.name),
        )
    } else {
        with_suggestion(
            world,
            &t.name,
            Diagnostic::error("E202", t.span, format!("unknown trait `{}`", t.name)),
        )
    };
    diags.push(diag);
}

fn index_impls(world: &mut World, diags: &mut Diagnostics) {
    for (idx, im) in world.impls.iter().enumerate() {
        let trait_ok = world.traits.contains_key(&im.trait_name.name);
        let device_ok = world.devices.contains_key(&im.device_name.name);
        if !trait_ok {
            let d = if world.devices.contains_key(&im.trait_name.name) {
                Diagnostic::error(
                    "E205",
                    im.trait_name.span,
                    format!("`{}` is a device, not a trait", im.trait_name.name),
                )
            } else {
                with_suggestion(
                    world,
                    &im.trait_name.name,
                    Diagnostic::error(
                        "E202",
                        im.trait_name.span,
                        format!("unknown trait `{}`", im.trait_name.name),
                    ),
                )
            };
            diags.push(d);
        }
        if !device_ok {
            let d = if world.traits.contains_key(&im.device_name.name) {
                Diagnostic::error(
                    "E205",
                    im.device_name.span,
                    format!(
                        "`{}` is a trait — `impl Trait for Device` implements a trait *for a device*",
                        im.device_name.name
                    ),
                )
            } else {
                with_suggestion(
                    world,
                    &im.device_name.name,
                    Diagnostic::error(
                        "E202",
                        im.device_name.span,
                        format!("unknown device `{}`", im.device_name.name),
                    ),
                )
            };
            diags.push(d);
        }
        if !(trait_ok && device_ok) {
            continue;
        }
        let key = (im.trait_name.name.clone(), im.device_name.name.clone());
        if let Some(&prev_idx) = world.impl_index.get(&key) {
            diags.push(
                Diagnostic::error(
                    "E303",
                    im.span,
                    format!(
                        "duplicate `impl {} for {}`",
                        im.trait_name.name, im.device_name.name
                    ),
                )
                .with_secondary(world.impls[prev_idx].span, "the earlier impl is here"),
            );
            continue;
        }
        world.impl_index.insert(key, idx);
    }
}

fn check_dup_names<'i>(
    names: impl Iterator<Item = &'i Ident>,
    what: &str,
    owner: &str,
    diags: &mut Diagnostics,
) {
    let mut seen: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
    for n in names {
        if let Some(prev) = seen.insert(n.name.as_str(), n.span) {
            diags.push(
                Diagnostic::error(
                    "E201",
                    n.span,
                    format!("duplicate {} `{}` on `{}`", what, n.name, owner),
                )
                .with_secondary(prev, "first declared here"),
            );
        }
    }
}

/// Attach RFC-016's closest-match help to an unknown-name diagnostic.
fn with_suggestion(world: &World, name: &str, d: Diagnostic) -> Diagnostic {
    match world.suggest(name) {
        Some(s) => d.with_help(format!("did you mean `{}`?", s)),
        None => d,
    }
}

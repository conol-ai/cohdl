//! Name resolution for the cohdl hardware-description language.
//!
//! This crate walks a parsed [`SourceFile`] AST, builds a [`SymbolTable`] of
//! all declarations, resolves `use` imports, and produces a
//! [`ResolvedSourceFile`] where every name reference carries a [`SymbolId`].
//!
//! Errors (undefined paths, duplicate definitions, visibility violations) are
//! collected with [`Span`] information rather than aborting on the first error.

pub mod typeck;

use std::collections::HashMap;

use cohdl_syntax::ast::{
    CallStmt, DesignBodyStmtKind, DesignDecl, DeviceDecl, Expr, ExprKind, FnBodyStmtKind, FnDecl,
    FnParamKind, GenericParamKind, Ident, InstStmt, NetStmt, PartDecl, Path, SourceFile, Span,
    TopLevelItem, TopLevelItemKind, TraitDecl, TypeAlias, TypeExpr, UseDecl,
};

// ── Symbol identifiers ─────────────────────────────────────────────────────

/// An opaque, unique identifier for a resolved symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

// ── Symbol kinds ────────────────────────────────────────────────────────────

/// The kind of declaration a symbol represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Trait,
    Device,
    Part,
    TypeAlias,
    Fn,
    Module,
    Design,
}

// ── Symbol ──────────────────────────────────────────────────────────────────

/// A resolved symbol in the symbol table.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Unique id.
    pub id: SymbolId,
    /// Simple name of the declaration.
    pub name: String,
    /// Fully-qualified path (e.g. `"power::decoupling"`).
    pub qualified_path: String,
    /// What kind of item this symbol is.
    pub kind: SymbolKind,
    /// Whether the symbol was declared `pub`.
    pub is_public: bool,
    /// The module path that owns this symbol (empty string for root).
    pub parent_module: String,
    /// Source location of the declaration.
    pub span: Span,
}

// ── Symbol table ────────────────────────────────────────────────────────────

/// Maps qualified paths to resolved declarations.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// All symbols, indexed by [`SymbolId`].
    symbols: Vec<Symbol>,
    /// Qualified-path → SymbolId lookup.
    by_path: HashMap<String, SymbolId>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            symbols: Vec::new(),
            by_path: HashMap::new(),
        }
    }

    /// Insert a new symbol. Returns `Err` with the existing symbol's span if a
    /// symbol at this qualified path already exists.
    fn insert(&mut self, symbol: Symbol) -> Result<SymbolId, Span> {
        if let Some(&existing) = self.by_path.get(&symbol.qualified_path) {
            return Err(self.symbols[existing.0 as usize].span);
        }
        let id = symbol.id;
        self.by_path.insert(symbol.qualified_path.clone(), id);
        self.symbols.push(symbol);
        Ok(id)
    }

    fn next_id(&self) -> SymbolId {
        SymbolId(self.symbols.len() as u32)
    }

    /// Look up a symbol by its fully-qualified path.
    pub fn lookup(&self, path: &str) -> Option<&Symbol> {
        self.by_path
            .get(path)
            .map(|id| &self.symbols[id.0 as usize])
    }

    /// Get a symbol by id.
    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0 as usize]
    }

    /// Iterate over all symbols.
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}

// ── Errors ──────────────────────────────────────────────────────────────────

/// A name-resolution error with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct SemaError {
    /// Human-readable description.
    pub message: String,
    /// Source location of the offending reference.
    pub span: Span,
}

impl SemaError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

// ── Resolved name reference ─────────────────────────────────────────────────

/// A name reference that has been resolved to a [`SymbolId`].
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedName {
    /// The original path segments (as written in source).
    pub path: Vec<String>,
    /// The resolved symbol.
    pub symbol_id: SymbolId,
    /// Source location.
    pub span: Span,
}

// ── Resolved source file ────────────────────────────────────────────────────

/// The result of name resolution: a symbol table plus all resolved references.
#[derive(Debug, Clone)]
pub struct ResolvedSourceFile {
    /// The global symbol table.
    pub symbols: SymbolTable,
    /// All resolved name references found in the file.
    pub resolved_names: Vec<ResolvedName>,
    /// All errors encountered during resolution (empty on success).
    pub errors: Vec<SemaError>,
}

// ── Import tracking ─────────────────────────────────────────────────────────

/// Tracks which names have been imported into a scope via `use` declarations.
/// Maps simple name → (qualified path it refers to, span of import).
type ImportMap = HashMap<String, (String, Span)>;

// ── Resolver ────────────────────────────────────────────────────────────────

/// Internal resolver state.
struct Resolver {
    table: SymbolTable,
    /// Per-module import maps. Key is the module path (empty string = root).
    imports: HashMap<String, ImportMap>,
    resolved: Vec<ResolvedName>,
    errors: Vec<SemaError>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            table: SymbolTable::new(),
            imports: HashMap::new(),
            resolved: Vec::new(),
            errors: Vec::new(),
        }
    }

    // ── Pass 1: collect declarations ────────────────────────────────────

    fn collect_items(&mut self, items: &[TopLevelItem], module_path: &str) {
        for item in items {
            let is_public = item.visibility.is_some();
            match &item.kind {
                TopLevelItemKind::Trait(t) => {
                    self.define(&t.name, SymbolKind::Trait, is_public, module_path);
                }
                TopLevelItemKind::Device(d) => {
                    self.define(&d.name, SymbolKind::Device, is_public, module_path);
                }
                TopLevelItemKind::Part(p) => {
                    self.define(&p.name, SymbolKind::Part, is_public, module_path);
                }
                TopLevelItemKind::TypeAlias(t) => {
                    self.define(&t.name, SymbolKind::TypeAlias, is_public, module_path);
                }
                TopLevelItemKind::Fn(f) => {
                    self.define(&f.name, SymbolKind::Fn, is_public, module_path);
                }
                TopLevelItemKind::Module(m) => {
                    self.define(&m.name, SymbolKind::Module, is_public, module_path);
                    let child_path = qualified(module_path, &m.name.name);
                    self.collect_items(&m.items, &child_path);
                }
                TopLevelItemKind::Design(d) => {
                    self.define(&d.name, SymbolKind::Design, is_public, module_path);
                }
                TopLevelItemKind::Use(_) | TopLevelItemKind::Mod(_) => {
                    // handled in pass 2
                }
            }
        }
    }

    fn define(&mut self, name: &Ident, kind: SymbolKind, is_public: bool, module_path: &str) {
        let qpath = qualified(module_path, &name.name);
        let id = self.table.next_id();
        let symbol = Symbol {
            id,
            name: name.name.clone(),
            qualified_path: qpath.clone(),
            kind,
            is_public,
            parent_module: module_path.to_string(),
            span: name.span,
        };
        if let Err(prev_span) = self.table.insert(symbol) {
            self.errors.push(SemaError::new(
                format!("duplicate definition of `{}`", qpath),
                name.span,
            ));
            // Also note the previous definition location
            self.errors.push(SemaError::new(
                format!("`{}` previously defined here", qpath),
                prev_span,
            ));
        }
    }

    // ── Pass 2: resolve use declarations ────────────────────────────────

    fn resolve_uses(&mut self, items: &[TopLevelItem], module_path: &str) {
        for item in items {
            match &item.kind {
                TopLevelItemKind::Use(u) => {
                    self.resolve_use_decl(u, module_path);
                }
                TopLevelItemKind::Module(m) => {
                    let child = qualified(module_path, &m.name.name);
                    self.resolve_uses(&m.items, &child);
                }
                _ => {}
            }
        }
    }

    fn resolve_use_decl(&mut self, use_decl: &UseDecl, current_module: &str) {
        let tree = &use_decl.tree;

        // Build the prefix path string, resolving relative to current module.
        let prefix_path = self.resolve_prefix_to_qualified(&tree.prefix, current_module);

        match &tree.group {
            None => {
                // `use a::b::c` — import the final segment
                if tree.prefix.is_empty() {
                    self.errors
                        .push(SemaError::new("empty use path", use_decl.span));
                    return;
                }
                let imported_name = tree.prefix.last().unwrap();
                self.import_single(
                    &prefix_path,
                    &imported_name.name,
                    current_module,
                    imported_name.span,
                );
            }
            Some(names) => {
                // `use a::b::{c, d}`
                for name in names {
                    let full = qualified(&prefix_path, &name.name);
                    self.import_single(&full, &name.name, current_module, name.span);
                }
            }
        }
    }

    /// Resolve a use-tree prefix to a fully-qualified path.
    /// Tries the path as-is first (absolute), then relative to current module.
    fn resolve_prefix_to_qualified(&self, segments: &[Ident], _current_module: &str) -> String {
        let path_str = segments
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        path_str
    }

    fn import_single(
        &mut self,
        full_path: &str,
        simple_name: &str,
        current_module: &str,
        span: Span,
    ) {
        // Check target exists
        if self.table.lookup(full_path).is_none() {
            self.errors.push(SemaError::new(
                format!("undefined path `{}`", full_path),
                span,
            ));
            return;
        }

        // Check visibility
        let sym = self.table.lookup(full_path).unwrap();
        if !sym.is_public && sym.parent_module != current_module {
            self.errors.push(SemaError::new(
                format!(
                    "`{}` is private and cannot be accessed from module `{}`",
                    full_path,
                    if current_module.is_empty() {
                        "<root>"
                    } else {
                        current_module
                    }
                ),
                span,
            ));
            return;
        }

        // Check for duplicate import
        let imports = self.imports.entry(current_module.to_string()).or_default();
        if let Some((prev_path, prev_span)) = imports.get(simple_name) {
            if prev_path != full_path {
                self.errors.push(SemaError::new(
                    format!(
                        "duplicate import of name `{}` (already imported as `{}`)",
                        simple_name, prev_path
                    ),
                    span,
                ));
                self.errors.push(SemaError::new(
                    format!("previous import of `{}` here", simple_name),
                    *prev_span,
                ));
            } else {
                self.errors.push(SemaError::new(
                    format!("duplicate import of `{}`", full_path),
                    span,
                ));
            }
            return;
        }

        imports.insert(simple_name.to_string(), (full_path.to_string(), span));
    }

    // ── Pass 3: resolve all name references in bodies ───────────────────

    fn resolve_references(&mut self, items: &[TopLevelItem], module_path: &str) {
        for item in items {
            match &item.kind {
                TopLevelItemKind::Trait(t) => self.resolve_trait(t, module_path),
                TopLevelItemKind::Device(d) => self.resolve_device(d, module_path),
                TopLevelItemKind::Part(p) => self.resolve_part(p, module_path),
                TopLevelItemKind::TypeAlias(t) => self.resolve_type_alias(t, module_path),
                TopLevelItemKind::Fn(f) => self.resolve_fn(f, module_path),
                TopLevelItemKind::Module(m) => {
                    let child = qualified(module_path, &m.name.name);
                    self.resolve_references(&m.items, &child);
                }
                TopLevelItemKind::Design(d) => self.resolve_design(d, module_path),
                TopLevelItemKind::Use(_) | TopLevelItemKind::Mod(_) => {}
            }
        }
    }

    fn resolve_trait(&mut self, t: &TraitDecl, module_path: &str) {
        if let Some(parents) = &t.parents {
            for bound in &parents.bounds {
                self.resolve_type_expr(bound, module_path);
            }
        }
    }

    fn resolve_device(&mut self, d: &DeviceDecl, module_path: &str) {
        if let Some(generics) = &d.generic_params {
            for p in &generics.params {
                match &p.kind {
                    GenericParamKind::Type(te) => self.resolve_type_expr(te, module_path),
                    GenericParamKind::ImplConstraint(tb) => {
                        for b in &tb.bounds {
                            self.resolve_type_expr(b, module_path);
                        }
                    }
                }
            }
        }
        if let Some(traits) = &d.impl_traits {
            for b in &traits.bounds {
                self.resolve_type_expr(b, module_path);
            }
        }
    }

    fn resolve_part(&mut self, p: &PartDecl, module_path: &str) {
        self.resolve_type_expr(&p.device_type, module_path);
    }

    fn resolve_type_alias(&mut self, t: &TypeAlias, module_path: &str) {
        if let Some(generics) = &t.generic_params {
            for p in &generics.params {
                match &p.kind {
                    GenericParamKind::Type(te) => self.resolve_type_expr(te, module_path),
                    GenericParamKind::ImplConstraint(tb) => {
                        for b in &tb.bounds {
                            self.resolve_type_expr(b, module_path);
                        }
                    }
                }
            }
        }
        self.resolve_type_expr(&t.target, module_path);
    }

    fn resolve_fn(&mut self, f: &FnDecl, module_path: &str) {
        if let Some(generics) = &f.generic_params {
            for p in &generics.params {
                match &p.kind {
                    GenericParamKind::Type(te) => self.resolve_type_expr(te, module_path),
                    GenericParamKind::ImplConstraint(tb) => {
                        for b in &tb.bounds {
                            self.resolve_type_expr(b, module_path);
                        }
                    }
                }
            }
        }
        for param in &f.params {
            match &param.kind {
                FnParamKind::Type(te) => self.resolve_type_expr(te, module_path),
                FnParamKind::ImplConstraint(tb) => {
                    for b in &tb.bounds {
                        self.resolve_type_expr(b, module_path);
                    }
                }
            }
        }
        for stmt in &f.body {
            match &stmt.kind {
                FnBodyStmtKind::Inst(inst) => self.resolve_inst(inst, module_path),
                FnBodyStmtKind::Net(net) => self.resolve_net(net, module_path),
                FnBodyStmtKind::Call(call) => self.resolve_call(call, module_path),
            }
        }
    }

    fn resolve_design(&mut self, d: &DesignDecl, module_path: &str) {
        for stmt in &d.body {
            match &stmt.kind {
                DesignBodyStmtKind::Inst(inst) => self.resolve_inst(inst, module_path),
                DesignBodyStmtKind::Net(net) => self.resolve_net(net, module_path),
                DesignBodyStmtKind::Call(call) => self.resolve_call(call, module_path),
            }
        }
    }

    fn resolve_inst(&mut self, inst: &InstStmt, module_path: &str) {
        self.resolve_type_expr(&inst.ty, module_path);
    }

    fn resolve_net(&mut self, _net: &NetStmt, _module_path: &str) {
        // Net endpoints refer to local instances/pins, not global symbols.
        // Full instance resolution is out of scope for name resolution.
    }

    fn resolve_call(&mut self, call: &CallStmt, module_path: &str) {
        self.resolve_path(&call.path, module_path);
        for arg in &call.args {
            self.resolve_expr(&arg.value, module_path);
        }
    }

    fn resolve_expr(&mut self, expr: &Expr, module_path: &str) {
        match &expr.kind {
            ExprKind::Type(te) => self.resolve_type_expr(te, module_path),
            ExprKind::FnCall(fc) => {
                self.resolve_path(&fc.path, module_path);
                for arg in &fc.args {
                    self.resolve_expr(arg, module_path);
                }
            }
            ExprKind::Binary(b) => {
                self.resolve_expr(&b.lhs, module_path);
                self.resolve_expr(&b.rhs, module_path);
            }
            ExprKind::Unary(u) => {
                self.resolve_expr(&u.operand, module_path);
            }
            ExprKind::Paren(e) => self.resolve_expr(e, module_path),
            ExprKind::DotPath(_)
            | ExprKind::EngineeringNumber(_)
            | ExprKind::Integer(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_) => {}
        }
    }

    fn resolve_type_expr(&mut self, te: &TypeExpr, module_path: &str) {
        self.resolve_path(&te.path, module_path);
        if let Some(args) = &te.generic_args {
            for arg in &args.args {
                self.resolve_expr(&arg.value, module_path);
            }
        }
    }

    /// Resolve a path to a symbol, checking visibility.
    fn resolve_path(&mut self, path: &Path, module_path: &str) {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();

        // Skip built-in type names that are not user-defined symbols.
        if segments.len() == 1 && is_builtin(segments[0]) {
            return;
        }

        // Try to find the symbol:
        // 1. As a fully-qualified path (joined with ::)
        // 2. Relative to current module
        // 3. Via imports in current module
        // 4. As a single-segment name in the root scope

        let full = segments.join("::");

        if let Some(sym) = self.table.lookup(&full) {
            let sym_id = sym.id;
            let sym_public = sym.is_public;
            let sym_parent = sym.parent_module.clone();
            if !sym_public
                && sym_parent != module_path
                && !module_path.starts_with(&format!("{}::", sym_parent))
            {
                self.errors.push(SemaError::new(
                    format!(
                        "`{}` is private and cannot be accessed from `{}`",
                        full,
                        if module_path.is_empty() {
                            "<root>"
                        } else {
                            module_path
                        }
                    ),
                    path.span,
                ));
                return;
            }
            self.resolved.push(ResolvedName {
                path: segments.iter().map(|s| s.to_string()).collect(),
                symbol_id: sym_id,
                span: path.span,
            });
            return;
        }

        // Try relative to current module
        if !module_path.is_empty() {
            let relative = qualified(module_path, &full);
            if let Some(sym) = self.table.lookup(&relative) {
                let sym_id = sym.id;
                self.resolved.push(ResolvedName {
                    path: segments.iter().map(|s| s.to_string()).collect(),
                    symbol_id: sym_id,
                    span: path.span,
                });
                return;
            }
        }

        // Try imports (only for single-segment names)
        if segments.len() == 1 {
            if let Some(imports) = self.imports.get(module_path) {
                if let Some((qualified_path, _)) = imports.get(segments[0]) {
                    if let Some(sym) = self.table.lookup(qualified_path) {
                        let sym_id = sym.id;
                        self.resolved.push(ResolvedName {
                            path: segments.iter().map(|s| s.to_string()).collect(),
                            symbol_id: sym_id,
                            span: path.span,
                        });
                        return;
                    }
                }
            }
        }

        self.errors.push(SemaError::new(
            format!("undefined symbol `{}`", full),
            path.span,
        ));
    }
}

/// Built-in type names that don't need resolution.
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "Pin"
            | "Net"
            | "Farads"
            | "Voltage"
            | "Ohms"
            | "Amps"
            | "Package"
            | "bool"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "f32"
            | "f64"
            | "String"
    )
}

/// Join a module path and a name into a qualified path.
fn qualified(module_path: &str, name: &str) -> String {
    if module_path.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", module_path, name)
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Run name resolution on a parsed source file.
///
/// Returns a [`ResolvedSourceFile`] containing the symbol table, all resolved
/// name references, and any errors encountered. Errors are collected
/// (resolution does not abort on the first error).
pub fn resolve(source: &SourceFile) -> ResolvedSourceFile {
    let mut resolver = Resolver::new();

    // Pass 1: collect all declarations into the symbol table.
    resolver.collect_items(&source.items, "");

    // Pass 2: resolve use declarations and build import maps.
    resolver.resolve_uses(&source.items, "");

    // Pass 3: resolve all name references in bodies.
    resolver.resolve_references(&source.items, "");

    ResolvedSourceFile {
        symbols: resolver.table,
        resolved_names: resolver.resolved,
        errors: resolver.errors,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl_parser::parse_source_file;

    /// Helper: parse source text and run name resolution.
    fn resolve_src(src: &str) -> ResolvedSourceFile {
        let sf = parse_source_file(src).expect("parse failed");
        resolve(&sf)
    }

    /// Helper: check that the error list contains a message matching the given substring.
    fn has_error(resolved: &ResolvedSourceFile, substr: &str) -> bool {
        resolved.errors.iter().any(|e| e.message.contains(substr))
    }

    // ── Basic declaration collection ────────────────────────────────────

    #[test]
    fn collects_top_level_declarations() {
        let src = r#"
            trait TwoTerminal {
                pins { A: Pin, B: Pin }
            }
            device MLCC: impl TwoTerminal {
                pins { A: 1, B: 2 }
            }
            fn helper(vdd: Net) {}
            type SmallCap = MLCC
        "#;
        let resolved = resolve_src(src);
        assert!(resolved.symbols.lookup("TwoTerminal").is_some());
        assert!(resolved.symbols.lookup("MLCC").is_some());
        assert!(resolved.symbols.lookup("helper").is_some());
        assert!(resolved.symbols.lookup("SmallCap").is_some());
    }

    #[test]
    fn collects_module_scoped_declarations() {
        let src = r#"
            module power {
                pub fn decoupling(vdd: Net) {}
                fn internal(gnd: Net) {}
            }
        "#;
        let resolved = resolve_src(src);
        assert!(resolved.symbols.lookup("power").is_some());
        assert!(resolved.symbols.lookup("power::decoupling").is_some());
        assert!(resolved.symbols.lookup("power::internal").is_some());
        assert!(
            resolved
                .symbols
                .lookup("power::decoupling")
                .unwrap()
                .is_public
        );
        assert!(
            !resolved
                .symbols
                .lookup("power::internal")
                .unwrap()
                .is_public
        );
    }

    // ── Missing symbol ──────────────────────────────────────────────────

    #[test]
    fn error_on_undefined_symbol() {
        let src = r#"
            design Board {
                inst c: NonExistent
            }
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "undefined symbol `NonExistent`"));
    }

    #[test]
    fn error_on_undefined_use_path() {
        let src = r#"
            use phantom::ghost
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "undefined path `phantom::ghost`"));
    }

    // ── Private access ──────────────────────────────────────────────────

    #[test]
    fn error_on_private_access_via_use() {
        let src = r#"
            module power {
                fn internal(vdd: Net) {}
            }
            use power::internal
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "private"));
        assert!(has_error(&resolved, "power::internal"));
    }

    #[test]
    fn error_on_private_access_via_path() {
        let src = r#"
            module power {
                fn secret(vdd: Net) {}
            }
            design Board {
                power::secret(vdd: Net)
            }
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "private"));
        assert!(has_error(&resolved, "power::secret"));
    }

    #[test]
    fn public_access_succeeds() {
        let src = r#"
            module power {
                pub fn decoupling(vdd: Net) {}
            }
            use power::decoupling
        "#;
        let resolved = resolve_src(src);
        // No visibility error
        assert!(!has_error(&resolved, "private"));
    }

    // ── Duplicate definition ────────────────────────────────────────────

    #[test]
    fn error_on_duplicate_definition() {
        let src = r#"
            trait Foo {
                pins { A: Pin }
            }
            trait Foo {
                pins { B: Pin }
            }
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "duplicate definition of `Foo`"));
    }

    // ── Duplicate import ────────────────────────────────────────────────

    #[test]
    fn error_on_duplicate_import_same_path() {
        let src = r#"
            module lib {
                pub trait Res {}
            }
            use lib::Res
            use lib::Res
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "duplicate import"));
    }

    #[test]
    fn error_on_duplicate_import_different_paths() {
        let src = r#"
            module a {
                pub trait Foo {}
            }
            module b {
                pub trait Foo {}
            }
            use a::Foo
            use b::Foo
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "duplicate import of name `Foo`"));
    }

    // ── Cross-module use ────────────────────────────────────────────────

    #[test]
    fn cross_module_use_resolves() {
        let src = r#"
            module passives {
                pub trait Capacitor {
                    pins { A: Pin, B: Pin }
                }
                pub device MLCC: impl Capacitor {
                    pins { A: 1, B: 2 }
                }
            }
            use passives::Capacitor
            use passives::MLCC
            design Board {
                inst c: MLCC
            }
        "#;
        let resolved = resolve_src(src);
        // The use declarations should succeed (no errors about the imports themselves)
        assert!(!has_error(
            &resolved,
            "undefined path `passives::Capacitor`"
        ));
        assert!(!has_error(&resolved, "undefined path `passives::MLCC`"));
        // The inst in Board should resolve MLCC via imports
        assert!(!has_error(&resolved, "undefined symbol `MLCC`"));
    }

    #[test]
    fn grouped_use_resolves() {
        let src = r#"
            module passives {
                pub trait Resistor {}
                pub trait Capacitor {
                    pins { A: Pin, B: Pin }
                }
            }
            use passives::{Resistor, Capacitor}
        "#;
        let resolved = resolve_src(src);
        assert!(!has_error(&resolved, "undefined path"));
        assert!(!has_error(&resolved, "private"));
    }

    // ── Scoped paths ────────────────────────────────────────────────────

    #[test]
    fn scoped_path_resolves_directly() {
        let src = r#"
            module power {
                pub fn decoupling(vdd: Net) {}
            }
            design Board {
                power::decoupling(vdd: Net)
            }
        "#;
        let resolved = resolve_src(src);
        assert!(!has_error(&resolved, "undefined"));
    }

    // ── Errors carry span info ──────────────────────────────────────────

    #[test]
    fn errors_carry_span() {
        let src = "use missing::symbol";
        let resolved = resolve_src(src);
        assert!(!resolved.errors.is_empty());
        let err = &resolved.errors[0];
        assert!(err.span.start < err.span.end);
    }

    // ── Multiple errors collected ───────────────────────────────────────

    #[test]
    fn collects_multiple_errors() {
        let src = r#"
            use ghost::one
            use ghost::two
            design Board {
                inst x: Phantom
            }
        "#;
        let resolved = resolve_src(src);
        // Should have at least 3 errors: two undefined use paths + one undefined inst type
        assert!(resolved.errors.len() >= 3);
    }

    // ── ResolvedName carries SymbolId ───────────────────────────────────

    #[test]
    fn resolved_names_carry_symbol_id() {
        let src = r#"
            trait Cap {
                pins { A: Pin, B: Pin }
            }
            device MLCC: impl Cap {
                pins { A: 1, B: 2 }
            }
        "#;
        let resolved = resolve_src(src);
        // The `impl Cap` reference in MLCC should produce a resolved name
        assert!(!resolved.resolved_names.is_empty());
        let cap_ref = resolved
            .resolved_names
            .iter()
            .find(|r| r.path == vec!["Cap"])
            .expect("should have resolved reference to Cap");
        let cap_sym = resolved.symbols.lookup("Cap").unwrap();
        assert_eq!(cap_ref.symbol_id, cap_sym.id);
    }

    // ── Nested modules ──────────────────────────────────────────────────

    #[test]
    fn nested_module_resolution() {
        let src = r#"
            module outer {
                pub module inner {
                    pub fn deep(v: Net) {}
                }
            }
            design Board {
                outer::inner::deep(v: Net)
            }
        "#;
        let resolved = resolve_src(src);
        assert!(resolved.symbols.lookup("outer::inner::deep").is_some());
        assert!(!has_error(&resolved, "undefined"));
    }

    #[test]
    fn private_nested_module_access_error() {
        let src = r#"
            module outer {
                module inner {
                    pub fn deep(v: Net) {}
                }
            }
            use outer::inner
        "#;
        let resolved = resolve_src(src);
        assert!(has_error(&resolved, "private"));
    }
}

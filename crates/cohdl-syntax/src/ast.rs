//! Typed AST for the cohdl hardware-description language.
//!
//! Every node carries a [`Span`] for source-location tracking and error
//! reporting.  The root of any parsed file is [`SourceFile`].

// ── Span ────────────────────────────────────────────────────────────────────

/// Byte-range into the original source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

// ── Identifier ──────────────────────────────────────────────────────────────

/// A single identifier token (e.g. `mcu`, `VDD_IO`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    /// The identifier text.
    pub name: String,
    /// Source location.
    pub span: Span,
}

// ── Paths ───────────────────────────────────────────────────────────────────

/// A possibly-qualified path such as `power::decoupling` or just `ident`.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// Path segments (at least one).
    pub segments: Vec<Ident>,
    /// Source location spanning the whole path.
    pub span: Span,
}

/// A dot-separated member access path such as `mcu.VDD_IO` or `self.spec.voltage_rating`.
#[derive(Debug, Clone, PartialEq)]
pub struct DotPath {
    /// The root path (may be scoped, e.g. `stm32::mcu`).
    pub root: Path,
    /// The dot-separated field names after the root.
    pub fields: Vec<Ident>,
    /// Source location.
    pub span: Span,
}

// ── Root node ───────────────────────────────────────────────────────────────

/// Root AST node for a single `.hdl` source file.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    /// All top-level items in declaration order.
    pub items: Vec<TopLevelItem>,
    /// Source location spanning the entire file.
    pub span: Span,
}

// ── Top-level item ──────────────────────────────────────────────────────────

/// A single top-level declaration, possibly prefixed with attributes and
/// a visibility modifier.
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelItem {
    /// Attributes applied to this item (e.g. `#[designator("U1")]`).
    pub attributes: Vec<Attribute>,
    /// Whether this item is `pub`.
    pub visibility: Option<Visibility>,
    /// The kind of item.
    pub kind: TopLevelItemKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for [`TopLevelItem`].
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelItemKind {
    /// `trait Foo { ... }`
    Trait(TraitDecl),
    /// `device Foo<...> { ... }`
    Device(DeviceDecl),
    /// `part foo: Type { ... }`
    Part(PartDecl),
    /// `type Foo = Bar<...>`
    TypeAlias(TypeAlias),
    /// `fn foo(...) { ... }`
    Fn(FnDecl),
    /// `module foo { ... }`
    Module(ModuleDecl),
    /// `design Foo { ... }`
    Design(DesignDecl),
    /// `use path::to::item`
    Use(UseDecl),
    /// `mod foo` (file-level module reference)
    Mod(ModDecl),
    /// `footprint_alias Name { kicad: "...", lceda: "..." }`
    FootprintAlias(FootprintAliasDecl),
}

// ── Visibility ──────────────────────────────────────────────────────────────

/// Visibility modifier (`pub`).
#[derive(Debug, Clone, PartialEq)]
pub struct Visibility {
    /// Source location of the `pub` keyword.
    pub span: Span,
}

// ── Attributes ──────────────────────────────────────────────────────────────

/// An attribute such as `#[designator("U1")]` or `#[allow(unconnected_pin)]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute name (e.g. `designator`, `allow`).
    pub name: Ident,
    /// Optional arguments inside the parentheses.
    pub args: Option<AttributeArgs>,
    /// Source location including the `#[` and `]`.
    pub span: Span,
}

/// Arguments to an attribute.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArgs {
    /// String literal arguments: `#[designator("U1")]`.
    Strings(Vec<StringLiteral>),
    /// Identifier arguments: `#[allow(unconnected_pin, voltage_derating)]`.
    Idents(Vec<Ident>),
    /// Key-value pairs: `#[footprint(kicad: "...", lceda: "...")]`.
    KeyValues(Vec<KeyValueArg>),
}

/// A single key-value argument in an attribute, e.g. `kicad: "..."`.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueArg {
    /// The key identifier.
    pub key: Ident,
    /// The string value.
    pub value: StringLiteral,
    /// Source location.
    pub span: Span,
}

/// A quoted string literal.
#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    /// The string value (without surrounding quotes).
    pub value: String,
    /// Source location.
    pub span: Span,
}

// ── Use / Mod declarations ──────────────────────────────────────────────────

/// A `use` import declaration such as `use power::decoupling` or
/// `use passives::{res_10k, mlcc_100nF}`.
#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    /// The import path, possibly ending with a group.
    pub tree: UseTree,
    /// Source location.
    pub span: Span,
}

/// The tree inside a `use` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct UseTree {
    /// Leading path segments before a possible group.
    pub prefix: Vec<Ident>,
    /// If the path ends with `::{a, b, c}`, the grouped names.
    /// `None` means a simple `use a::b` import.
    pub group: Option<Vec<Ident>>,
    /// Source location.
    pub span: Span,
}

/// A bare `mod foo` declaration (references an external file).
#[derive(Debug, Clone, PartialEq)]
pub struct ModDecl {
    /// Module name.
    pub name: Ident,
    /// Source location.
    pub span: Span,
}

// ── Generics ────────────────────────────────────────────────────────────────

/// A list of generic parameters enclosed in `< >` at a definition site.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParams {
    /// The individual parameters.
    pub params: Vec<GenericParam>,
    /// Source location.
    pub span: Span,
}

/// A single generic parameter such as `C: Farads`, `pkg: Package = C0402`,
/// or `P: impl Capacitor + Polarized`.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    /// Parameter name.
    pub name: Ident,
    /// The kind / constraint of this parameter.
    pub kind: GenericParamKind,
    /// Optional default value expression.
    pub default: Option<Expr>,
    /// Source location.
    pub span: Span,
}

/// The constraint on a generic parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParamKind {
    /// A type or built-in kind such as `Farads`, `Voltage`, `Ohms`,
    /// `Package`, `Net`, or any user-defined type.
    Type(TypeExpr),
    /// An `impl Trait` constraint (possibly multi-trait via `+`).
    ImplConstraint(TraitBound),
}

/// A list of generic arguments enclosed in `< >` at a usage site.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgs {
    /// The individual named arguments.
    pub args: Vec<GenericArg>,
    /// Source location.
    pub span: Span,
}

/// A single named generic argument such as `C: 100nF` or `pkg: C0402`.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericArg {
    /// Argument name.
    pub name: Ident,
    /// Value expression.
    pub value: Expr,
    /// Source location.
    pub span: Span,
}

// ── Type expressions ────────────────────────────────────────────────────────

/// A type expression such as `MLCC<C: 100nF, V: 10V, pkg: C0402>` or just `Pin`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr {
    /// The type path (may be scoped).
    pub path: Path,
    /// Optional generic arguments.
    pub generic_args: Option<GenericArgs>,
    /// Source location.
    pub span: Span,
}

/// One or more trait bounds separated by `+`.
#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    /// Each bound is a type expression (trait name, possibly with generics).
    pub bounds: Vec<TypeExpr>,
    /// Source location.
    pub span: Span,
}

// ── Expressions ─────────────────────────────────────────────────────────────

/// An expression used in spec values, generic defaults, rule assertions, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    /// The expression kind.
    pub kind: ExprKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for [`Expr`].
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// An engineering-notation number literal such as `100nF`, `10k`, `3.3V`, `48`.
    EngineeringNumber(EngineeringNumber),
    /// An integer literal.
    Integer(u64),
    /// A string literal.
    String(StringLiteral),
    /// A boolean literal (`true` / `false`).
    Bool(bool),
    /// A type expression used as a value (e.g. an identifier, a generic type).
    Type(TypeExpr),
    /// A dot-path member access (e.g. `self.spec.voltage_rating`).
    DotPath(DotPath),
    /// A function call expression (e.g. `net_voltage(self.A, self.B)`).
    FnCall(FnCallExpr),
    /// A binary operation (e.g. `a + b`, `x <= y * 0.8`).
    Binary(BinaryExpr),
    /// A unary operation (e.g. `-x`, `!flag`).
    Unary(UnaryExpr),
    /// A parenthesized sub-expression.
    Paren(Box<Expr>),
}

/// An engineering-notation number such as `100nF`, `10k`, `3.3V`, `22R`.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineeringNumber {
    /// The raw numeric text (e.g. `"100"`, `"3.3"`).
    pub number: String,
    /// Optional suffix (e.g. `"nF"`, `"k"`, `"V"`, `"R"`).
    pub suffix: Option<String>,
    /// Source location.
    pub span: Span,
}

/// A binary expression such as `a <= b * 0.8`.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    /// Left-hand side.
    pub lhs: Box<Expr>,
    /// The operator.
    pub op: BinaryOp,
    /// Right-hand side.
    pub rhs: Box<Expr>,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// `==`
    Eq,
    /// `!=`
    Ne,
}

/// A unary expression such as `-x` or `!flag`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    /// The operator.
    pub op: UnaryOp,
    /// The operand.
    pub operand: Box<Expr>,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-`
    Neg,
    /// `!`
    Not,
}

/// A function call expression such as `net_voltage(self.A, self.B)`.
#[derive(Debug, Clone, PartialEq)]
pub struct FnCallExpr {
    /// The function path.
    pub path: Path,
    /// Positional arguments.
    pub args: Vec<Expr>,
    /// Source location.
    pub span: Span,
}

// ── Interpolated strings (rule messages) ────────────────────────────────────

/// An interpolated string such as `"voltage {v}V exceeds {max}V"`.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedString {
    /// The raw string content (with `{expr}` fragments unprocessed).
    pub raw: String,
    /// Source location.
    pub span: Span,
}

// ── Trait declaration ───────────────────────────────────────────────────────

/// A `trait` declaration defining an electrical-behaviour contract.
///
/// ```hdl
/// trait Capacitor: TwoTerminal {
///   pins { A: Pin, B: Pin }
///   spec { capacitance: Farads, voltage_rating: Voltage }
///   designator_prefix: "C"
///   rule voltage_exceed(level: Error) { ... }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    /// Trait name.
    pub name: Ident,
    /// Optional parent traits (super-traits).
    pub parents: Option<TraitBound>,
    /// Body items in declaration order.
    pub body: Vec<TraitBodyItem>,
    /// Source location.
    pub span: Span,
}

/// An item inside a trait body.
#[derive(Debug, Clone, PartialEq)]
pub enum TraitBodyItem {
    /// A `pins { ... }` block.
    Pins(PinsBlock),
    /// A `spec { ... }` block.
    Spec(SpecBlock),
    /// A `rule name(level: ...) { ... }` block.
    Rule(RuleBlock),
    /// `designator_prefix: "C"`.
    DesignatorPrefix(DesignatorPrefix),
}

/// A `designator_prefix: "C"` declaration inside a trait.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignatorPrefix {
    /// The prefix string (e.g. `"C"`, `"R"`, `"U"`).
    pub prefix: StringLiteral,
    /// Source location.
    pub span: Span,
}

// ── Footprint alias ─────────────────────────────────────────────────────────

/// A `footprint_alias` declaration mapping backend names to footprint strings.
///
/// ```hdl
/// footprint_alias LQFP64_10x10 {
///     kicad: "Package_QFP:LQFP-64_10x10mm_P0.5mm"
///     lceda: "LQFP-64_L10.0-W10.0-P0.50-LS12.0-BL"
///     default: "LQFP-64"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FootprintAliasDecl {
    /// Alias name.
    pub name: Ident,
    /// Backend → footprint string mappings.
    pub entries: Vec<FootprintMapEntry>,
    /// Source location.
    pub span: Span,
}

/// A single backend → footprint string mapping entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FootprintMapEntry {
    /// Backend name (e.g. `"kicad"`, `"lceda"`, `"default"`).
    pub backend: Ident,
    /// Footprint string value.
    pub value: StringLiteral,
    /// Source location.
    pub span: Span,
}

// ── Pins block ──────────────────────────────────────────────────────────────

/// A `pins` block, optionally qualified by a package name.
///
/// ```hdl
/// pins[LQFP48] {
///     VDD_CORE: 1
///     GND: [8, 23, 35, 47]
///     pin_bus!(PA, 10, 8)
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PinsBlock {
    /// Optional package qualifier (e.g. `LQFP48`).
    pub qualifier: Option<Ident>,
    /// Pin entries in declaration order.
    pub entries: Vec<PinEntry>,
    /// Source location.
    pub span: Span,
}

/// A single entry inside a `pins` block.
#[derive(Debug, Clone, PartialEq)]
pub struct PinEntry {
    /// The kind of pin entry.
    pub kind: PinEntryKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for [`PinEntry`].
#[derive(Debug, Clone, PartialEq)]
pub enum PinEntryKind {
    /// A single pin mapping: `VDD_CORE: 1` or `VDD_CORE: A1` (BGA).
    Single {
        /// Pin name.
        name: Ident,
        /// Pin number/identifier (e.g. `"1"`, `"A1"`).
        number: String,
    },
    /// A pin mapped to a list of numbers: `GND: [8, 23, 35, 47]`.
    List {
        /// Pin name.
        name: Ident,
        /// Pin numbers/identifiers.
        numbers: Vec<String>,
    },
    /// A pin mapped to a contiguous range: `DATA: [0..7]`.
    Range {
        /// Pin name.
        name: Ident,
        /// Inclusive start pin number.
        start: u64,
        /// Inclusive end pin number.
        end: u64,
    },
    /// A pin-bus macro invocation: `pin_bus!(PA, 10, 8)`.
    BusMacro {
        /// Bus name prefix.
        name: Ident,
        /// Starting pin number.
        start_pin: u64,
        /// Number of pins in the bus.
        count: u64,
    },
    /// A typed pin declaration (in trait pins): `A: Pin`.
    Typed {
        /// Pin name.
        name: Ident,
        /// Pin type (e.g. `Pin`).
        ty: Ident,
    },
}

// ── Spec block ──────────────────────────────────────────────────────────────

/// A `spec { ... }` block declaring electrical specifications,
/// optionally qualified by a package variant: `spec[LQFP48] { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecBlock {
    /// Optional package variant qualifier (e.g. `LQFP48` in `spec[LQFP48] { ... }`).
    pub qualifier: Option<Ident>,
    /// Spec entries in declaration order.
    pub entries: Vec<SpecEntry>,
    /// Source location.
    pub span: Span,
}

/// A single entry in a `spec` block, e.g. `capacitance: Farads` or
/// `capacitance: C`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecEntry {
    /// Spec field name.
    pub name: Ident,
    /// Value (expression or inline footprint map).
    pub value: SpecEntryValue,
    /// Source location.
    pub span: Span,
}

/// The value of a spec entry.
#[derive(Debug, Clone, PartialEq)]
pub enum SpecEntryValue {
    /// A normal expression value (e.g. `capacitance: C`, `voltage_rating: 3.3V`).
    Expr(Expr),
    /// An inline backend map (only valid for `footprint`).
    /// `footprint { kicad: "...", lceda: "...", default: "..." }`
    FootprintMap(Vec<FootprintMapEntry>),
}

// ── Rule block ──────────────────────────────────────────────────────────────

/// A DRC rule block inside a trait or device.
///
/// ```hdl
/// rule voltage_exceed(level: Error) {
///   assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
///   message: "..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBlock {
    /// Rule name.
    pub name: Ident,
    /// Severity level (`Error` or `Warning`).
    pub level: RuleLevel,
    /// The assertion expression.
    pub assertion: Expr,
    /// The diagnostic message (may contain interpolation).
    pub message: InterpolatedString,
    /// Source location.
    pub span: Span,
}

/// Severity level for a DRC rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleLevel {
    /// Blocks output.
    Error,
    /// Non-blocking diagnostic.
    Warning,
}

// ── Device declaration ──────────────────────────────────────────────────────

/// A `device` declaration with optional generics and trait implementations.
///
/// ```hdl
/// device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
///   package: pkg
///   pins { A: 1, B: 2 }
///   spec { capacitance: C, voltage_rating: V }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceDecl {
    /// Device name.
    pub name: Ident,
    /// Optional generic parameters.
    pub generic_params: Option<GenericParams>,
    /// Optional `impl Trait + Trait` clause.
    pub impl_traits: Option<TraitBound>,
    /// Body items in declaration order.
    pub body: Vec<DeviceBodyItem>,
    /// Source location.
    pub span: Span,
}

/// An item inside a device body.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceBodyItem {
    /// `package: pkg` declaration.
    Package(PackageDecl),
    /// A `pins { ... }` or `pins[QUALIFIER] { ... }` block.
    Pins(PinsBlock),
    /// A `spec { ... }` block.
    Spec(SpecBlock),
    /// A DRC `rule` block.
    Rule(RuleBlock),
}

/// A `package: <path>` declaration inside a device.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageDecl {
    /// The package expression (identifier or scoped path).
    pub path: Path,
    /// Source location.
    pub span: Span,
}

// ── Part declaration ────────────────────────────────────────────────────────

/// A `part` declaration binding a generic device to a real MPN with AVL entries.
///
/// ```hdl
/// part mlcc_100nF_0402: MLCC<C: 100nF, V: 10V, pkg: C0402> {
///   primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
///   alt     { mfr: "Murata",  mpn: "GRM155R61C104KA88D" }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PartDecl {
    /// Part name.
    pub name: Ident,
    /// The base device type with generic arguments.
    pub device_type: TypeExpr,
    /// AVL (Approved Vendor List) entries.
    pub avl_entries: Vec<AvlEntry>,
    /// Source location.
    pub span: Span,
}

/// An AVL entry (primary or alternate) inside a part declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct AvlEntry {
    /// Whether this is the `primary` or an `alt` source.
    pub kind: AvlKind,
    /// Fields inside the entry (e.g. `mfr`, `mpn`).
    pub fields: Vec<AvlField>,
    /// Source location.
    pub span: Span,
}

/// Discriminant for primary vs. alternate AVL entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvlKind {
    /// The primary (preferred) vendor source.
    Primary,
    /// An alternate vendor source.
    Alt,
}

/// A single field inside an AVL entry, e.g. `mfr: "Samsung"`.
#[derive(Debug, Clone, PartialEq)]
pub struct AvlField {
    /// Field name (e.g. `mfr`, `mpn`).
    pub name: Ident,
    /// String value.
    pub value: StringLiteral,
    /// Source location.
    pub span: Span,
}

// ── Type alias ──────────────────────────────────────────────────────────────

/// A `type` alias declaration.
///
/// ```hdl
/// type SmallCap<C: Farads> = MLCC<C: C, V: 10V, pkg: C0402>
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    /// Alias name.
    pub name: Ident,
    /// Optional generic parameters on the alias itself.
    pub generic_params: Option<GenericParams>,
    /// The aliased type.
    pub target: TypeExpr,
    /// Source location.
    pub span: Span,
}

// ── Function declaration ────────────────────────────────────────────────────

/// A `fn` declaration defining a reusable sub-circuit template.
///
/// ```hdl
/// fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
///   inst c: cap
///   net vdd: c.A
///   net gnd: c.B
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    /// Function name.
    pub name: Ident,
    /// Optional generic parameters.
    pub generic_params: Option<GenericParams>,
    /// Named parameters.
    pub params: Vec<FnParam>,
    /// Function body statements.
    pub body: Vec<FnBodyStmt>,
    /// Source location.
    pub span: Span,
}

/// A single named parameter of a function.
#[derive(Debug, Clone, PartialEq)]
pub struct FnParam {
    /// Parameter name.
    pub name: Ident,
    /// Parameter type or `impl Trait` constraint.
    pub kind: FnParamKind,
    /// Source location.
    pub span: Span,
}

/// The type/constraint of a function parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum FnParamKind {
    /// A concrete type expression (e.g. `Net`, `Package`).
    Type(TypeExpr),
    /// An `impl Trait` constraint (e.g. `impl Capacitor + Polarized`).
    ImplConstraint(TraitBound),
}

/// A statement inside a function body.
#[derive(Debug, Clone, PartialEq)]
pub struct FnBodyStmt {
    /// Attributes on this statement.
    pub attributes: Vec<Attribute>,
    /// The statement kind.
    pub kind: FnBodyStmtKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for function body statements.
#[derive(Debug, Clone, PartialEq)]
pub enum FnBodyStmtKind {
    /// An `inst` statement.
    Inst(InstStmt),
    /// A `net` statement.
    Net(NetStmt),
    /// A function call statement.
    Call(CallStmt),
}

// ── Module declaration ──────────────────────────────────────────────────────

/// A `module` declaration grouping items under a namespace.
///
/// ```hdl
/// module power {
///   pub fn decoupling(...) { ... }
///   fn _internal_helper(...) { ... }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    /// Module name.
    pub name: Ident,
    /// Items declared inside the module.
    pub items: Vec<TopLevelItem>,
    /// Source location.
    pub span: Span,
}

// ── Design declaration ──────────────────────────────────────────────────────

/// A `design` declaration representing a top-level board or assembly.
///
/// ```hdl
/// design MainBoard {
///   inst mcu: STM32F103<pkg: LQFP64>
///   decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DesignDecl {
    /// Design name.
    pub name: Ident,
    /// Body statements.
    pub body: Vec<DesignBodyStmt>,
    /// Source location.
    pub span: Span,
}

/// A statement inside a design body.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignBodyStmt {
    /// Attributes on this statement.
    pub attributes: Vec<Attribute>,
    /// The statement kind.
    pub kind: DesignBodyStmtKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for design body statements.
#[derive(Debug, Clone, PartialEq)]
pub enum DesignBodyStmtKind {
    /// An `inst` statement.
    Inst(InstStmt),
    /// A `net` statement.
    Net(NetStmt),
    /// A function call statement.
    Call(CallStmt),
    /// A `footprint_override { ... }` block.
    FootprintOverride(FootprintOverrideBlock),
}

/// A `footprint_override { ... }` block inside a design.
#[derive(Debug, Clone, PartialEq)]
pub struct FootprintOverrideBlock {
    /// Override entries mapping device types to footprints.
    pub entries: Vec<FootprintOverrideEntry>,
    /// Source location.
    pub span: Span,
}

/// A single entry in a `footprint_override` block.
#[derive(Debug, Clone, PartialEq)]
pub struct FootprintOverrideEntry {
    /// The device type path.
    pub device: Path,
    /// What to override with.
    pub value: FootprintOverrideValue,
    /// Source location.
    pub span: Span,
}

/// The value of a footprint override entry.
#[derive(Debug, Clone, PartialEq)]
pub enum FootprintOverrideValue {
    /// A raw string: `-> "HouseLib:LQFP-64_Special"`
    String(StringLiteral),
    /// An alias reference: `-> LQFP64_10x10`
    AliasRef(Path),
}

// ── Statements ──────────────────────────────────────────────────────────────

/// An `inst` statement declaring a component instance.
///
/// ```hdl
/// inst mcu: STM32F103<pkg: LQFP64>
/// inst r_sense: Resistor<R: 100m> { primary { mfr: "Vishay", mpn: "..." } }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct InstStmt {
    /// Instance name.
    pub name: Ident,
    /// The device/part type.
    pub ty: TypeExpr,
    /// Optional inline AVL entries (for inline part definitions).
    pub avl_entries: Option<Vec<AvlEntry>>,
    /// Source location.
    pub span: Span,
}

/// A `net` statement connecting pins.
///
/// ```hdl
/// net vdd: mcu.VDD_IO, c.A
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NetStmt {
    /// The net target (left-hand side of `:`).
    pub target: NetEndpoint,
    /// One or more endpoints connected to this net.
    pub endpoints: Vec<NetEndpoint>,
    /// Source location.
    pub span: Span,
}

/// A net endpoint — either a simple identifier or a dot-path.
#[derive(Debug, Clone, PartialEq)]
pub struct NetEndpoint {
    /// The endpoint kind.
    pub kind: NetEndpointKind,
    /// Source location.
    pub span: Span,
}

/// Discriminant for [`NetEndpoint`].
#[derive(Debug, Clone, PartialEq)]
pub enum NetEndpointKind {
    /// A bare identifier such as `GND` or `VDD`.
    Ident(Ident),
    /// A dot-path such as `mcu.VDD_IO` or `c.A`.
    DotPath(DotPath),
}

/// A function call statement with named arguments.
///
/// ```hdl
/// decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CallStmt {
    /// The function path (simple or scoped).
    pub path: Path,
    /// Named arguments.
    pub args: Vec<CallArg>,
    /// Source location.
    pub span: Span,
}

/// A single named argument in a function call.
#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    /// Argument name.
    pub name: Ident,
    /// Argument value.
    pub value: Expr,
    /// Source location.
    pub span: Span,
}

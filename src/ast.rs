//! AST — every node carries a span (Constitution hard constraint).
//!
//! Shapes follow the Accepted-RFC examples exactly (note 10 + RFC-001…007);
//! constructs with no Accepted RFC follow docs/provisional-syntax.md.

use crate::span::Span;
use crate::units::{UnitType, UnitValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Item {
    /// `pub` is accepted and recorded; the MVP's flat scope doesn't enforce it.
    pub is_pub: bool,
    /// RFC-012 `#[intent("...")]` rationale — opaque metadata (value + the
    /// attribute's own span, kept so `fmt` can preserve comments around the
    /// attribute). Never READ by any checking/emission pass. (Honesty note,
    /// review 3: the guarantee is by-discipline, not fully structural — the
    /// AST that check functions receive does carry this field; what is
    /// enforced is that nothing looks at it, pinned by the byte-identity
    /// tests in tests/intent.rs.)
    pub intent: Option<(String, Span)>,
    /// Where the declaration proper begins (`pub`/keyword) — after any
    /// attributes. `span` covers the attributes too; `fmt` needs both.
    pub decl_span: Span,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Trait(TraitDef),
    Device(DeviceDef),
    Impl(ImplDef),
    Fn(FnDef),
    Part(PartDef),
    Design(DesignDef),
}

impl ItemKind {
    pub fn name(&self) -> Option<&Ident> {
        match self {
            ItemKind::Trait(t) => Some(&t.name),
            ItemKind::Device(d) => Some(&d.name),
            ItemKind::Fn(f) => Some(&f.name),
            ItemKind::Part(p) => Some(&p.name),
            ItemKind::Design(d) => Some(&d.name),
            ItemKind::Impl(_) => None,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            ItemKind::Trait(_) => "trait",
            ItemKind::Device(_) => "device",
            ItemKind::Impl(_) => "impl",
            ItemKind::Fn(_) => "fn",
            ItemKind::Part(_) => "part",
            ItemKind::Design(_) => "design",
        }
    }
}

// ---------------------------------------------------------------------------
// Shared pieces

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Obligation {
    Required,
    Optional,
}

impl Obligation {
    pub fn keyword(self) -> &'static str {
        match self {
            Obligation::Required => "required",
            Obligation::Optional => "optional",
        }
    }
}

/// Pin role (provisional-syntax.md §3). Consumed only by the driver DRC rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinRole {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
}

impl PinRole {
    pub fn name(self) -> &'static str {
        match self {
            PinRole::Input => "input",
            PinRole::Output => "output",
            PinRole::Bidirectional => "bidirectional",
            PinRole::Passive => "passive",
            PinRole::PowerIn => "power_in",
            PinRole::PowerOut => "power_out",
        }
    }

    pub fn from_name(s: &str) -> Option<PinRole> {
        Some(match s {
            "input" => PinRole::Input,
            "output" => PinRole::Output,
            "bidirectional" => PinRole::Bidirectional,
            "passive" => PinRole::Passive,
            "power_in" => PinRole::PowerIn,
            "power_out" => PinRole::PowerOut,
            _ => return None,
        })
    }

    /// Driver-type pins per provisional-syntax.md §3 (used by D003/D004).
    pub fn is_driver(self) -> bool {
        matches!(self, PinRole::Output | PinRole::PowerOut)
    }
}

/// Reference to one of the ten unit types by name, e.g. `Capacitance`.
#[derive(Debug, Clone)]
pub struct UnitTypeRef {
    pub unit: UnitType,
    pub span: Span,
}

/// `#[name("arg", …)]`
#[derive(Debug, Clone)]
pub struct Attr {
    pub name: Ident,
    pub args: Vec<(String, Span)>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Traits (RFC-003)

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: Ident,
    /// Sub-trait bounds: `trait Capacitor: TwoTerminal`.
    pub super_traits: Vec<Ident>,
    /// `designator_prefix: "C"` (provisional §6, from RFC-001's example).
    pub designator_prefix: Option<(String, Span)>,
    /// Abstract pin roles: `required A: pin`.
    pub pins: Vec<TraitPin>,
    /// Required spec fields with unit types: `capacitance: Capacitance`.
    pub specs: Vec<TraitSpecField>,
    /// Span of the first `pins { … }` / `spec { … }` block — kept so `fmt`
    /// can preserve comments around and inside trait bodies.
    pub pins_span: Option<Span>,
    pub spec_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct TraitPin {
    /// Omitted obligation defaults to `required` (the strictest reading).
    pub obligation: Obligation,
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitSpecField {
    pub name: Ident,
    pub ty: UnitTypeRef,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Devices (RFC-003; pins per RFC-002, roles + variants per RFC-008)

#[derive(Debug, Clone)]
pub struct DeviceDef {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    /// RFC-008 `variants { C0402, C0603 }` — empty when the device has none.
    pub variants: Vec<Ident>,
    /// Span of the whole `variants { … }` block, for source-order layout
    /// (`cohdl fmt`); `None` when the device has no variants block.
    pub variants_span: Option<Span>,
    /// `pins { … }` blocks; `variant` is the `pins[VARIANT]` qualifier.
    pub pin_blocks: Vec<PinBlock>,
    /// `spec { … }` blocks; `variant` is the `spec[VARIANT]` qualifier.
    pub spec_blocks: Vec<SpecBlock>,
}

#[derive(Debug, Clone)]
pub struct PinBlock {
    pub variant: Option<Ident>,
    pub pins: Vec<DevicePin>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SpecBlock {
    pub variant: Option<Ident>,
    pub fields: Vec<DeviceSpecField>,
    pub span: Span,
}

impl DeviceDef {
    pub fn has_variants(&self) -> bool {
        !self.variants.is_empty()
    }

    /// The pin set an instance of `variant` sees. For variant-less devices
    /// pass `None` (the bare block). Validation guarantees exactly one
    /// matching block exists in a well-formed device.
    pub fn pins_for(&self, variant: Option<&str>) -> &[DevicePin] {
        self.pin_blocks
            .iter()
            .find(|b| b.variant.as_ref().map(|v| v.name.as_str()) == variant)
            .map(|b| b.pins.as_slice())
            .unwrap_or(&[])
    }

    /// The merged spec fields an instance of `variant` sees: the base
    /// `spec {}` block, with `spec[VARIANT]` entries overriding same-named
    /// fields and appending new ones (RFC-008).
    pub fn spec_fields_for(&self, variant: Option<&str>) -> Vec<&DeviceSpecField> {
        let mut out: Vec<&DeviceSpecField> = Vec::new();
        for block in self.spec_blocks.iter().filter(|b| b.variant.is_none()) {
            out.extend(block.fields.iter());
        }
        if let Some(v) = variant {
            for block in self
                .spec_blocks
                .iter()
                .filter(|b| b.variant.as_ref().is_some_and(|x| x.name == v))
            {
                for field in &block.fields {
                    if let Some(slot) = out.iter_mut().find(|f| f.name.name == field.name.name) {
                        *slot = field; // override
                    } else {
                        out.push(field); // addition
                    }
                }
            }
        }
        out
    }
}

/// A physical pin number: `"1"`, `"57"`, or BGA-style `"A3"`.
#[derive(Debug, Clone)]
pub struct PinNumber {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DevicePin {
    pub obligation: Obligation,
    pub name: Ident,
    /// One or more physical pin numbers (`required GND: 2, 3, 4` is a pin
    /// bus — one logical pin, several physical pins, all wired together).
    pub numbers: Vec<PinNumber>,
    /// The explicit role annotation — required on every device pin
    /// (RFC-008; a missing role is E901). `None` only survives parse-error
    /// recovery, after the diagnostic has already been emitted.
    pub role: Option<(PinRole, Span)>,
    pub span: Span,
}

impl DevicePin {
    /// The pin's role. `Passive` only as post-E901 recovery poison.
    pub fn role_or_default(&self) -> PinRole {
        self.role.map_or(PinRole::Passive, |(r, _)| r)
    }
}

#[derive(Debug, Clone)]
pub struct DeviceSpecField {
    pub name: Ident,
    pub value: SpecValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SpecValue {
    /// `capacitance: 100nF`
    Lit(UnitValue, Span),
    /// `capacitance: C` — reference to one of the device's generic params.
    GenericRef(Ident),
}

// ---------------------------------------------------------------------------
// Generics (RFC-007)

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: Ident,
    pub bound: GenericBound,
    /// Visible default — valid only on unit-type parameters (E406 otherwise).
    pub default: Option<(UnitValue, Span)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum GenericBound {
    /// `C: Capacitance`
    Unit(UnitTypeRef),
    /// `D: Capacitor + Polarized`
    Traits(Vec<Ident>),
}

/// A generic argument at a use site: `MLCC<100nF, V>`, `foo::<MLCC>(…)`.
#[derive(Debug, Clone)]
pub enum GenericArg {
    Unit(UnitValue, Span),
    /// A name: an in-scope generic parameter, or a device/part name for a
    /// trait-bound parameter. Resolved during checking.
    Name(Ident),
    /// A bare number — syntactically accepted so the type checker can reject
    /// it with the precise E404/E111 diagnostic rather than a parse error.
    Number(String, Span),
}

impl GenericArg {
    pub fn span(&self) -> Span {
        match self {
            GenericArg::Unit(_, s) => *s,
            GenericArg::Name(i) => i.span,
            GenericArg::Number(_, s) => *s,
        }
    }
}

/// `Name<args>[VARIANT]` in type position (`inst c: MLCC<100nF, V>[C0603]`,
/// part device refs). The `[VARIANT]` selector is RFC-008: required exactly
/// when the device declares `variants {}`.
#[derive(Debug, Clone)]
pub struct TypeRef {
    pub name: Ident,
    pub generic_args: Vec<GenericArg>,
    pub variant: Option<Ident>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Impls (RFC-003)

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub trait_name: Ident,
    pub device_name: Ident,
    /// Explicit role → device-pin mapping (only when names differ):
    /// `pins { A: Anode, B: Cathode }`.
    pub pin_map: Vec<MapEntry>,
    /// Explicit trait-spec-field → device-spec-field mapping.
    pub spec_map: Vec<MapEntry>,
    /// Spans of the mapping sub-blocks (comment preservation in `fmt`).
    pub pins_span: Option<Span>,
    pub spec_span: Option<Span>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MapEntry {
    /// The trait's required role/field name.
    pub role: Ident,
    /// The device's own pin/spec name it maps to.
    pub target: Ident,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Fns (RFC-006/007)

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<FnParam>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub name: Ident,
    pub ty: FnParamTy,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum FnParamTy {
    /// `pin: Pin`
    Pin(Span),
    /// `target: D` — a name that must resolve to one of the fn's own
    /// trait-bound generic parameters.
    Generic(Ident),
    /// `target: impl Capacitor + Polarized` — sugar for an anonymous
    /// trait-bound generic parameter (RFC-007).
    ImplTrait(Vec<Ident>, Span),
}

// ---------------------------------------------------------------------------
// Statements (fn and design bodies)

#[derive(Debug, Clone)]
pub enum Stmt {
    Inst(InstStmt),
    Net(NetStmt),
    Nc(NcStmt),
    Call(CallStmt),
    /// RFC-013 `layout { … }` — layout-constraint metadata, structurally
    /// checked but never affecting connectivity or emitted netlist bytes.
    Layout(LayoutBlock),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Inst(s) => s.span,
            Stmt::Net(s) => s.span,
            Stmt::Nc(s) => s.span,
            Stmt::Call(s) => s.span,
            Stmt::Layout(s) => s.span,
        }
    }
}

// ---------------------------------------------------------------------------
// Layout constraints (RFC-013 — the door)

#[derive(Debug, Clone)]
pub struct LayoutBlock {
    pub constraints: Vec<LayoutConstraint>,
    pub span: Span,
}

/// One layout constraint. Net references are net *names* (not pin refs),
/// resolved against the design/fn's declared nets during expansion.
#[derive(Debug, Clone)]
pub enum LayoutConstraint {
    /// `net_class NAME { net, net, … }` — a named group sharing a layout
    /// treatment (e.g. impedance control).
    NetClass {
        name: Ident,
        nets: Vec<Ident>,
        span: Span,
    },
    /// `diff_pair(net_p, net_n)` — exactly two nets (checked: E1003).
    DiffPair { nets: Vec<Ident>, span: Span },
    /// `length_match(net, net, …) [tolerance: "..."]` — two or more nets
    /// (checked: E1004). Tolerance is an opaque pass-through string (CoHDL has
    /// no length unit; the value is never enforced — RFC-013 Failure modes).
    LengthMatch {
        nets: Vec<Ident>,
        tolerance: Option<(String, Span)>,
        span: Span,
    },
}

impl LayoutConstraint {
    pub fn span(&self) -> Span {
        match self {
            LayoutConstraint::NetClass { span, .. }
            | LayoutConstraint::DiffPair { span, .. }
            | LayoutConstraint::LengthMatch { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstStmt {
    /// Recognized: `#[designator("U7")]` (RFC-005). `#[intent(...)]` and
    /// `#[placement_hint(...)]` are split out into their own fields at parse
    /// time, never left here.
    pub attrs: Vec<Attr>,
    /// RFC-012 opaque `#[intent("...")]` metadata (never compiled); the span
    /// is the attribute's own, for comment-preserving `fmt`.
    pub intent: Option<(String, Span)>,
    /// RFC-013 opaque `#[placement_hint("...")]` layout metadata — inst-only,
    /// zero-impact (rides into `layout.json`, never into `.net`/BOM/designators).
    pub placement_hint: Option<(String, Span)>,
    pub name: Ident,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NetStmt {
    /// `None` for the anonymous form `net _: …`.
    pub name: Option<Ident>,
    pub annotation: Option<NetAnnotation>,
    pub members: Vec<PinRef>,
    /// RFC-012 opaque `#[intent("...")]` metadata (never compiled).
    pub intent: Option<(String, Span)>,
    pub span: Span,
}

/// Provisional §4: `net VDD [3.3V]: …` / `net GND [gnd]: …`.
#[derive(Debug, Clone)]
pub enum NetAnnotation {
    Voltage(UnitValue, Span),
    Gnd(Span),
}

impl NetAnnotation {
    pub fn span(&self) -> Span {
        match self {
            NetAnnotation::Voltage(_, s) | NetAnnotation::Gnd(s) => *s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NcStmt {
    pub members: Vec<PinRef>,
    /// RFC-012 opaque `#[intent("...")]` metadata (never compiled).
    pub intent: Option<(String, Span)>,
    pub span: Span,
}

/// `mcu.VDD` (instance/param pin), `target.A` (trait role on a generic-typed
/// param), or a bare name (`pin` param passthrough, or an instance being
/// passed as a call argument).
#[derive(Debug, Clone)]
pub struct PinRef {
    pub base: Ident,
    pub pin: Option<Ident>,
    pub span: Span,
}

impl std::fmt::Display for PinRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.pin {
            Some(p) => write!(f, "{}.{}", self.base.name, p.name),
            None => f.write_str(&self.base.name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallStmt {
    pub callee: Ident,
    /// Turbofish args: `decoupling_cap::<V>(…)`.
    pub generic_args: Vec<GenericArg>,
    pub args: Vec<PinRef>,
    /// RFC-012 opaque `#[intent("...")]` metadata (never compiled).
    pub intent: Option<(String, Span)>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Parts (provisional §2)

#[derive(Debug, Clone)]
pub struct PartDef {
    pub name: Ident,
    pub device: TypeRef,
    pub primary: AvlEntry,
    pub alts: Vec<AvlEntry>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AvlEntry {
    pub fields: Vec<AvlField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AvlField {
    pub name: Ident,
    pub value: String,
    pub span: Span,
}

impl AvlEntry {
    pub fn field(&self, name: &str) -> Option<&AvlField> {
        self.fields.iter().find(|f| f.name.name == name)
    }
}

// ---------------------------------------------------------------------------
// Designs

#[derive(Debug, Clone)]
pub struct DesignDef {
    pub name: Ident,
    pub body: Vec<Stmt>,
}

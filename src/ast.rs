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
    /// RFC-017 `#[doc("relative/path")]` reference documents — one or MORE
    /// per declaration (unlike intent's at-most-one). Opaque metadata: the
    /// compiler never opens the referenced files; surfaced by tooling (LSP
    /// hover) only.
    pub docs: Vec<(String, Span)>,
    /// Where the declaration proper begins (`pub`/keyword) — after any
    /// attributes. `span` covers the attributes too; `fmt` needs both.
    pub decl_span: Span,
    pub kind: ItemKind,
    pub span: Span,
}

/// RFC-016 `use package::module::Name;` — imports exactly one name into the
/// current FILE's local scope. `path` holds every segment; the last one is
/// the local name the import binds.
#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<Ident>,
    pub span: Span,
}

impl UseDecl {
    /// The imported local name (the path's last segment).
    pub fn local(&self) -> &Ident {
        self.path.last().expect("use path is never empty")
    }

    /// The full path as written, `::`-joined.
    pub fn path_text(&self) -> String {
        self.path
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }
}

/// RFC-018 pad vocabulary (closed sets — extension needs a follow-up RFC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadShape {
    Rect,
    Circle,
    Oval,
    Annulus,
}

impl PadShape {
    pub fn name(self) -> &'static str {
        match self {
            PadShape::Rect => "rect",
            PadShape::Circle => "circle",
            PadShape::Oval => "oval",
            PadShape::Annulus => "annulus",
        }
    }
    pub fn from_name(s: &str) -> Option<PadShape> {
        Some(match s {
            "rect" => PadShape::Rect,
            "circle" => PadShape::Circle,
            "oval" => PadShape::Oval,
            "annulus" => PadShape::Annulus,
            _ => return None,
        })
    }
    /// `(w, h)` for rect/oval; `(d)` for circle.
    pub fn size_arity(self) -> usize {
        match self {
            PadShape::Circle => 1,
            PadShape::Rect | PadShape::Oval | PadShape::Annulus => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadLayer {
    TopCopper,
    BottomCopper,
    ThroughAll,
}

impl PadLayer {
    pub fn name(self) -> &'static str {
        match self {
            PadLayer::TopCopper => "top_copper",
            PadLayer::BottomCopper => "bottom_copper",
            PadLayer::ThroughAll => "through_all",
        }
    }
    pub fn from_name(s: &str) -> Option<PadLayer> {
        Some(match s {
            "top_copper" => PadLayer::TopCopper,
            "bottom_copper" => PadLayer::BottomCopper,
            "through_all" => PadLayer::ThroughAll,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadPlating {
    Smd,
    PlatedThroughHole,
}

/// One 45-degree corner cut on an otherwise rectangular pad.  The vocabulary
/// is intentionally bounded: it covers manufacturer land patterns that call
/// out a single orientation/chamfer feature without opening the pad grammar to
/// arbitrary, backend-dependent polygons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl PadCorner {
    pub fn name(self) -> &'static str {
        match self {
            PadCorner::TopLeft => "top_left",
            PadCorner::TopRight => "top_right",
            PadCorner::BottomLeft => "bottom_left",
            PadCorner::BottomRight => "bottom_right",
        }
    }

    pub fn from_name(s: &str) -> Option<PadCorner> {
        Some(match s {
            "top_left" => PadCorner::TopLeft,
            "top_right" => PadCorner::TopRight,
            "bottom_left" => PadCorner::BottomLeft,
            "bottom_right" => PadCorner::BottomRight,
            _ => return None,
        })
    }
}

/// Paste override for an SMD pad.  Absence on [`PadDef`] preserves the
/// historical behaviour (paste equals copper); `None` suppresses paste,
/// `Rect` emits one centered rectangular stencil aperture, and `Circle`
/// emits one circular aperture whose diameter is independent of copper.
#[derive(Debug, Clone)]
pub enum PadPaste {
    None,
    Rect(UnitValue, UnitValue),
    Circle(UnitValue),
    SegmentedAnnulus(Box<[UnitValue; 3]>),
}

impl PadPlating {
    pub fn name(self) -> &'static str {
        match self {
            PadPlating::Smd => "smd",
            PadPlating::PlatedThroughHole => "plated_through_hole",
        }
    }
    pub fn from_name(s: &str) -> Option<PadPlating> {
        Some(match s {
            "smd" => PadPlating::Smd,
            "plated_through_hole" => PadPlating::PlatedThroughHole,
            _ => return None,
        })
    }
}

/// RFC-018 `pad NAME { shape/size/layer/plating[/drill] }` — one reusable
/// pad definition, referenced (never inlined) by footprints.
#[derive(Debug, Clone)]
pub struct PadDef {
    pub name: Ident,
    /// `None` only survives a missing-field parse error (reported).
    pub shape: Option<(PadShape, Span)>,
    /// `(w, h)` or `(d)` — arity validated against the shape.
    pub size: Vec<UnitValue>,
    pub size_span: Option<Span>,
    pub layer: Option<(PadLayer, Span)>,
    pub plating: Option<(PadPlating, Span)>,
    /// Required iff `plating: plated_through_hole`.
    pub drill: Option<(PadDrill, Span)>,
    /// Optional single 45-degree corner cut: `(corner, cut length, span)`.
    pub chamfer: Option<(PadCorner, UnitValue, Span)>,
    /// Optional radius applied to all four corners of a rectangular SMD pad.
    pub corner_radius: Option<(UnitValue, Span)>,
    /// Optional local solder-mask expansion around the authored copper shape.
    pub mask_expansion: Option<(UnitValue, Span)>,
    /// Optional paste override; absent means paste follows copper.
    pub paste: Option<(PadPaste, Span)>,
    pub span: Span,
}

/// A plated pad's hole. `drill: D` is a round hole (the original RFC-018
/// form); `drill: (w, l)` is a SLOT of that width and length — what a
/// connector's shield leg or any rectangular tab actually needs.
///
/// The two forms mirror RFC-023's `mount_hole` convention exactly: a scalar
/// for the circular case, a `(w, h)` tuple for the elongated one. Slots have
/// no Accepted RFC yet — see docs/provisional-syntax.md.
#[derive(Debug, Clone)]
pub enum PadDrill {
    /// `drill: D` — a round hole of diameter D.
    Round(UnitValue),
    /// `drill: (width, length)` — an obround slot.
    Slot(UnitValue, UnitValue),
}

impl PadDrill {
    /// Every dimension the form carries, for uniform per-value validation.
    pub fn values(&self) -> Vec<&UnitValue> {
        match self {
            PadDrill::Round(d) => vec![d],
            PadDrill::Slot(w, l) => vec![w, l],
        }
    }

    /// The hole's minor axis — a round hole's diameter, a slot's width. This
    /// is the drill a fabricator routes the slot with, and what IPC-2581's
    /// single-scalar `<Hole>` carries (the full extent rides the primitive).
    pub fn minor(&self) -> &UnitValue {
        match self {
            PadDrill::Round(d) => d,
            PadDrill::Slot(w, l) => {
                if w.femto <= l.femto {
                    w
                } else {
                    l
                }
            }
        }
    }
}

/// One `pad N: PadSymbol at (x, y) [rotate ANGLE]` placement inside a
/// footprint body. RFC-025: `rotate` reuses RFC-020's closed {0, 90, 180, 270}
/// set — a placement-time fact layered on the referenced pad symbol's own
/// (never-mutated) geometry. 0 = unrotated, the pre-RFC-025 meaning.
#[derive(Debug, Clone)]
pub struct PadPlace {
    /// Must match one of the bound device's physical pin numbers.
    pub number: PinNumber,
    /// The referenced pad symbol (fq after resolution).
    pub pad: Ident,
    pub x: UnitValue,
    pub y: UnitValue,
    /// RFC-025; `u16::MAX` marks an unparseable value (fails the closed-set
    /// check at declaration, same convention as `Placement::rotate`).
    pub rotate: u16,
    pub span: Span,
}

/// RFC-022 mount_hole plating — a closed two-value set (no `smd`: a mount_hole
/// is definitionally a hole, never a surface pad).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountHolePlating {
    NonPlated,
    Plated,
}

impl MountHolePlating {
    pub fn name(self) -> &'static str {
        match self {
            MountHolePlating::NonPlated => "non_plated",
            MountHolePlating::Plated => "plated",
        }
    }
    pub fn from_name(s: &str) -> Option<MountHolePlating> {
        Some(match s {
            "non_plated" => MountHolePlating::NonPlated,
            "plated" => MountHolePlating::Plated,
            _ => return None,
        })
    }
}

/// RFC-023: a `mount_hole`'s geometry. Shape-dependent, mirroring `pad`'s own
/// established convention exactly: a `circle` (explicit or defaulted) carries a
/// scalar `diameter D`; a `rect`/`oval` carries `size: (w, h)`.
#[derive(Debug, Clone)]
pub enum MountHoleGeom {
    /// `diameter D` — legal only for `circle`.
    Diameter(UnitValue),
    /// `size: (w, h)` — legal only for `rect` / `oval`.
    Size(Vec<UnitValue>, Span),
}

impl MountHoleGeom {
    /// The shape this geometry form belongs to, for mismatch diagnostics.
    pub fn field_name(&self) -> &'static str {
        match self {
            MountHoleGeom::Diameter(_) => "diameter",
            MountHoleGeom::Size(..) => "size:",
        }
    }
    /// A `size:` tuple carries its own span; a scalar `diameter` has none of
    /// its own, so callers pass the whole declaration's span as the fallback.
    pub fn span(&self, fallback: Span) -> Span {
        match self {
            MountHoleGeom::Diameter(_) => fallback,
            MountHoleGeom::Size(_, s) => *s,
        }
    }
}

/// RFC-022, extended by RFC-023:
/// `mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D | size: (w, h)]`
/// — a mechanical locating hole in a footprint body. `N` is a locating-hole-local
/// counter, entirely DISJOINT from `pad`'s pin-bound numbering: it is never
/// checked against the bound device's declared pins. Always spans `through_all`.
#[derive(Debug, Clone)]
pub struct MountHole {
    /// A footprint-local counter, disjoint from pad numbers (never a pin).
    pub number: PinNumber,
    pub plating: MountHolePlating,
    /// RFC-023: optional. Absence defaults to `circle`, which preserves the
    /// meaning of every `mount_hole` written before the shape/size extension.
    pub shape: Option<(PadShape, Span)>,
    pub x: UnitValue,
    pub y: UnitValue,
    /// Required regardless of plating; its form must match `shape`.
    pub geom: MountHoleGeom,
    pub span: Span,
}

impl MountHole {
    /// The effective shape: explicit if written, else RFC-023's `circle` default.
    pub fn shape_or_default(&self) -> PadShape {
        self.shape.map(|(s, _)| s).unwrap_or(PadShape::Circle)
    }
}

/// RFC-031 silkscreen fill vocabulary — a closed two-value set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilkFill {
    None,
    Solid,
}

impl SilkFill {
    pub fn name(self) -> &'static str {
        match self {
            SilkFill::None => "none",
            SilkFill::Solid => "solid",
        }
    }
    pub fn from_name(s: &str) -> Option<SilkFill> {
        Some(match s {
            "none" => SilkFill::None,
            "solid" => SilkFill::Solid,
            _ => return None,
        })
    }
}

/// RFC-031: one drawable silkscreen primitive. The set is closed to the four
/// shapes real silkscreen art needs, deliberately NOT reusing `PadShape` —
/// a pad models copper/mask/drill area, a silk graphic models a stroke.
#[derive(Debug, Clone)]
pub enum SilkGraphic {
    /// `line from (x1, y1) to (x2, y2) width W`
    Line {
        from: (UnitValue, UnitValue),
        to: (UnitValue, UnitValue),
        width: UnitValue,
    },
    /// `circle at (x, y) radius R width W [fill F]` — defaults `none`.
    Circle {
        at: (UnitValue, UnitValue),
        radius: UnitValue,
        width: UnitValue,
        fill: SilkFill,
    },
    /// `arc at (x, y) radius R start_angle A1 end_angle A2 width W` — angles in
    /// whole degrees, 0..=360 (a genuinely continuous need, unlike the cardinal
    /// set `rotate` uses for whole-component orientation).
    Arc {
        at: (UnitValue, UnitValue),
        radius: UnitValue,
        start_angle: i32,
        end_angle: i32,
        width: UnitValue,
    },
    /// `polygon [(x1, y1), …] [fill F]` — three or more vertices; defaults
    /// `solid` (the common case is a filled pin-1 triangle).
    Polygon {
        points: Vec<(UnitValue, UnitValue)>,
        fill: SilkFill,
    },
}

/// RFC-031 `pin_1_marker … shape SHAPE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin1Shape {
    Dot,
    Triangle,
}

/// RFC-031 `polarity_marker … shape SHAPE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarityShape {
    Band,
    Arrow,
}

/// One statement inside a `silkscreen { … }` block: either a raw primitive or
/// one of the two semantic markers, which are sugar that EXPANDS to primitives
/// (`emit::silk`) rather than a second drawing mechanism.
#[derive(Debug, Clone)]
pub enum SilkItem {
    Graphic(SilkGraphic, Span),
    /// `pin_1_marker near pad N shape SHAPE`
    Pin1Marker {
        pad: PinNumber,
        shape: Pin1Shape,
        span: Span,
    },
    /// `polarity_marker cathode_pin N shape SHAPE`
    PolarityMarker {
        cathode_pad: PinNumber,
        shape: PolarityShape,
        span: Span,
    },
}

impl SilkItem {
    pub fn span(&self) -> Span {
        match self {
            SilkItem::Graphic(_, s) | SilkItem::Pin1Marker { span: s, .. } => *s,
            SilkItem::PolarityMarker { span: s, .. } => *s,
        }
    }
}

/// RFC-031 `silkscreen { … }` — a footprint-body block of drawable graphics.
#[derive(Debug, Clone)]
pub struct SilkscreenBlock {
    pub items: Vec<SilkItem>,
    pub span: Span,
}

/// A `shape: … , at: (x, y), size: (…)` block. Two footprint constructs share
/// this record: `courtyard` (the placement keep-out) and `window` (a board
/// CUTOUT the part needs — see `FootprintDef::window`).
#[derive(Debug, Clone)]
pub struct Courtyard {
    pub shape: (PadShape, Span),
    pub at: (UnitValue, UnitValue),
    pub size: Vec<UnitValue>,
    pub size_span: Span,
    pub span: Span,
}

/// RFC-017/018: `pub footprint NAME { … }` — a named, resolvable footprint.
/// RFC-018 gave the body real content: pad placements + optional courtyard
/// and silkscreen reference. An EMPTY body remains legal — it is RFC-017's
/// stage-one placeholder (the pad-count check only applies once pads exist).
#[derive(Debug, Clone)]
pub struct FootprintDef {
    /// RFC-021: for the closed IPC-7351 family set (QFP/QFN/SOIC/SOP/SOT/BGA/
    /// CHIP/MELF) the identifier itself IS the IPC-7351B designator (with `-`
    /// → `_`), checked for grammar well-formedness and against this footprint's
    /// own pad geometry (pin count + pitch). A name outside those families is
    /// an ordinary RFC-016 identifier, unchecked. There is no separate field.
    pub name: Ident,
    pub pads: Vec<PadPlace>,
    /// RFC-022 mechanical locating holes — numbered disjoint from `pads`.
    pub mount_holes: Vec<MountHole>,
    pub courtyard: Option<Courtyard>,
    /// A board cutout this part needs: the light window a reverse-mount LED
    /// fires through. Deliberately NOT a `mount_hole` — that is a drilled
    /// hole, this is routed board edge (KiCad's `Edge.Cuts`), and conflating
    /// them would emit a 3.4mm "drill" where a fabricator needs a milled
    /// aperture. Provisional; see docs/provisional-syntax.md.
    ///
    /// Boxed because a window is rare — only reverse-mount parts carry one —
    /// and inlining a second shape record here makes `ItemKind::Footprint` the
    /// outsized enum variant for every footprint that has no window.
    pub window: Option<Box<Courtyard>>,
    /// RFC-031 silkscreen graphics. Boxed for the same reason `window` is: most
    /// footprints have none, and inlining the block would make
    /// `ItemKind::Footprint` the outsized enum variant for all of them.
    pub silkscreen: Option<Box<SilkscreenBlock>>,
    pub silkscreen_ref: Option<(UnitValue, UnitValue, Span)>,
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
    /// RFC-018 reusable pad definition.
    Pad(PadDef),
    /// RFC-017/018 footprint (pad placements; empty = stage-one placeholder).
    Footprint(FootprintDef),
    /// RFC-016 `use path::Name;` — a file-scoped import, not a declaration.
    Use(UseDecl),
}

impl ItemKind {
    pub fn name(&self) -> Option<&Ident> {
        match self {
            ItemKind::Trait(t) => Some(&t.name),
            ItemKind::Device(d) => Some(&d.name),
            ItemKind::Fn(f) => Some(&f.name),
            ItemKind::Part(p) => Some(&p.name),
            ItemKind::Design(d) => Some(&d.name),
            ItemKind::Pad(p) => Some(&p.name),
            ItemKind::Footprint(f) => Some(&f.name),
            ItemKind::Impl(_) | ItemKind::Use(_) => None,
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
            ItemKind::Pad(_) => "pad",
            ItemKind::Footprint(_) => "footprint",
            ItemKind::Use(_) => "use",
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

/// Reference to one of the eleven unit types by name, e.g. `Capacitance`.
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
    /// A pragmatic extension beyond RFC-013's net-level constraints: an
    /// optional rectangular board outline (center `at` + `size`), the single
    /// closed board perimeter a downstream layout tool (Quilter, RFC-015)
    /// needs. Board-level, not net-level — at most one, design top level only.
    /// Ledgered in docs/compliance-report.md as an extension pending an RFC.
    pub board_outline: Option<BoardOutline>,
    /// `place <inst> at (x, y)` — fixed (locked) component positions. A
    /// downstream placement tool treats these as pre-placed and positions only
    /// the rest; used for board-edge/mechanical parts (connectors, mounting).
    /// Same pragmatic-extension status as `board_outline`.
    pub placements: Vec<Placement>,
    pub span: Span,
}

/// `place <inst> at (x, y) [rotate ANGLE]` — a locked, optionally-rotated
/// placement of one instance (RFC-020). `rotate` is one of the closed set
/// {0, 90, 180, 270}; 0 (unrotated) is the default when omitted.
#[derive(Debug, Clone)]
pub struct Placement {
    pub inst: Ident,
    /// RFC-024: `place NAME[i] at (…)` — always exactly one element, never a
    /// range (each element needs its own coordinates).
    pub index: Option<(i64, Span)>,
    pub at: (UnitValue, UnitValue),
    pub rotate: u16,
    /// RFC-026: `side top | bottom` — which outer face the whole component
    /// sits on. `Top` is the default and the pre-RFC-026 meaning; mirroring a
    /// bottom-side footprint is emitter work, never computed here.
    pub side: PlacementSide,
    /// The `side` clause's span when written (diagnostics); `None` = defaulted.
    pub side_span: Option<Span>,
    pub span: Span,
}

/// RFC-026's closed two-value side set. There are exactly two outer faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementSide {
    Top,
    Bottom,
}

impl PlacementSide {
    pub fn name(self) -> &'static str {
        match self {
            PlacementSide::Top => "top",
            PlacementSide::Bottom => "bottom",
        }
    }
    pub fn from_name(s: &str) -> Option<PlacementSide> {
        Some(match s {
            "top" => PlacementSide::Top,
            "bottom" => PlacementSide::Bottom,
            _ => return None,
        })
    }
}

/// `board_outline: "path.dxf"` (RFC-020) — a reference to a DXF file from
/// which `cohdl build` extracts one closed outline entity (see `crate::dxf`).
/// The body is just the path string; the real geometry is resolved at build.
#[derive(Debug, Clone)]
pub struct BoardOutline {
    pub path: String,
    pub path_span: Span,
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
    /// RFC-027: an optional trailing `[differential_impedance: R,
    /// single_ended_impedance: R, frequency: F]` bracket; omitting it
    /// preserves RFC-013's original form exactly.
    DiffPair {
        nets: Vec<Ident>,
        differential_impedance: Option<UnitValue>,
        single_ended_impedance: Option<UnitValue>,
        frequency: Option<UnitValue>,
        span: Span,
    },
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
    /// RFC-027 inst-target physics attributes (at most one of each kind).
    pub phys: Vec<PhysAttr>,
    pub name: Ident,
    /// RFC-024: `inst NAME: [Device; N]` — an array-typed instance of fixed
    /// length N. `None` is the ordinary single-instance form. When set, `NAME`
    /// alone is NEVER a valid reference; every use must be indexed `NAME[i]`.
    pub array_len: Option<(i64, Span)>,
    pub ty: TypeRef,
    pub span: Span,
}

/// RFC-027: one structured Quilter physics-constraint attribute — the closed
/// seven-name set riding the existing `#[name(...)]` bracket syntax. Unlike
/// the opaque-string `Attr` shape (RFC-012), each carries its own real, parsed
/// argument grammar. Which statement kind each variant may sit on is enforced
/// at parse (net-only vs inst-only).
#[derive(Debug, Clone)]
pub enum PhysAttr {
    /// `#[ground(primary | secondary [, region_pour])]` — net-only.
    Ground {
        primary: bool,
        region_pour: bool,
        span: Span,
    },
    /// `#[high_current(CURRENT [, power_pour])]` — net-only.
    HighCurrent {
        current: UnitValue,
        power_pour: bool,
        span: Span,
    },
    /// `#[impedance(RESISTANCE, frequency: FREQUENCY)]` — net-only.
    Impedance {
        impedance: UnitValue,
        frequency: UnitValue,
        span: Span,
    },
    /// `#[bypass(TARGET, CAPACITANCE)]` — on the capacitor's own inst.
    /// RFC-028: `TARGET` is either `INST.PIN` (pin: Some) or a bare
    /// `Pin`-typed `fn` parameter (pin: None), the same two forms `PinRef`
    /// already has everywhere else in the language.
    Bypass {
        inst: Ident,
        /// RFC-024: `#[bypass(NAME[i].PIN, …)]` — an array element is a valid
        /// instance reference here as anywhere else.
        index: Option<(i64, Span)>,
        pin: Option<Ident>,
        capacitance: UnitValue,
        span: Span,
    },
    /// `#[crystal_oscillator(PARENT, PIN_1, PIN_2)]` — on the crystal's inst.
    CrystalOscillator {
        parent: Ident,
        pin1: Ident,
        pin2: Ident,
        span: Span,
    },
    /// `#[switching_converter(inductor: I [, input_capacitor: C]
    /// [, output_capacitor: C])]` — on the converter's own inst.
    SwitchingConverter {
        inductor: Ident,
        input_capacitor: Option<Ident>,
        output_capacitor: Option<Ident>,
        span: Span,
    },
    /// `#[bga_fanout]` — bare, on a BGA's own inst.
    BgaFanout { span: Span },
}

impl PhysAttr {
    pub fn span(&self) -> Span {
        match self {
            PhysAttr::Ground { span, .. }
            | PhysAttr::HighCurrent { span, .. }
            | PhysAttr::Impedance { span, .. }
            | PhysAttr::Bypass { span, .. }
            | PhysAttr::CrystalOscillator { span, .. }
            | PhysAttr::SwitchingConverter { span, .. }
            | PhysAttr::BgaFanout { span } => *span,
        }
    }
    /// The attribute's name, for diagnostics and fmt.
    pub fn name(&self) -> &'static str {
        match self {
            PhysAttr::Ground { .. } => "ground",
            PhysAttr::HighCurrent { .. } => "high_current",
            PhysAttr::Impedance { .. } => "impedance",
            PhysAttr::Bypass { .. } => "bypass",
            PhysAttr::CrystalOscillator { .. } => "crystal_oscillator",
            PhysAttr::SwitchingConverter { .. } => "switching_converter",
            PhysAttr::BgaFanout { .. } => "bga_fanout",
        }
    }
    /// True for the net-target kinds (the rest are inst-target).
    pub fn is_net_attr(&self) -> bool {
        matches!(
            self,
            PhysAttr::Ground { .. } | PhysAttr::HighCurrent { .. } | PhysAttr::Impedance { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct NetStmt {
    /// `None` for the anonymous form `net _: …`.
    pub name: Option<Ident>,
    pub annotation: Option<NetAnnotation>,
    /// RFC-027 net-target physics attributes (at most one of each kind).
    pub phys: Vec<PhysAttr>,
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

/// RFC-024: an index selector on an array-typed instance reference.
///
/// `Single` — `NAME[i]`, a literal index — is valid EVERYWHERE an ordinary
/// instance reference is: net members, `place`, `decouple`, and `fn`-call
/// arguments. `Range`/`List` are fan-out SUGAR over `Single`, and remain
/// scoped to net-member lists only (placing or decoupling "a range at once"
/// has no single sensible meaning).
#[derive(Debug, Clone)]
pub enum IndexSel {
    /// `[i]` — one element. The real reference form.
    Single(i64, Span),
    /// `[START..=END]`, or `[START..=END step STEP]` when `step` is written.
    Range {
        start: i64,
        end: i64,
        /// Always >= 1; absent in source means 1.
        step: i64,
        /// True when `step` was written, so `fmt` round-trips it.
        explicit_step: bool,
        span: Span,
    },
    /// `[i1, i2, i3, …]` — a genuinely irregular set a stride can't express.
    List(Vec<i64>, Span),
}

impl IndexSel {
    /// The selected indices, in written order.
    pub fn indices(&self) -> Vec<i64> {
        match self {
            IndexSel::Single(i, _) => vec![*i],
            IndexSel::Range {
                start, end, step, ..
            } => {
                let step = (*step).max(1);
                let mut out = Vec::new();
                let mut i = *start;
                while i <= *end {
                    out.push(i);
                    i += step;
                }
                out
            }
            IndexSel::List(v, _) => v.clone(),
        }
    }
    pub fn span(&self) -> Span {
        match self {
            IndexSel::Single(_, span) | IndexSel::Range { span, .. } | IndexSel::List(_, span) => {
                *span
            }
        }
    }
}

impl std::fmt::Display for IndexSel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexSel::Single(i, _) => write!(f, "[{}]", i),
            IndexSel::Range {
                start,
                end,
                step,
                explicit_step,
                ..
            } => {
                write!(f, "[{}..={}", start, end)?;
                // An implicit stride of 1 is never spelled out, so an
                // unstrided range round-trips byte-identically.
                if *explicit_step {
                    write!(f, " step {}", step)?;
                }
                f.write_str("]")
            }
            IndexSel::List(v, _) => {
                let items: Vec<String> = v.iter().map(|i| i.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
        }
    }
}

/// `mcu.VDD` (instance/param pin), `target.A` (trait role on a generic-typed
/// param), or a bare name (`pin` param passthrough, or an instance being
/// passed as a call argument).
#[derive(Debug, Clone)]
pub struct PinRef {
    pub base: Ident,
    /// RFC-024: `base[…]` selecting several array elements at once. `None` for
    /// an ordinary single-instance reference.
    pub index: Option<IndexSel>,
    pub pin: Option<Ident>,
    pub span: Span,
}

impl std::fmt::Display for PinRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.base.name)?;
        // RFC-024: the index selector belongs to the base name, so it must be
        // rendered here — `fmt` prints net members through this impl.
        if let Some(sel) = &self.index {
            write!(f, "{}", sel)?;
        }
        if let Some(p) = &self.pin {
            write!(f, ".{}", p.name)?;
        }
        Ok(())
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
    /// RFC-017: `footprint:` holds a SYMBOL reference (resolved via
    /// RFC-016), never a string. The Ident's name is the fq path after
    /// resolution.
    pub footprint: Option<Ident>,
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

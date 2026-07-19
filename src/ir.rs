//! Flat post-expansion design IR — the "connects" rung of the verdict ladder.
//!
//! Everything downstream (designators, residual DRC, emitters) consumes this.
//! All collections are ordered (BTree/sorted Vec) so output is byte-stable.

use crate::span::Span;
use crate::units::UnitValue;
use std::collections::{BTreeMap, BTreeSet};

/// A fully-expanded, monomorphized design.
#[derive(Debug)]
pub struct DesignIr {
    pub name: String,
    /// Keyed by hierarchical path (`Board::__fn0_power_rail::c`).
    pub instances: BTreeMap<String, IrInstance>,
    /// Merged electrical nets, sorted by emitted name.
    pub nets: Vec<IrNet>,
    /// Pins explicitly marked not-connected: (instance path, logical pin).
    pub nc_pins: BTreeSet<(String, String)>,
    /// RFC-013 layout constraints, resolved to IR net names. Rides into the
    /// separate `layout.json` artifact — never the `.net`/BOM connectivity data.
    pub layout: LayoutIr,
}

/// Resolved layout constraints (RFC-013). Net references are IR net names.
/// Emission order is source/collection order — deterministic per source.
#[derive(Debug, Default)]
pub struct LayoutIr {
    pub net_classes: Vec<LayoutNetClass>,
    pub diff_pairs: Vec<LayoutDiffPair>,
    pub length_matches: Vec<LayoutLengthMatch>,
    /// The rectangular board perimeter (pragmatic extension beyond RFC-013 —
    /// see `ast::BoardOutline`). Projects into the IPC-2581 `Step/Profile`
    /// (RFC-015, the Quilter handoff) and into `layout.json`.
    pub board_outline: Option<BoardOutlineIr>,
    /// Locked component placements (`place <inst> at (x, y)`), resolved to IR
    /// instance paths. A placement tool treats these as pre-placed.
    pub placements: Vec<LayoutPlacement>,
}

impl LayoutIr {
    pub fn is_empty(&self) -> bool {
        self.net_classes.is_empty()
            && self.diff_pairs.is_empty()
            && self.length_matches.is_empty()
            && self.board_outline.is_none()
            && self.placements.is_empty()
    }
}

/// A board outline (RFC-020): the referenced DXF `path` (validated as a
/// project-relative string at assembly, E1006) and the geometry extracted from
/// it at `cohdl build`. `geom` is `None` until the build step resolves the file
/// (`cohdl check` never reads the DXF); the emitters use it when present.
#[derive(Debug)]
pub struct BoardOutlineIr {
    pub path: String,
    pub span: crate::span::Span,
    pub geom: Option<crate::dxf::Outline>,
}

/// A locked placement: the IR instance path, its fixed origin (`Length` values
/// in geometry range), and a closed-set rotation {0,90,180,270} — all checked
/// at assembly (E1007).
#[derive(Debug)]
pub struct LayoutPlacement {
    pub path: String,
    pub at: (UnitValue, UnitValue),
    pub rotate: u16,
    /// RFC-026: which outer face the component sits on.
    pub side: crate::ast::PlacementSide,
}

#[derive(Debug)]
pub struct LayoutNetClass {
    pub name: String,
    pub nets: Vec<String>,
}

#[derive(Debug)]
pub struct LayoutDiffPair {
    /// (positive, negative) — pair order is significant, preserved as written.
    pub p: String,
    pub n: String,
}

#[derive(Debug)]
pub struct LayoutLengthMatch {
    pub nets: Vec<String>,
    /// Opaque pass-through tolerance (never enforced by CoHDL).
    pub tolerance: Option<String>,
}

#[derive(Debug)]
pub struct IrInstance {
    /// `Board::__fn0_power_rail::c` — RFC-006 call-chain naming.
    pub path: String,
    pub device: String,
    /// RFC-008: the selected package/footprint variant (None for devices
    /// without `variants {}`). Determines the instance's pin layout and
    /// merged spec set.
    pub variant: Option<String>,
    /// Fully-concrete spec values after substitution.
    pub specs: BTreeMap<String, UnitValue>,
    /// Part binding (by-name at expansion; by-exact-match filled at build).
    pub part: Option<String>,
    pub designator_override: Option<(String, Span)>,
    /// Assigned by the allocator (RFC-005) during `build`.
    pub designator: Option<String>,
    /// RFC-013 opaque `#[placement_hint("...")]` — layout metadata for the
    /// separate `layout.json`; never influences designators or the netlist.
    pub placement_hint: Option<String>,
    /// Traits the device implements (checked impls) — used by DRC D002 and
    /// the designator-prefix rule.
    pub impl_traits: BTreeSet<String>,
    /// The source span of the `inst` statement that produced this instance
    /// (survives fn expansion — diagnostics stay precise).
    pub span: Span,
}

#[derive(Debug)]
pub struct IrNet {
    /// Emitted name (provisional §5's deterministic naming rule).
    pub name: String,
    pub voltage: Option<UnitValue>,
    pub is_gnd: bool,
    /// (instance path, logical pin name), deduplicated and sorted.
    pub members: BTreeSet<(String, String)>,
    /// Span of the first (in source order) contributing declaration.
    pub span: Span,
}

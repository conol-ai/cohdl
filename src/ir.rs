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

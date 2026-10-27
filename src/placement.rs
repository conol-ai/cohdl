//! Shared placement composition for subdesigns and RFC-034 supplier mappings.
//!
//! The authoring frame is +y-down (KiCad's board frame); [`crate::trig::rotate`]
//! is counter-clockwise in the +y-up IPC frame, so a board-frame rotation uses
//! its inverse angle. Coordinates remain exact femto-mm integers throughout.

use crate::ast::PlacementSide;
use crate::span::Span;
use crate::units::{UnitType, UnitValue};

/// The data half of a `place` statement (target resolved separately).
#[derive(Clone)]
pub struct PlaceData {
    pub at: (UnitValue, UnitValue),
    pub rotate: u16,
    pub side: PlacementSide,
    pub span: Span,
}

/// Compose an RFC-034 supplier footprint mapping with an instance placement.
///
/// RFC-034 §2 defines the mapping in the instance footprint's local frame with
/// `side: Top`, even when the instance itself is on the bottom. In particular,
/// a mapping is not a request to change the instance's side.
pub fn compose_supplier_mapping(
    instance: &PlaceData,
    at_femto: (i128, i128),
    rotate: u16,
) -> PlaceData {
    let mapping = PlaceData {
        at: (length_value(at_femto.0), length_value(at_femto.1)),
        rotate,
        side: PlacementSide::Top,
        // A supplier mapping has no source span yet; retain the instance span.
        // Subdesign composition continues to retain the child's span below.
        span: instance.span,
    };
    compose_place(instance, &mapping)
}

/// Transform a child placement, relative to a parent's origin, through the
/// parent's anchor placement. On the back side, mirror x BEFORE rotating, flip
/// the side, and REVERSE the child's own rotation (a reflection).
pub fn compose_place(parent: &PlaceData, child: &PlaceData) -> PlaceData {
    let bottom = parent.side == PlacementSide::Bottom;
    let (dx, dy) = (child.at.0.femto, child.at.1.femto);
    let dx = if bottom { -dx } else { dx };
    let inv = ((360 - (parent.rotate as u32 % 360)) % 360) as u16;
    let (rx, ry) = crate::trig::rotate(dx, dy, inv);
    let x = parent.at.0.femto + rx;
    let y = parent.at.1.femto + ry;
    let rotate = if bottom {
        ((parent.rotate as u32 + 360 - child.rotate as u32) % 360) as u16
    } else {
        ((parent.rotate as u32 + child.rotate as u32) % 360) as u16
    };
    let side = match (bottom, child.side) {
        (false, s) => s,
        (true, PlacementSide::Top) => PlacementSide::Bottom,
        (true, PlacementSide::Bottom) => PlacementSide::Top,
    };
    PlaceData {
        at: (length_value(x), length_value(y)),
        rotate,
        side,
        span: child.span,
    }
}

/// A computed `Length` value: exact femto-mm integer, canonical mm text.
pub(crate) fn length_value(femto: i128) -> UnitValue {
    UnitValue {
        unit: UnitType::Length,
        femto,
        text: format!("{}mm", crate::emit::geom::mm_femto(femto)),
    }
}

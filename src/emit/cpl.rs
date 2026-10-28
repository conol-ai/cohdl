//! RFC-034 §3.1: supplier-neutral component placement list.
//!
//! Placement coordinates stay in exact femto-mm integers.  Only the final
//! decimal projection rounds (half away from zero, to four decimal places).

use crate::ast::PlacementSide;
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::World;
use crate::span::Span;
use crate::supplier::SupplierSelection;
use std::fmt::Write as _;

const FEMTO_PER_TEN_THOUSANDTH_MM: i128 = 100_000_000_000;

pub struct UnplacedInstance {
    pub designator: String,
    pub span: Span,
}

pub struct NeutralCpl {
    pub csv: String,
    pub unplaced: Vec<UnplacedInstance>,
}

/// Build the neutral CPL whenever the design itself has at least one real
/// placement.  In particular, emitter staging positions never enter here.
fn emit_cpl(
    world: &World,
    ir: &DesignIr,
    supplier: Option<&SupplierSelection>,
) -> Option<NeutralCpl> {
    if ir.layout.placements.is_empty() {
        return None;
    }

    let origin = ir
        .layout
        .board_outline
        .as_ref()
        .and_then(|outline| outline.geom.as_ref())
        .map(|outline| {
            let extent = outline.extent();
            (extent.0 .0, extent.1 .1)
        });
    let mut rows = Vec::new();
    let mut unplaced = Vec::new();

    for inst in ir.instances.values() {
        let Some(part) = inst.part.as_ref().and_then(|name| world.parts.get(name)) else {
            continue;
        };
        let Some(footprint) = part
            .primary
            .footprint
            .as_ref()
            .and_then(|fp| world.footprints.get(&fp.name))
        else {
            continue;
        };
        // `build_artifacts` also serves --logical-only.  Such a footprint is
        // rejected later by the physical preflight, but must not reach the
        // physical pad projection's source-unspecified-hole invariant here.
        if footprint
            .mount_holes
            .iter()
            .any(|hole| hole.plating == crate::ast::MountHolePlating::SourceUnspecified)
        {
            continue;
        }
        // Use this instance's own bound footprint.  Paste apertures are SMD
        // pad plans too, but their empty number correctly excludes them.
        if !crate::emit::kicad_mod::pad_plans(world, footprint)
            .iter()
            .any(|pad| !pad.number.is_empty() && pad.kind == "smd")
        {
            continue;
        }
        let designator = inst.designator.as_deref().unwrap_or("?");
        let Some(placement) = ir
            .layout
            .placements
            .iter()
            .find(|placement| placement.path == inst.path)
        else {
            unplaced.push(UnplacedInstance {
                designator: designator.to_string(),
                span: inst.span,
            });
            continue;
        };

        let mapped = supplier.map_or_else(
            || crate::placement::PlaceData {
                at: (placement.at.0.clone(), placement.at.1.clone()),
                rotate: placement.rotate,
                side: placement.side,
                span: inst.span,
            },
            |selection| {
                let mapping = selection
                    .get(&inst.path)
                    .and_then(|instance| instance.selected.as_ref())
                    .map(|selected| selected.mapping)
                    .unwrap_or_default();
                crate::placement::compose_supplier_mapping(
                    &crate::placement::PlaceData {
                        at: (placement.at.0.clone(), placement.at.1.clone()),
                        rotate: placement.rotate,
                        side: placement.side,
                        span: inst.span,
                    },
                    mapping.at,
                    mapping.rotate,
                )
            },
        );
        let (x, y) = match origin {
            Some((left, lower_y)) => (mapped.at.0.femto - left, lower_y - mapped.at.1.femto),
            None => (mapped.at.0.femto, -mapped.at.1.femto),
        };
        let (layer, rotation) = match mapped.side {
            PlacementSide::Top => ("Top", mapped.rotate % 360),
            PlacementSide::Bottom => ("Bottom", (mapped.rotate + 180) % 360),
        };
        rows.push((
            crate::emit::designator_sort_key(designator),
            format!(
                "{},{},{},{},{}\n",
                designator,
                format_coordinate(x),
                format_coordinate(y),
                layer,
                rotation
            ),
        ));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    unplaced.sort_by(|a, b| {
        crate::emit::designator_sort_key(&a.designator)
            .cmp(&crate::emit::designator_sort_key(&b.designator))
    });
    let mut csv = String::from("Designator,Mid X,Mid Y,Layer,Rotation\n");
    for (_, row) in rows {
        csv.push_str(&row);
    }
    Some(NeutralCpl { csv, unplaced })
}

pub fn emit_neutral_cpl(world: &World, ir: &DesignIr) -> Option<NeutralCpl> {
    emit_cpl(world, ir, None)
}

/// The supplier file is the neutral row set, origin and angle convention with
/// exactly one preceding per-instance footprint-frame composition.
pub(crate) fn emit_supplier_cpl(
    world: &World,
    ir: &DesignIr,
    supplier: &SupplierSelection,
) -> Option<String> {
    emit_cpl(world, ir, Some(supplier)).map(|cpl| cpl.csv)
}

/// E815 belongs to physical build only.  The caller invokes this after the
/// physical-export preflight and only when the CPL is actually written.
pub fn report_unplaced(diags: &mut Diagnostics, cpl: &NeutralCpl) {
    if cpl.unplaced.is_empty() {
        return;
    }
    let count = cpl.unplaced.len();
    let designators = cpl
        .unplaced
        .iter()
        .map(|instance| instance.designator.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let message = if count == 1 {
        format!("1 unplaced machine-placed instance: {designators}")
    } else {
        format!("{count} unplaced machine-placed instances: {designators}")
    };
    // Unplaced instances have no placement span; anchor the design-level
    // warning to the first omitted instance's declaration instead.
    diags.push(Diagnostic::warning("E815", cpl.unplaced[0].span, message));
}

fn format_coordinate(femto: i128) -> String {
    let magnitude = femto.unsigned_abs();
    let unit = FEMTO_PER_TEN_THOUSANDTH_MM as u128;
    let scaled = (magnitude + unit / 2) / unit;
    let negative = femto < 0 && scaled != 0;
    let mut out = if negative {
        "-".to_string()
    } else {
        String::new()
    };
    let _ = write!(out, "{}.{:04}mm", scaled / 10_000, scaled % 10_000);
    out
}

#[cfg(test)]
mod tests {
    use super::format_coordinate;

    #[test]
    fn coordinates_round_half_away_from_zero_without_floats() {
        const F: i128 = 100_000_000_000;
        assert_eq!(format_coordinate(0), "0.0000mm");
        assert_eq!(format_coordinate(F / 2), "0.0001mm");
        assert_eq!(format_coordinate(F / 2 - 1), "0.0000mm");
        assert_eq!(format_coordinate(-F / 2), "-0.0001mm");
        assert_eq!(format_coordinate(-F / 2 + 1), "0.0000mm");
        assert_eq!(format_coordinate(123_450 * F), "12.3450mm");
    }
}

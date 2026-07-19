//! RFC-013 layout-constraint artifact (`layout.json`).
//!
//! A versioned, byte-stable projection of the design's layout constraints and
//! placement hints — the separate output that rides alongside the `.net`/BOM,
//! never merged into connectivity data (that is the RFC-013 zero-impact
//! guarantee). Emitted only when the design actually carries layout metadata.
//! Hand-rolled and deterministic: instances iterate in path order (BTreeMap),
//! constraints in source/collection order.

use crate::emit::json::json_str;
use crate::ir::DesignIr;
use std::fmt::Write as _;

/// Bumped only on a breaking change to this artifact's shape.
pub const SCHEMA_VERSION: u32 = 1;

/// The `layout.json` document, or `None` when the design carries no layout
/// metadata at all (no `layout {}` constraints and no `#[placement_hint]`).
pub fn emit_layout_json(ir: &DesignIr) -> Option<String> {
    // Placement hints, in instance-path order (deterministic).
    let hints: Vec<(&str, &str, &str)> = ir
        .instances
        .values()
        .filter_map(|i| {
            i.placement_hint
                .as_deref()
                .map(|h| (i.designator.as_deref().unwrap_or(""), i.path.as_str(), h))
        })
        .collect();

    if ir.layout.is_empty() && hints.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"schema_version\": {},", SCHEMA_VERSION);

    array(&mut out, "net_classes", &ir.layout.net_classes, |nc| {
        format!(
            "{{ \"name\": {}, \"nets\": [{}] }}",
            json_str(&nc.name),
            str_list(&nc.nets)
        )
    });
    array(&mut out, "diff_pairs", &ir.layout.diff_pairs, |dp| {
        format!(
            "{{ \"p\": {}, \"n\": {} }}",
            json_str(&dp.p),
            json_str(&dp.n)
        )
    });
    array(
        &mut out,
        "length_matches",
        &ir.layout.length_matches,
        |lm| {
            let tol = match &lm.tolerance {
                Some(t) => json_str(t),
                None => "null".to_string(),
            };
            format!(
                "{{ \"nets\": [{}], \"tolerance\": {} }}",
                str_list(&lm.nets),
                tol
            )
        },
    );
    // Board outline (RFC-020): the referenced DXF `source` plus the extracted
    // closed loop (start + one segment per edge). `null` when the design
    // declares none, or when it declares a path but the DXF was not resolved
    // (e.g. `check` never reads the file).
    use crate::emit::geom::mm_femto;
    match ir
        .layout
        .board_outline
        .as_ref()
        .and_then(|b| b.geom.as_ref().map(|g| (b, g)))
    {
        Some((bo, g)) => {
            let seg = |s: &crate::dxf::Seg| {
                match s {
                crate::dxf::Seg::Line { to } => format!(
                    "{{ \"type\": \"line\", \"to\": [{}, {}] }}",
                    mm_femto(to.0),
                    mm_femto(to.1)
                ),
                crate::dxf::Seg::Arc { to, center, clockwise } => format!(
                    "{{ \"type\": \"arc\", \"to\": [{}, {}], \"center\": [{}, {}], \"clockwise\": {} }}",
                    mm_femto(to.0),
                    mm_femto(to.1),
                    mm_femto(center.0),
                    mm_femto(center.1),
                    clockwise
                ),
            }
            };
            let segs: Vec<String> = g.segs.iter().map(seg).collect();
            let _ = writeln!(
                out,
                "  \"board_outline\": {{ \"source\": {}, \"start\": [{}, {}], \"segments\": [{}] }},",
                json_str(&bo.path),
                mm_femto(g.start.0),
                mm_femto(g.start.1),
                segs.join(", ")
            );
        }
        None => out.push_str("  \"board_outline\": null,\n"),
    }
    // Locked component placements (`place <inst> at … [rotate N] [side S]`),
    // path order. RFC-026: `side` is emitted only for `bottom` — a top-side
    // placement's JSON is byte-identical to its pre-RFC-026 form.
    array(&mut out, "placements", &ir.layout.placements, |p| {
        let side = match p.side {
            crate::ast::PlacementSide::Top => String::new(),
            crate::ast::PlacementSide::Bottom => ", \"side\": \"bottom\"".to_string(),
        };
        format!(
            "{{ \"instance\": {}, \"at\": [{}, {}], \"rotate\": {}{} }}",
            json_str(&p.path),
            crate::emit::geom::mm(&p.at.0),
            crate::emit::geom::mm(&p.at.1),
            p.rotate,
            side
        )
    });
    array(
        &mut out,
        "placement_hints",
        &hints,
        |(designator, instance, hint)| {
            format!(
                "{{ \"designator\": {}, \"instance\": {}, \"hint\": {} }}",
                json_str(designator),
                json_str(instance),
                json_str(hint)
            )
        },
    );

    // Trim the trailing comma left by the last array, then close.
    while out.ends_with(",\n") {
        out.truncate(out.len() - 2);
        out.push('\n');
    }
    out.push_str("}\n");
    Some(out)
}

/// Emit `"key": [ … ],` with one element per line, via `render`.
fn array<T>(out: &mut String, key: &str, items: &[T], render: impl Fn(&T) -> String) {
    if items.is_empty() {
        let _ = writeln!(out, "  \"{}\": [],", key);
        return;
    }
    let _ = writeln!(out, "  \"{}\": [", key);
    for (i, item) in items.iter().enumerate() {
        let comma = if i + 1 < items.len() { "," } else { "" };
        let _ = writeln!(out, "    {}{}", render(item), comma);
    }
    out.push_str("  ],\n");
}

/// A comma-space-separated list of JSON-escaped strings.
fn str_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| json_str(s))
        .collect::<Vec<_>>()
        .join(", ")
}

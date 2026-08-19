//! Package API documentation JSON (`cohdl docs` — docs/apidocs.md).
//!
//! A deterministic, versioned re-projection of the resolved `World` for the
//! registry's API explorer: every named top-level declaration of ONE package,
//! its impls, and the dependency-owned pads/footprints/devices its previews
//! need, inlined under `foreign`. Hand-rolled (zero external dependencies)
//! and byte-stable: inputs are BTreeMap iterations or explicitly sorted, all
//! geometry renders through the shared `geom::mm` canonical formatter, and
//! silkscreen markers are expanded through the SAME `emit::silk::graphics`
//! the `.kicad_mod`/IPC-2581 emitters consume — the preview cannot drift
//! from the shipped artifacts.
//!
//! This is a NEW artifact: emitting it never changes a verdict, diagnostic,
//! designator, or any existing artifact's bytes (pinned in tests/apidocs.rs).

use crate::ast::{
    self, FnParamTy, GenericBound, MountHoleGeom, PadDrill, PadPaste, SilkFill, SilkGraphic,
    SilkItem, SpecValue, Stmt,
};
use crate::emit::geom;
use crate::emit::json::json_str;
use crate::pipeline::Checked;
use crate::resolve::{short, World};
use crate::span::SourceMap;
use std::collections::BTreeSet;

/// Bumped only on a breaking change to this schema's *shape*.
pub const SCHEMA_VERSION: u32 = 1;

/// The `[package]` metadata recorded in the document header.
pub struct PackageMeta<'a> {
    /// The manifest name, UNSANITIZED (`@st/stm32`), as the registry knows it.
    pub name: &'a str,
    pub version: &'a str,
    pub description: Option<&'a str>,
    pub license: Option<&'a str>,
    pub repository: Option<&'a str>,
}

/// One resolved dependency, in pipeline order (std first, then name order).
pub struct DepMeta {
    /// Unsanitized registry name.
    pub name: String,
    pub version: String,
    /// Whether the dependency's `.cohdl` files live under `src/` (the
    /// published layout) — a bare directory of files (the `--std` override
    /// escape hatch) has no `src/` segment in its tar paths.
    pub src_layout: bool,
}

/// The rendered document plus the item count (for CLI reporting).
pub struct Rendered {
    pub json: String,
    pub items: usize,
}

// ---------------------------------------------------------------------------
// A minimal JSON value tree. The existing emitters build strings directly;
// this document nests deeply enough (items → footprint → silk → points) that
// explicit comma/indent bookkeeping would be the main bug surface. Field
// order is the insertion order of each `Vec` — fixed by construction — so
// output stays byte-stable.

enum Val {
    /// Pre-rendered token (numbers, booleans).
    Raw(String),
    Str(String),
    Arr(Vec<Val>),
    Obj(Vec<(&'static str, Val)>),
}

fn raw(s: impl Into<String>) -> Val {
    Val::Raw(s.into())
}

fn s(v: impl Into<String>) -> Val {
    Val::Str(v.into())
}

fn strs<I: IntoIterator<Item = String>>(items: I) -> Val {
    Val::Arr(items.into_iter().map(Val::Str).collect())
}

fn write_val(out: &mut String, v: &Val, indent: usize) {
    match v {
        Val::Raw(t) => out.push_str(t),
        Val::Str(t) => out.push_str(&json_str(t)),
        Val::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            let pad = " ".repeat(indent + 2);
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad);
                write_val(out, item, indent + 2);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            out.push_str(&" ".repeat(indent));
            out.push(']');
        }
        Val::Obj(fields) => {
            if fields.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            let pad = " ".repeat(indent + 2);
            for (i, (key, val)) in fields.iter().enumerate() {
                out.push_str(&pad);
                out.push_str(&json_str(key));
                out.push_str(": ");
                write_val(out, val, indent + 2);
                out.push_str(if i + 1 < fields.len() { ",\n" } else { "\n" });
            }
            out.push_str(&" ".repeat(indent));
            out.push('}');
        }
    }
}

// ---------------------------------------------------------------------------
// Small shared projections

fn root_of(fq: &str) -> &str {
    fq.split("::").next().unwrap_or(fq)
}

/// A file display name with `/` separators regardless of platform — the
/// same normalization `pipeline::infer_module` applies, and the separator
/// tar paths (and therefore `GET /api/doc?path=`) always use. Emitted
/// bytes must not depend on the build platform.
fn norm_display(d: &str) -> String {
    d.replace('\\', "/")
}

/// `module` = the fq path minus its last segment.
fn module_of(fq: &str) -> &str {
    fq.rsplit_once("::").map(|(m, _)| m).unwrap_or(fq)
}

fn mm(v: &crate::units::UnitValue) -> Val {
    s(geom::mm(v))
}

fn mm_pair(x: &crate::units::UnitValue, y: &crate::units::UnitValue) -> Val {
    Val::Arr(vec![mm(x), mm(y)])
}

fn mm_list(vs: &[crate::units::UnitValue]) -> Val {
    Val::Arr(vs.iter().map(mm).collect())
}

fn generics_val(generics: &[ast::GenericParam]) -> Val {
    Val::Arr(
        generics
            .iter()
            .map(|g| {
                let mut fields: Vec<(&'static str, Val)> = vec![("name", s(&g.name.name))];
                let bound = match &g.bound {
                    GenericBound::Unit(u) => Val::Obj(vec![("unit", s(u.unit.type_name()))]),
                    GenericBound::Traits(ts) => {
                        Val::Obj(vec![("traits", strs(ts.iter().map(|t| t.name.clone())))])
                    }
                };
                fields.push(("bound", bound));
                if let Some((default, _)) = &g.default {
                    fields.push(("default", s(&default.text)));
                }
                Val::Obj(fields)
            })
            .collect(),
    )
}

fn generic_arg_text(arg: &ast::GenericArg) -> String {
    match arg {
        ast::GenericArg::Unit(v, _) => v.text.clone(),
        ast::GenericArg::Name(i) => i.name.clone(),
        ast::GenericArg::Number(n, _) => n.clone(),
    }
}

/// The shared inst/call/net summary of a fn or design body.
fn body_summary(body: &[Stmt]) -> Vec<(&'static str, Val)> {
    let mut insts = Vec::new();
    let mut calls: BTreeSet<String> = BTreeSet::new();
    let mut nets = 0usize;
    for stmt in body {
        match stmt {
            Stmt::Inst(i) => {
                let mut fields: Vec<(&'static str, Val)> =
                    vec![("name", s(&i.name.name)), ("type", s(&i.ty.name.name))];
                if let Some((len, _)) = &i.array_len {
                    fields.push(("array", raw(len.to_string())));
                }
                if !i.ty.generic_args.is_empty() {
                    fields.push(("args", strs(i.ty.generic_args.iter().map(generic_arg_text))));
                }
                if let Some(v) = &i.ty.variant {
                    fields.push(("variant", s(&v.name)));
                }
                insts.push(Val::Obj(fields));
            }
            Stmt::Call(c) => {
                calls.insert(c.callee.name.clone());
            }
            Stmt::Net(_) => nets += 1,
            Stmt::Nc(_) | Stmt::Layout(_) => {}
        }
    }
    let mut out: Vec<(&'static str, Val)> = Vec::new();
    if !insts.is_empty() {
        out.push(("insts", Val::Arr(insts)));
    }
    if !calls.is_empty() {
        out.push(("calls", strs(calls)));
    }
    out.push(("nets", raw(nets.to_string())));
    out
}

fn avl_entry_val(entry: &ast::AvlEntry) -> Val {
    let mut fields: Vec<(&'static str, Val)> = vec![(
        "fields",
        Val::Arr(
            entry
                .fields
                .iter()
                .map(|f| Val::Obj(vec![("name", s(&f.name.name)), ("value", s(&f.value))]))
                .collect(),
        ),
    )];
    if let Some(fp) = &entry.footprint {
        fields.push(("footprint", s(&fp.name)));
    }
    Val::Obj(fields)
}

fn pin_rows(pins: &[ast::DevicePin]) -> Val {
    Val::Arr(
        pins.iter()
            .map(|p| {
                Val::Obj(vec![
                    ("name", s(&p.name.name)),
                    ("obligation", s(p.obligation.keyword())),
                    ("numbers", strs(p.numbers.iter().map(|n| n.text.clone()))),
                    ("role", s(p.role_or_default().name())),
                ])
            })
            .collect(),
    )
}

fn spec_rows(fields: &[&ast::DeviceSpecField]) -> Val {
    Val::Arr(
        fields
            .iter()
            .map(|f| {
                let mut row: Vec<(&'static str, Val)> = vec![("name", s(&f.name.name))];
                match &f.value {
                    SpecValue::Lit(v, _) => row.push(("value", s(&v.text))),
                    SpecValue::GenericRef(g) => row.push(("generic", s(&g.name))),
                }
                Val::Obj(row)
            })
            .collect(),
    )
}

fn courtyard_val(c: &ast::Courtyard) -> Val {
    Val::Obj(vec![
        ("shape", s(c.shape.0.name())),
        ("at", mm_pair(&c.at.0, &c.at.1)),
        ("size", mm_list(&c.size)),
    ])
}

// ---------------------------------------------------------------------------
// Kind payloads

fn trait_payload(world: &World, fq: &str, t: &ast::TraitDef) -> Val {
    let _ = (world, fq);
    let mut fields: Vec<(&'static str, Val)> = vec![(
        "super_traits",
        strs(t.super_traits.iter().map(|st| st.name.clone())),
    )];
    if let Some((prefix, _)) = &t.designator_prefix {
        fields.push(("designator_prefix", s(prefix)));
    }
    fields.push((
        "pins",
        Val::Arr(
            t.pins
                .iter()
                .map(|p| {
                    Val::Obj(vec![
                        ("name", s(&p.name.name)),
                        ("obligation", s(p.obligation.keyword())),
                    ])
                })
                .collect(),
        ),
    ));
    fields.push((
        "specs",
        Val::Arr(
            t.specs
                .iter()
                .map(|f| {
                    Val::Obj(vec![
                        ("name", s(&f.name.name)),
                        ("type", s(f.ty.unit.type_name())),
                    ])
                })
                .collect(),
        ),
    ));
    Val::Obj(fields)
}

fn device_payload(world: &World, fq: &str, d: &ast::DeviceDef) -> Val {
    let mut fields: Vec<(&'static str, Val)> = Vec::new();
    if !d.generics.is_empty() {
        fields.push(("generics", generics_val(&d.generics)));
    }
    if !d.variants.is_empty() {
        fields.push(("variants", strs(d.variants.iter().map(|v| v.name.clone()))));
    }
    fields.push(("designator_prefix", s(world.designator_prefix(fq))));

    // Render-ready per-variant views: what an instantiation of that variant
    // sees (merged specs per RFC-008), one unlabeled entry for a variant-less
    // device.
    let variants: Vec<Option<&str>> = if d.variants.is_empty() {
        vec![None]
    } else {
        d.variants.iter().map(|v| Some(v.name.as_str())).collect()
    };
    fields.push((
        "pins",
        Val::Arr(
            variants
                .iter()
                .map(|v| {
                    let mut entry: Vec<(&'static str, Val)> = Vec::new();
                    if let Some(v) = v {
                        entry.push(("variant", s(*v)));
                    }
                    entry.push(("pins", pin_rows(d.pins_for(*v))));
                    Val::Obj(entry)
                })
                .collect(),
        ),
    ));
    fields.push((
        "specs",
        Val::Arr(
            variants
                .iter()
                .map(|v| {
                    let mut entry: Vec<(&'static str, Val)> = Vec::new();
                    if let Some(v) = v {
                        entry.push(("variant", s(*v)));
                    }
                    entry.push(("fields", spec_rows(&d.spec_fields_for(*v))));
                    Val::Obj(entry)
                })
                .collect(),
        ),
    ));
    Val::Obj(fields)
}

fn fn_payload(f: &ast::FnDef) -> Val {
    let mut fields: Vec<(&'static str, Val)> = Vec::new();
    if !f.generics.is_empty() {
        fields.push(("generics", generics_val(&f.generics)));
    }
    fields.push((
        "params",
        Val::Arr(
            f.params
                .iter()
                .map(|p| {
                    let ty = match &p.ty {
                        FnParamTy::Pin(_) => Val::Obj(vec![("kind", s("pin"))]),
                        FnParamTy::Generic(g) => {
                            Val::Obj(vec![("kind", s("generic")), ("name", s(&g.name))])
                        }
                        FnParamTy::ImplTrait(ts, _) => Val::Obj(vec![
                            ("kind", s("impl")),
                            ("traits", strs(ts.iter().map(|t| t.name.clone()))),
                        ]),
                    };
                    Val::Obj(vec![("name", s(&p.name.name)), ("type", ty)])
                })
                .collect(),
        ),
    ));
    fields.extend(body_summary(&f.body));
    Val::Obj(fields)
}

fn part_payload(p: &ast::PartDef) -> Val {
    let mut fields: Vec<(&'static str, Val)> = vec![("device", s(&p.device.name.name))];
    if !p.device.generic_args.is_empty() {
        fields.push((
            "args",
            strs(p.device.generic_args.iter().map(generic_arg_text)),
        ));
    }
    if let Some(v) = &p.device.variant {
        fields.push(("variant", s(&v.name)));
    }
    fields.push(("primary", avl_entry_val(&p.primary)));
    if !p.alts.is_empty() {
        fields.push(("alts", Val::Arr(p.alts.iter().map(avl_entry_val).collect())));
    }
    Val::Obj(fields)
}

fn pad_payload(p: &ast::PadDef) -> Val {
    let mut fields: Vec<(&'static str, Val)> = Vec::new();
    if let Some((shape, _)) = &p.shape {
        fields.push(("shape", s(shape.name())));
    }
    if !p.size.is_empty() {
        fields.push(("size", mm_list(&p.size)));
    }
    if let Some((layer, _)) = &p.layer {
        fields.push(("layer", s(layer.name())));
    }
    if let Some((plating, _)) = &p.plating {
        fields.push(("plating", s(plating.name())));
    }
    if let Some((drill, _)) = &p.drill {
        let v = match drill {
            PadDrill::Round(d) => Val::Obj(vec![("round", mm(d))]),
            PadDrill::Slot(w, l) => Val::Obj(vec![("slot", Val::Arr(vec![mm(w), mm(l)]))]),
        };
        fields.push(("drill", v));
    }
    if let Some((corner, cut, _)) = &p.chamfer {
        fields.push((
            "chamfer",
            Val::Obj(vec![("corner", s(corner.name())), ("cut", mm(cut))]),
        ));
    }
    if let Some((r, _)) = &p.corner_radius {
        fields.push(("corner_radius", mm(r)));
    }
    if let Some((m, _)) = &p.mask_expansion {
        fields.push(("mask_expansion", mm(m)));
    }
    if let Some((paste, _)) = &p.paste {
        let v = match paste {
            PadPaste::None => s("none"),
            PadPaste::Rect(w, h) => Val::Obj(vec![("rect", Val::Arr(vec![mm(w), mm(h)]))]),
            PadPaste::SegmentedAnnulus(vals) => Val::Obj(vec![(
                "segmented_annulus",
                Val::Arr(vals.iter().map(mm).collect()),
            )]),
        };
        fields.push(("paste", v));
    }
    Val::Obj(fields)
}

fn silk_graphic_val(g: &SilkGraphic) -> Val {
    match g {
        SilkGraphic::Line { from, to, width } => Val::Obj(vec![
            ("kind", s("line")),
            ("from", mm_pair(&from.0, &from.1)),
            ("to", mm_pair(&to.0, &to.1)),
            ("width", mm(width)),
        ]),
        SilkGraphic::Circle {
            at,
            radius,
            width,
            fill,
        } => Val::Obj(vec![
            ("kind", s("circle")),
            ("at", mm_pair(&at.0, &at.1)),
            ("radius", mm(radius)),
            ("width", mm(width)),
            (
                "fill",
                raw(if *fill == SilkFill::Solid {
                    "true"
                } else {
                    "false"
                }),
            ),
        ]),
        SilkGraphic::Arc {
            at,
            radius,
            start_angle,
            end_angle,
            width,
        } => Val::Obj(vec![
            ("kind", s("arc")),
            ("at", mm_pair(&at.0, &at.1)),
            ("radius", mm(radius)),
            ("start_angle", raw(start_angle.to_string())),
            ("end_angle", raw(end_angle.to_string())),
            ("width", mm(width)),
        ]),
        SilkGraphic::Polygon { points, fill } => Val::Obj(vec![
            ("kind", s("polygon")),
            (
                "points",
                Val::Arr(points.iter().map(|(x, y)| mm_pair(x, y)).collect()),
            ),
            (
                "fill",
                raw(if *fill == SilkFill::Solid {
                    "true"
                } else {
                    "false"
                }),
            ),
        ]),
    }
}

fn footprint_payload(world: &World, fp: &ast::FootprintDef) -> Val {
    if crate::check::footprints::is_placeholder(fp) {
        return Val::Obj(vec![("placeholder", raw("true"))]);
    }
    let mut fields: Vec<(&'static str, Val)> = vec![("placeholder", raw("false"))];
    fields.push((
        "pads",
        Val::Arr(
            fp.pads
                .iter()
                .map(|place| {
                    let mut row: Vec<(&'static str, Val)> = vec![
                        ("number", s(&place.number.text)),
                        ("pad", s(&place.pad.name)),
                        ("x", mm(&place.x)),
                        ("y", mm(&place.y)),
                    ];
                    if place.rotate != 0 && place.rotate != u16::MAX {
                        row.push(("rotate", raw(place.rotate.to_string())));
                    }
                    Val::Obj(row)
                })
                .collect(),
        ),
    ));
    if !fp.mount_holes.is_empty() {
        fields.push((
            "mount_holes",
            Val::Arr(
                fp.mount_holes
                    .iter()
                    .map(|mh| {
                        let mut row: Vec<(&'static str, Val)> = vec![
                            ("number", s(&mh.number.text)),
                            ("plating", s(mh.plating.name())),
                            ("shape", s(mh.shape_or_default().name())),
                            ("x", mm(&mh.x)),
                            ("y", mm(&mh.y)),
                        ];
                        match &mh.geom {
                            MountHoleGeom::Diameter(d) => row.push(("diameter", mm(d))),
                            MountHoleGeom::Size(vs, _) => row.push(("size", mm_list(vs))),
                        }
                        Val::Obj(row)
                    })
                    .collect(),
            ),
        ));
    }
    if let Some(c) = &fp.courtyard {
        fields.push(("courtyard", courtyard_val(c)));
    }
    if let Some(w) = &fp.window {
        fields.push(("window", courtyard_val(w)));
    }
    if let Some((x, y, _)) = &fp.silkscreen_ref {
        fields.push(("silkscreen_ref", Val::Obj(vec![("at", mm_pair(x, y))])));
    }
    if let Some(block) = &fp.silkscreen {
        let markers: Vec<Val> = block
            .items
            .iter()
            .filter_map(|item| match item {
                SilkItem::Pin1Marker { pad, shape, .. } => Some(Val::Obj(vec![
                    ("kind", s("pin_1_marker")),
                    ("pad", s(&pad.text)),
                    (
                        "shape",
                        s(match shape {
                            ast::Pin1Shape::Dot => "dot",
                            ast::Pin1Shape::Triangle => "triangle",
                        }),
                    ),
                ])),
                SilkItem::PolarityMarker {
                    cathode_pad, shape, ..
                } => Some(Val::Obj(vec![
                    ("kind", s("polarity_marker")),
                    ("pad", s(&cathode_pad.text)),
                    (
                        "shape",
                        s(match shape {
                            ast::PolarityShape::Band => "band",
                            ast::PolarityShape::Arrow => "arrow",
                        }),
                    ),
                ])),
                SilkItem::Graphic(..) => None,
            })
            .collect();
        if !markers.is_empty() {
            fields.push(("markers", Val::Arr(markers)));
        }
    }
    // The EXPANDED graphics — the identical projection both geometry emitters
    // consume (markers already resolved to primitives with the compiler's own
    // standoff/size math).
    let silk = crate::emit::silk::graphics(world, fp);
    if !silk.is_empty() {
        fields.push((
            "silk",
            Val::Arr(silk.iter().map(silk_graphic_val).collect()),
        ));
    }
    Val::Obj(fields)
}

fn design_payload(d: &ast::DesignDef) -> Val {
    Val::Obj(body_summary(&d.body))
}

// ---------------------------------------------------------------------------
// Items

struct ItemRef<'a> {
    /// The symbol-table key: fq path, or the bare name for a design.
    fq: &'a str,
    kind: &'static str,
    is_pub: bool,
    span: crate::span::Span,
    /// The key `World.docs`/`World.intents` use (module::name — differs from
    /// `fq` only for designs, whose symbol key is bare).
    meta_key: String,
    module: String,
}

fn item_val(world: &World, sm: &SourceMap, item: &ItemRef<'_>, file: String) -> Val {
    let line = sm.line_col(item.span.file, item.span.start).line;
    let mut fields: Vec<(&'static str, Val)> = vec![
        ("fq", s(item.fq)),
        ("name", s(short(item.fq))),
        ("kind", s(item.kind)),
        ("pub", raw(if item.is_pub { "true" } else { "false" })),
        ("module", s(&item.module)),
        ("file", s(file)),
        ("line", raw(line.to_string())),
    ];
    if let Some(intent) = world.intents.get(&item.meta_key) {
        fields.push(("intent", s(intent)));
    }
    if let Some(docs) = world.docs.get(&item.meta_key) {
        fields.push(("docs", strs(docs.iter().cloned())));
    }
    let payload: Option<(&'static str, Val)> = match item.kind {
        "trait" => world
            .traits
            .get(item.fq)
            .map(|t| ("trait", trait_payload(world, item.fq, t))),
        "device" => world
            .devices
            .get(item.fq)
            .map(|d| ("device", device_payload(world, item.fq, d))),
        "fn" => world.fns.get(item.fq).map(|f| ("fn", fn_payload(f))),
        "part" => world.parts.get(item.fq).map(|p| ("part", part_payload(p))),
        "pad" => world.pads.get(item.fq).map(|p| ("pad", pad_payload(p))),
        "footprint" => world
            .footprints
            .get(item.fq)
            .map(|f| ("footprint", footprint_payload(world, f))),
        "design" => world
            .designs
            .get(item.fq)
            .map(|d| ("design", design_payload(d))),
        _ => None,
    };
    if let Some((key, val)) = payload {
        fields.push((key, val));
    }
    Val::Obj(fields)
}

/// The dependency-owned declarations this package's previews need inlined:
/// pads referenced by local footprints, footprints (plus their pads) and
/// devices referenced by local parts.
fn foreign_fqs(world: &World, root: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let add_footprint_pads = |fp: &ast::FootprintDef, out: &mut BTreeSet<String>| {
        for place in &fp.pads {
            let pad_fq = &place.pad.name;
            if root_of(pad_fq) != root && world.pads.contains_key(pad_fq) {
                out.insert(pad_fq.clone());
            }
        }
    };
    for (fq, fp) in &world.footprints {
        if root_of(fq) == root {
            add_footprint_pads(fp, &mut out);
        }
    }
    for (fq, part) in &world.parts {
        if root_of(fq) != root {
            continue;
        }
        let device_fq = &part.device.name.name;
        if root_of(device_fq) != root && world.devices.contains_key(device_fq) {
            out.insert(device_fq.clone());
        }
        for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
            let Some(fp_ref) = &entry.footprint else {
                continue;
            };
            let fp_fq = &fp_ref.name;
            if root_of(fp_fq) == root {
                continue;
            }
            let Some(fp) = world.footprints.get(fp_fq) else {
                continue;
            };
            out.insert(fp_fq.clone());
            add_footprint_pads(fp, &mut out);
        }
    }
    out
}

/// A dependency file's display name (`<dep>/rel`) as the path inside that
/// package's own published tar (`src/rel` for the ordinary layout; bare
/// `rel` for a src-less directory of files).
fn foreign_file(display: &str, deps: &[DepMeta]) -> String {
    let display = norm_display(display);
    for dep in deps {
        if let Some(rel) = display.strip_prefix(&format!("{}/", dep.name)) {
            return if dep.src_layout {
                format!("src/{rel}")
            } else {
                rel.to_string()
            };
        }
    }
    display
}

// ---------------------------------------------------------------------------

/// Render the API-docs document for the checked package. `checked` must be
/// error-free (the caller refuses to emit on `diags.has_errors()`); `deps`
/// is the resolved dependency set in pipeline order.
pub fn render(checked: &Checked, pkg: &PackageMeta<'_>, deps: &[DepMeta]) -> Rendered {
    let world = &checked.world;
    let sm = &checked.sm;
    let root = crate::pipeline::package_root(pkg.name);
    let dep_names: Vec<String> = deps.iter().map(|d| d.name.clone()).collect();

    // --- package + dependencies -------------------------------------------
    let mut package_fields: Vec<(&'static str, Val)> = vec![
        ("name", s(pkg.name)),
        ("version", s(pkg.version)),
        ("root", s(&root)),
    ];
    if let Some(d) = pkg.description {
        package_fields.push(("description", s(d)));
    }
    if let Some(l) = pkg.license {
        package_fields.push(("license", s(l)));
    }
    if let Some(r) = pkg.repository {
        package_fields.push(("repository", s(r)));
    }
    let deps_val = Val::Arr(
        deps.iter()
            .map(|d| {
                Val::Obj(vec![
                    ("name", s(&d.name)),
                    ("version", s(&d.version)),
                    ("root", s(crate::pipeline::package_root(&d.name))),
                ])
            })
            .collect(),
    );

    // --- local items (symbols are fq-sorted; designs merge by bare name) ---
    let mut refs: Vec<ItemRef<'_>> = Vec::new();
    for (fq, sym) in &world.symbols {
        if root_of(fq) != root {
            continue;
        }
        refs.push(ItemRef {
            fq,
            kind: sym.kind,
            is_pub: sym.is_pub,
            span: sym.span,
            meta_key: fq.clone(),
            module: module_of(fq).to_string(),
        });
    }
    for (name, d) in &world.designs {
        let file = norm_display(sm.name(d.name.span.file));
        if !file.starts_with("src/") {
            continue; // a dependency's design — not this package's
        }
        // Designs are project-global and never importable; the explorer
        // treats them as visible. Their docs/intent keys are moduled even
        // though the design itself is bare-named.
        let module = crate::pipeline::infer_module(&root, &dep_names, &file).module;
        refs.push(ItemRef {
            fq: name,
            kind: "design",
            is_pub: true,
            span: d.name.span,
            meta_key: format!("{}::{}", module, name),
            module,
        });
    }
    refs.sort_by(|a, b| a.fq.cmp(b.fq));
    let items: Vec<Val> = refs
        .iter()
        .map(|r| item_val(world, sm, r, norm_display(sm.name(r.span.file))))
        .collect();
    let item_count = items.len();

    // --- impls declared in this package -----------------------------------
    let mut impl_entries: Vec<&ast::ImplDef> = world
        .impls
        .iter()
        .filter(|im| norm_display(sm.name(im.span.file)).starts_with("src/"))
        .collect();
    impl_entries.sort_by(|a, b| {
        (&a.trait_name.name, &a.device_name.name).cmp(&(&b.trait_name.name, &b.device_name.name))
    });
    let impls_val = Val::Arr(
        impl_entries
            .iter()
            .map(|im| {
                let line = sm.line_col(im.span.file, im.span.start).line;
                let mut fields: Vec<(&'static str, Val)> = vec![
                    ("trait", s(&im.trait_name.name)),
                    ("device", s(&im.device_name.name)),
                    ("file", s(norm_display(sm.name(im.span.file)))),
                    ("line", raw(line.to_string())),
                ];
                if let Some(resolved) = world
                    .resolved_impls
                    .get(&(im.trait_name.name.clone(), im.device_name.name.clone()))
                {
                    if !resolved.pin_map.is_empty() {
                        fields.push((
                            "pin_map",
                            Val::Arr(
                                resolved
                                    .pin_map
                                    .iter()
                                    .map(|(role, pin)| {
                                        Val::Obj(vec![("role", s(role)), ("pin", s(pin))])
                                    })
                                    .collect(),
                            ),
                        ));
                    }
                    if !resolved.spec_map.is_empty() {
                        fields.push((
                            "spec_map",
                            Val::Arr(
                                resolved
                                    .spec_map
                                    .iter()
                                    .map(|(field, spec)| {
                                        Val::Obj(vec![("field", s(field)), ("spec", s(spec))])
                                    })
                                    .collect(),
                            ),
                        ));
                    }
                }
                Val::Obj(fields)
            })
            .collect(),
    );

    // --- foreign items -----------------------------------------------------
    let foreign_val = Val::Arr(
        foreign_fqs(world, &root)
            .iter()
            .filter_map(|fq| {
                let sym = world.symbols.get(fq)?;
                let item = ItemRef {
                    fq,
                    kind: sym.kind,
                    is_pub: sym.is_pub,
                    span: sym.span,
                    meta_key: fq.clone(),
                    module: module_of(fq).to_string(),
                };
                let file = foreign_file(sm.name(sym.span.file), deps);
                Some(item_val(world, sm, &item, file))
            })
            .collect(),
    );

    // --- document ----------------------------------------------------------
    let doc = Val::Obj(vec![
        ("schema_version", raw(SCHEMA_VERSION.to_string())),
        (
            "generator",
            s(format!("cohdl {}", env!("CARGO_PKG_VERSION"))),
        ),
        ("package", Val::Obj(package_fields)),
        ("dependencies", deps_val),
        ("items", Val::Arr(items)),
        ("impls", impls_val),
        ("foreign", foreign_val),
    ]);
    let mut json = String::new();
    write_val(&mut json, &doc, 0);
    json.push('\n');
    Rendered {
        json,
        items: item_count,
    }
}

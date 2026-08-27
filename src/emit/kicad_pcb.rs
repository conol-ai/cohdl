//! Native `.kicad_pcb` board emitter — `build --emit kicad_pcb`.
//!
//! Writes a KiCad 10 board file (format `(version 20260206)`) directly from
//! the checked IR: every instance's footprint embedded with its pads bound
//! to nets by name, placements applied in KiCad's own board frame, and the
//! RFC-020 board outline drawn on Edge.Cuts. This replaces the
//! pcbnew-scripted `tools/kicad_board.py` flow — no KiCad installation is
//! consulted, and the output is byte-stable.
//!
//! Frame: CoHDL's authoring frame IS KiCad's board frame (+y down), so
//! placement and footprint-local coordinates pass through verbatim — the
//! IPC-2581 emitter's y-negation is that document's +y-up requirement, not
//! this one's. Rotation math is exact (integer fast paths; the file stores
//! angles, never rotated coordinates).
//!
//! RFC-026 back side, in KiCad's on-disk representation (pinned empirically
//! against pcbnew-written boards): a `side: bottom` instance keeps its
//! authored (x, y), takes `(layer "B.Cu")`, and its footprint-local content
//! is stored Y-NEGATED with 180° folded into the angle — KiCad's canonical
//! decomposition of the left/right flip (x-mirror = y-mirror + 180°).
//! Concretely: footprint angle = authored rotation + 180 (normalized to
//! (-180, 180]); every local y (pads, graphics, texts) negates; every F.*
//! layer becomes B.*; texts gain `(justify mirror)`; a pad's own RFC-025
//! rotation REVERSES (reflection: R·Mirror·r = (R-r)·Mirror); an asymmetric
//! chamfer corner swaps vertically. The absolute pad delta this encodes is
//! Rot(fp_angle)·(lx, -ly) — for a pad at local (1, 2) on an unrotated
//! bottom part: Rot(180)·(1, -2) = (-1, 2), the LEFT_RIGHT flip the 82%
//! Quilter failure taught us to verify empirically.
//!
//! Geometry derivation (pad plans, body graphics, silkscreen expansion) is
//! shared with the `.kicad_mod` emitter — one derivation, two dialects, so
//! the two projections cannot drift.

use crate::emit::geom;
use crate::emit::kicad_mod::{
    body_graphics, chamfer_xy_list, file_base, footprint_attr, pad_plans, paste_xy_list,
    BodyGraphic, DrillPlan, GfxLayer, LayerSet, PadPlan, PlanBody,
};
use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::BTreeMap;
use std::fmt::Write as _;

const FEMTO_MM: i128 = 1_000_000_000_000_000;

/// File header through the end of `(setup …)`: the block KiCad 10 writes for
/// a board of ours, verbatim (generator identity aside). `generator_version`
/// declares the format generation targeted, fixed for byte stability.
const HEADER: &str = r#"(kicad_pcb
	(version 20260206)
	(generator "cohdl")
	(generator_version "10.0")
	(general
		(thickness 1.6)
		(legacy_teardrops no)
	)
	(paper "A4")
	(layers
		(0 "F.Cu" signal)
		(2 "B.Cu" signal)
		(9 "F.Adhes" user "F.Adhesive")
		(11 "B.Adhes" user "B.Adhesive")
		(13 "F.Paste" user)
		(15 "B.Paste" user)
		(5 "F.SilkS" user "F.Silkscreen")
		(7 "B.SilkS" user "B.Silkscreen")
		(1 "F.Mask" user)
		(3 "B.Mask" user)
		(17 "Dwgs.User" user "User.Drawings")
		(19 "Cmts.User" user "User.Comments")
		(21 "Eco1.User" user "User.Eco1")
		(23 "Eco2.User" user "User.Eco2")
		(25 "Edge.Cuts" user)
		(27 "Margin" user)
		(31 "F.CrtYd" user "F.Courtyard")
		(29 "B.CrtYd" user "B.Courtyard")
		(35 "F.Fab" user)
		(33 "B.Fab" user)
		(39 "User.1" user)
		(41 "User.2" user)
		(43 "User.3" user)
		(45 "User.4" user)
	)
	(setup
		(pad_to_mask_clearance 0)
		(allow_soldermask_bridges_in_footprints no)
		(tenting
			(front yes)
			(back yes)
		)
		(covering
			(front no)
			(back no)
		)
		(plugging
			(front no)
			(back no)
		)
		(capping no)
		(filling no)
		(pcbplotparams
			(layerselection 0x00000000_00000000_55555555_5755f5ff)
			(plot_on_all_layers_selection 0x00000000_00000000_00000000_00000000)
			(disableapertmacros no)
			(usegerberextensions no)
			(usegerberattributes yes)
			(usegerberadvancedattributes yes)
			(creategerberjobfile yes)
			(dashed_line_dash_ratio 12)
			(dashed_line_gap_ratio 3)
			(svgprecision 4)
			(plotframeref no)
			(mode 1)
			(useauxorigin no)
			(pdf_front_fp_property_popups yes)
			(pdf_back_fp_property_popups yes)
			(pdf_metadata yes)
			(pdf_single_document no)
			(dxfpolygonmode yes)
			(dxfimperialunits yes)
			(dxfusepcbnewfont yes)
			(psnegative no)
			(psa4output no)
			(plot_black_and_white yes)
			(sketchpadsonfab no)
			(plotpadnumbers no)
			(hidednponfab no)
			(sketchdnponfab yes)
			(crossoutdnponfab yes)
			(subtractmaskfromsilk no)
			(outputformat 1)
			(mirror no)
			(drillshape 1)
			(scaleselection 1)
			(outputdirectory "")
		)
	)
"#;

pub fn emit_kicad_pcb(world: &World, ir: &DesignIr, package_name: &str) -> String {
    let insts = crate::emit::ipc2581::sorted_instances(ir);
    let placed = crate::emit::ipc2581::placed_positions(ir);
    let staged = staging(world, ir, &insts, &placed);
    let rotations: BTreeMap<&str, u16> = ir
        .layout
        .placements
        .iter()
        .map(|p| (p.path.as_str(), p.rotate))
        .collect();
    let bottoms: BTreeMap<&str, bool> = ir
        .layout
        .placements
        .iter()
        .map(|p| (p.path.as_str(), p.side == crate::ast::PlacementSide::Bottom))
        .collect();
    let nets = net_map(world, ir);

    let mut s = String::new();
    s.push_str(HEADER);
    for inst in &insts {
        let refdes = inst.designator.as_deref().unwrap_or("?");
        let fq = crate::emit::ipc2581::part_fields(world, inst).2;
        let (x, y) = placed
            .get(&inst.path)
            .or_else(|| staged.get(&inst.path))
            .copied()
            .unwrap_or((0, 0));
        let rot = i32::from(rotations.get(inst.path.as_str()).copied().unwrap_or(0));
        let bottom = bottoms.get(inst.path.as_str()).copied().unwrap_or(false);
        let fp_angle = norm180(rot + if bottom { 180 } else { 0 });
        let value = crate::emit::kicad::principal_value(world, inst);
        let fp = world.footprints.get(&fq);

        let _ = writeln!(s, "\t(footprint {}", quote(&file_base(&fq)));
        let _ = writeln!(s, "\t\t(layer \"{}.Cu\")", if bottom { "B" } else { "F" });
        let _ = writeln!(s, "\t\t(uuid \"{}\")", uuid(package_name, refdes, "fp", 0));
        let _ = writeln!(
            s,
            "\t\t(at {} {}{})",
            geom::mm_femto(x),
            geom::mm_femto(y),
            angle_suffix(fp_angle)
        );
        // The four properties KiCad expects, in its order. Their angle tracks
        // the footprint's rotation; on the back the text representation adds
        // its own 180 (the flip fold-in cancels back out to the authored
        // rotation) and mirrors its justification.
        let prop_angle = norm360(fp_angle + if bottom { 180 } else { 0 });
        let ref_anchor = fp
            .and_then(|f| f.silkscreen_ref.as_ref())
            .map(|(ax, ay, _)| (ax.femto, ay.femto))
            .unwrap_or((0, 0));
        let ref_at = (ref_anchor.0, flip_y(ref_anchor.1, bottom));
        property(
            &mut s,
            package_name,
            refdes,
            "Reference",
            refdes,
            ref_at,
            prop_angle,
            if bottom { "B.SilkS" } else { "F.SilkS" },
            false,
            bottom,
        );
        let fab = if bottom { "B.Fab" } else { "F.Fab" };
        property(
            &mut s,
            package_name,
            refdes,
            "Value",
            &value,
            (0, 0),
            prop_angle,
            fab,
            false,
            bottom,
        );
        property(
            &mut s,
            package_name,
            refdes,
            "Datasheet",
            "",
            (0, 0),
            prop_angle,
            fab,
            true,
            bottom,
        );
        property(
            &mut s,
            package_name,
            refdes,
            "Description",
            "",
            (0, 0),
            prop_angle,
            fab,
            true,
            bottom,
        );
        let attr = fp.map(|f| footprint_attr(world, f)).unwrap_or("smd");
        let _ = writeln!(s, "\t\t(attr {})", attr);
        s.push_str("\t\t(duplicate_pad_numbers_are_jumpers no)\n");
        if let Some(fp) = fp {
            for (i, g) in body_graphics(world, fp).into_iter().enumerate() {
                graphic(&mut s, package_name, refdes, i, g, bottom);
            }
            for (i, p) in pad_plans(world, fp).into_iter().enumerate() {
                pad(&mut s, package_name, refdes, i, &p, fp_angle, bottom, &nets);
            }
        }
        s.push_str("\t\t(embedded_fonts no)\n");
        s.push_str("\t)\n");
    }
    outline(&mut s, ir, package_name);
    s.push_str("\t(embedded_fonts no)\n");
    s.push_str(")\n");
    s
}

// ---------------------------------------------------------------------------
// Angles and the back-side transform
// ---------------------------------------------------------------------------

/// Footprint-level angle convention: pcbnew serializes into (-180, 180].
fn norm180(deg: i32) -> i32 {
    let d = deg.rem_euclid(360);
    if d > 180 {
        d - 360
    } else {
        d
    }
}

/// Pad/property angle convention: [0, 360).
fn norm360(deg: i32) -> i32 {
    deg.rem_euclid(360)
}

/// The third `(at …)` argument — omitted when the angle is 0.
fn angle_suffix(deg: i32) -> String {
    if deg != 0 {
        format!(" {}", deg)
    } else {
        String::new()
    }
}

fn flip_y(y: i128, bottom: bool) -> i128 {
    if bottom {
        -y
    } else {
        y
    }
}

/// The vertical chamfer-corner swap of the on-disk y-mirror representation
/// (the horizontal component of the authored left/right flip lives in the
/// folded-in 180°).
fn vswap(c: crate::ast::PadCorner) -> crate::ast::PadCorner {
    use crate::ast::PadCorner::*;
    match c {
        TopLeft => BottomLeft,
        TopRight => BottomRight,
        BottomLeft => TopLeft,
        BottomRight => TopRight,
    }
}

// ---------------------------------------------------------------------------
// Nets
// ---------------------------------------------------------------------------

/// (designator, physical pad number) → net name. One logical pin fans out to
/// every one of its physical pad numbers (RFC-008 variant-selected), and in
/// the board every same-numbered pad copy (exposed land + thermal vias)
/// takes the net through this same lookup. `nc` pins are absent — a pad with
/// no row simply omits its `(net …)` clause, KiCad's own representation.
fn net_map<'a>(world: &World, ir: &'a DesignIr) -> BTreeMap<(String, String), &'a str> {
    let mut out = BTreeMap::new();
    for net in &ir.nets {
        for (path, pin) in &net.members {
            let inst = &ir.instances[path];
            let refdes = inst.designator.as_deref().unwrap_or("?");
            let device = &world.devices[&inst.device];
            if let Some(dev_pin) = device
                .pins_for(inst.variant.as_deref())
                .iter()
                .find(|p| p.name.name == *pin)
            {
                for num in &dev_pin.numbers {
                    out.insert((refdes.to_string(), num.text.clone()), net.name.as_str());
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Footprint children
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn property(
    s: &mut String,
    pkg: &str,
    refdes: &str,
    name: &str,
    value: &str,
    at: (i128, i128),
    angle: i32,
    layer: &str,
    hide: bool,
    mirror: bool,
) {
    let _ = writeln!(s, "\t\t(property {} {}", quote(name), quote(value));
    let _ = writeln!(
        s,
        "\t\t\t(at {} {} {})",
        geom::mm_femto(at.0),
        geom::mm_femto(at.1),
        angle
    );
    let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer);
    if hide {
        s.push_str("\t\t\t(hide yes)\n");
    }
    let _ = writeln!(s, "\t\t\t(uuid \"{}\")", uuid(pkg, refdes, name, 0));
    s.push_str("\t\t\t(effects\n\t\t\t\t(font\n\t\t\t\t\t(size 1.27 1.27)\n\t\t\t\t)\n");
    if mirror {
        s.push_str("\t\t\t\t(justify mirror)\n");
    }
    s.push_str("\t\t\t)\n\t\t)\n");
}

/// One footprint body graphic in the board dialect. Coordinates arrive at
/// 10^-16 mm; a bottom-side instance negates local y (the mirror's stored
/// half) and moves every front layer to its back counterpart.
fn graphic(s: &mut String, pkg: &str, refdes: &str, ordinal: usize, g: BodyGraphic, bottom: bool) {
    let g16 = |v: i128| geom::render(v, 16);
    let fy = |v: i128| if bottom { -v } else { v };
    let layer = |l: GfxLayer| match (l, bottom) {
        (GfxLayer::Silk, false) => "F.SilkS",
        (GfxLayer::Silk, true) => "B.SilkS",
        (GfxLayer::Courtyard, false) => "F.CrtYd",
        (GfxLayer::Courtyard, true) => "B.CrtYd",
        (GfxLayer::EdgeCuts, _) => "Edge.Cuts",
    };
    let fill = |f: bool| if f { "yes" } else { "no" };
    let id = uuid(pkg, refdes, "gfx", ordinal);
    match g {
        BodyGraphic::Line {
            from,
            to,
            width,
            layer: l,
        } => {
            let _ = writeln!(s, "\t\t(fp_line");
            let _ = writeln!(s, "\t\t\t(start {} {})", g16(from.0), g16(fy(from.1)));
            let _ = writeln!(s, "\t\t\t(end {} {})", g16(to.0), g16(fy(to.1)));
            stroke(s, width, "solid");
            let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer(l));
            let _ = writeln!(s, "\t\t\t(uuid \"{}\")", id);
            s.push_str("\t\t)\n");
        }
        BodyGraphic::Circle {
            center,
            end,
            width,
            fill: f,
            layer: l,
        } => {
            let _ = writeln!(s, "\t\t(fp_circle");
            let _ = writeln!(s, "\t\t\t(center {} {})", g16(center.0), g16(fy(center.1)));
            let _ = writeln!(s, "\t\t\t(end {} {})", g16(end.0), g16(fy(end.1)));
            stroke(s, width, "solid");
            let _ = writeln!(s, "\t\t\t(fill {})", fill(f));
            let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer(l));
            let _ = writeln!(s, "\t\t\t(uuid \"{}\")", id);
            s.push_str("\t\t)\n");
        }
        BodyGraphic::Arc {
            start,
            mid,
            end,
            width,
            layer: l,
        } => {
            let _ = writeln!(s, "\t\t(fp_arc");
            let _ = writeln!(s, "\t\t\t(start {} {})", g16(start.0), g16(fy(start.1)));
            let _ = writeln!(s, "\t\t\t(mid {} {})", g16(mid.0), g16(fy(mid.1)));
            let _ = writeln!(s, "\t\t\t(end {} {})", g16(end.0), g16(fy(end.1)));
            stroke(s, width, "solid");
            let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer(l));
            let _ = writeln!(s, "\t\t\t(uuid \"{}\")", id);
            s.push_str("\t\t)\n");
        }
        BodyGraphic::Rect {
            start,
            end,
            width,
            fill: f,
            layer: l,
        } => {
            let _ = writeln!(s, "\t\t(fp_rect");
            let _ = writeln!(s, "\t\t\t(start {} {})", g16(start.0), g16(fy(start.1)));
            let _ = writeln!(s, "\t\t\t(end {} {})", g16(end.0), g16(fy(end.1)));
            stroke(s, width, "solid");
            let _ = writeln!(s, "\t\t\t(fill {})", fill(f));
            let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer(l));
            let _ = writeln!(s, "\t\t\t(uuid \"{}\")", id);
            s.push_str("\t\t)\n");
        }
        BodyGraphic::Poly {
            points,
            width,
            fill: f,
            layer: l,
        } => {
            let pts: Vec<String> = points
                .iter()
                .map(|(x, y)| format!("(xy {} {})", g16(*x), g16(fy(*y))))
                .collect();
            let _ = writeln!(s, "\t\t(fp_poly");
            s.push_str("\t\t\t(pts\n");
            let _ = writeln!(s, "\t\t\t\t{}", pts.join(" "));
            s.push_str("\t\t\t)\n");
            stroke(s, width, "solid");
            let _ = writeln!(s, "\t\t\t(fill {})", fill(f));
            let _ = writeln!(s, "\t\t\t(layer \"{}\")", layer(l));
            let _ = writeln!(s, "\t\t\t(uuid \"{}\")", id);
            s.push_str("\t\t)\n");
        }
    }
}

fn stroke(s: &mut String, width: i128, kind: &str) {
    let _ = writeln!(
        s,
        "\t\t\t(stroke\n\t\t\t\t(width {})\n\t\t\t\t(type {})\n\t\t\t)",
        geom::mm_femto(width),
        kind
    );
}

fn board_layers(set: LayerSet, bottom: bool) -> &'static str {
    let set = if bottom {
        match set {
            LayerSet::Front => LayerSet::Back,
            LayerSet::FrontNoPaste => LayerSet::BackNoPaste,
            LayerSet::Back => LayerSet::Front,
            LayerSet::BackNoPaste => LayerSet::FrontNoPaste,
            LayerSet::PasteFront => LayerSet::PasteBack,
            LayerSet::PasteBack => LayerSet::PasteFront,
            LayerSet::CuMask => LayerSet::CuMask,
        }
    } else {
        set
    };
    // Board files list pad layers in KiCad's canonical Cu/Mask/Paste order
    // (the .kicad_mod dialect's Cu/Paste/Mask is equally valid on load; this
    // matches what pcbnew itself writes for a board).
    match set {
        LayerSet::CuMask => "\"*.Cu\" \"*.Mask\"",
        LayerSet::Front => "\"F.Cu\" \"F.Mask\" \"F.Paste\"",
        LayerSet::FrontNoPaste => "\"F.Cu\" \"F.Mask\"",
        LayerSet::Back => "\"B.Cu\" \"B.Mask\" \"B.Paste\"",
        LayerSet::BackNoPaste => "\"B.Cu\" \"B.Mask\"",
        LayerSet::PasteFront => "\"F.Paste\"",
        LayerSet::PasteBack => "\"B.Paste\"",
    }
}

#[allow(clippy::too_many_arguments)]
fn pad(
    s: &mut String,
    pkg: &str,
    refdes: &str,
    ordinal: usize,
    p: &PadPlan,
    fp_angle: i32,
    bottom: bool,
    nets: &BTreeMap<(String, String), &str>,
) {
    // RFC-025 composed with the flip: a reflection reverses a pad-local
    // rotation, and the file's pad angle is absolute (footprint + local).
    let local = i32::from(p.rotate);
    let pad_angle = norm360(fp_angle + if bottom { -local } else { local });
    let _ = writeln!(s, "\t\t(pad {} {} {}", quote(&p.number), p.kind, p.kshape);
    let _ = writeln!(
        s,
        "\t\t\t(at {} {}{})",
        geom::mm_femto(p.x),
        geom::mm_femto(flip_y(p.y, bottom)),
        angle_suffix(pad_angle)
    );
    let _ = writeln!(
        s,
        "\t\t\t(size {} {})",
        geom::mm_femto(p.size.0),
        geom::mm_femto(p.size.1)
    );
    match &p.drill {
        Some(DrillPlan::Round(v)) => {
            let _ = writeln!(s, "\t\t\t(drill {})", geom::mm_femto(*v));
        }
        Some(DrillPlan::Slot(w, l)) => {
            let _ = writeln!(
                s,
                "\t\t\t(drill oval {} {})",
                geom::mm_femto(*w),
                geom::mm_femto(*l)
            );
        }
        None => {}
    }
    let _ = writeln!(s, "\t\t\t(layers {})", board_layers(p.layers, bottom));
    if p.kind == "thru_hole" {
        s.push_str("\t\t\t(remove_unused_layers no)\n");
    }
    if let Some((ratio, corner)) = &p.chamfer {
        let corner = if bottom { vswap(*corner) } else { *corner };
        s.push_str("\t\t\t(roundrect_rratio 0)\n");
        let _ = writeln!(s, "\t\t\t(chamfer_ratio {})", ratio);
        let _ = writeln!(s, "\t\t\t(chamfer {})", corner.name());
    }
    if let Some(ratio) = &p.corner_radius {
        let _ = writeln!(s, "\t\t\t(roundrect_rratio {})", ratio);
    }
    if !p.number.is_empty() {
        if let Some(net) = nets.get(&(refdes.to_string(), p.number.clone())) {
            let _ = writeln!(s, "\t\t\t(net {})", quote(net));
        }
    }
    if let Some(m) = p.mask_expansion {
        let _ = writeln!(s, "\t\t\t(solder_mask_margin {})", geom::mm_femto(m));
    }
    match &p.body {
        PlanBody::Flat => {}
        PlanBody::AnnulusRing { mid_radius, stroke } => {
            s.push_str(
                "\t\t\t(options\n\t\t\t\t(clearance outline)\n\t\t\t\t(anchor circle)\n\t\t\t)\n",
            );
            s.push_str("\t\t\t(primitives\n\t\t\t\t(gr_circle\n");
            s.push_str("\t\t\t\t\t(center 0 0)\n");
            let _ = writeln!(s, "\t\t\t\t\t(end {} 0)", geom::mm_femto(*mid_radius));
            let _ = writeln!(s, "\t\t\t\t\t(width {})", geom::mm_femto(*stroke));
            s.push_str("\t\t\t\t\t(fill no)\n\t\t\t\t)\n\t\t\t)\n");
        }
        PlanBody::ChamferPoly { points } => {
            let points: Vec<(i128, i128)> = points
                .iter()
                .map(|(x, y)| (*x, flip_y(*y, bottom)))
                .collect();
            s.push_str(
                "\t\t\t(options\n\t\t\t\t(clearance outline)\n\t\t\t\t(anchor rect)\n\t\t\t)\n",
            );
            s.push_str("\t\t\t(primitives\n\t\t\t\t(gr_poly\n\t\t\t\t\t(pts\n");
            let _ = writeln!(s, "\t\t\t\t\t\t{}", chamfer_xy_list(&points));
            s.push_str(
                "\t\t\t\t\t)\n\t\t\t\t\t(width 0)\n\t\t\t\t\t(fill yes)\n\t\t\t\t)\n\t\t\t)\n",
            );
        }
        PlanBody::PastePoly { points } => {
            let points: Vec<(i128, i128)> = points
                .iter()
                .map(|(x, y)| (*x, flip_y(*y, bottom)))
                .collect();
            s.push_str(
                "\t\t\t(options\n\t\t\t\t(clearance outline)\n\t\t\t\t(anchor circle)\n\t\t\t)\n",
            );
            s.push_str("\t\t\t(primitives\n\t\t\t\t(gr_poly\n\t\t\t\t\t(pts\n");
            let _ = writeln!(s, "\t\t\t\t\t\t{}", paste_xy_list(&points));
            s.push_str(
                "\t\t\t\t\t)\n\t\t\t\t\t(width 0)\n\t\t\t\t\t(fill yes)\n\t\t\t\t)\n\t\t\t)\n",
            );
        }
    }
    let _ = writeln!(s, "\t\t\t(uuid \"{}\")", uuid(pkg, refdes, "pad", ordinal));
    s.push_str("\t\t)\n");
}

// ---------------------------------------------------------------------------
// Board outline (RFC-020 DXF geometry, authoring frame = KiCad frame)
// ---------------------------------------------------------------------------

fn outline(s: &mut String, ir: &DesignIr, pkg: &str) {
    let Some(g) = ir
        .layout
        .board_outline
        .as_ref()
        .and_then(|b| b.geom.as_ref())
    else {
        return;
    };
    let width = FEMTO_MM / 10; // 0.1 mm — the established Edge.Cuts stroke
    let mut prev = g.start;
    for (i, seg) in g.segs.iter().enumerate() {
        let id = uuid(pkg, "__outline", "seg", i);
        match seg {
            crate::dxf::Seg::Line { to } => {
                let _ = writeln!(s, "\t(gr_line");
                let _ = writeln!(
                    s,
                    "\t\t(start {} {})",
                    geom::mm_femto(prev.0),
                    geom::mm_femto(prev.1)
                );
                let _ = writeln!(
                    s,
                    "\t\t(end {} {})",
                    geom::mm_femto(to.0),
                    geom::mm_femto(to.1)
                );
                outline_stroke(s, width);
                s.push_str("\t\t(layer \"Edge.Cuts\")\n");
                let _ = writeln!(s, "\t\t(uuid \"{}\")", id);
                s.push_str("\t)\n");
                prev = *to;
            }
            crate::dxf::Seg::Arc {
                to,
                center,
                clockwise,
            } => {
                let (mx, my) = arc_mid(prev, *to, *center, *clockwise);
                let _ = writeln!(s, "\t(gr_arc");
                let _ = writeln!(
                    s,
                    "\t\t(start {} {})",
                    geom::mm_femto(prev.0),
                    geom::mm_femto(prev.1)
                );
                let _ = writeln!(s, "\t\t(mid {} {})", geom::mm_femto(mx), geom::mm_femto(my));
                let _ = writeln!(
                    s,
                    "\t\t(end {} {})",
                    geom::mm_femto(to.0),
                    geom::mm_femto(to.1)
                );
                outline_stroke(s, width);
                s.push_str("\t\t(layer \"Edge.Cuts\")\n");
                let _ = writeln!(s, "\t\t(uuid \"{}\")", id);
                s.push_str("\t)\n");
                prev = *to;
            }
        }
    }
}

fn outline_stroke(s: &mut String, width: i128) {
    let _ = writeln!(
        s,
        "\t\t(stroke\n\t\t\t(width {})\n\t\t\t(type default)\n\t\t)",
        geom::mm_femto(width)
    );
}

/// The KiCad arc midpoint: the point on the arc halfway along its sweep,
/// honoring the DXF winding (an arc may exceed 180°, where the naive radii
/// bisector lands on the wrong side). Float trigonometry is confined to
/// this tessellation step — the silk fp_arc precedent — and rounds ONCE to
/// nanometers (KiCad's own internal resolution), so the last-bit noise of a
/// platform's libm sits five orders of magnitude below the rounding step.
fn arc_mid(
    from: (i128, i128),
    to: (i128, i128),
    center: (i128, i128),
    clockwise: bool,
) -> (i128, i128) {
    let fx = (from.0 - center.0) as f64;
    let fy = (from.1 - center.1) as f64;
    let tx = (to.0 - center.0) as f64;
    let ty = (to.1 - center.1) as f64;
    let r = fx.hypot(fy);
    let a0 = fy.atan2(fx);
    let a1 = ty.atan2(tx);
    // `clockwise` is the DXF bulge direction in the DXF's mathematical
    // (+y-up) coordinate system.  The numeric coordinates pass through to
    // KiCad unchanged, so choose the midpoint in that same coordinate system;
    // KiCad's +y-down display supplies the visual handedness flip itself.
    let tau = std::f64::consts::TAU;
    let sweep = if clockwise {
        let mut d = a1 - a0;
        if d >= 0.0 {
            d -= tau;
        }
        d
    } else {
        let mut d = a1 - a0;
        if d <= 0.0 {
            d += tau;
        }
        d
    };
    let am = a0 + sweep / 2.0;
    const NM: f64 = 1_000_000.0; // femto per nanometer
    let round_nm = |v: f64| ((v / NM).round() as i128) * 1_000_000;
    (
        center.0 + round_nm(r * am.cos()),
        center.1 + round_nm(r * am.sin()),
    )
}

// ---------------------------------------------------------------------------
// Staging for unplaced instances
// ---------------------------------------------------------------------------

/// Where unplaced instances go. With a board outline: the same shelf-packed
/// grid just outside it that the IPC-2581 document stages (one convention,
/// two emitters). Without one there is nothing to stage against, and
/// stacking everything at (0, 0) — the IPC placeholder — would overlap and
/// short every footprint in a board file, so fall back to the plain staging
/// grid the retired pcbnew script used: 12 mm pitch from (40, 40), eight
/// per row, in designator order.
fn staging(
    world: &World,
    ir: &DesignIr,
    insts: &[&crate::ir::IrInstance],
    placed: &BTreeMap<String, (i128, i128)>,
) -> BTreeMap<String, (i128, i128)> {
    let shelf = crate::emit::ipc2581::staging_positions(world, ir, insts, placed);
    if !shelf.is_empty() || insts.iter().all(|i| placed.contains_key(&i.path)) {
        return shelf;
    }
    let mut out = BTreeMap::new();
    let (mut col, mut row) = (0i128, 0i128);
    for inst in insts {
        if placed.contains_key(&inst.path) {
            continue;
        }
        out.insert(
            inst.path.clone(),
            ((40 + col * 12) * FEMTO_MM, (40 + row * 12) * FEMTO_MM),
        );
        col += 1;
        if col == 8 {
            col = 0;
            row += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Deterministic identity
// ---------------------------------------------------------------------------

/// Deterministic RFC-4122-shaped uuid from stable identity, so the emitted
/// board is byte-stable across builds (pcbnew's own uuids are random v4 —
/// the one part of its output a reproducible emitter must not copy).
fn uuid(pkg: &str, refdes: &str, kind: &str, ordinal: usize) -> String {
    let mut h = crate::hash::Sha256::new();
    h.update(b"cohdl.kicad_pcb\x00");
    h.update(pkg.as_bytes());
    h.update(b"\x00");
    h.update(refdes.as_bytes());
    h.update(b"\x00");
    h.update(kind.as_bytes());
    h.update(b"\x00");
    h.update(ordinal.to_string().as_bytes());
    let d = h.finish();
    let mut b = [0u8; 16];
    b.copy_from_slice(&d[..16]);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let hexed: Vec<String> = b.iter().map(|x| format!("{:02x}", x)).collect();
    let hexed = hexed.join("");
    format!(
        "{}-{}-{}-{}-{}",
        &hexed[0..8],
        &hexed[8..12],
        &hexed[12..16],
        &hexed[16..20],
        &hexed[20..32]
    )
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod arc_mid_tests {
    use super::{arc_mid, FEMTO_MM};

    #[test]
    fn dxf_counterclockwise_quarter_uses_the_minor_arc() {
        let got = arc_mid((FEMTO_MM, 0), (0, FEMTO_MM), (0, 0), false);
        assert_eq!(got, (707_106_781_000_000, 707_106_781_000_000));
    }

    #[test]
    fn dxf_clockwise_quarter_uses_the_minor_arc() {
        let got = arc_mid((FEMTO_MM, 0), (0, -FEMTO_MM), (0, 0), true);
        assert_eq!(got, (707_106_781_000_000, -707_106_781_000_000));
    }

    #[test]
    fn winding_selects_the_major_arc_when_requested() {
        let got = arc_mid((FEMTO_MM, 0), (0, FEMTO_MM), (0, 0), true);
        assert_eq!(got, (-707_106_781_000_000, -707_106_781_000_000));
    }
}

//! RFC-015: IPC-2581 (revision B1) emitter — the partner-handoff artifact.
//!
//! An IPC-2581 document carrying the REAL physical geometry a layout partner
//! (Quilter) routes — not just the logical design. It emits the logical design
//! (netlist, components, resolved specs, RFC-013 constraints) and the board
//! outline (`Step/Profile`, from RFC-020's DXF); a `DictionaryStandard` of
//! primitive shapes, `PadStackDef`s (copper + mask + paste + a plated
//! `PadstackHoleDef` for through-hole), and real physical layers
//! (F.Cu/B.Cu/masks/paste) + a stackup; side-aware `LayerFeature`s of every
//! PLACED copper/mask/paste feature — including repeated placements of one
//! electrical number — at absolute board positions, tied to their component
//! pin (`PinRef`) and net (`Set/@net`); and accurate `Component/@mountType`
//! (SMT vs THMT) and
//! `Pin/@mountType` (`SURFACE_MOUNT_PAD` / `THROUGH_HOLE_PIN`).
//! (This physical layer was added to fix .co/invalid-ipc2581.xml — an
//! XSD-valid document that carried only abstract `Package/Pin` land patterns
//! showed no copper/holes/ratsnest in Quilter.) The `Package` land pattern is
//! still emitted too, with a non-degenerate `Outline` derived from the pad
//! extents when a footprint deliberately omits its courtyard.
//!
//! What is NOT yet done — and the `COHDL_COMPLETENESS` /`FunctionMode` markers
//! say so, never overclaiming (DR-021): final component PLACEMENT (components
//! that aren't `place`-locked are STAGED just outside the outline — the "please
//! place me" idiom) and ROUTING (no copper traces). Hence the marker
//! `logical-complete,placement-staged,unrouted`. Still omitted: the pad's
//! `silkscreen_ref` (present in the `.kicad_mod`).
//!
//! Hand-rolled XML in the project's existing emitter style (same discipline
//! as the hand-rolled JSON in `json.rs`/`layout.rs`): the populated subset
//! of the schema is small and fixed, every ordering is an explicit sort, and
//! the output is byte-stable — which is also why every schema-required
//! `xsd:dateTime` is the fixed epoch instant, never the wall clock (same
//! source + same std → same bytes is a Constitution hard constraint).
//!
//! Schema ground truth: `tests/schema/IPC-2581B1.xsd` (the IPC 2581
//! Consortium's published copy); `tests/ipc2581.rs` validates every fixture
//! document against it and cross-checks fidelity against the `.net`/BOM/
//! `layout.json` emitters.

use crate::emit::geom;
use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// The completeness marker (Non-goals: geometry/outline/stackup are not
/// CoHDL concepts yet; the receiving tool must be able to detect that).
pub const COMPLETENESS: &str = "logical-complete,placement-staged,unrouted";

/// Fixed timestamp for every schema-required `xsd:dateTime`: byte-stable
/// output is a hard constraint, so the wall clock never enters an artifact.
const EPOCH: &str = "1970-01-01T00:00:00Z";

/// Emit the IPC-2581B1 document for a built design. Requires designators
/// assigned and parts bound (call only after `build_artifacts` succeeded).
pub fn emit_ipc2581(world: &World, ir: &DesignIr, package_name: &str) -> String {
    let insts = sorted_instances(ir);
    let bom = bom_groups(world, ir);
    let enterprises = enterprise_map(&bom);
    let packages = package_table(world, ir);
    // Designators become the componentKey — sanitize them ONCE, collision-
    // free, and use the same spelling everywhere a refdes appears
    // (Component/@refDes, RefDes/@name, PinRef/@componentRef). NOTE (review
    // F7): the vendored B1 schema's `componentKeyRef` keyref binds
    // `LogicalNetPin/@componentRef` and `RefDes/@name` to `componentKey`,
    // NOT `PinRef/@componentRef` — so PinRef agreement is NOT XSD-enforced.
    // We route every refdes through this one table so the emitter guarantees
    // it, and tests/ipc2581.rs asserts it semantically rather than relying
    // on the schema.
    let refdes_map = refdes_table(ir);
    // Component staging (Quilter): a layout tool treats components INSIDE the
    // board outline as pre-placed/locked and only places/routes components
    // left OUTSIDE it (docs.quilter.ai). So when a board outline exists, stage
    // every component in a deterministic, non-overlapping grid just OUTSIDE the
    // outline — the IPC/Quilter idiom for "unplaced, please place me" — instead
    // of piling them all at (0,0) (inside the outline = 49 locked components
    // stacked at the board center). No outline → keep the (0,0) placeholder.
    // Locked placements (`place <inst> at …`) take their fixed position and
    // are NOT staged — a placement tool treats them as pre-placed. Everything
    // else is staged outside the outline.
    let placed = placed_positions(ir);
    let rotations: BTreeMap<String, u16> = ir
        .layout
        .placements
        .iter()
        .map(|p| (p.path.clone(), p.rotate))
        .collect();
    // RFC-026: which outer face each placed component sits on.
    let sides: BTreeMap<String, crate::ast::PlacementSide> = ir
        .layout
        .placements
        .iter()
        .map(|p| (p.path.clone(), p.side))
        .collect();
    let staging = staging_positions(world, ir, &insts, &placed);
    // The physical model (the copper Quilter routes): every pad placed at its
    // absolute board position, deduplicated into primitives + padstacks.
    let positions: BTreeMap<String, (i128, i128)> = insts
        .iter()
        .map(|i| {
            let p = placed
                .get(&i.path)
                .or_else(|| staging.get(&i.path))
                .copied()
                .unwrap_or((0, 0));
            (i.path.clone(), p)
        })
        .collect();
    let mut phys = build_physical(world, &insts, &positions, &rotations, &sides, &refdes_map);
    // Close the shape dictionary over every Package/Pin as well: pin geometry
    // is emitted as a `StandardPrimitiveRef` into the same `DictionaryStandard`
    // the padstacks use (the encoding every mainstream consumer implements —
    // an inline primitive under `<Pin>` is schema-valid via the StandardShape
    // substitution group but invisible to a real importer), so every package
    // pad shape must have a `PRIM_n` entry even if no placed instance
    // contributed it. Same guards + (w, h) derivation as `build_physical`.
    for footprint in packages.keys() {
        let Some(fp) = world.footprints.get(footprint) else {
            continue;
        };
        for place in &fp.pads {
            let Some(pad) = world.pads.get(&place.pad.name) else {
                continue;
            };
            let (Some((shape, _)), Some(_)) = (&pad.shape, &pad.plating) else {
                continue;
            };
            let (w, h) = match pad.size.as_slice() {
                [d] => (d.femto, d.femto),
                [w, h, ..] => (w.femto, h.femto),
                [] => continue,
            };
            dedup(&mut phys.prims, copper_prim(pad, *shape, w, h));
        }
    }
    let phys = phys; // frozen — the dictionary is closed from here on
    let net_of = net_membership(world, ir, &refdes_map);
    // A component is through-hole-mounted (THMT) if it has an electrical
    // number implemented only by PTH lands. Thermal vias repeated under an SMD
    // exposed-pad number remain physical holes but do not change mount type.
    // An RFC-022 mount_hole is mechanical and likewise does not change it.
    let smd_terminals: BTreeSet<(&str, &str)> = phys
        .pads
        .iter()
        .filter(|p| !p.tht && !p.hole)
        .map(|p| (p.refdes.as_str(), p.pin.as_str()))
        .collect();
    let tht_refdes: BTreeSet<&str> = phys
        .pads
        .iter()
        .filter(|p| {
            p.tht && !p.hole && !smd_terminals.contains(&(p.refdes.as_str(), p.pin.as_str()))
        })
        .map(|p| p.refdes.as_str())
        .collect();
    let name = sanitize(package_name, false);
    let step = sanitize(&ir.name, false);

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<IPC-2581 revision=\"B1\" xmlns=\"http://webstds.ipc.org/2581\">\n");

    // ---- Content: what the document contains, and for whom ----
    out.push_str("  <Content roleRef=\"Owner\">\n");
    // `mode` names the data FUNCTION the document serves (assembly-level:
    // BOM + components + nets + land patterns + padstacks), not a completeness
    // claim — real consumers key their fabrication-layer rendering off a
    // standard mode and skip LayerFeatures under USERDEF. The honest
    // completeness disclosure stays in @comment and COHDL_COMPLETENESS.
    let _ = writeln!(
        out,
        "    <FunctionMode mode=\"ASSEMBLY\" level=\"1\" comment=\"{} — layout not yet performed\"/>",
        esc(COMPLETENESS)
    );
    let _ = writeln!(out, "    <StepRef name=\"{}\"/>", esc(&step));
    // The physical stack, top→bottom — every layer the padstacks/features
    // reference is declared here (a consumer may build its renderable-layer
    // list from these Content declarations). The full `FAB_LAYERS` set, in the
    // same order the reference exporter (KiCad) lists it.
    for (layer, _func, _side, _thickness) in FAB_LAYERS {
        let _ = writeln!(out, "    <LayerRef name=\"{}\"/>", layer);
    }
    if !bom.is_empty() {
        let _ = writeln!(out, "    <BomRef name=\"{}-bom\"/>", esc(&name));
        let _ = writeln!(out, "    <AvlRef name=\"{}-avl\"/>", esc(&name));
    }
    // Primitive-shape dictionary the padstacks + placed pads reference.
    emit_dictionary(&mut out, &phys);
    out.push_str("  </Content>\n");

    // ---- LogisticHeader: one owner role/enterprise/person (schema-required
    // minimums), plus one Enterprise per manufacturer so the AVL's vendor
    // references resolve (the XSD enforces that keyref).
    out.push_str("  <LogisticHeader>\n");
    out.push_str("    <Role id=\"Owner\" roleFunction=\"OWNER\"/>\n");
    out.push_str("    <Enterprise id=\"cohdl\" code=\"NONE\"/>\n");
    for id in enterprises.values() {
        let _ = writeln!(out, "    <Enterprise id=\"{}\" code=\"NONE\"/>", esc(id));
    }
    out.push_str("    <Person name=\"cohdl\" enterpriseRef=\"cohdl\" roleRef=\"Owner\"/>\n");
    out.push_str("  </LogisticHeader>\n");

    // ---- HistoryRecord (schema-required; fixed instants, see EPOCH) ----
    let _ = writeln!(
        out,
        "  <HistoryRecord number=\"1\" origination=\"{EPOCH}\" software=\"cohdl\" lastChange=\"{EPOCH}\">"
    );
    let _ = writeln!(
        out,
        "    <FileRevision fileRevisionId=\"1\" comment=\"{}\">",
        esc(COMPLETENESS)
    );
    let _ = writeln!(
        out,
        "      <SoftwarePackage name=\"cohdl\" vendor=\"conol.ai\" revision=\"{}\">",
        esc(env!("CARGO_PKG_VERSION"))
    );
    out.push_str("        <Certification certificationStatus=\"SELFTEST\"/>\n");
    out.push_str("      </SoftwarePackage>\n");
    out.push_str("    </FileRevision>\n");
    out.push_str("  </HistoryRecord>\n");

    // ---- Bom: one item per MPN group, exactly the BOM CSV's grouping ----
    if !bom.is_empty() {
        let _ = writeln!(out, "  <Bom name=\"{}-bom\">", esc(&name));
        let _ = writeln!(
            out,
            "    <BomHeader assembly=\"{}\" revision=\"1\"/>",
            esc(&name)
        );
        for g in &bom {
            let _ = writeln!(
                out,
                "    <BomItem OEMDesignNumberRef=\"{}\" quantity=\"{}\" category=\"ELECTRICAL\">",
                esc(&g.key),
                g.refdes.len()
            );
            for (refdes, footprint) in &g.refdes {
                let _ = writeln!(
                    out,
                    "      <RefDes name=\"{}\" packageRef=\"{}\" populate=\"true\"/>",
                    esc(&refdes_map[refdes]),
                    esc(&packages[footprint])
                );
            }
            out.push_str("      <Characteristics category=\"ELECTRICAL\">\n");
            for (k, v) in [("MPN", &g.mpn), ("MFR", &g.mfr), ("VALUE", &g.value)] {
                let _ = writeln!(
                    out,
                    "        <Textual textualCharacteristicName=\"{}\" textualCharacteristicValue=\"{}\"/>",
                    k,
                    esc(v)
                );
            }
            out.push_str("      </Characteristics>\n");
            out.push_str("    </BomItem>\n");
        }
        out.push_str("  </Bom>\n");
    }

    // ---- Ecad: header specs (RFC-013 constraints) + the one step ----
    let _ = writeln!(out, "  <Ecad name=\"{}\">", esc(&name));
    out.push_str("    <CadHeader units=\"MILLIMETER\">\n");
    emit_layout_specs(&mut out, ir);
    out.push_str("    </CadHeader>\n");
    out.push_str("    <CadData>\n");
    emit_layers(&mut out);
    emit_stackup(&mut out);
    let _ = writeln!(out, "      <Step name=\"{}\">", esc(&step));
    let _ = writeln!(
        out,
        "        <NonstandardAttribute name=\"COHDL_COMPLETENESS\" type=\"STRING\" value=\"{}\"/>",
        esc(COMPLETENESS)
    );
    // Reusable padstack definitions (copper/mask/paste + plated holes).
    emit_padstacks(&mut out, &phys);
    out.push_str("        <Datum x=\"0\" y=\"0\"/>\n");

    // Board outline → Step/Profile: the single closed board perimeter a
    // downstream layout tool (Quilter) seeds placement/routing against.
    // RFC-020: real geometry extracted from a referenced DXF (straight
    // segments become PolyStepSegment, arc bulges become PolyStepCurve). All
    // coordinates are exact over the femto integers (emit::geom / emit::dxf).
    // Absent when the design declares no `board_outline`, or when it does but
    // the DXF wasn't resolved (the completeness marker still says minimal).
    if let Some(g) = ir
        .layout
        .board_outline
        .as_ref()
        .and_then(|b| b.geom.as_ref())
    {
        out.push_str("        <Profile>\n");
        out.push_str("          <Polygon>\n");
        // The whole outline is projected into IPC's +y-up frame: every y is
        // negated, and each arc's winding flips (a y-reflection reverses the
        // sense of rotation), so the board reads the same as in CoHDL/KiCad.
        let _ = writeln!(
            out,
            "            <PolyBegin x=\"{}\" y=\"{}\"/>",
            geom::mm_femto(g.start.0),
            geom::mm_femto_y(g.start.1)
        );
        for seg in &g.segs {
            match seg {
                crate::dxf::Seg::Line { to } => {
                    let _ = writeln!(
                        out,
                        "            <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                        geom::mm_femto(to.0),
                        geom::mm_femto_y(to.1)
                    );
                }
                crate::dxf::Seg::Arc {
                    to,
                    center,
                    clockwise,
                } => {
                    let _ = writeln!(
                        out,
                        "            <PolyStepCurve x=\"{}\" y=\"{}\" centerX=\"{}\" centerY=\"{}\" clockwise=\"{}\"/>",
                        geom::mm_femto(to.0),
                        geom::mm_femto_y(to.1),
                        geom::mm_femto(center.0),
                        geom::mm_femto_y(center.1),
                        !clockwise
                    );
                }
            }
        }
        out.push_str("            <LineDesc lineEnd=\"NONE\" lineWidth=\"0.1\"/>\n");
        out.push_str("          </Polygon>\n");
        out.push_str("        </Profile>\n");
    }

    // Packages: one per distinct footprint symbol. RFC-018: a pad-bearing
    // footprint projects REAL geometry (courtyard outline + one Pin per
    // distinct electrical pad number); an RFC-017 stage-one placeholder keeps the zero-size idiom
    // (the completeness marker declares that).
    for (footprint, pkg) in &packages {
        let comment = if pkg != footprint {
            format!(" comment=\"{}\"", esc(footprint))
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "        <Package name=\"{}\" type=\"OTHER\" pinOneOrientation=\"OTHER\"{}>",
            esc(pkg),
            comment
        );
        let fp = world
            .footprints
            .get(footprint)
            .filter(|f| !crate::check::footprints::is_placeholder(f));
        match fp.and_then(|f| f.courtyard.as_ref()) {
            // A courtyard becomes the package outline (the schema's Outline
            // requires a Polygon, so a CIRCLE courtyard projects as its
            // bounding square — a disclosed approximation; .kicad_mod keeps
            // the true circle). Corners are computed exactly over the femto
            // integers (emit::geom) — never floats.
            Some(c) if !c.size.is_empty() => {
                let (w, h) = match c.size.as_slice() {
                    [d] => (d, d),
                    [w, h, ..] => (w, h),
                    [] => unreachable!(),
                };
                out.push_str("          <Outline>\n");
                out.push_str("            <Polygon>\n");
                // y corners negated: IPC +y-up vs CoHDL +y-down.
                let corners = [
                    (geom::corner_lo(&c.at.0, w), geom::corner_lo_y(&c.at.1, h)),
                    (geom::corner_hi(&c.at.0, w), geom::corner_lo_y(&c.at.1, h)),
                    (geom::corner_hi(&c.at.0, w), geom::corner_hi_y(&c.at.1, h)),
                    (geom::corner_lo(&c.at.0, w), geom::corner_hi_y(&c.at.1, h)),
                ];
                let _ = writeln!(
                    out,
                    "              <PolyBegin x=\"{}\" y=\"{}\"/>",
                    corners[0].0, corners[0].1
                );
                for (x, y) in corners.iter().skip(1).chain([corners[0].clone()].iter()) {
                    let _ = writeln!(
                        out,
                        "              <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                        x, y
                    );
                }
                out.push_str("            </Polygon>\n");
                out.push_str("            <LineDesc lineEnd=\"NONE\" lineWidth=\"0.05\"/>\n");
                out.push_str("          </Outline>\n");
            }
            // No courtyard: derive a real bounding-box outline from the pad
            // extents (a footprint may deliberately omit its courtyard — e.g.
            // the castellated header — and a degenerate (0,0)-(0,0) outline
            // would make the package invisible in a viewer). Only truly empty
            // (placeholder) footprints keep the zero-size idiom.
            _ if fp.is_some_and(|f| !f.pads.is_empty()) => {
                let (lo_x, lo_y, hi_x, hi_y) = footprint_bbox(world, footprint);
                let corners = [(lo_x, lo_y), (hi_x, lo_y), (hi_x, hi_y), (lo_x, hi_y)];
                out.push_str("          <Outline>\n");
                out.push_str("            <Polygon>\n");
                // y negated: IPC +y-up vs CoHDL +y-down.
                let _ = writeln!(
                    out,
                    "              <PolyBegin x=\"{}\" y=\"{}\"/>",
                    geom::mm_femto(corners[0].0),
                    geom::mm_femto_y(corners[0].1)
                );
                for (x, y) in corners.iter().skip(1).chain([corners[0]].iter()) {
                    let _ = writeln!(
                        out,
                        "              <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                        geom::mm_femto(*x),
                        geom::mm_femto_y(*y)
                    );
                }
                out.push_str("            </Polygon>\n");
                out.push_str("            <LineDesc lineEnd=\"NONE\" lineWidth=\"0.05\"/>\n");
                out.push_str("          </Outline>\n");
            }
            _ => {
                out.push_str("          <Outline>\n");
                out.push_str("            <Polygon>\n");
                out.push_str("              <PolyBegin x=\"0\" y=\"0\"/>\n");
                out.push_str("              <PolyStepSegment x=\"0\" y=\"0\"/>\n");
                out.push_str("            </Polygon>\n");
                out.push_str("            <LineDesc lineEnd=\"NONE\" lineWidth=\"0\"/>\n");
                out.push_str("          </Outline>\n");
            }
        }
        if let Some(f) = fp {
            // A single electrical terminal may be implemented by several
            // physical lands (EP copper/paste/vias/back land). Package/Pin is
            // logical, so choose one deterministic representative per number:
            // footprint-local top SMD before bottom SMD before PTH, then the
            // largest land.
            // A thermal via authored first must never misclassify the logical
            // terminal as through-hole. build_physical below still emits every
            // authored placement as a physical feature.
            let mut logical_pins: BTreeMap<&str, (&crate::ast::PadPlace, &crate::ast::PadDef)> =
                BTreeMap::new();
            let mut logical_order: Vec<&str> = Vec::new();
            let mut seen_numbers: BTreeSet<&str> = BTreeSet::new();
            for place in &f.pads {
                let Some(pad) = world.pads.get(&place.pad.name) else {
                    continue;
                };
                let (Some(_), Some(_)) = (&pad.shape, &pad.plating) else {
                    continue;
                };
                if seen_numbers.insert(place.number.text.as_str()) {
                    logical_order.push(place.number.text.as_str());
                }
                logical_pins
                    .entry(place.number.text.as_str())
                    .and_modify(|current| {
                        if logical_pin_cmp((place, pad), *current).is_lt() {
                            *current = (place, pad);
                        }
                    })
                    .or_insert((place, pad));
            }
            for number in logical_order {
                let Some(&(place, pad)) = logical_pins.get(number) else {
                    continue;
                };
                let (Some((shape, _)), Some((plating, _))) = (&pad.shape, &pad.plating) else {
                    continue;
                };
                let (pin_type, mount_type) = match plating {
                    crate::ast::PadPlating::Smd => ("SURFACE", "SURFACE_MOUNT_PAD"),
                    // An electrical through-hole PIN (not a non-electrical
                    // mounting HOLE) — every CoHDL PTH pad is a device terminal.
                    crate::ast::PadPlating::PlatedThroughHole => ("THRU", "THROUGH_HOLE_PIN"),
                };
                // Every CoHDL pad placement is a device terminal (see THRU
                // note above) — never a mechanical-only pin.
                let _ = writeln!(
                    out,
                    "          <Pin number=\"{}\" type=\"{}\" electricalType=\"ELECTRICAL\" mountType=\"{}\">",
                    esc(&sanitize(number, true)),
                    pin_type,
                    mount_type
                );
                // Package/Pin describes the footprint-local land pattern, so
                // its representative placement must carry the pad-local turn
                // as well as its centre and primitive.  The board-level
                // LayerFeature below composes this with the component turn;
                // here there is no component transform yet.
                if place.rotate != 0 {
                    let _ = writeln!(out, "            <Xform rotation=\"{}\"/>", place.rotate);
                }
                let _ = writeln!(
                    out,
                    "            <Location x=\"{}\" y=\"{}\"/>",
                    geom::mm(&place.x),
                    geom::mm_y(&place.y) // y negated: IPC +y-up (see build_physical)
                );
                // Geometry as a `StandardPrimitiveRef` into the shared
                // DictionaryStandard — the encoding consumers implement (an
                // inline primitive is schema-valid but real importers resolve
                // only the dictionary form; pads rendered nowhere). The
                // dictionary was closed over every package pad up front, so
                // the lookup is total for any sized pad.
                match pad.size.as_slice() {
                    [] => {
                        // Arity errors were reported at declaration check;
                        // keep the document well-formed with a zero circle.
                        out.push_str("            <Circle diameter=\"0\"/>\n");
                    }
                    sz => {
                        let (w, h) = match sz {
                            [d] => (d.femto, d.femto),
                            [w, h, ..] => (w.femto, h.femto),
                            [] => unreachable!(),
                        };
                        let want = copper_prim(pad, *shape, w, h);
                        let idx = phys
                            .prims
                            .iter()
                            .position(|e| *e == want)
                            .expect("pin shape in dictionary (closure pre-pass)");
                        let _ = writeln!(
                            out,
                            "            <StandardPrimitiveRef id=\"PRIM_{}\"/>",
                            idx
                        );
                    }
                }
                out.push_str("          </Pin>\n");
            }
        }
        out.push_str("        </Package>\n");
    }

    // Components: designator order, resolved specs + placement hint as
    // machine-readable attributes. Location is the staged position outside the
    // board outline (Quilter places from there) or (0,0) when no outline.
    for inst in &insts {
        let refdes = inst.designator.as_deref().unwrap_or("?");
        let (mpn, _mfr, footprint) = part_fields(world, inst);
        // Real mount type: THMT when at least one distinct electrical terminal
        // is implemented exclusively by PTH placements. Repeated thermal vias
        // under an SMD terminal do not change the component from SMT. The
        // component's physical side is carried independently by layerRef.
        let mount = if tht_refdes.contains(refdes_map[refdes].as_str()) {
            "THMT"
        } else {
            "SMT"
        };
        // RFC-026: a bottom-side placement rides layerRef="B.Cu"; everything
        // else (staged included) stays on the front, the pre-RFC-026 value.
        let bottom = matches!(
            sides.get(&inst.path),
            Some(crate::ast::PlacementSide::Bottom)
        );
        let _ = writeln!(
            out,
            "        <Component refDes=\"{}\" packageRef=\"{}\" part=\"{}\" layerRef=\"{}\" mountType=\"{}\">",
            esc(&refdes_map[refdes]),
            esc(&packages[&footprint]),
            esc(&mpn),
            if bottom { "B.Cu" } else { "F.Cu" },
            mount
        );
        let _ = writeln!(
            out,
            "          <NonstandardAttribute name=\"COHDL_DEVICE\" type=\"STRING\" value=\"{}\"/>",
            esc(&inst.device)
        );
        let _ = writeln!(
            out,
            "          <NonstandardAttribute name=\"COHDL_PATH\" type=\"STRING\" value=\"{}\"/>",
            esc(&inst.path)
        );
        for (field, value) in &inst.specs {
            let _ = writeln!(
                out,
                "          <NonstandardAttribute name=\"COHDL_SPEC_{}\" type=\"STRING\" value=\"{}\"/>",
                esc(field),
                esc(&value.text)
            );
        }
        if let Some(hint) = &inst.placement_hint {
            let _ = writeln!(
                out,
                "          <NonstandardAttribute name=\"COHDL_PLACEMENT_HINT\" type=\"STRING\" value=\"{}\"/>",
                esc(hint)
            );
        }
        // RFC-020: a locked placement's rotation rides Component/Xform
        // (schema order: NonstandardAttribute*, Xform?, Location). Staged and
        // unplaced components are unrotated (no Xform).
        let rot = rotations.get(&inst.path).copied().unwrap_or(0);
        if rot != 0 || bottom {
            let mut attrs = String::new();
            if rot != 0 {
                let _ = write!(attrs, " rotation=\"{}\"", rot);
            }
            if bottom {
                // RFC-026: IPC-2581's own per-component mirror attribute.
                attrs.push_str(" mirror=\"true\"");
            }
            let _ = writeln!(out, "          <Xform{}/>", attrs);
        }
        let (lx, ly) = match placed.get(&inst.path).or_else(|| staging.get(&inst.path)) {
            // y negated: IPC-2581 +y-up vs CoHDL +y-down (see build_physical).
            Some((x, y)) => (geom::mm_femto(*x), geom::mm_femto_y(*y)),
            None => ("0".to_string(), "0".to_string()),
        };
        let _ = writeln!(out, "          <Location x=\"{}\" y=\"{}\"/>", lx, ly);
        out.push_str("        </Component>\n");
    }

    // Logical nets: one per merged net, one PinRef per PHYSICAL pin — the
    // exact node set the KiCad emitter writes (fidelity by construction,
    // enforced against the `.net` text in tests/ipc2581.rs).
    for net in &ir.nets {
        let class = if net.is_gnd {
            "GROUND"
        } else if net.voltage.is_some() {
            "POWER"
        } else {
            "SIGNAL"
        };
        let _ = writeln!(
            out,
            "        <LogicalNet name=\"{}\" netClass=\"{}\">",
            esc(&sanitize(&net.name, true)),
            class
        );
        for (refdes, pin) in physical_pins(world, ir, net) {
            let _ = writeln!(
                out,
                "          <PinRef componentRef=\"{}\" pin=\"{}\"/>",
                esc(&refdes_map[&refdes]),
                esc(&pin)
            );
        }
        out.push_str("        </LogicalNet>\n");
    }
    // Every placed copper/mask/enabled-paste feature on its resolved physical
    // side, tied to its component pin + net — the geometry a layout tool routes.
    emit_layer_features(&mut out, &phys, &net_of);
    out.push_str("      </Step>\n");
    out.push_str("    </CadData>\n");
    out.push_str("  </Ecad>\n");

    // ---- Avl: the approved-vendor list backing the Bom (the XSD requires
    // every BomItem's OEMDesignNumberRef to resolve to an AvlItem) ----
    if !bom.is_empty() {
        let _ = writeln!(out, "  <Avl name=\"{}-avl\">", esc(&name));
        let _ = writeln!(
            out,
            "    <AvlHeader title=\"{}\" source=\"cohdl\" author=\"cohdl\" datetime=\"{EPOCH}\" version=\"1\"/>",
            esc(&name)
        );
        for g in &bom {
            let _ = writeln!(out, "    <AvlItem OEMDesignNumber=\"{}\">", esc(&g.key));
            out.push_str("      <AvlVmpn chosen=\"true\">\n");
            // @name uses the group's collision-free key (adversarial
            // finding: two distinct MPNs must never collapse to one AvlMpn,
            // let alone alias a competing vendor's exact MPN); the TRUE MPN
            // rides @other, the schema's free-string attribute.
            let _ = writeln!(
                out,
                "        <AvlMpn name=\"{}\" other=\"{}\"/>",
                esc(&g.key),
                esc(&g.mpn)
            );
            let _ = writeln!(
                out,
                "        <AvlVendor enterpriseRef=\"{}\"/>",
                esc(enterprises
                    .get(&g.mfr)
                    .map(String::as_str)
                    .unwrap_or("cohdl"))
            );
            out.push_str("      </AvlVmpn>\n");
            out.push_str("    </AvlItem>\n");
        }
        out.push_str("  </Avl>\n");
    }

    out.push_str("</IPC-2581>\n");
    out
}

/// RFC-013 constraints as `CadHeader/Spec` entries — IPC-2581's own place
/// for named design specifications. One `Spec` per constraint; members and
/// the opaque tolerance ride `General/Property` entries.
fn emit_layout_specs(out: &mut String, ir: &DesignIr) {
    let spec = |out: &mut String, name: &str, comment: &str, props: &[(&str, &str)]| {
        let _ = writeln!(out, "      <Spec name=\"{}\">", esc(name));
        let _ = writeln!(
            out,
            "        <General type=\"OTHER\" comment=\"{}\">",
            esc(comment)
        );
        for (k, v) in props {
            let _ = writeln!(
                out,
                "          <Property name=\"{}\" text=\"{}\"/>",
                esc(k),
                esc(v)
            );
        }
        out.push_str("        </General>\n");
        out.push_str("      </Spec>\n");
    };
    for nc in &ir.layout.net_classes {
        let props: Vec<(&str, &str)> = nc.nets.iter().map(|n| ("net", n.as_str())).collect();
        spec(
            out,
            &format!("cohdl:net_class:{}", nc.name),
            "RFC-013 net_class",
            &props,
        );
    }
    for dp in &ir.layout.diff_pairs {
        spec(
            out,
            &format!("cohdl:diff_pair:{}:{}", dp.p, dp.n),
            "RFC-013 diff_pair",
            &[("positive", dp.p.as_str()), ("negative", dp.n.as_str())],
        );
    }
    for (i, lm) in ir.layout.length_matches.iter().enumerate() {
        let mut props: Vec<(&str, &str)> = lm.nets.iter().map(|n| ("net", n.as_str())).collect();
        if let Some(tol) = &lm.tolerance {
            props.push(("tolerance", tol.as_str()));
        }
        spec(
            out,
            &format!("cohdl:length_match:{}", i + 1),
            "RFC-013 length_match",
            &props,
        );
    }
}

/// One BOM group per MPN — the exact grouping `bom.rs` writes to CSV.
struct BomGroup {
    /// The `OEMDesignNumber` key: the MPN sanitized to the XSD's shortName
    /// charset, disambiguated on (unlikely) post-sanitize collisions.
    key: String,
    mpn: String,
    mfr: String,
    value: String,
    /// (designator, footprint), designator order.
    refdes: Vec<(String, String)>,
}

/// Accumulator per (MPN, manufacturer): (value, refdes+footprint list).
type GroupAcc = (String, Vec<(String, String)>);

fn bom_groups(world: &World, ir: &DesignIr) -> Vec<BomGroup> {
    // Keyed by (MPN, manufacturer), NOT MPN alone: an MPN is not globally
    // unique without its manufacturer, and the language permits two parts
    // that share an MPN under different manufacturers. Keying by MPN alone
    // silently dropped the second manufacturer's identity and value from
    // both the CSV and the XML (review F5). MPN stays the primary sort key
    // (BOM readability); manufacturer only breaks genuine ties.
    let mut groups: BTreeMap<(String, String), GroupAcc> = BTreeMap::new();
    for inst in ir.instances.values() {
        let refdes = inst.designator.clone().unwrap_or_else(|| "?".to_string());
        let (mpn, mfr, footprint) = part_fields(world, inst);
        let value = principal_value(inst);
        groups
            .entry((mpn, mfr))
            .or_insert_with(|| (value, Vec::new()))
            .1
            .push((refdes, footprint));
    }
    let mut used = BTreeSet::new();
    let mut out = Vec::new();
    for ((mpn, mfr), (value, mut refdes)) in groups {
        refdes.sort_by_key(|(d, _)| crate::emit::designator_sort_key(d));
        let mut key = sanitize(&mpn, false);
        while !used.insert(key.clone()) {
            key.push('_');
        }
        out.push(BomGroup {
            key,
            mpn,
            mfr,
            value,
            refdes,
        });
    }
    out
}

/// footprint name → sanitized, collision-free Package name (deterministic:
/// footprints iterate in sorted order).
///
/// The name is emitted verbatim as BOTH the `<Package name>` and every
/// `<Component packageRef>`/`<RefDes packageRef>`, so they always stay matched.
/// The CoHDL fully-qualified separator `::` is collapsed to a single `-`
/// (KiCad's own convention: `rpi_pico2-CHIP_0805`) rather than kept as a colon:
/// `:` is the XML QName/namespace delimiter, and a consumer whose pad-resolution
/// path treats a package name as an NCName (or splits it on `:`) would fail to
/// bind pins to their land pattern. The XSD's `qualifiedNameType` permits the
/// colon, so this is a consumer-safety choice, not a validity fix — it converges
/// onto the colon-free shape every reference IPC-2581 exporter emits.
fn package_table(world: &World, ir: &DesignIr) -> BTreeMap<String, String> {
    let footprints: BTreeSet<String> = ir
        .instances
        .values()
        .map(|i| part_fields(world, i).2)
        .collect();
    let mut used = BTreeSet::new();
    let mut table = BTreeMap::new();
    for f in footprints {
        // `::` → `-` first (a single separator, not the `__` that char-wise
        // colon-stripping would give), then sanitize disallowing any stray
        // colon as a backstop.
        let mut name = sanitize(&f.replace("::", "-"), false);
        while !used.insert(name.clone()) {
            name.push('_');
        }
        table.insert(f, name);
    }
    table
}

fn manufacturers(bom: &[BomGroup]) -> BTreeSet<String> {
    bom.iter().map(|g| g.mfr.clone()).collect()
}

/// The Enterprise id for a manufacturer name (xsd:string — no charset
/// restriction, but never empty/colliding with the fixed "cohdl" id).
fn enterprise_id(mfr: &str) -> String {
    if mfr.is_empty() {
        "unknown-manufacturer".to_string()
    } else {
        format!("mfr:{}", mfr)
    }
}

/// (MPN, manufacturer, footprint) — same source as the KiCad/BOM emitters.
fn part_fields(world: &World, inst: &crate::ir::IrInstance) -> (String, String, String) {
    let part = inst.part.as_ref().and_then(|p| world.parts.get(p));
    let field = |name: &str| -> String {
        part.and_then(|p| p.primary.field(name))
            .map(|f| f.value.clone())
            .unwrap_or_default()
    };
    // RFC-017: the footprint identity is the resolved symbol's fq path.
    let footprint = part
        .and_then(|p| p.primary.footprint.as_ref())
        .map(|f| f.name.clone())
        .unwrap_or_default();
    (field("mpn"), field("mfr"), footprint)
}

/// One femto-mm. Staging geometry is computed on the same exact-integer scale
/// the lexer/`emit::geom` use, so it is byte-stable and renders canonically.
const FEMTO_MM: i128 = 1_000_000_000_000_000;

/// Grow the running bbox `(min_x, min_y, max_x, max_y)` to include a rect
/// centered at `(cx, cy)` with half-extents `(hw, hh)` — all femto-mm.
fn union_bbox(b: &mut Option<(i128, i128, i128, i128)>, cx: i128, cy: i128, hw: i128, hh: i128) {
    let r = (cx - hw, cy - hh, cx + hw, cy + hh);
    *b = Some(match *b {
        None => r,
        Some(o) => (o.0.min(r.0), o.1.min(r.1), o.2.max(r.2), o.3.max(r.3)),
    });
}

/// A footprint's bounding box in femto-mm `(min_x, min_y, max_x, max_y)` — the
/// union of every pad extent and the courtyard. An empty (RFC-017 placeholder)
/// footprint has no geometry, so it gets a nominal 1×1 mm box, enough to stage
/// it beside its neighbors without a zero-size overlap.
fn footprint_bbox(world: &World, fp_name: &str) -> (i128, i128, i128, i128) {
    const NOMINAL_HALF: i128 = FEMTO_MM / 2; // 0.5mm → a 1×1mm nominal box
    let mut b: Option<(i128, i128, i128, i128)> = None;
    if let Some(fp) = world.footprints.get(fp_name) {
        for place in &fp.pads {
            if let Some(pad) = world.pads.get(&place.pad.name) {
                let (w, h) = match (pad.shape.map(|(shape, _)| shape), pad.size.as_slice()) {
                    (Some(crate::ast::PadShape::Annulus), [outer, _]) => (outer.femto, outer.femto),
                    (_, [w, h]) => (w.femto, h.femto),
                    (_, [d]) => (d.femto, d.femto),
                    _ => continue,
                };
                union_bbox(&mut b, place.x.femto, place.y.femto, w / 2, h / 2);
            }
        }
        if let Some(c) = &fp.courtyard {
            let (w, h) = match c.size.as_slice() {
                [w, h] => (w.femto, h.femto),
                [d] => (d.femto, d.femto),
                _ => (0, 0),
            };
            if w > 0 && h > 0 {
                union_bbox(&mut b, c.at.0.femto, c.at.1.femto, w / 2, h / 2);
            }
        }
    }
    b.unwrap_or((-NOMINAL_HALF, -NOMINAL_HALF, NOMINAL_HALF, NOMINAL_HALF))
}

/// Locked placements (`place <inst> at (x, y)`) as component path → origin in
/// femto-mm. A placement tool treats these as pre-placed/fixed.
fn placed_positions(ir: &DesignIr) -> BTreeMap<String, (i128, i128)> {
    ir.layout
        .placements
        .iter()
        .map(|p| (p.path.clone(), (p.at.0.femto, p.at.1.femto)))
        .collect()
}

/// Component staging positions (component path → origin in femto-mm), keyed so
/// the emit loop can look each one up. Empty when the design declares no
/// `board_outline` (nothing to stage against — components keep the (0,0)
/// placeholder). Otherwise: a deterministic shelf-packed grid immediately to
/// the RIGHT of the outline, so every component's full footprint lies outside
/// the perimeter (Quilter's "please place me" signal) and no two overlap.
/// Locked (`placed`) components are skipped — they keep their fixed position.
fn staging_positions(
    world: &World,
    ir: &DesignIr,
    insts: &[&crate::ir::IrInstance],
    placed: &BTreeMap<String, (i128, i128)>,
) -> BTreeMap<String, (i128, i128)> {
    let mut out = BTreeMap::new();
    // Stage against the RESOLVED outline geometry's bounding box (RFC-020).
    let Some(g) = ir
        .layout
        .board_outline
        .as_ref()
        .and_then(|b| b.geom.as_ref())
    else {
        return out;
    };
    let ((ob_lo_x, _ob_lo_y), (ob_hi_x, ob_hi_y)) = g.bbox;
    let board_w = ob_hi_x - ob_lo_x;
    let margin = 5 * FEMTO_MM;
    let gap = 2 * FEMTO_MM;
    let out_right = ob_hi_x;
    let out_top = ob_hi_y;
    let start_x = out_right + margin;
    // Shelf width ≈ the board width, but never narrower than the widest
    // component (so no component overflows its shelf and the block stays a
    // tidy rectangle beside the board rather than a long strip).
    let bbox = |inst: &crate::ir::IrInstance| footprint_bbox(world, &part_fields(world, inst).2);
    let to_stage: Vec<&crate::ir::IrInstance> = insts
        .iter()
        .copied()
        .filter(|i| !placed.contains_key(&i.path))
        .collect();
    let widest = to_stage
        .iter()
        .map(|i| {
            let (lo_x, _, hi_x, _) = bbox(i);
            hi_x - lo_x
        })
        .max()
        .unwrap_or(0);
    let limit_x = start_x + board_w.max(widest);
    let mut cursor_x = start_x;
    let mut row_top = out_top; // rows extend downward from the board's top edge
    let mut row_h = 0i128;
    for inst in &to_stage {
        let (lo_x, lo_y, hi_x, hi_y) = bbox(inst);
        let (bw, bh) = (hi_x - lo_x, hi_y - lo_y);
        if cursor_x > start_x && cursor_x + bw > limit_x {
            row_top -= row_h + gap;
            cursor_x = start_x;
            row_h = 0;
        }
        // Left edge of the bbox at cursor_x, top at row_top; the ORIGIN is
        // offset by the bbox's own min corner (footprints aren't origin-centered).
        out.insert(inst.path.clone(), (cursor_x - lo_x, (row_top - bh) - lo_y));
        cursor_x += bw + gap;
        row_h = row_h.max(bh);
    }
    out
}

/// Same principal-value rule as the KiCad/BOM emitters.
fn principal_value(inst: &crate::ir::IrInstance) -> String {
    const PRINCIPAL: [&str; 4] = ["capacitance", "resistance", "inductance", "frequency"];
    for field in PRINCIPAL {
        if let Some(v) = inst.specs.get(field) {
            return v.text.clone();
        }
    }
    crate::resolve::short(&inst.device).to_string()
}

fn sorted_instances(ir: &DesignIr) -> Vec<&crate::ir::IrInstance> {
    let mut insts: Vec<_> = ir.instances.values().collect();
    insts.sort_by_key(|i| crate::emit::designator_sort_key(i.designator.as_deref().unwrap_or("")));
    insts
}

/// designator → sanitized, collision-free XML spelling. The designator is
/// the document's `componentKey`; the XSD enforces uniqueness AND that
/// `RefDes/@name` / `PinRef/@componentRef` resolve to it, so every refdes
/// must be spelled through this one table. Deterministic: designators
/// iterate in designator order; post-sanitize collisions (possible only
/// when a `designator_prefix` carries non-identifier characters) get `_`
/// suffixes in that order.
fn refdes_table(ir: &DesignIr) -> BTreeMap<String, String> {
    let mut designators: Vec<String> = ir
        .instances
        .values()
        .map(|i| i.designator.clone().unwrap_or_else(|| "?".to_string()))
        .collect();
    designators.sort_by_key(|d| crate::emit::designator_sort_key(d));
    let mut used = BTreeSet::new();
    let mut table = BTreeMap::new();
    for d in designators {
        if table.contains_key(&d) {
            continue; // injectivity guarantees this is unreachable; defensive
        }
        let mut name = sanitize(&d, true);
        while !used.insert(name.clone()) {
            name.push('_');
        }
        table.insert(d, name);
    }
    table
}

/// One entry per PHYSICAL pin of each connected logical pin — the same
/// expansion the KiCad emitter performs for its `node` list, same order.
fn physical_pins(world: &World, ir: &DesignIr, net: &crate::ir::IrNet) -> Vec<(String, String)> {
    let mut nodes: Vec<(String, String)> = Vec::new();
    for (path, pin) in &net.members {
        let inst = &ir.instances[path];
        let refdes = inst.designator.as_deref().unwrap_or("?").to_string();
        let device = &world.devices[&inst.device];
        if let Some(dev_pin) = device
            .pins_for(inst.variant.as_deref())
            .iter()
            .find(|p| p.name.name == *pin)
        {
            for num in &dev_pin.numbers {
                nodes.push((refdes.clone(), num.text.clone()));
            }
        }
    }
    nodes.sort_by(|a, b| {
        (crate::emit::designator_sort_key(&a.0), pin_sort_key(&a.1))
            .cmp(&(crate::emit::designator_sort_key(&b.0), pin_sort_key(&b.1)))
    });
    nodes
}

fn pin_sort_key(pin: &str) -> (u64, String) {
    match pin.parse::<u64>() {
        Ok(n) => (n, String::new()),
        Err(_) => (u64::MAX, pin.to_string()),
    }
}

/// XML attribute escaping (attributes are double-quoted here), hardened for
/// hostile-but-legal CoHDL strings (adversarial findings):
///
/// - tab/CR/LF become character references — a literal tab in an attribute
///   value is folded to a space by every conforming parser (XML 1.0 §3.3.3
///   attribute-value normalization), which would silently diverge from
///   layout.json/BOM and can collide XSD key values;
/// - characters XML 1.0 forbids OUTRIGHT (C0 controls other than
///   tab/LF/CR — illegal even as character references) are replaced with
///   U+FFFD. This is the emitter's ONE non-value-preserving projection,
///   disclosed in docs/ipc2581.md; the alternative is a non-well-formed
///   document no parser will open.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' => out.push_str("&#9;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            // Every scalar XML 1.0 forbids OUTRIGHT (illegal even as a
            // character reference) is replaced with U+FFFD. This is the
            // full `Char` predicate, not just C0: U+FFFE/U+FFFF are equally
            // forbidden and previously slipped through the catch-all
            // (review F3), yielding a well-formed-looking build that
            // `xmllint` rejects. Disclosed lossy projection (docs/ipc2581.md).
            c if !xml_char_ok(c) => out.push('\u{FFFD}'),
            _ => out.push(c),
        }
    }
    out
}

/// The XML 1.0 `Char` production (§2.2): tab/LF/CR, U+0020–U+D7FF,
/// U+E000–U+FFFD, U+10000–U+10FFFF. Surrogates cannot be a Rust `char`;
/// U+FFFE, U+FFFF, and the C0 controls other than tab/LF/CR are the
/// reachable exclusions.
fn xml_char_ok(c: char) -> bool {
    matches!(c,
        '\u{9}' | '\u{A}' | '\u{D}'
        | '\u{20}'..='\u{D7FF}'
        | '\u{E000}'..='\u{FFFD}'
        | '\u{10000}'..='\u{10FFFF}')
}

/// The lossy char projection `esc` applies (forbidden scalar → U+FFFD),
/// WITHOUT the XML entity escaping — the key on which post-projection
/// collisions must be resolved. Two manufacturer strings differing only in
/// distinct forbidden control characters both project here to the same
/// bytes, so an id/key table built over this value can `_`-disambiguate
/// them before they reach the schema as duplicate keys (review F3).
fn project_forbidden(s: &str) -> String {
    s.chars()
        .map(|c| if xml_char_ok(c) { c } else { '\u{FFFD}' })
        .collect()
}

/// Raw manufacturer name → its collision-free `Enterprise/@id`. Built once
/// over the sorted manufacturer set and used for both the `<Enterprise>`
/// declarations and every `AvlVendor/@enterpriseRef`, so the two always
/// agree and no two manufacturers ever share an id (the schema's
/// `enterpriseKey` forbids duplicate ids — review F3).
fn enterprise_map(bom: &[BomGroup]) -> BTreeMap<String, String> {
    let mut used = BTreeSet::new();
    used.insert("cohdl".to_string()); // the fixed owner Enterprise id
    let mut table = BTreeMap::new();
    for mfr in manufacturers(bom) {
        let mut id = project_forbidden(&enterprise_id(&mfr));
        while !used.insert(id.clone()) {
            id.push('_');
        }
        table.insert(mfr, id);
    }
    table
}

/// Restrict a name to the XSD's identifier charsets: `qualifiedNameType`
/// (`allow_colon`) or `shortName` (without). Anything outside becomes `_`;
/// empty input becomes `_` (the patterns allow empty, but an empty key/ref
/// is useless to a consumer).
fn sanitize(s: &str, allow_colon: bool) -> String {
    // The vendored-XSD character classes (review F6): both `qualifiedNameType`
    // and `shortName` allow `<` and `>`, which the old invented set stripped
    // — diverging the XML refdes (`R<1` → `R_1`) from the `.net`/CSV. Colon
    // is qualifiedNameType-only. `shortName` also allows `#`/`/`, but the BOM
    // `key` is emitted in BOTH a shortName slot (`OEMDesignNumber`) and a
    // qualifiedNameType slot (`AvlMpn/@name`), so those two are deliberately
    // NOT admitted here — the intersection is what is always safe.
    let ok = |c: char| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '+' | '<' | '>' => true,
        ':' => allow_colon,
        _ => false,
    };
    let mut out: String = s.chars().map(|c| if ok(c) { c } else { '_' }).collect();
    if out.is_empty() {
        out.push('_');
    }
    out
}

// ===========================================================================
// RFC-015 physical model — the real copper Quilter shows and routes.
//
// Beyond the abstract `Package/Pin` land pattern, an importable IPC-2581 must
// carry: a `DictionaryStandard` of primitive shapes, `PadStackDef`s (copper +
// mask + paste + plated-hole), and side-aware `LayerFeature`s of every PLACED
// physical feature at absolute board positions, each tied to its component pin
// (and thus its net).
// Without these a consumer (Quilter) can parse the document but sees only
// package courtyards — no pads, holes, or ratsnest. (Finding: .co/invalid-
// ipc2581.xml.)

/// A reusable primitive shape (femto). Circle stores `w == h == diameter`.
#[derive(PartialEq, Eq, Clone)]
struct Prim {
    shape: crate::ast::PadShape,
    w: i128,
    h: i128,
    /// One 45-degree corner cut, measured along both adjacent edges.
    chamfer: Option<(crate::ast::PadCorner, i128)>,
    /// Radius applied to all four corners of a rectangular primitive.
    corner_radius: Option<i128>,
    polygon: Option<Vec<(i128, i128)>>,
}

/// A reusable padstack: a primitive plus an optional through-hole. `plated` is
/// false only for an RFC-022 non_plated mount_hole (a bare drilled hole, no
/// copper); every electrical pad and every plated mount_hole is `plated`.
#[derive(PartialEq, Eq, Clone)]
struct PadStack {
    /// Copper primitive (and package-pin geometry).
    prim: usize,
    /// Solder-mask opening; differs from copper when `mask_expansion` is set.
    mask_prim: usize,
    /// Solder-paste aperture; `None` suppresses paste for this SMD pad.
    paste_prims: Vec<usize>,
    tht: bool,
    drill: i128,
    plated: bool,
    /// Physical face of an SMD padstack after combining the footprint pad's
    /// local copper layer with the component-side flip. `true` gets B-layer
    /// pad defs. Always `false` for THT (one definition spans both faces).
    bottom: bool,
}

/// One placed copper pad — absolute board position + the component pin it is.
/// `hole` marks an RFC-022 mount_hole: a mechanical locating hole with no net
/// and no device pin (no `PinRef`, no copper unless plated).
struct PlacedPad {
    refdes: String,
    pin: String,
    padstack: usize,
    prim: usize,
    x: i128,
    y: i128,
    rot: u16,
    tht: bool,
    hole: bool,
    /// Physical face after combining pad-local layer and component side. An
    /// SMD placement lands on only that face's copper/mask/paste layers; a
    /// through-hole placement spans both faces regardless.
    bottom: bool,
}

struct Physical {
    prims: Vec<Prim>,
    padstacks: Vec<PadStack>,
    pads: Vec<PlacedPad>,
    /// RFC-031 silkscreen, already transformed to board coordinates. Every
    /// primitive is reduced to a filled polygon (a stroke IS a rectangle of the
    /// stroke's width) because IPC's `Features` accepts a `StandardShape` —
    /// `Line`/`Arc` are `Simple` and cannot appear there, while `Contour`
    /// carries any of them faithfully.
    silk: Vec<PlacedSilk>,
}

/// One silkscreen polygon on the board: a closed ring, plus which side's
/// silkscreen layer it belongs to.
struct PlacedSilk {
    points: Vec<(i128, i128)>,
    bottom: bool,
}

/// Rotate a pad offset by any whole-degree angle (CCW).
///
/// Delegates to [`crate::trig`], whose fixed-point table is exact at
/// 0/90/180/270 — so this stays bit-identical to the exact-swap implementation
/// it replaces for every placement authored under RFC-020's closed set.
fn rotate(px: i128, py: i128, rot: u16) -> (i128, i128) {
    crate::trig::rotate(px, py, rot)
}

fn dedup<T: PartialEq>(v: &mut Vec<T>, x: T) -> usize {
    match v.iter().position(|e| *e == x) {
        Some(i) => i,
        None => {
            v.push(x);
            v.len() - 1
        }
    }
}

fn copper_prim(pad: &crate::ast::PadDef, shape: crate::ast::PadShape, w: i128, h: i128) -> Prim {
    Prim {
        shape,
        w,
        h,
        chamfer: pad
            .chamfer
            .as_ref()
            .map(|(corner, cut, _)| (*corner, cut.femto)),
        corner_radius: pad.corner_radius.as_ref().map(|(radius, _)| radius.femto),
        polygon: None,
    }
}

/// Stable preference for the one Package/Pin representative of a repeated
/// electrical number. Physical layer features retain all placements.
fn logical_pin_cmp(
    a: (&crate::ast::PadPlace, &crate::ast::PadDef),
    b: (&crate::ast::PadPlace, &crate::ast::PadDef),
) -> std::cmp::Ordering {
    fn layer_rank(pad: &crate::ast::PadDef) -> u8 {
        match pad.plating.as_ref().map(|(p, _)| p) {
            Some(crate::ast::PadPlating::Smd) => {
                if matches!(pad.layer, Some((crate::ast::PadLayer::BottomCopper, _))) {
                    1
                } else {
                    0
                }
            }
            Some(crate::ast::PadPlating::PlatedThroughHole) => 2,
            None => 3,
        }
    }
    fn spans(pad: &crate::ast::PadDef) -> (i128, i128) {
        let (w, h) = match pad.size.as_slice() {
            [d] => (d.femto, d.femto),
            [w, h, ..] => (w.femto, h.femto),
            [] => (0, 0),
        };
        (w.max(h), w.min(h))
    }
    let (ap, ad) = a;
    let (bp, bd) = b;
    let (amax, amin) = spans(ad);
    let (bmax, bmin) = spans(bd);
    layer_rank(ad)
        .cmp(&layer_rank(bd))
        // Within one physical kind, prefer the most descriptive primary land.
        .then_with(|| bmax.cmp(&amax))
        .then_with(|| bmin.cmp(&amin))
        // Source-order-independent tie breakers.
        .then_with(|| ap.x.femto.cmp(&bp.x.femto))
        .then_with(|| ap.y.femto.cmp(&bp.y.femto))
        .then_with(|| ap.pad.name.cmp(&bp.pad.name))
        .then_with(|| ap.rotate.cmp(&bp.rotate))
}

fn mirror_prim_x(mut prim: Prim) -> Prim {
    prim.chamfer = prim.chamfer.map(|(corner, cut)| {
        let mirrored = match corner {
            crate::ast::PadCorner::TopLeft => crate::ast::PadCorner::TopRight,
            crate::ast::PadCorner::TopRight => crate::ast::PadCorner::TopLeft,
            crate::ast::PadCorner::BottomRight => crate::ast::PadCorner::BottomLeft,
            crate::ast::PadCorner::BottomLeft => crate::ast::PadCorner::BottomRight,
        };
        (mirrored, cut)
    });
    prim
}

/// Expanding a 45-degree chamfered polygon by `m` grows its edge cut by
/// `(2 - sqrt(2)) * m`.  The fixed-point coefficient is rounded at CoHDL's
/// 10^-15 geometry resolution; the decomposition avoids overflow for every
/// value admitted by `MAX_GEOM_FEMTO`.
fn chamfer_expansion(m: i128) -> i128 {
    const SCALE: i128 = 1_000_000_000_000_000;
    const TWO_MINUS_SQRT_TWO: i128 = 585_786_437_626_905;
    (m / SCALE) * TWO_MINUS_SQRT_TWO + ((m % SCALE) * TWO_MINUS_SQRT_TWO) / SCALE
}

fn mask_prim(pad: &crate::ast::PadDef, copper: &Prim) -> Prim {
    let margin = pad.mask_expansion.as_ref().map_or(0, |(m, _)| m.femto);
    Prim {
        shape: copper.shape,
        w: copper.w + 2 * margin,
        h: if matches!(copper.shape, crate::ast::PadShape::Annulus) {
            copper.h - 2 * margin
        } else {
            copper.h + 2 * margin
        },
        chamfer: copper
            .chamfer
            .map(|(corner, cut)| (corner, cut + chamfer_expansion(margin))),
        corner_radius: copper.corner_radius.map(|radius| radius + margin),
        polygon: copper.polygon.clone(),
    }
}

fn sector_polygon(
    outer: i128,
    inner: i128,
    plan: crate::resolve::SegmentedAnnulusSectorPlan,
) -> Vec<(i128, i128)> {
    let outer_r = outer as f64 / 2.0;
    let inner_r = inner as f64 / 2.0;
    let conservative_inner = inner_r / (plan.step_angle / 2.0).cos();
    let mut points = Vec::with_capacity(plan.vertices);
    for i in 0..=plan.segments {
        let angle = plan.start_angle + plan.step_angle * i as f64;
        points.push((
            (outer_r * angle.cos()).round() as i128,
            (outer_r * angle.sin()).round() as i128,
        ));
    }
    for i in (0..=plan.segments).rev() {
        let angle = plan.start_angle + plan.step_angle * i as f64;
        points.push((
            (conservative_inner * angle.cos()).round() as i128,
            (conservative_inner * angle.sin()).round() as i128,
        ));
    }
    points
}

fn paste_prims(pad: &crate::ast::PadDef, copper: &Prim) -> Vec<Prim> {
    match &pad.paste {
        None => vec![copper.clone()],
        Some((crate::ast::PadPaste::None, _)) => Vec::new(),
        Some((crate::ast::PadPaste::Rect(w, h), _)) => vec![Prim {
            shape: crate::ast::PadShape::Rect,
            w: w.femto,
            h: h.femto,
            chamfer: None,
            corner_radius: None,
            polygon: None,
        }],
        Some((crate::ast::PadPaste::SegmentedAnnulus(values), _)) => {
            let [outer, inner, gap] = values.as_ref();
            let Some(plan) =
                crate::resolve::segmented_annulus_plan(outer.femto, inner.femto, gap.femto)
            else {
                return Vec::new();
            };
            plan.sectors
                .into_iter()
                .map(|sector| Prim {
                    shape: crate::ast::PadShape::Annulus,
                    w: outer.femto,
                    h: inner.femto,
                    chamfer: None,
                    corner_radius: None,
                    polygon: Some(sector_polygon(outer.femto, inner.femto, sector)),
                })
                .collect()
        }
    }
}

/// Walk every instance's footprint pads, deduplicating exact copper/mask/paste
/// geometries into primitives and their plating/drill/physical-side combination
/// into padstacks, then place every authored physical occurrence on the board.
fn build_physical(
    world: &World,
    insts: &[&crate::ir::IrInstance],
    positions: &BTreeMap<String, (i128, i128)>,
    rotations: &BTreeMap<String, u16>,
    sides: &BTreeMap<String, crate::ast::PlacementSide>,
    refdes_map: &BTreeMap<String, String>,
) -> Physical {
    let mut prims: Vec<Prim> = Vec::new();
    let mut padstacks: Vec<PadStack> = Vec::new();
    let mut pads: Vec<PlacedPad> = Vec::new();
    let mut silk: Vec<PlacedSilk> = Vec::new();
    for inst in insts {
        let raw = inst.designator.as_deref().unwrap_or("?");
        let refdes = refdes_map[raw].clone();
        let (cx, cy) = positions.get(&inst.path).copied().unwrap_or((0, 0));
        let rot = rotations.get(&inst.path).copied().unwrap_or(0);
        // RFC-026: bottom-side pads mirror their local x BEFORE rotation —
        // the same order KiCad's own Flip-then-orient applies, so the board
        // built from this document matches pcbnew's native convention.
        let bottom = matches!(
            sides.get(&inst.path),
            Some(crate::ast::PlacementSide::Bottom)
        );
        let fp_name = part_fields(world, inst).2;
        let Some(fp) = world.footprints.get(&fp_name) else {
            continue;
        };
        // RFC-031: the footprint's silkscreen, markers already expanded, moved
        // onto the board with the SAME mirror-then-rotate order pads use.
        for ring in crate::emit::silk::polygons(world, fp) {
            let pts = ring
                .iter()
                .map(|(x, y)| {
                    let lx = if bottom { -*x } else { *x };
                    let (ox, oy) = rotate(lx, -*y, rot);
                    (cx + ox, -cy + oy)
                })
                .collect();
            silk.push(PlacedSilk {
                points: pts,
                bottom,
            });
        }
        for place in &fp.pads {
            let Some(pad) = world.pads.get(&place.pad.name) else {
                continue;
            };
            let (Some((shape, _)), Some((plating, _))) = (&pad.shape, &pad.plating) else {
                continue;
            };
            let (w, h) = match pad.size.as_slice() {
                [d] => (d.femto, d.femto),
                [w, h, ..] => (w.femto, h.femto),
                [] => continue,
            };
            let tht = matches!(plating, crate::ast::PadPlating::PlatedThroughHole);
            // PadDef.layer is local to the footprint. Flipping a component
            // swaps front/back, so a locally-bottom SMD land on a top-side
            // component (or locally-top on a bottom-side component) lands on
            // the physical back. THT pads span both faces regardless.
            let pad_bottom = !tht
                && (bottom ^ matches!(pad.layer, Some((crate::ast::PadLayer::BottomCopper, _))));
            let drill = if tht {
                // A slot reports its MINOR axis, exactly as an oval mount hole
                // does: IPC's `<Hole>` carries one scalar and the full extent
                // is already described by the padstack primitive.
                pad.drill
                    .as_ref()
                    .map(|(d, _)| d.minor().femto)
                    .unwrap_or(0)
            } else {
                0
            };
            // Flipping a component reflects its pad-local geometry as well as
            // its center offset. This matters for asymmetric chamfers; a
            // roundrect/oval remains byte-identical under the reflection.
            let copper = if bottom {
                mirror_prim_x(copper_prim(pad, *shape, w, h))
            } else {
                copper_prim(pad, *shape, w, h)
            };
            let mask = mask_prim(pad, &copper);
            let paste = if tht {
                Vec::new()
            } else {
                paste_prims(pad, &copper)
            };
            let prim = dedup(&mut prims, copper);
            let mask_prim = dedup(&mut prims, mask);
            let paste_prims = paste.into_iter().map(|p| dedup(&mut prims, p)).collect();
            let padstack = dedup(
                &mut padstacks,
                PadStack {
                    prim,
                    mask_prim,
                    paste_prims,
                    tht,
                    drill,
                    plated: true,
                    bottom: pad_bottom,
                },
            );
            // IPC-2581 is +y-up; CoHDL/KiCad author +y-down. Project by negating
            // every y — the local pad offset BEFORE rotation, and the component
            // position — while keeping the rotation value. (Verified against
            // KiCad's own `--version B` export: this reproduces its placement of
            // rotated, y-offset pads exactly; a naive reflection of the final
            // absolute position does not.)
            let lx = if bottom {
                -place.x.femto
            } else {
                place.x.femto
            };
            let (ox, oy) = rotate(lx, -place.y.femto, rot);
            pads.push(PlacedPad {
                refdes: refdes.clone(),
                pin: sanitize(&place.number.text, true),
                padstack,
                prim,
                x: cx + ox,
                y: -cy + oy,
                // Reflection reverses a pad-local rotation: R(component) ×
                // MirrorX × R(pad) = R(component - pad) × MirrorX.
                // Position is unaffected (rotation is about the pad centre).
                rot: if bottom {
                    (rot + 360 - (place.rotate % 360)) % 360
                } else {
                    (rot + place.rotate) % 360
                },
                tht,
                hole: false,
                bottom: pad_bottom,
            });
        }
        // RFC-022 mechanical locating holes — placed as through-holes with no
        // net and no PinRef. non_plated has no copper (a bare hole); plated
        // carries a copper ring sized to the drill.
        for mh in &fp.mount_holes {
            // RFC-023: a circle carries one `diameter`, a rect/oval a `size:
            // (w, h)`. Both project as hole geometry with no net reference.
            let (w, h) = match &mh.geom {
                crate::ast::MountHoleGeom::Diameter(d) => (d.femto, d.femto),
                crate::ast::MountHoleGeom::Size(dims, _) => (
                    dims.first().map_or(0, |v| v.femto),
                    dims.get(1).map_or(0, |v| v.femto),
                ),
            };
            if w <= 0 || h <= 0 {
                continue; // non-positive: reported at declaration check
            }
            let plated = matches!(mh.plating, crate::ast::MountHolePlating::Plated);
            let prim = dedup(
                &mut prims,
                Prim {
                    shape: mh.shape_or_default(),
                    w,
                    h,
                    chamfer: None,
                    corner_radius: None,
                    polygon: None,
                },
            );
            let padstack = dedup(
                &mut padstacks,
                PadStack {
                    prim,
                    mask_prim: prim,
                    paste_prims: Vec::new(),
                    tht: true,
                    // IPC's `<Hole>` carries a single scalar diameter, so a
                    // non-circular hole reports its MINOR axis — the slot
                    // width, the conventional drill for a slotted hole. The
                    // full (w, h) extent is already carried by `prim`.
                    drill: w.min(h),
                    plated,
                    bottom: false,
                },
            );
            // RFC-026: a hole spans the board either way, but its POSITION
            // still mirrors with a bottom-side component.
            let lx = if bottom { -mh.x.femto } else { mh.x.femto };
            let (ox, oy) = rotate(lx, -mh.y.femto, rot);
            pads.push(PlacedPad {
                refdes: refdes.clone(),
                pin: String::new(),
                padstack,
                prim,
                x: cx + ox,
                y: -cy + oy,
                rot,
                tht: true,
                hole: true,
                bottom,
            });
        }
    }
    Physical {
        prims,
        padstacks,
        pads,
        silk,
    }
}

/// The primitive-shape element (`RectCenter`/`Circle`/`Oval`, `RectRound` for
/// rounded rectangles, or `Contour` for a one-corner chamfer), shared by the
/// `DictionaryStandard` entry, padstack pad defs, and placed features.
fn prim_body(p: &Prim) -> String {
    if let Some(points) = &p.polygon {
        let mut s = String::from("<Contour><Polygon>");
        let (x0, y0) = points[0];
        let _ = write!(
            s,
            "<PolyBegin x=\"{}\" y=\"{}\"/>",
            geom::mm_femto(x0),
            geom::mm_femto(-y0)
        );
        for (x, y) in points.iter().skip(1).chain(points.first()) {
            let _ = write!(
                s,
                "<PolyStepSegment x=\"{}\" y=\"{}\"/>",
                geom::mm_femto(*x),
                geom::mm_femto(-*y)
            );
        }
        s.push_str("<FillDesc fillProperty=\"FILL\"/></Polygon></Contour>");
        return s;
    }
    if let Some((corner, cut)) = p.chamfer {
        // Coordinates are carried in tenths of a femto-mm so odd dimensions
        // retain their exact half.  Author coordinates are +Y-down; negate Y
        // when projecting the contour into IPC-2581's +Y-up frame.
        let left = -p.w * 5;
        let right = p.w * 5;
        let top = -p.h * 5;
        let bottom = p.h * 5;
        let cut = cut * 10;
        let points: Vec<(i128, i128)> = match corner {
            crate::ast::PadCorner::TopLeft => vec![
                (left + cut, top),
                (right, top),
                (right, bottom),
                (left, bottom),
                (left, top + cut),
            ],
            crate::ast::PadCorner::TopRight => vec![
                (left, top),
                (right - cut, top),
                (right, top + cut),
                (right, bottom),
                (left, bottom),
            ],
            crate::ast::PadCorner::BottomRight => vec![
                (left, top),
                (right, top),
                (right, bottom - cut),
                (right - cut, bottom),
                (left, bottom),
            ],
            crate::ast::PadCorner::BottomLeft => vec![
                (left, top),
                (right, top),
                (right, bottom),
                (left + cut, bottom),
                (left, bottom - cut),
            ],
        };
        let mut s = String::from("<Contour><Polygon>");
        let (x0, y0) = points[0];
        let _ = write!(
            s,
            "<PolyBegin x=\"{}\" y=\"{}\"/>",
            geom::scaled(x0, 16),
            geom::scaled(-y0, 16)
        );
        for (x, y) in points.iter().skip(1).chain(points.first()) {
            let _ = write!(
                s,
                "<PolyStepSegment x=\"{}\" y=\"{}\"/>",
                geom::scaled(*x, 16),
                geom::scaled(-*y, 16)
            );
        }
        s.push_str("<FillDesc fillProperty=\"FILL\"/></Polygon></Contour>");
        return s;
    }
    if let Some(radius) = p.corner_radius {
        return format!(
            "<RectRound width=\"{}\" height=\"{}\" radius=\"{}\" upperRight=\"true\" upperLeft=\"true\" lowerLeft=\"true\" lowerRight=\"true\"/>",
            geom::mm_femto(p.w),
            geom::mm_femto(p.h),
            geom::mm_femto(radius)
        );
    }
    match p.shape {
        crate::ast::PadShape::Annulus => {
            let outer = p.w / 2;
            let inner = p.h / 2;
            format!(
                "<Contour><Polygon><PolyBegin x=\"{}\" y=\"0\"/><PolyStepCurve x=\"{}\" y=\"0\" centerX=\"0\" centerY=\"0\" clockwise=\"false\"/><PolyStepCurve x=\"{}\" y=\"0\" centerX=\"0\" centerY=\"0\" clockwise=\"false\"/><FillDesc fillProperty=\"FILL\"/></Polygon><Cutout><PolyBegin x=\"{}\" y=\"0\"/><PolyStepCurve x=\"{}\" y=\"0\" centerX=\"0\" centerY=\"0\" clockwise=\"true\"/><PolyStepCurve x=\"{}\" y=\"0\" centerX=\"0\" centerY=\"0\" clockwise=\"true\"/><FillDesc fillProperty=\"FILL\"/></Cutout></Contour>",
                geom::mm_femto(outer),
                geom::mm_femto(-outer),
                geom::mm_femto(outer),
                geom::mm_femto(inner),
                geom::mm_femto(-inner),
                geom::mm_femto(inner)
            )
        }
        crate::ast::PadShape::Circle => format!("<Circle diameter=\"{}\"/>", geom::mm_femto(p.w)),
        crate::ast::PadShape::Rect => format!(
            "<RectCenter width=\"{}\" height=\"{}\"/>",
            geom::mm_femto(p.w),
            geom::mm_femto(p.h)
        ),
        crate::ast::PadShape::Oval => format!(
            "<Oval width=\"{}\" height=\"{}\"/>",
            geom::mm_femto(p.w),
            geom::mm_femto(p.h)
        ),
    }
}

/// The `DictionaryStandard` (Content section): one primitive per unique shape.
fn emit_dictionary(out: &mut String, phys: &Physical) {
    if phys.prims.is_empty() {
        return;
    }
    out.push_str("    <DictionaryStandard units=\"MILLIMETER\">\n");
    for (i, p) in phys.prims.iter().enumerate() {
        let _ = writeln!(
            out,
            "      <EntryStandard id=\"PRIM_{}\">{}</EntryStandard>",
            i,
            prim_body(p)
        );
    }
    out.push_str("    </DictionaryStandard>\n");
}

/// The physical layer set (CadData): the full silkscreen/paste/mask/copper/
/// dielectric stack a consumer needs (replaces the single synthetic `TOP`
/// layer), plus the board outline and the through-board drill span the plated
/// holes live on. Every layer any `PadstackPadDef`, `LayerFeature`, or
/// `StackupLayer` references must be declared here — the fabrication stack is
/// listed top→bottom, mirroring the reference exporter (KiCad), so a consumer
/// that builds its renderable-layer model from these declarations sees the mask
/// apertures the copper pads are revealed through.
fn emit_layers(out: &mut String) {
    for (name, func, side, _thickness) in FAB_LAYERS {
        let _ = writeln!(
            out,
            "      <Layer name=\"{}\" layerFunction=\"{}\" polarity=\"POSITIVE\" side=\"{}\"/>",
            name, func, side
        );
    }
    out.push_str(
        "      <Layer name=\"Edge.Cuts\" layerFunction=\"BOARD_OUTLINE\" polarity=\"POSITIVE\" side=\"ALL\"/>\n",
    );
    // The plated through-holes are emitted as located `<Hole>` features on
    // this span layer (the board-level form hole-rendering consumers key on;
    // the padstack-level `PadstackHoleDef` alone stays invisible).
    out.push_str(
        "      <Layer name=\"F.Cu_B.Cu\" layerFunction=\"DRILL\" polarity=\"POSITIVE\" side=\"ALL\">\n",
    );
    out.push_str("        <Span fromLayer=\"F.Cu\" toLayer=\"B.Cu\"/>\n");
    out.push_str("      </Layer>\n");
}

/// The fabrication stack, top→bottom: (name, layerFunction, side, thickness mm).
/// Non-zero thicknesses sum to the 1.6mm overall (0.01 mask + 0.035 Cu + 1.51
/// dielectric + 0.035 Cu + 0.01 mask). Silkscreen/paste are zero-thickness
/// process layers. This is exactly the set + order KiCad's `--version B`
/// exporter emits, which Quilter renders.
const FAB_LAYERS: [(&str, &str, &str, &str); 9] = [
    ("F.Silkscreen", "SILKSCREEN", "TOP", "0"),
    ("F.Paste", "SOLDERPASTE", "TOP", "0"),
    ("F.Mask", "SOLDERMASK", "TOP", "0.01"),
    ("F.Cu", "CONDUCTOR", "TOP", "0.035"),
    ("DIELECTRIC_1", "DIELCORE", "INTERNAL", "1.51"),
    ("B.Cu", "CONDUCTOR", "BOTTOM", "0.035"),
    ("B.Mask", "SOLDERMASK", "BOTTOM", "0.01"),
    ("B.Paste", "SOLDERPASTE", "BOTTOM", "0"),
    ("B.Silkscreen", "SILKSCREEN", "BOTTOM", "0"),
];

/// The stackup (CadData): the full top→bottom fabrication sequence, one
/// `StackupLayer` per `FAB_LAYERS` row. Listing the mask & paste layers (not
/// just the copper) matters: a consumer that builds its renderable-layer list
/// from the `StackupLayer` sequence needs the mask rows present, or it never
/// composites "copper pad revealed through a mask aperture" and shows no pads.
fn emit_stackup(out: &mut String) {
    out.push_str("      <Stackup name=\"stackup\" overallThickness=\"1.6\" tolPlus=\"0\" tolMinus=\"0\" whereMeasured=\"MASK\">\n");
    out.push_str(
        "        <StackupGroup name=\"grp\" thickness=\"1.6\" tolPlus=\"0\" tolMinus=\"0\">\n",
    );
    for (i, (l, _func, _side, thickness)) in FAB_LAYERS.iter().enumerate() {
        let _ = writeln!(
            out,
            "          <StackupLayer layerOrGroupRef=\"{}\" thickness=\"{}\" tolPlus=\"0\" tolMinus=\"0\" sequence=\"{}\"/>",
            l, thickness, i
        );
    }
    out.push_str("        </StackupGroup>\n");
    out.push_str("      </Stackup>\n");
}

/// The `PadStackDef`s (Step, before `Datum`): an SMD stack has copper/mask and
/// optional paste on exactly its physical face; a THT stack has copper/mask on
/// both faces plus a plated `PadstackHoleDef`.
fn emit_padstacks(out: &mut String, phys: &Physical) {
    for (i, ps) in phys.padstacks.iter().enumerate() {
        let _ = writeln!(out, "        <PadStackDef name=\"PADSTACK_{}\">", i);
        if ps.tht && ps.drill > 0 {
            let _ = writeln!(
                out,
                "          <PadstackHoleDef name=\"HOLE_{}\" diameter=\"{}\" platingStatus=\"{}\" plusTol=\"0\" minusTol=\"0\" x=\"0\" y=\"0\"/>",
                i,
                geom::mm_femto(ps.drill),
                if ps.plated { "PLATED" } else { "NONPLATED" }
            );
        }
        // The copper/mask/paste layers this padstack lands on. An RFC-022
        // non_plated mount_hole is a bare drilled hole — no copper at all.
        let mut layers: Vec<(&str, usize)> = Vec::new();
        if ps.plated && ps.tht {
            layers.extend([
                ("F.Cu", ps.prim),
                ("F.Mask", ps.mask_prim),
                ("B.Cu", ps.prim),
                ("B.Mask", ps.mask_prim),
            ]);
        } else if ps.plated && ps.bottom {
            // An SMD padstack resolved to the physical back lands on B-layers.
            layers.extend([("B.Cu", ps.prim), ("B.Mask", ps.mask_prim)]);
            for paste in &ps.paste_prims {
                layers.push(("B.Paste", *paste));
            }
        } else if ps.plated {
            layers.extend([("F.Cu", ps.prim), ("F.Mask", ps.mask_prim)]);
            for paste in &ps.paste_prims {
                layers.push(("F.Paste", *paste));
            }
        }
        for (layer, prim) in layers {
            let _ = writeln!(
                out,
                "          <PadstackPadDef layerRef=\"{}\" padUse=\"REGULAR\"><Location x=\"0\" y=\"0\"/><StandardPrimitiveRef id=\"PRIM_{}\"/></PadstackPadDef>",
                layer, prim
            );
        }
        out.push_str("        </PadStackDef>\n");
    }
}

/// The `LayerFeature`s (Step, after `LogicalNet`): every physical placement is
/// instanced on each layer its padstack lands on. THT placements span both
/// copper/mask faces; SMD placements use only their resolved physical face's
/// copper/mask and optional paste. Each feature is tied to its component pin
/// (`PinRef`) and net (`Set/@net`), positioned + rotated on the board.
/// The mask/paste features matter for visibility: a consumer compositing
/// "visible pad = copper through a mask aperture" sees no pads at all when
/// only the copper is instanced. A final drill `LayerFeature` (`F.Cu_B.Cu`)
/// carries one located, plated `<Hole>` per through-hole pad.
/// The per-layer pad filter mirrors `emit_padstacks`' side-aware layer lists
/// exactly; suppressed paste apertures never create paste features.
fn emit_layer_features(
    out: &mut String,
    phys: &Physical,
    net_of: &BTreeMap<(String, String), String>,
) {
    // Per-layer membership (RFC-026 side-aware): a THT pad spans both outer
    // faces regardless of its component's side; an SMD pad lands only on its
    // own side's copper/mask/paste.
    type LayerFilter = fn(&PlacedPad) -> bool;
    let layers: [(&str, LayerFilter); 6] = [
        ("F.Cu", |p| p.tht || !p.bottom),
        ("B.Cu", |p| p.tht || p.bottom),
        ("F.Mask", |p| p.tht || !p.bottom),
        ("B.Mask", |p| p.tht || p.bottom),
        ("F.Paste", |p| !p.tht && !p.bottom),
        ("B.Paste", |p| !p.tht && p.bottom),
    ];
    for (layer, keep) in layers {
        let pads: Vec<&PlacedPad> = phys
            .pads
            .iter()
            // A non_plated mount_hole (padstack not plated) has no copper — it
            // appears only on the drill layer below, never here.
            .filter(|p| phys.padstacks[p.padstack].plated)
            .filter(|p| keep(p))
            .filter(|p| {
                !layer.ends_with(".Paste") || !phys.padstacks[p.padstack].paste_prims.is_empty()
            })
            .collect();
        if pads.is_empty() {
            continue;
        }
        let _ = writeln!(out, "        <LayerFeature layerRef=\"{}\">", layer);
        for p in pads {
            let layer_prims = if layer.ends_with(".Mask") {
                vec![phys.padstacks[p.padstack].mask_prim]
            } else if layer.ends_with(".Paste") {
                phys.padstacks[p.padstack].paste_prims.clone()
            } else {
                vec![p.prim]
            };
            for layer_prim in layer_prims {
                let net = net_of.get(&(p.refdes.clone(), p.pin.clone()));
                match net {
                    Some(n) => {
                        let _ = writeln!(out, "          <Set net=\"{}\">", esc(n));
                    }
                    None => out.push_str("          <Set>\n"),
                }
                let _ = writeln!(
                    out,
                    "            <Pad padstackDefRef=\"PADSTACK_{}\">",
                    p.padstack
                );
                if p.rot != 0 {
                    let _ = writeln!(out, "              <Xform rotation=\"{}\"/>", p.rot);
                }
                let _ = writeln!(
                    out,
                    "              <Location x=\"{}\" y=\"{}\"/>",
                    geom::mm_femto(p.x),
                    geom::mm_femto(p.y)
                );
                let _ = writeln!(
                    out,
                    "              <StandardPrimitiveRef id=\"PRIM_{}\"/>",
                    layer_prim
                );
                // A mount_hole is mechanical — no device pin to reference.
                if !p.hole {
                    let _ = writeln!(
                        out,
                        "              <PinRef componentRef=\"{}\" pin=\"{}\"/>",
                        esc(&p.refdes),
                        esc(&p.pin)
                    );
                }
                out.push_str("            </Pad>\n");
                out.push_str("          </Set>\n");
            }
        }
        out.push_str("        </LayerFeature>\n");
    }
    // RFC-031: silkscreen. IPC-2581 had no silkscreen output at all before
    // this; each polygon becomes a filled `Contour` under `Features`, which is
    // the format's own native way to carry arbitrary silk art (`Line`/`Arc`
    // are `Simple` and are not valid `Feature`s).
    for (layer, want_bottom) in [("F.Silkscreen", false), ("B.Silkscreen", true)] {
        let rings: Vec<&PlacedSilk> = phys
            .silk
            .iter()
            .filter(|r| r.bottom == want_bottom && r.points.len() >= 3)
            .collect();
        if rings.is_empty() {
            continue;
        }
        let _ = writeln!(out, "        <LayerFeature layerRef=\"{}\">", layer);
        for r in rings {
            out.push_str("          <Set>\n");
            out.push_str("            <Features>\n");
            out.push_str("              <Location x=\"0\" y=\"0\"/>\n");
            out.push_str("              <Contour>\n");
            let (x0, y0) = r.points[0];
            let _ = writeln!(
                out,
                "                <Polygon>\n                  <PolyBegin x=\"{}\" y=\"{}\"/>",
                geom::mm_femto(x0),
                geom::mm_femto(y0)
            );
            for (x, y) in r.points.iter().skip(1) {
                let _ = writeln!(
                    out,
                    "                  <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                    geom::mm_femto(*x),
                    geom::mm_femto(*y)
                );
            }
            // Close the ring explicitly — a Contour polygon is a closed region.
            let _ = writeln!(
                out,
                "                  <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                geom::mm_femto(x0),
                geom::mm_femto(y0)
            );
            out.push_str("                  <FillDesc fillProperty=\"FILL\"/>\n");
            out.push_str("                </Polygon>\n");
            out.push_str("              </Contour>\n");
            out.push_str("            </Features>\n");
            out.push_str("          </Set>\n");
        }
        out.push_str("        </LayerFeature>\n");
    }
    // The drill layer: one located, plated `<Hole>` per through-hole pad —
    // the board-level projection of the padstacks' `PadstackHoleDef` (which
    // is defined at padstack origin and never placed; hole-rendering
    // consumers key on these features). `Set/@geometry` names the padstack,
    // mirroring the reference-exporter corpus.
    let holes: Vec<(usize, &PlacedPad)> = phys
        .pads
        .iter()
        .filter(|p| p.tht && phys.padstacks[p.padstack].drill > 0)
        .enumerate()
        .collect();
    if !holes.is_empty() {
        out.push_str("        <LayerFeature layerRef=\"F.Cu_B.Cu\">\n");
        for (k, p) in holes {
            let net = net_of.get(&(p.refdes.clone(), p.pin.clone()));
            match net {
                Some(n) => {
                    let _ = writeln!(
                        out,
                        "          <Set geometry=\"PADSTACK_{}\" net=\"{}\">",
                        p.padstack,
                        esc(n)
                    );
                }
                None => {
                    let _ = writeln!(out, "          <Set geometry=\"PADSTACK_{}\">", p.padstack);
                }
            }
            let _ = writeln!(
                out,
                "            <Hole name=\"H{}\" diameter=\"{}\" platingStatus=\"{}\" plusTol=\"0\" minusTol=\"0\" x=\"{}\" y=\"{}\"/>",
                k,
                geom::mm_femto(phys.padstacks[p.padstack].drill),
                if phys.padstacks[p.padstack].plated { "PLATED" } else { "NONPLATED" },
                geom::mm_femto(p.x),
                geom::mm_femto(p.y)
            );
            out.push_str("          </Set>\n");
        }
        out.push_str("        </LayerFeature>\n");
    }
}

/// (sanitized refdes, sanitized pin) → net name, for tying placed copper to a
/// net — the same physical-pin expansion the LogicalNets use.
fn net_membership(
    world: &World,
    ir: &DesignIr,
    refdes_map: &BTreeMap<String, String>,
) -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    for net in &ir.nets {
        let nn = sanitize(&net.name, true);
        for (refdes, pin) in physical_pins(world, ir, net) {
            out.insert(
                (refdes_map[&refdes].clone(), sanitize(&pin, true)),
                nn.clone(),
            );
        }
    }
    out
}

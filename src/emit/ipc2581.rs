//! RFC-015: IPC-2581 (revision B1) emitter — the partner-handoff artifact.
//!
//! A *partially-specified* IPC-2581 document: logical design complete
//! (netlist, components, resolved specs, RFC-013 layout constraints) and,
//! since RFC-018, PARTIAL footprint geometry (per-pad `Package/Pin` +
//! courtyard `Outline`) for pad-bearing footprints — the `Pin` subset omits
//! the footprint's `silkscreen_ref` (present in the `.kicad_mod`). The pad's
//! plating now rides the `Pin/@mountType` attribute (SMD → `SURFACE_MOUNT_PAD`,
//! PTH → `THROUGH_HOLE_HOLE`), narrowing review R5-8: the copper `layer` and
//! the exact `drill` diameter still have no home on `PinType` (they need a
//! Step-level `PadStackDef`, deferred — moot for this all-SMD, single-layer
//! board where every pad is `top_copper`/`smd`/drill-less). Physical layout
//! is otherwise still minimal: no routing, no stackup, and no real PLACEMENT —
//! when a board outline exists, components are STAGED in a grid just outside
//! the outline (a layout tool like Quilter treats components inside the
//! outline as locked/pre-placed and only places the ones left outside, so
//! staging outside is the idiom for "unplaced, please place me"; without an
//! outline they keep the `(0,0)` placeholder). A `Step/Profile`
//! board outline IS emitted when the design declares a `board_outline` (a
//! pragmatic extension beyond RFC-015's original cut —
//! docs/compliance-report.md); with real per-pad footprints (RFC-018) and
//! that outline, `--emit ipc2581` now carries the board boundary + land
//! patterns + netlist a layout partner (Quilter) needs, though component
//! placement is still theirs to perform. The document says so, visibly and
//! machine-readably: the
//! `FunctionMode` comment and a `COHDL_COMPLETENESS` attribute both carry
//! the `logical-complete,physical-minimal` marker (DR-021 calls this the
//! single most load-bearing design decision of the RFC — the output must
//! never silently overclaim completeness).
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
pub const COMPLETENESS: &str = "logical-complete,physical-minimal";

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
    let staging = staging_positions(world, ir, &insts);
    let name = sanitize(package_name, false);
    let step = sanitize(&ir.name, false);

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<IPC-2581 revision=\"B1\" xmlns=\"http://webstds.ipc.org/2581\">\n");

    // ---- Content: what the document contains, and for whom ----
    out.push_str("  <Content roleRef=\"Owner\">\n");
    let _ = writeln!(
        out,
        "    <FunctionMode mode=\"USERDEF\" level=\"1\" comment=\"{} — layout not yet performed\"/>",
        esc(COMPLETENESS)
    );
    let _ = writeln!(out, "    <StepRef name=\"{}\"/>", esc(&step));
    out.push_str("    <LayerRef name=\"TOP\"/>\n");
    if !bom.is_empty() {
        let _ = writeln!(out, "    <BomRef name=\"{}-bom\"/>", esc(&name));
        let _ = writeln!(out, "    <AvlRef name=\"{}-avl\"/>", esc(&name));
    }
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
    out.push_str(
        "      <Layer name=\"TOP\" layerFunction=\"CONDUCTOR\" side=\"TOP\" polarity=\"POSITIVE\"/>\n",
    );
    let _ = writeln!(out, "      <Step name=\"{}\">", esc(&step));
    let _ = writeln!(
        out,
        "        <NonstandardAttribute name=\"COHDL_COMPLETENESS\" type=\"STRING\" value=\"{}\"/>",
        esc(COMPLETENESS)
    );
    out.push_str("        <Datum x=\"0\" y=\"0\"/>\n");

    // Board outline → Step/Profile: the single closed board perimeter a
    // downstream layout tool (Quilter) seeds placement/routing against. A
    // pragmatic extension beyond RFC-015's original physical-minimal cut
    // (which had no outline) — its presence is what lets `--emit ipc2581`
    // carry a real board boundary; corners are exact over the femto integers
    // (emit::geom), never floats. Absent when the design declares no
    // `board_outline` (the completeness marker still says physical-minimal).
    if let Some(bo) = &ir.layout.board_outline {
        let (cx, cy) = &bo.at;
        let (w, h) = &bo.size;
        let corners = [
            (geom::corner_lo(cx, w), geom::corner_lo(cy, h)),
            (geom::corner_hi(cx, w), geom::corner_lo(cy, h)),
            (geom::corner_hi(cx, w), geom::corner_hi(cy, h)),
            (geom::corner_lo(cx, w), geom::corner_hi(cy, h)),
        ];
        out.push_str("        <Profile>\n");
        out.push_str("          <Polygon>\n");
        let _ = writeln!(
            out,
            "            <PolyBegin x=\"{}\" y=\"{}\"/>",
            corners[0].0, corners[0].1
        );
        for (x, y) in corners.iter().skip(1).chain([corners[0].clone()].iter()) {
            let _ = writeln!(
                out,
                "            <PolyStepSegment x=\"{}\" y=\"{}\"/>",
                x, y
            );
        }
        out.push_str("            <LineDesc lineEnd=\"NONE\" lineWidth=\"0.1\"/>\n");
        out.push_str("          </Polygon>\n");
        out.push_str("        </Profile>\n");
    }

    // Packages: one per distinct footprint symbol. RFC-018: a pad-bearing
    // footprint projects REAL geometry (courtyard outline + one Pin per
    // pad); an RFC-017 stage-one placeholder keeps the zero-size idiom
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
                let corners = [
                    (geom::corner_lo(&c.at.0, w), geom::corner_lo(&c.at.1, h)),
                    (geom::corner_hi(&c.at.0, w), geom::corner_lo(&c.at.1, h)),
                    (geom::corner_hi(&c.at.0, w), geom::corner_hi(&c.at.1, h)),
                    (geom::corner_lo(&c.at.0, w), geom::corner_hi(&c.at.1, h)),
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
            for place in &f.pads {
                let Some(pad) = world.pads.get(&place.pad.name) else {
                    continue;
                };
                let (Some((shape, _)), Some((plating, _))) = (&pad.shape, &pad.plating) else {
                    continue;
                };
                let (pin_type, mount_type) = match plating {
                    crate::ast::PadPlating::Smd => ("SURFACE", "SURFACE_MOUNT_PAD"),
                    crate::ast::PadPlating::PlatedThroughHole => ("THRU", "THROUGH_HOLE_HOLE"),
                };
                let _ = writeln!(
                    out,
                    "          <Pin number=\"{}\" type=\"{}\" mountType=\"{}\">",
                    esc(&sanitize(&place.number.text, true)),
                    pin_type,
                    mount_type
                );
                let _ = writeln!(
                    out,
                    "            <Location x=\"{}\" y=\"{}\"/>",
                    geom::mm(&place.x),
                    geom::mm(&place.y)
                );
                match (shape, pad.size.as_slice()) {
                    (crate::ast::PadShape::Circle, [d]) => {
                        let _ = writeln!(out, "            <Circle diameter=\"{}\"/>", geom::mm(d));
                    }
                    (crate::ast::PadShape::Rect, [w, h]) => {
                        let _ = writeln!(
                            out,
                            "            <RectCenter width=\"{}\" height=\"{}\"/>",
                            geom::mm(w),
                            geom::mm(h)
                        );
                    }
                    (crate::ast::PadShape::Oval, [w, h]) => {
                        let _ = writeln!(
                            out,
                            "            <Oval width=\"{}\" height=\"{}\"/>",
                            geom::mm(w),
                            geom::mm(h)
                        );
                    }
                    _ => {
                        // Arity errors were reported at declaration check;
                        // keep the document well-formed with a zero circle.
                        out.push_str("            <Circle diameter=\"0\"/>\n");
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
        let _ = writeln!(
            out,
            "        <Component refDes=\"{}\" packageRef=\"{}\" part=\"{}\" layerRef=\"TOP\" mountType=\"OTHER\">",
            esc(&refdes_map[refdes]),
            esc(&packages[&footprint]),
            esc(&mpn)
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
        let (lx, ly) = match staging.get(&inst.path) {
            Some((x, y)) => (geom::mm_femto(*x), geom::mm_femto(*y)),
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
fn package_table(world: &World, ir: &DesignIr) -> BTreeMap<String, String> {
    let footprints: BTreeSet<String> = ir
        .instances
        .values()
        .map(|i| part_fields(world, i).2)
        .collect();
    let mut used = BTreeSet::new();
    let mut table = BTreeMap::new();
    for f in footprints {
        let mut name = sanitize(&f, true);
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
                let (w, h) = match pad.size.as_slice() {
                    [w, h] => (w.femto, h.femto),
                    [d] => (d.femto, d.femto),
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

/// Component staging positions (component path → origin in femto-mm), keyed so
/// the emit loop can look each one up. Empty when the design declares no
/// `board_outline` (nothing to stage against — components keep the (0,0)
/// placeholder). Otherwise: a deterministic shelf-packed grid immediately to
/// the RIGHT of the outline, so every component's full footprint lies outside
/// the perimeter (Quilter's "please place me" signal) and no two overlap.
fn staging_positions(
    world: &World,
    ir: &DesignIr,
    insts: &[&crate::ir::IrInstance],
) -> BTreeMap<String, (i128, i128)> {
    let mut out = BTreeMap::new();
    let Some(bo) = &ir.layout.board_outline else {
        return out;
    };
    let margin = 5 * FEMTO_MM;
    let gap = 2 * FEMTO_MM;
    let out_right = bo.at.0.femto + bo.size.0.femto / 2;
    let out_top = bo.at.1.femto + bo.size.1.femto / 2;
    let start_x = out_right + margin;
    // Shelf width ≈ the board width, but never narrower than the widest
    // component (so no component overflows its shelf and the block stays a
    // tidy rectangle beside the board rather than a long strip).
    let bbox = |inst: &crate::ir::IrInstance| footprint_bbox(world, &part_fields(world, inst).2);
    let widest = insts
        .iter()
        .map(|i| {
            let (lo_x, _, hi_x, _) = bbox(i);
            hi_x - lo_x
        })
        .max()
        .unwrap_or(0);
    let limit_x = start_x + bo.size.0.femto.max(widest);
    let mut cursor_x = start_x;
    let mut row_top = out_top; // rows extend downward from the board's top edge
    let mut row_h = 0i128;
    for inst in insts {
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

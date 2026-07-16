//! RFC-015 IPC-2581 emitter conformance.
//!
//! The two mandatory gradeability properties (DR-021):
//!
//! 1. **Schema validity** — every fixture document validates against the
//!    real `IPC-2581B1.xsd` (the IPC 2581 Consortium's published copy,
//!    vendored at `tests/schema/IPC-2581B1.xsd`), via `xmllint --schema`.
//!    xmllint ships with macOS and is installed in CI (the authoritative
//!    gate); if it is genuinely absent locally the validity tests skip with
//!    a loud warning rather than failing unrelated work.
//! 2. **Fidelity equivalence** — the document's netlist/component/spec/BOM/
//!    constraint content must agree with what the KiCad `.net`, BOM CSV,
//!    `layout.json` emitters and the shared IR report for the same design
//!    ("two consumers of the same data must agree", the RFC-010/RFC-014
//!    discipline) — over the fixture corpus AND both repo examples.
//!
//! Plus: byte-determinism, the completeness marker, XML escaping/name
//! sanitization (incl. the adversarial-round regressions: control chars,
//! tab normalization, hostile designator prefixes, AVL MPN collisions),
//! zero impact on the existing artifacts, and the CLI surface
//! (`--emit ipc2581`, the `--json` build key, stale-file removal).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files, Checked};
use std::path::{Path, PathBuf};
use std::process::Command;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn xsd() -> PathBuf {
    manifest().join("tests/schema/IPC-2581B1.xsd")
}

/// Build a single-file design through the library pipeline and emit all four
/// artifacts. Panics on any check/build failure (fixtures must be clean).
struct Built {
    checked: Checked,
    netlist: String,
    bom: String,
    layout: Option<String>,
    xml: String,
}

/// A closed 51x21mm rectangle outline on Edge.Cuts (centered at origin), for
/// fixtures that declare `board_outline: "…"` — the DXF an in-memory loader
/// returns (RFC-020; tests are FS-free).
const DXF_51X21: &str = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n\
    10\n-25.5\n20\n-10.5\n10\n25.5\n20\n-10.5\n10\n25.5\n20\n10.5\n10\n-25.5\n20\n10.5\n0\nENDSEC\n";
/// A 30x20mm rectangle for the placement fixture.
const DXF_30X20: &str = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n\
    10\n-15\n20\n-10\n10\n15\n20\n-10\n10\n15\n20\n10\n10\n-15\n20\n10\n0\nENDSEC\n";

fn build(name: &str, src: &str) -> Built {
    build_with_dxf(name, src, DXF_51X21)
}

fn build_with_dxf(name: &str, src: &str, dxf: &str) -> Built {
    let files = vec![(format!("{}.cohdl", name), src.to_string())];
    let mut checked = check_files(&files, None).expect("design selection");
    assert!(
        !checked.diags.has_errors(),
        "fixture `{}` must check cleanly:\n{}",
        name,
        checked.diags.render(&checked.sm)
    );
    cohdl::pipeline::resolve_board_outline(&mut checked, |_| Ok(dxf.to_string()));
    let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, name);
    Built {
        netlist: artifacts.netlist,
        bom: artifacts.bom,
        layout: artifacts.layout,
        xml,
        checked,
    }
}

/// Build a repo example project (with the real std) and emit the document.
fn build_example(dir: &str) -> Built {
    let root = manifest();
    let proj = cohdl::project::load_project(&root.join(dir), Some(&root.join("std"))).unwrap();
    let mut checked = check_files(&proj.files, proj.top.as_deref()).unwrap();
    assert!(!checked.diags.has_errors());
    // RFC-020: resolve the example's real DXF outline from disk.
    let proj_dir = proj.dir.clone();
    cohdl::pipeline::resolve_board_outline(&mut checked, |p| {
        std::fs::read_to_string(proj_dir.join(p)).map_err(|e| e.to_string())
    });
    let artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, &proj.name);
    Built {
        netlist: artifacts.netlist,
        bom: artifacts.bom,
        layout: artifacts.layout,
        xml,
        checked,
    }
}

/// Validate one document against the vendored XSD. Returns false (with a
/// loud warning) only when xmllint itself is unavailable.
fn xsd_validate(name: &str, xml: &str) -> bool {
    let dir = std::env::temp_dir().join(format!("cohdl-ipc2581-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.xml", name));
    std::fs::write(&path, xml).unwrap();
    let out = match Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(xsd())
        .arg(&path)
        .output()
    {
        Ok(out) => out,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("WARNING: xmllint not found — schema validity NOT checked locally (CI is the authoritative gate)");
            return false;
        }
        Err(e) => panic!("xmllint failed to run: {}", e),
    };
    assert!(
        out.status.success(),
        "`{}` does not validate against IPC-2581B1.xsd:\n{}",
        name,
        String::from_utf8_lossy(&out.stderr)
    );
    true
}

// ---------------------------------------------------------------------------
// Fixtures.

/// Two-resistor board (clean, no layout metadata).
const BASIC: &str = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } spec { resistance: 1kohm } }
pub footprint TFP {}
pub part R1K: Res { primary { mfr: "Yageo", mpn: "RC0402FR-071KL", footprint: TFP } }
design B {
    inst r1: R1K
    inst r2: R1K
    net N: r1.A, r2.A
    net GND [gnd]: r1.B, r2.B
}
"#;

/// A board exercising every RFC-013 construct + a POWER net + a pin bus.
const WITH_LAYOUT: &str = r#"
pub device Mcu { pins { required DP: 1 [bidirectional], required DM: 2 [bidirectional], required VDD: 3 [power_in], required GND: 4, 5 [power_in] } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint TFP {}
pub part MCU_P: Mcu { primary { mfr: "Acme", mpn: "MCU-1", footprint: TFP } }
pub part R0: Res { primary { mfr: "Yageo", mpn: "RC-0", footprint: TFP } }
design B {
    #[placement_hint("near the USB connector")]
    inst u1: MCU_P
    inst r1: R0
    inst r2: R0
    net USB_DP: u1.DP, r1.A
    net USB_DM: u1.DM, r2.A
    net VDD [3.3V]: u1.VDD, r1.B
    net GND [gnd]: u1.GND, r2.B
    layout {
        net_class HighSpeed { USB_DP, USB_DM }
        diff_pair(USB_DP, USB_DM)
        length_match(USB_DP, USB_DM) [tolerance: "0.15mm"]
    }
}
"#;

/// XML-hostile strings in MPN/MFR (footprints are symbols since RFC-017 —
/// identifier-safe by construction, so only free-text part fields can be
/// hostile now).
const NASTY: &str = r#"
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint TFP {}
pub part P1: D { primary { mfr: "ACME & Co 'Ltd'", mpn: "MPN <X> & 'Y'", footprint: TFP } }
design B {
    inst d1: P1
    inst d2: P1
    net N: d1.A, d2.A
    net M: d1.B, d2.B
}
"#;

// ---------------------------------------------------------------------------
// Tiny test-local XML scanners (no XML dependency, same spirit as the local
// JSON parser in tests/json_output.rs).

/// Every element named `tag`: the raw text of its start tag (attrs only).
fn elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let open = format!("<{} ", tag);
    let mut from = 0;
    while let Some(pos) = xml[from..].find(&open) {
        let start = from + pos;
        let end = start + xml[start..].find('>').expect("closed tag");
        out.push(&xml[start..end]);
        from = end;
    }
    out
}

/// The blocks `<tag …> … </tag>` (start tag text, inner text).
fn blocks<'a>(xml: &'a str, tag: &str) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    let open = format!("<{} ", tag);
    let close = format!("</{}>", tag);
    let mut from = 0;
    while let Some(pos) = xml[from..].find(&open) {
        let start = from + pos;
        let head_end = start + xml[start..].find('>').expect("closed tag");
        let body_end = head_end + xml[head_end..].find(&close).expect("closing tag");
        out.push((&xml[start..head_end], &xml[head_end + 1..body_end]));
        from = body_end;
    }
    out
}

/// Attribute value from a start-tag text, XML-unescaped.
fn attr(elem: &str, name: &str) -> Option<String> {
    let needle = format!(" {}=\"", name);
    let pos = elem.find(&needle)? + needle.len();
    let end = pos + elem[pos..].find('"')?;
    Some(
        elem[pos..end]
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&"),
    )
}

// ---------------------------------------------------------------------------
// Property 1: schema validity, over the fixture corpus + both repo examples.

#[test]
fn schema_validity_over_corpus() {
    let mut ran = false;
    for (name, src) in [
        ("basic", BASIC),
        ("with-layout", WITH_LAYOUT),
        ("nasty", NASTY),
    ] {
        ran |= xsd_validate(name, &build(name, src).xml);
    }
    for dir in ["examples/sensor-node", "examples/rpi-pico2"] {
        let b = build_example(dir);
        ran |= xsd_validate(dir.rsplit('/').next().unwrap(), &b.xml);
    }
    if !ran {
        eprintln!("WARNING: schema validity not checked (no xmllint)");
    }
}

// ---------------------------------------------------------------------------
// Property 2: fidelity equivalence against the .net / BOM CSV / layout.json.

/// (ref, pin) node set per net name, parsed from the KiCad `.net` text.
fn net_nodes(netlist: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut out = Vec::new();
    for chunk in netlist.split("(net (code ").skip(1) {
        let name_start = chunk.find("(name \"").unwrap() + 7;
        let name_end = name_start + chunk[name_start..].find('"').unwrap();
        let name = chunk[name_start..name_end].to_string();
        let mut nodes = Vec::new();
        let body_end = chunk.find("\n    )").unwrap_or(chunk.len());
        for node in chunk[..body_end].split("(node (ref \"").skip(1) {
            let r_end = node.find('"').unwrap();
            let r = node[..r_end].to_string();
            let p_start = node.find("(pin \"").unwrap() + 6;
            let p_end = p_start + node[p_start..].find('"').unwrap();
            nodes.push((r, node[p_start..p_end].to_string()));
        }
        out.push((name, nodes));
    }
    out
}

/// Netlist/component fidelity: the same refdes order as the `.net`'s comps
/// (modulo the documented XSD-charset sanitization, which is identity for
/// every clean designator), and every net's physical node list exactly.
fn assert_net_fidelity(name: &str, b: &Built) {
    let comps = elements(&b.xml, "Component");
    let net_refs: Vec<String> = b
        .netlist
        .split("(comp (ref \"")
        .skip(1)
        .map(|c| c[..c.find('"').unwrap()].to_string())
        .collect();
    let xml_refs: Vec<String> = comps.iter().map(|c| attr(c, "refDes").unwrap()).collect();
    assert_eq!(xml_refs, net_refs, "[{}] component refdes order/set", name);

    let nets = blocks(&b.xml, "LogicalNet");
    let kicad = net_nodes(&b.netlist);
    assert_eq!(nets.len(), kicad.len(), "[{}] net count", name);
    for ((head, body), (kname, knodes)) in nets.iter().zip(&kicad) {
        assert_eq!(&attr(head, "name").unwrap(), kname, "[{}] net name", name);
        let pins: Vec<(String, String)> = elements(body, "PinRef")
            .iter()
            .map(|p| (attr(p, "componentRef").unwrap(), attr(p, "pin").unwrap()))
            .collect();
        assert_eq!(&pins, knodes, "[{}] net `{}` nodes", name, kname);
    }
}

/// BOM fidelity: one BomItem per CSV row, same refdes list/quantity/values.
fn assert_bom_fidelity(name: &str, b: &Built) {
    // CSV rows: "RefDes","Value","MPN","Manufacturer",Qty — the first four
    // quoted (with "" escaping), the count bare.
    let rows: Vec<Vec<String>> = b
        .bom
        .lines()
        .skip(1)
        .map(|l| {
            let (quoted, qty) = l.rsplit_once(',').unwrap();
            let mut cells: Vec<String> = quoted
                .trim_matches('"')
                .split("\",\"")
                .map(|c| c.replace("\"\"", "\""))
                .collect();
            cells.push(qty.to_string());
            cells
        })
        .collect();
    let items = blocks(&b.xml, "BomItem");
    assert_eq!(items.len(), rows.len(), "[{}] BOM group count", name);
    for ((head, body), row) in items.iter().zip(&rows) {
        let refdes: Vec<String> = elements(body, "RefDes")
            .iter()
            .map(|r| attr(r, "name").unwrap())
            .collect();
        assert_eq!(refdes.join(","), row[0], "[{}] refdes list", name);
        let qty: usize = row[4].parse().unwrap();
        assert_eq!(
            attr(head, "quantity").unwrap(),
            qty.to_string(),
            "[{}] quantity",
            name
        );
        let textual = |k: &str| -> String {
            elements(body, "Textual")
                .iter()
                .find(|t| attr(t, "textualCharacteristicName").as_deref() == Some(k))
                .and_then(|t| attr(t, "textualCharacteristicValue"))
                .unwrap()
        };
        assert_eq!(textual("VALUE"), row[1], "[{}] value", name);
        assert_eq!(textual("MPN"), row[2], "[{}] mpn", name);
        assert_eq!(textual("MFR"), row[3], "[{}] mfr", name);
    }
}

/// Spec fidelity (adversarial finding: previously untested — deleting the
/// spec emission passed the whole suite): every Component's COHDL_SPEC_*
/// attribute map equals the IR's resolved spec map for that instance, keyed
/// through COHDL_PATH. Every emitter consumes the same IR, so this pins the
/// document's spec content to the shared source of truth.
fn assert_spec_fidelity(name: &str, b: &Built) {
    use std::collections::BTreeMap;
    let ir = b.checked.ir.as_ref().unwrap();
    let comps = blocks(&b.xml, "Component");
    assert_eq!(
        comps.len(),
        ir.instances.len(),
        "[{}] component count",
        name
    );
    for (_, body) in &comps {
        let attrs = elements(body, "NonstandardAttribute");
        let path = attrs
            .iter()
            .find(|a| attr(a, "name").as_deref() == Some("COHDL_PATH"))
            .and_then(|a| attr(a, "value"))
            .unwrap_or_else(|| panic!("[{}] component missing COHDL_PATH", name));
        let inst = &ir.instances[&path];
        let xml_specs: BTreeMap<String, String> = attrs
            .iter()
            .filter_map(|a| {
                let field = attr(a, "name")?.strip_prefix("COHDL_SPEC_")?.to_string();
                Some((field, attr(a, "value").unwrap()))
            })
            .collect();
        let ir_specs: BTreeMap<String, String> = inst
            .specs
            .iter()
            .map(|(k, v)| (k.clone(), v.text.clone()))
            .collect();
        assert_eq!(xml_specs, ir_specs, "[{}] specs for `{}`", name, path);
    }
}

/// Constraint fidelity, generically against the parsed layout.json (not
/// hand-picked names): every net_class/diff_pair/length_match/hint in the
/// RFC-013 artifact appears in the document with the same members.
fn assert_constraint_fidelity(name: &str, b: &Built) {
    let Some(layout) = &b.layout else { return };
    let doc: serde_json::Value = serde_json::from_str(layout).unwrap();
    let arr = |k: &str| doc[k].as_array().cloned().unwrap_or_default();
    let specs = blocks(&b.xml, "Spec");
    let spec = |sname: &str| -> &str {
        specs
            .iter()
            .find(|(h, _)| attr(h, "name").as_deref() == Some(sname))
            .map(|(_, body)| *body)
            .unwrap_or_else(|| panic!("[{}] missing Spec `{}`", name, sname))
    };
    let props = |body: &str, key: &str| -> Vec<String> {
        elements(body, "Property")
            .iter()
            .filter(|p| attr(p, "name").as_deref() == Some(key))
            .map(|p| attr(p, "text").unwrap())
            .collect()
    };
    let strs = |v: &serde_json::Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect()
    };
    for nc in arr("net_classes") {
        let body = spec(&format!("cohdl:net_class:{}", nc["name"].as_str().unwrap()));
        assert_eq!(
            props(body, "net"),
            strs(&nc["nets"]),
            "[{}] net_class",
            name
        );
    }
    for dp in arr("diff_pairs") {
        let (p, n) = (dp["p"].as_str().unwrap(), dp["n"].as_str().unwrap());
        let body = spec(&format!("cohdl:diff_pair:{}:{}", p, n));
        assert_eq!(props(body, "positive"), [p], "[{}] diff_pair p", name);
        assert_eq!(props(body, "negative"), [n], "[{}] diff_pair n", name);
    }
    for (i, lm) in arr("length_matches").iter().enumerate() {
        let body = spec(&format!("cohdl:length_match:{}", i + 1));
        assert_eq!(props(body, "net"), strs(&lm["nets"]), "[{}] lm nets", name);
        match lm["tolerance"].as_str() {
            Some(t) => assert_eq!(props(body, "tolerance"), [t], "[{}] tolerance", name),
            None => assert!(
                props(body, "tolerance").is_empty(),
                "[{}] no tolerance",
                name
            ),
        }
    }
    // Every placement hint in layout.json rides exactly one component.
    let xml_hints: Vec<String> = elements(&b.xml, "NonstandardAttribute")
        .iter()
        .filter(|a| attr(a, "name").as_deref() == Some("COHDL_PLACEMENT_HINT"))
        .map(|a| attr(a, "value").unwrap())
        .collect();
    let json_hints: Vec<String> = arr("placement_hints")
        .iter()
        .map(|h| h["hint"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(xml_hints, json_hints, "[{}] placement hints", name);
}

/// The full fidelity battery over the fixture corpus AND both repo examples
/// (the examples are the only boards with multi-pin buses across many parts
/// — adversarial finding: they were previously only schema-checked).
#[test]
fn fidelity_over_corpus_and_examples() {
    let mut builds: Vec<(String, Built)> = [
        ("basic", BASIC),
        ("with-layout", WITH_LAYOUT),
        ("nasty", NASTY),
    ]
    .into_iter()
    .map(|(n, s)| (n.to_string(), build(n, s)))
    .collect();
    for dir in ["examples/sensor-node", "examples/rpi-pico2"] {
        builds.push((dir.to_string(), build_example(dir)));
    }
    for (name, b) in &builds {
        assert_net_fidelity(name, b);
        assert_bom_fidelity(name, b);
        assert_spec_fidelity(name, b);
        assert_constraint_fidelity(name, b);
    }
}

#[test]
fn net_class_attribute_mirrors_annotations() {
    // [gnd] → GROUND, voltage → POWER, else SIGNAL.
    let b = build("classes", WITH_LAYOUT);
    let class_of = |net: &str| -> String {
        blocks(&b.xml, "LogicalNet")
            .iter()
            .find(|(h, _)| attr(h, "name").as_deref() == Some(net))
            .map(|(h, _)| attr(h, "netClass").unwrap())
            .unwrap()
    };
    assert_eq!(class_of("GND"), "GROUND");
    assert_eq!(class_of("VDD"), "POWER");
    assert_eq!(class_of("USB_DP"), "SIGNAL");
}

// ---------------------------------------------------------------------------
// Determinism, marker, escaping.

#[test]
fn emission_is_byte_deterministic() {
    let a = build("det", WITH_LAYOUT).xml;
    let b = build("det", WITH_LAYOUT).xml;
    assert_eq!(a, b, "same source must emit identical bytes");
    // No wall-clock leakage: every dateTime is the fixed epoch instant.
    assert!(
        !a.contains("202"),
        "a live timestamp leaked into the artifact"
    );
}

#[test]
fn completeness_marker_is_machine_readable() {
    let b = build("marker", BASIC);
    // The machine-readable attribute…
    let marker = elements(&b.xml, "NonstandardAttribute")
        .into_iter()
        .find(|a| attr(a, "name").as_deref() == Some("COHDL_COMPLETENESS"))
        .map(|a| a.to_string())
        .expect("COHDL_COMPLETENESS attribute present");
    assert_eq!(
        attr(&marker, "value").as_deref(),
        Some("logical-complete,placement-staged,unrouted")
    );
    // …and the human-visible FunctionMode comment.
    let fm = elements(&b.xml, "FunctionMode")[0];
    assert!(attr(fm, "comment")
        .unwrap()
        .contains("logical-complete,placement-staged,unrouted"));
}

#[test]
fn hostile_strings_are_escaped_and_names_sanitized() {
    let b = build("nasty", NASTY);
    // Raw MPN/MFR strings survive (escaped) in the BOM characteristics.
    let items = blocks(&b.xml, "BomItem");
    let (_, body) = &items[0];
    let textual = |k: &str| -> String {
        elements(body, "Textual")
            .iter()
            .find(|t| attr(t, "textualCharacteristicName").as_deref() == Some(k))
            .and_then(|t| attr(t, "textualCharacteristicValue"))
            .unwrap()
    };
    assert_eq!(textual("MPN"), "MPN <X> & 'Y'");
    assert_eq!(textual("MFR"), "ACME & Co 'Ltd'");
    // RFC-017: footprints are SYMBOLS now — package names derive from the fq
    // module path, identifier-safe by construction (the hostile-footprint-
    // string case is unrepresentable since the migration; sanitization remains
    // as defense-in-depth only). The CoHDL `::` separator is collapsed to a
    // single `-` for the IPC-2581 Package name / packageRef so a consumer whose
    // pad-resolution path treats the name as an NCName (splitting on the XML
    // `:` delimiter) still binds pins to their land pattern.
    let pkg = elements(&b.xml, "Package")[0];
    let name = attr(pkg, "name").unwrap();
    assert_eq!(
        name, "main-TFP",
        "package name is the colon-free fq footprint symbol"
    );
    // Because `::` → `-` changes the name, the raw fq symbol is preserved as
    // the Package `comment` (traceability back to the CoHDL declaration).
    assert_eq!(
        attr(pkg, "comment").as_deref(),
        Some("main::TFP"),
        "raw fq footprint symbol preserved in the comment"
    );
    // The Component packageRef must still match the Package name byte-for-byte.
    let comp = elements(&b.xml, "Component")[0];
    assert_eq!(
        attr(comp, "packageRef").as_deref(),
        Some("main-TFP"),
        "packageRef stays matched to the sanitized Package name"
    );
    let _ = &b.checked;
}

// ---------------------------------------------------------------------------
// Adversarial-verification regressions (RFC-015 round 1).

// Finding (high): XML-1.0-illegal control characters in CoHDL strings made
// the document non-well-formed; literal tabs were silently normalized to
// spaces by conforming parsers (diverging from layout.json/BOM) and could
// collide XSD key values ("ACME\tCo" vs "ACME Co" as enterprise ids).
#[test]
fn control_characters_stay_well_formed_and_tabs_survive() {
    let src = "\
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint TFP {}
pub part P1: D { primary { mfr: \"ACME\tCo\", mpn: \"MPN\u{8}X\", footprint: TFP } }
pub part P2: D { primary { mfr: \"ACME Co\", mpn: \"OTHER\", footprint: TFP } }
design B {
    #[placement_hint(\"near\tUSB\")]
    inst d1: P1
    inst d2: P2
    net N: d1.A, d2.A
    net M: d1.B, d2.B
    layout {
        length_match(N, M) [tolerance: \"0.15\tmm\"]
    }
}
";
    let b = build("ctrl", src);
    // Well-formed AND schema-valid, including the two tab-distinct
    // manufacturers as distinct enterprise keys.
    xsd_validate("ctrl", &b.xml);
    // Tabs are character references, so conforming parsers preserve them.
    assert!(
        b.xml.contains("ACME&#9;Co"),
        "tab must survive as &#9;:\n{}",
        b.xml
    );
    assert!(b.xml.contains("near&#9;USB"), "hint tab as &#9;");
    assert!(b.xml.contains("0.15&#9;mm"), "tolerance tab as &#9;");
    // The XML-illegal 0x08 is replaced with U+FFFD (disclosed lossy case).
    assert!(
        b.xml.contains("MPN\u{FFFD}X"),
        "illegal control char replaced with U+FFFD"
    );
    assert!(!b.xml.contains('\u{8}'), "no raw control bytes in the XML");
}

// Finding (high): a trait `designator_prefix` may carry any string; raw
// designators reached RefDes/@name (XSD pattern violation) and two prefixes
// sanitizing identically produced duplicate componentKeys. All refdes
// spellings now route through one collision-free table.
#[test]
fn hostile_designator_prefixes_stay_schema_valid_and_distinct() {
    let src = "\
pub trait TA { designator_prefix: \"R<\" pins { required A: pin } }
pub trait TB { designator_prefix: \"R>\" pins { required A: pin } }
pub device Da { pins { A: 1 [passive], B: 2 [passive] } }
pub device Db { pins { A: 1 [passive], B: 2 [passive] } }
impl TA for Da {}
impl TB for Db {}
pub footprint TFP {}
pub part PA: Da { primary { mfr: \"m\", mpn: \"a\", footprint: TFP } }
pub part PB: Db { primary { mfr: \"m\", mpn: \"b\", footprint: TFP } }
design B {
    inst a: PA
    inst b: PB
    net N: a.A, b.A
    net M: a.B, b.B
}
";
    let b = build("prefix", src);
    xsd_validate("prefix", &b.xml);
    // Distinct components stay distinct after sanitization…
    let refs: Vec<String> = elements(&b.xml, "Component")
        .iter()
        .map(|c| attr(c, "refDes").unwrap())
        .collect();
    let unique: std::collections::BTreeSet<&String> = refs.iter().collect();
    assert_eq!(
        refs.len(),
        unique.len(),
        "refdes must be unique: {:?}",
        refs
    );
    // …and every RefDes/@name and PinRef/@componentRef uses exactly those
    // spellings (the XSD keyrefs require it; asserted here for clarity).
    for r in elements(&b.xml, "RefDes") {
        let n = attr(r, "name").unwrap();
        assert!(refs.contains(&n), "RefDes `{}` not a componentKey", n);
    }
    for p in elements(&b.xml, "PinRef") {
        let n = attr(p, "componentRef").unwrap();
        assert!(refs.contains(&n), "PinRef `{}` not a componentKey", n);
    }
}

// Finding (medium): distinct MPNs whose sanitized forms collide collapsed to
// one AvlMpn/@name — one vendor's item literally naming a competitor's MPN.
// @name now uses the collision-free group key; the TRUE MPN rides @other.
#[test]
fn avl_mpns_stay_distinct_and_carry_raw_mpn() {
    let src = "\
pub device Da { pins { A: 1 [passive], B: 2 [passive] } }
pub device Db { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint TFP {}
pub part PA: Da { primary { mfr: \"NXP\", mpn: \"BAT54S/SOT23\", footprint: TFP } }
pub part PB: Db { primary { mfr: \"Other\", mpn: \"BAT54S_SOT23\", footprint: TFP } }
design B {
    inst a: PA
    inst b: PB
    net N: a.A, b.A
    net M: a.B, b.B
}
";
    let b = build("avl", src);
    xsd_validate("avl", &b.xml);
    let mpns: Vec<(String, String)> = elements(&b.xml, "AvlMpn")
        .iter()
        .map(|m| (attr(m, "name").unwrap(), attr(m, "other").unwrap()))
        .collect();
    let names: std::collections::BTreeSet<&String> = mpns.iter().map(|(n, _)| n).collect();
    assert_eq!(
        mpns.len(),
        names.len(),
        "AvlMpn names must be unique: {:?}",
        mpns
    );
    let raw: Vec<&str> = mpns.iter().map(|(_, o)| o.as_str()).collect();
    assert!(
        raw.contains(&"BAT54S/SOT23"),
        "raw MPN preserved: {:?}",
        raw
    );
    assert!(
        raw.contains(&"BAT54S_SOT23"),
        "raw MPN preserved: {:?}",
        raw
    );
    // Each AvlItem's name matches its BomItem key (the XSD keyref) and its
    // vendor: the NXP-backed item must carry the NXP MPN, not the other's.
    let nxp_item = blocks(&b.xml, "AvlItem")
        .into_iter()
        .find(|(_, body)| body.contains("mfr:NXP"))
        .expect("NXP AvlItem");
    let nxp_mpn = elements(nxp_item.1, "AvlMpn")
        .first()
        .and_then(|m| attr(m, "other"))
        .unwrap();
    assert_eq!(nxp_mpn, "BAT54S/SOT23", "vendor keeps its own MPN");
}

// ---------------------------------------------------------------------------
// Zero impact + CLI surface (the real binary).

fn cohdl() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cohdl"));
    c.env("COHDL_STD", manifest().join("std"));
    c
}

fn make_project(root: &Path, main_src: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), main_src).unwrap();
}

#[test]
fn cli_emit_writes_documents_and_stays_zero_impact() {
    let tmp = std::env::temp_dir().join(format!("cohdl-ipc-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, BASIC);

    // Build WITHOUT --emit: no .xml.
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let read = |p: &str| std::fs::read_to_string(tmp.join(p)).unwrap();
    let (net0, bom0, lock0) = (
        read("out/t.net"),
        read("out/t-bom.csv"),
        read("design.lock"),
    );
    assert!(!tmp.join("out/t.xml").exists(), "no .xml without --emit");

    // Build WITH --emit ipc2581: .xml appears; nothing else moves a byte.
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(tmp.join("out/t.xml").exists());
    assert!(read("out/t.xml").contains("<IPC-2581 revision=\"B1\""));
    assert_eq!(
        read("out/t.net"),
        net0,
        "--emit must not change the netlist"
    );
    assert_eq!(
        read("out/t-bom.csv"),
        bom0,
        "--emit must not change the BOM"
    );
    assert_eq!(
        read("design.lock"),
        lock0,
        "--emit must not change the lock"
    );

    // --json: the build object gains the ipc2581 key (present only w/ flag).
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"ipc2581\":"), "{}", stdout);

    // Rebuild without the flag: stale .xml is removed, key absent.
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!stdout.contains("\"ipc2581\":"), "{}", stdout);
    assert!(
        !tmp.join("out/t.xml").exists(),
        "stale IPC-2581 document must be removed on a build without --emit"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_emit_flag_matrix() {
    let tmp = std::env::temp_dir().join(format!("cohdl-ipc-flags-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, BASIC);

    // --emit is build-only — and command compatibility outranks value
    // validity (adversarial finding: `fmt --emit bogus` used to recommend
    // `ipc2581`, which fmt would then reject anyway).
    for cmd in ["check", "fmt"] {
        for value in ["ipc2581", "bogus"] {
            let out = cohdl()
                .args([cmd, tmp.to_str().unwrap(), "--emit", value])
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(2), "--emit rejected on {}", cmd);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(
                stderr.contains("is not valid with"),
                "command error must outrank the value error ({} --emit {}):\n{}",
                cmd,
                value,
                stderr
            );
        }
    }
    for value in ["ipc2581", "bogus"] {
        let out = cohdl().args(["lsp", "--emit", value]).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("takes no flags"),
            "lsp rejects --emit before the value is judged"
        );
    }

    // Unknown format value (on build, where --emit is legal).
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--emit", "gerber"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("valid: ipc2581"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// Fourth-review (2026-07-14) regressions.

/// F7: the fidelity gate must actually check Component attributes and prove
/// the semantic PinRef→Component reference the XSD keyref does NOT enforce.
#[test]
fn component_attributes_and_pinrefs_are_semantically_faithful() {
    let b = build("frev", WITH_LAYOUT);
    xsd_validate("frev", &b.xml);
    let comps = blocks(&b.xml, "Component");
    let refset: std::collections::BTreeSet<String> = comps
        .iter()
        .map(|(h, _)| attr(h, "refDes").unwrap())
        .collect();
    let packages: std::collections::BTreeSet<String> = elements(&b.xml, "Package")
        .iter()
        .map(|p| attr(p, "name").unwrap())
        .collect();
    for (head, body) in &comps {
        // Every component carries a non-empty MPN (@part), a packageRef that
        // resolves to a declared Package, and its COHDL_DEVICE attribute —
        // mutating any of these must now fail a test (previously invisible).
        assert!(
            !attr(head, "part").unwrap().is_empty(),
            "Component @part must be a real MPN:\n{}",
            head
        );
        assert!(
            packages.contains(&attr(head, "packageRef").unwrap()),
            "@packageRef must resolve to a Package: {}",
            attr(head, "packageRef").unwrap()
        );
        assert!(
            body.contains("name=\"COHDL_DEVICE\""),
            "each Component names its device"
        );
    }
    // Every PinRef/@componentRef resolves to a real Component/@refDes. The
    // vendored XSD's keyref binds LogicalNetPin (which we do not emit), not
    // PinRef, so this invariant is the emitter's to guarantee and the
    // test's to check (review F7).
    for net in elements_multiline(&b.xml, "LogicalNet") {
        for pr in elements(&net, "PinRef") {
            let cref = attr(pr, "componentRef").unwrap();
            assert!(
                refset.contains(&cref),
                "PinRef componentRef `{}` has no Component:\n{:?}",
                cref,
                refset
            );
        }
    }
}

/// F5: two parts sharing an MPN under different manufacturers must both keep
/// their identity — no vendor or value silently erased in CSV or XML.
#[test]
fn duplicate_mpn_across_manufacturers_keeps_both() {
    let src = "\
pub device Da { pins { A: 1 [passive] } spec { resistance: 1kohm } }
pub device Db { pins { A: 1 [passive] } spec { resistance: 2kohm } }
pub footprint TFP {}
pub part PA: Da { primary { mfr: \"Alpha\", mpn: \"SHARED\", footprint: TFP } }
pub part PB: Db { primary { mfr: \"Beta\", mpn: \"SHARED\", footprint: TFP } }
design B { inst a: PA  inst b: PB  net N: a.A, b.A }
";
    let b = build("dupmpn", src);
    xsd_validate("dupmpn", &b.xml);
    // Both manufacturers survive in the CSV.
    assert!(b.bom.contains("Alpha"), "Alpha row:\n{}", b.bom);
    assert!(b.bom.contains("Beta"), "Beta row (was dropped):\n{}", b.bom);
    assert!(
        b.bom.contains("1kohm") && b.bom.contains("2kohm"),
        "both values:\n{}",
        b.bom
    );
    // Two distinct AVL items, two enterprises.
    assert_eq!(
        blocks(&b.xml, "AvlItem").len(),
        2,
        "two AVL items:\n{}",
        b.xml
    );
    let ents: Vec<String> = elements(&b.xml, "Enterprise")
        .iter()
        .map(|e| attr(e, "id").unwrap())
        .collect();
    assert!(ents.iter().any(|e| e.contains("Alpha")), "{:?}", ents);
    assert!(ents.iter().any(|e| e.contains("Beta")), "{:?}", ents);
}

/// F3: XML-1.0-forbidden scalars beyond the C0 block (U+FFFE/U+FFFF) must be
/// projected, and two manufacturers differing only in distinct control
/// characters must not collapse to one Enterprise id (schema enterpriseKey).
#[test]
fn forbidden_scalars_projected_and_enterprise_ids_unique() {
    // U+FFFE in a manufacturer name — previously slipped through esc() and
    // produced a document xmllint rejects.
    let src = format!(
        "pub device D {{ pins {{ A: 1 [passive] }} }}
pub footprint TFP {{}}
pub part P: D {{ primary {{ mfr: \"Ac{}me\", mpn: \"M1\", footprint: TFP }} }}
design B {{ inst a: P  net N: a.A }}",
        '\u{FFFE}'
    );
    let b = build("fffe", &src);
    xsd_validate("fffe", &b.xml); // must still validate

    // Two manufacturers differing only in distinct C0 controls both project
    // to `…\u{FFFD}`; the enterprise-id table must `_`-disambiguate them.
    let src = format!(
        "pub device Da {{ pins {{ A: 1 [passive] }} }}
pub device Db {{ pins {{ A: 1 [passive] }} }}
pub footprint TFP {{}}
pub part PA: Da {{ primary {{ mfr: \"Ac{}me\", mpn: \"MA\", footprint: TFP }} }}
pub part PB: Db {{ primary {{ mfr: \"Ac{}me\", mpn: \"MB\", footprint: TFP }} }}
design B {{ inst a: PA  inst b: PB  net N: a.A, b.A }}",
        '\u{8}', '\u{B}'
    );
    let b = build("c0", &src);
    xsd_validate("c0", &b.xml); // duplicate enterpriseKey would fail here
    let ents: Vec<String> = elements(&b.xml, "Enterprise")
        .iter()
        .map(|e| attr(e, "id").unwrap())
        .collect();
    let uniq: std::collections::BTreeSet<&String> = ents.iter().collect();
    assert_eq!(
        ents.len(),
        uniq.len(),
        "enterprise ids must be unique: {:?}",
        ents
    );
}

/// F6: the review's designator-divergence reproduction (`R<1` becoming `R_1`
/// in the XML alone) is UNREACHABLE at HEAD — E804 rejects any designator
/// that is not `[A-Z]+[0-9]+` at the source, so `<`/`>` never enters a
/// designator to diverge in the first place (the review's "make the source
/// rule reject them" resolution, already in place). `sanitize` was
/// nonetheless widened to the true XSD charset (`<`/`>` are legal in both
/// `qualifiedNameType` and `shortName`) as defense-in-depth; this test pins
/// the source-side guarantee that makes the divergence impossible.
#[test]
fn designator_special_chars_are_rejected_at_source() {
    let files = vec![(
        "d.cohdl".to_string(),
        "pub device R { pins { A: 1 [passive] } }
pub footprint TFP {}
pub part RP: R { primary { mfr: \"Y\", mpn: \"M1\", footprint: TFP } }
design B { #[designator(\"R<1\")] inst a: RP  net N: a.A }
"
        .to_string(),
    )];
    let mut checked = check_files(&files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let r = checked.diags.render(&checked.sm);
    assert!(
        r.contains("E804") && r.contains("not a valid designator"),
        "a designator with `<` must be rejected at the source:\n{}",
        r
    );
}

/// Helper: container elements whose body spans multiple lines (LogicalNet).
fn elements_multiline(xml: &str, tag: &str) -> Vec<String> {
    blocks(xml, tag)
        .into_iter()
        .map(|(h, b)| format!("{}{}", h, b))
        .collect()
}

/// A board that declares a rectangular outline.
const WITH_OUTLINE: &str = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub pad P { shape: rect size: (0.5mm, 0.6mm) layer: top_copper plating: smd }
pub footprint FP { pad 1: P at (-0.5mm, 0mm) pad 2: P at (0.5mm, 0mm) courtyard { shape: rect, at: (0mm, 0mm), size: (2mm, 1mm) } }
pub part R0: Res { primary { mfr: "m", mpn: "n", footprint: FP } }
design B {
    inst r1: R0
    net N: r1.A
    nc: r1.B
    layout { board_outline: "o.dxf" }
}
"#;

#[test]
fn board_outline_emits_profile_polygon() {
    let b = build("outline", WITH_OUTLINE);
    // `<Profile>` has no attributes (the `blocks` helper matches `<Tag `), so
    // check the emitted ring directly. 51x21 centered at origin → ±25.5/±10.5.
    assert_eq!(
        b.xml.matches("<Profile>").count(),
        1,
        "exactly one Profile:\n{}",
        b.xml
    );
    // y negated throughout: the Profile is projected into IPC's +y-up frame
    // (matching KiCad's export) from the CoHDL/DXF +y-down outline.
    for needle in [
        "<Profile>",
        "<PolyBegin x=\"-25.5\" y=\"10.5\"/>",
        "<PolyStepSegment x=\"25.5\" y=\"10.5\"/>",
        "<PolyStepSegment x=\"25.5\" y=\"-10.5\"/>",
        "<PolyStepSegment x=\"-25.5\" y=\"-10.5\"/>",
        "<PolyStepSegment x=\"-25.5\" y=\"10.5\"/>",
    ] {
        assert!(
            b.xml.contains(needle),
            "Profile missing {}:\n{}",
            needle,
            b.xml
        );
    }
    // The Profile sits between Datum and the first Package (schema sequence).
    let (datum, pkg) = (
        b.xml.find("<Datum").unwrap(),
        b.xml.find("<Package ").unwrap(),
    );
    let prof = b.xml.find("<Profile>").unwrap();
    assert!(
        datum < prof && prof < pkg,
        "Profile must be ordered Datum < Profile < Package"
    );
    xsd_validate("outline", &b.xml);
}

fn component_location(block: &str) -> (f64, f64) {
    let l = block.split("<Location").nth(1).expect("Location");
    let x = attr(&format!("<Location{}", l), "x")
        .unwrap()
        .parse()
        .unwrap();
    let y = attr(&format!("<Location{}", l), "y")
        .unwrap()
        .parse()
        .unwrap();
    (x, y)
}

#[test]
fn components_stage_outside_board_outline() {
    // Quilter locks components INSIDE the outline and only places those
    // outside it, so a component must be staged outside (not piled at 0,0).
    let b = build("stage", WITH_OUTLINE);
    let comp = blocks(&b.xml, "Component")[0].1;
    let (x, _y) = component_location(comp);
    // Outline is 51x21 at origin → right edge 25.5mm; the staged origin sits
    // past the +5mm margin. Definitely not the (0,0) interior placeholder.
    assert!(
        x > 25.5,
        "component must be staged outside the outline, got x={}",
        x
    );
    assert!(
        !comp.contains("<Location x=\"0\" y=\"0\"/>"),
        "component still at origin:\n{}",
        comp
    );
}

const WITH_PLACE: &str = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub device Con { pins { P: 1 [passive] } }
pub pad P { shape: rect size: (0.5mm, 0.6mm) layer: top_copper plating: smd }
pub footprint FP { pad 1: P at (-0.5mm, 0mm) pad 2: P at (0.5mm, 0mm) courtyard { shape: rect, at: (0mm, 0mm), size: (2mm, 1mm) } }
pub footprint FPC { pad 1: P at (0mm, 0mm) }
pub part R0: Res { primary { mfr: "m", mpn: "n", footprint: FP } }
pub part J0: Con { primary { mfr: "m", mpn: "c", footprint: FPC } }
design B {
    inst r1: R0
    inst j1: J0
    net N: r1.A, j1.P
    nc: r1.B
    layout {
        board_outline: "o.dxf"
        place j1 at (3mm, -4mm)
    }
}
"#;

#[test]
fn placed_component_locks_at_position_and_is_not_staged() {
    let b = build_with_dxf("place", WITH_PLACE, DXF_30X20);
    // Identify components by their package name (the `-`-joined fq footprint,
    // colon-free for consumer safety): FPC = the placed connector, FP = the
    // staged resistor.
    let locs: Vec<(String, (f64, f64))> = blocks(&b.xml, "Component")
        .into_iter()
        .map(|(h, body)| (attr(h, "packageRef").unwrap(), component_location(body)))
        .collect();
    let find = |suffix: &str| locs.iter().find(|(p, _)| p.ends_with(suffix)).unwrap().1;
    // The placed connector is locked at its `place j1 at (3, -4)` position; its
    // y is negated to +4 in the IPC +y-up frame (matching KiCad's export).
    assert_eq!(
        find("-FPC"),
        (3.0, 4.0),
        "placed component not locked at (3,-4)->(3,4) in +y-up"
    );
    // The resistor is staged OUTSIDE the 30mm-wide outline (right of +15mm).
    assert!(
        find("-FP").0 > 15.0,
        "staged resistor should be outside the outline, got {:?}",
        find("-FP")
    );
    assert!(xsd_validate("place", &b.xml));
}

#[test]
fn components_stay_at_origin_without_outline() {
    // No board outline → nothing to stage against → keep the (0,0) placeholder.
    let b = build("no-stage", BASIC);
    assert!(!b.xml.contains("<Profile>"), "BASIC has no outline");
    let comp = blocks(&b.xml, "Component")[0].1;
    assert_eq!(
        component_location(comp),
        (0.0, 0.0),
        "no-outline component must stay at origin"
    );
}

#[test]
fn no_board_outline_emits_no_profile() {
    // WITH_LAYOUT carries net-level constraints but no board_outline.
    let b = build("no-outline", WITH_LAYOUT);
    assert!(
        !b.xml.contains("<Profile>"),
        "unexpected Profile:\n{}",
        b.xml
    );
}

#[test]
fn smd_pins_carry_mount_type() {
    // R5-8 narrowing: pad plating now rides Pin/@mountType.
    let b = build("mount", WITH_OUTLINE);
    let pins = elements(&b.xml, "Pin");
    assert!(!pins.is_empty(), "footprint FP has pads → Pins expected");
    for p in &pins {
        assert_eq!(
            attr(p, "mountType").as_deref(),
            Some("SURFACE_MOUNT_PAD"),
            "smd pad Pin must carry mountType:\n{}",
            p
        );
    }
}

/// A board with a through-hole pad + an SMD pad, for the physical-model check.
const WITH_PHYSICAL: &str = r#"
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub pad SMD { shape: rect size: (0.5mm, 0.6mm) layer: top_copper plating: smd }
pub pad PTH { shape: circle size: (0.9mm) layer: through_all plating: plated_through_hole drill: 0.5mm }
pub footprint FP { pad 1: PTH at (-1mm, 0mm) pad 2: SMD at (1mm, 0mm) courtyard { shape: rect, at: (0mm, 0mm), size: (3mm, 2mm) } }
pub part CON: D { primary { mfr: "m", mpn: "n", footprint: FP } }
design B {
    inst j1: CON
    net N: j1.A
    nc: j1.B
    layout { board_outline: "o.dxf" }
}
"#;

#[test]
fn emits_physical_padstacks_layers_and_placed_copper() {
    let b = build("physical", WITH_PHYSICAL);
    // Real physical structures (the .co/invalid-ipc2581.xml fix).
    for needle in [
        "<DictionaryStandard",
        "<Layer name=\"F.Cu\"",
        "<Layer name=\"B.Cu\"",
        "<Stackup ",
        "<PadStackDef ",
        "<PadstackPadDef ",
        "<LayerFeature layerRef=\"F.Cu\">",
        "<Pad padstackDefRef=",
        "<PinRef componentRef=",
    ] {
        assert!(
            b.xml.contains(needle),
            "physical structure missing {}:\n…",
            needle
        );
    }
    // The THT pad produces a plated hole with the real drill diameter.
    assert!(
        b.xml.contains("<PadstackHoleDef") && b.xml.contains("diameter=\"0.5\""),
        "no plated hole for the through-hole pad"
    );
    assert!(
        b.xml.contains("platingStatus=\"PLATED\""),
        "hole not plated"
    );
    // A THT component is THMT; its through-hole pin lands on B.Cu too.
    assert!(
        b.xml.contains("mountType=\"THMT\""),
        "connector should be THMT"
    );
    assert!(
        b.xml.contains("<LayerFeature layerRef=\"B.Cu\">"),
        "THT pad missing from B.Cu"
    );
    // Still schema-valid with all the physical structures.
    assert!(xsd_validate("physical", &b.xml));
}

#[test]
fn placed_copper_is_tied_to_net_and_pin() {
    let b = build("physical", WITH_PHYSICAL);
    // The F.Cu feature's pad carries both its net (Set/@net) and its pin (PinRef).
    let fcu = b
        .xml
        .split("<LayerFeature layerRef=\"F.Cu\">")
        .nth(1)
        .unwrap();
    assert!(
        fcu.contains("<Set net=\"N\">"),
        "pad not tied to its net:\n{}",
        &fcu[..300.min(fcu.len())]
    );
    assert!(
        fcu.contains("componentRef=\"U1\" pin=\"1\""),
        "pad not tied to its pin"
    );
}

#[test]
fn courtyardless_footprint_gets_nondegenerate_outline() {
    // A footprint with pads but no courtyard must get a real bbox outline, not
    // the degenerate (0,0)-(0,0) polygon (which hid J1 in Quilter).
    let src = WITH_PHYSICAL.replace(
        " courtyard { shape: rect, at: (0mm, 0mm), size: (3mm, 2mm) }",
        "",
    );
    let b = build("nocourt", &src);
    let pkg = b.xml.split("<Package ").nth(1).unwrap();
    let outline = pkg.split("</Outline>").next().unwrap();
    assert!(
        !outline.contains(
            "<PolyBegin x=\"0\" y=\"0\"/>\n              <PolyStepSegment x=\"0\" y=\"0\"/>"
        ),
        "degenerate outline:\n{}",
        outline
    );
    assert!(
        outline.contains("<PolyBegin x=\"-1.45\""),
        "outline not from pad extents:\n{}",
        outline
    );
}

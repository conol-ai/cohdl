//! RFC-033 named loop nets (PR-43 review fix, conservative rule).
//!
//! A directly authored NAMED `net NAME: …` inside a circuit `for` body is
//! rejected statically with a single E1406 whose primary span is exactly the
//! net name; the diagnostic names the actual net and spells out the
//! outside-loop + anonymous-`net _` alternative. Anonymous `net _` inside
//! loops, named nets outside loops, and private named nets inside helper fns
//! CALLED from loops keep their old semantics (per-frame isolation).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::collections::BTreeSet;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn netlist(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    build_artifacts(&mut checked, &LockState::default())
        .expect("build")
        .netlist
}

/// Single passive-pin library — no obligation/multi-driver noise.
const LIB: &str = r#"pub device D { pins { A: 1 [passive] } }
pub device H { pins { V: 1 [passive] } }
pub footprint FP {}
pub part P: D { primary { mfr: "m", mpn: "p", footprint: FP } }
pub part HP: H { primary { mfr: "m", mpn: "h", footprint: FP } }
"#;

/// The one E1406 for `net NAME:` inside `body`, asserting: exactly one
/// error, the message names NAME, the alternative is spelled out, and the
/// primary span is exactly the NAME identifier (byte-exact).
fn assert_named_loop_net_e1406(src: &str, name: &str) -> String {
    let (chk, r) = check(src);
    let errors: Vec<_> = chk
        .diags
        .iter()
        .filter(|d| d.severity == cohdl::diag::Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "exactly one error:\n{r}");
    let d = errors[0];
    assert_eq!(d.code, "E1406", "{r}");
    assert!(
        d.message.contains(name),
        "message names {name}:\n{}",
        d.message
    );
    let text = format!("{} {}", d.message, d.help.join(" ")).to_lowercase();
    assert!(
        text.contains("outside") && (text.contains("anonymous") || text.contains("net _")),
        "outside-loop + anonymous alternative:\n{r}"
    );
    // byte-exact span of the NAME token after `net `.
    let off = src.find(&format!("net {name}:")).unwrap() + 4;
    let span = d.primary.span;
    assert_eq!(span.start as usize, off, "span start:\n{r}");
    assert_eq!(span.end as usize, off + name.len(), "span end:\n{r}");
    r
}

#[test]
fn named_net_in_design_loop_is_e1406_at_exact_name_span() {
    let src = format!(
        "{LIB}design Board {{ inst h: HP inst d: [P; 3] for wire: i in 0..3 {{ net BAD: h.V, d[i].A }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

#[test]
fn named_net_in_loop_reports_once_no_iteration_fanout_n50() {
    let src =
        format!("{LIB}design Board {{ inst h: HP for wire: i in 0..50 {{ net BAD: h.V }} }}\n");
    assert_named_loop_net_e1406(&src, "BAD");
}

#[test]
fn named_net_in_empty_loop_still_rejected() {
    let src = format!(
        "{LIB}design Board {{ inst h: HP net _: h.V for wire: i in 0..0 {{ net BAD: h.V }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

#[test]
fn named_net_in_nested_empty_loop_rejected() {
    let src = format!(
        "{LIB}design Board {{ inst h: HP net _: h.V for row: r in 0..0 {{ for col: c in 0..3 {{ net BAD: h.V }} }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

#[test]
fn named_net_in_loop_of_uncalled_fn_rejected_statically() {
    let src = format!(
        "{LIB}pub fn unused(p: Pin) {{ for wire: i in 0..0 {{ net BAD: p }} }}\ndesign Board {{ inst h: HP net _: h.V }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

#[test]
fn named_net_in_loop_of_unused_subdesign_rejected_statically() {
    let src = format!(
        "{LIB}pub subdesign Unused {{ ports {{ optional P: Pin }} for wire: i in 0..0 {{ net BAD: P }} }}\ndesign Board {{ inst h: HP net _: h.V }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

/// The original review reproducer: shadowing an outer rail from inside a
/// loop is the same single E1406.
#[test]
fn review_reproducer_shadowing_outer_vcc_rejected() {
    let src = format!(
        "{LIB}design Board {{ inst h: HP inst d: [P; 3] net VCC [5V]: h.V for wire: i in 0..3 {{ net VCC: d[i].A }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "VCC");
}

/// The same name at DIFFERENT definition sites reports once per site —
/// single-site dedup must never swallow a second location.
#[test]
fn same_name_at_different_sites_each_reports() {
    let src = format!(
        "{LIB}pub fn f1(a: Pin) {{ for w: i in 0..0 {{ net BAD: a }} }}\npub fn f2(a: Pin) {{ for w: i in 0..0 {{ net BAD: a }} }}\ndesign Board {{ inst h: HP net _: h.V }}\n"
    );
    let (chk, r) = check(&src);
    let e1406: Vec<_> = chk.diags.iter().filter(|d| d.code == "E1406").collect();
    assert_eq!(e1406.len(), 2, "one E1406 per definition site:\n{r}");
    assert_ne!(
        e1406[0].primary.span, e1406[1].primary.span,
        "distinct spans:\n{r}"
    );
}

/// A helper fn whose body carries the violation and is CALLED from a loop
/// still reports exactly once — the static definition-site diagnostic must
/// not be doubled by expansion. The call sites are real (0..3) and every
/// pin is pre-connected so no other diagnostic can hide the count.
#[test]
fn called_helper_with_named_loop_net_reports_once() {
    let src = format!(
        "{LIB}pub fn hook(a: Pin) {{ for wire: i in 0..0 {{ net BAD: a }} }}\ndesign Board {{ inst h: HP net VCC [5V]: h.V inst d: [P; 3] for wire: i in 0..3 {{ net _: h.V, d[i].A hook(d[i].A) }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

/// An instantiated subdesign with the violation likewise reports once,
/// against an otherwise fully legal design.
#[test]
fn instantiated_subdesign_with_named_loop_net_reports_once() {
    let src = format!(
        "{LIB}pub subdesign S {{ ports {{ optional P: Pin }} for wire: i in 0..0 {{ net BAD: P }} }}\ndesign Board {{ inst h: HP net VCC [5V]: h.V subdesign s: S {{ P: h.V }} }}\n"
    );
    assert_named_loop_net_e1406(&src, "BAD");
}

/// Legal pattern: outer named rail + anonymous loop connections through the
/// shared pin. The full VCC endpoint set, voltage and manufacturing name
/// survive; the GND variant keeps name + is_gnd.
#[test]
fn outer_named_rail_with_anonymous_loop_connections_full_ir() {
    let good = format!(
        "{LIB}design Board {{\n inst h: HP\n inst d: [P; 3]\n net VCC [5V]: h.V\n for wire: i in 0..3 {{ net _: h.V, d[i].A }}\n}}\n"
    );
    let (chk, r) = check(&good);
    assert!(!chk.diags.has_errors(), "{r}");
    let ir = chk.ir.as_ref().unwrap();
    let expected: BTreeSet<(String, String)> = [
        ("Board::h", "V"),
        ("Board::d_0", "A"),
        ("Board::d_1", "A"),
        ("Board::d_2", "A"),
    ]
    .iter()
    .map(|(i, p)| (i.to_string(), p.to_string()))
    .collect();
    assert_eq!(ir.nets.len(), 1, "{r}");
    let net = &ir.nets[0];
    assert_eq!(net.name, "VCC");
    assert_eq!(net.members, expected, "full four-endpoint set");
    assert!(!net.is_gnd);
    assert_eq!(net.voltage.as_ref().map(|v| v.text.as_str()), Some("5V"));

    let ground = good.replace("net VCC [5V]: h.V", "net GND [gnd]: h.V");
    let (chk, r) = check(&ground);
    assert!(!chk.diags.has_errors(), "{r}");
    let ir = chk.ir.as_ref().unwrap();
    assert_eq!(ir.nets.len(), 1, "{r}");
    let net = &ir.nets[0];
    assert_eq!(net.name, "GND");
    assert_eq!(net.members, expected, "same topology as the VCC case");
    assert!(net.is_gnd, "ground attribute preserved");
    assert_eq!(net.voltage, None);

    // real artifact: manufacturing name present in the emitted netlist
    let text = netlist(&ground);
    assert!(text.contains("GND"), "{text}");
    assert!(
        !text.contains("__for_wire_"),
        "no leftover loop-frame nets:\n{text}"
    );
}

/// A private named net inside a helper fn CALLED from a loop keeps the old
/// per-frame isolation: three separate nets, no short between iterations.
#[test]
fn helper_private_named_net_called_from_loop_stays_isolated() {
    let src = format!(
        "{LIB}pub fn hook(a: Pin, b: Pin) {{ net PRIVATE: a, b }}\ndesign Board {{ inst a: [P; 3] inst b: [P; 3] for wire: i in 0..3 {{ hook(a[i].A, b[i].A) }} }}\n"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "helper private net must not be banned:\n{r}"
    );
    let ir = chk.ir.as_ref().unwrap();
    assert_eq!(ir.nets.len(), 3, "one private net per iteration:\n{r}");
    for (i, net) in ir.nets.iter().enumerate() {
        assert_eq!(
            net.name,
            format!("__for_wire_{i}::__fn0_hook::PRIVATE"),
            "frame-private naming:\n{r}"
        );
        let expected: BTreeSet<(String, String)> = [
            (format!("Board::a_{i}"), "A".to_string()),
            (format!("Board::b_{i}"), "A".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            net.members, expected,
            "iteration {i} full endpoints, no cross-frame short"
        );
    }
}

/// Anonymous loop nets and outer named nets are untouched by the rule.
#[test]
fn anonymous_loop_nets_and_outer_named_nets_unaffected() {
    let src = format!(
        "{LIB}design Board {{\n inst h: HP\n inst d: [P; 3]\n net VCC [5V]: h.V\n for wire: i in 0..3 {{ net _: h.V, d[i].A }}\n}}\n"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    let ir = chk.ir.as_ref().unwrap();
    assert_eq!(ir.nets.len(), 1, "{r}");
    assert_eq!(ir.nets[0].name, "VCC");
    assert_eq!(ir.nets[0].members.len(), 4);
}

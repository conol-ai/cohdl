use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

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
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts.expect("build").netlist
}

const LIB: &str = r#"
pub trait Cap { designator_prefix: "C" }
pub device CapDev { pins { A: 1 [passive], B: 2 [passive] } }
impl Cap for CapDev {}
pub footprint FP {}
pub part C100N: CapDev { primary { mfr: "m", mpn: "c", footprint: FP } }
pub device Host { pins { P: 1 [passive], Q: 2 [passive] } }
pub part HOST: Host { primary { mfr: "m", mpn: "h", footprint: FP } }
"#;

#[test]
fn fn_const_int_param_binds_and_forwards() {
    let src = format!("{LIB}
pub fn inner<const N: Int>(p: Pin) {{ inst c: [C100N; N]  net _: p, c[0..=(N - 1)].A, c[0..=(N - 1)].B }}
pub fn outer<const N: Int = 2>(p: Pin) {{ inner::<N + 1>(p) }}
design Board {{ inst h: HOST  outer(h.P)  nc: h.Q }}");
    let (c, r) = check(&src);
    assert!(!c.diags.has_errors(), "{r}");
    let n = netlist(&src);
    assert_eq!(
        n.matches("(comp (ref \"C").count(),
        3,
        "N=2 default → inner gets 3"
    );
}

#[test]
fn subdesign_const_int_param() {
    let src = format!("{LIB}
pub subdesign Bank<const N: Int> {{ ports {{ required IN: Pin }}  inst c: [C100N; N]  net _: IN, c[0..=(N - 1)].A, c[0..=(N - 1)].B }}
design Board {{ inst h: HOST  subdesign b: Bank<4> {{ IN: h.P }}  nc: h.Q }}");
    let n = netlist(&src);
    assert_eq!(n.matches("(comp (ref \"C").count(), 4);
}

#[test]
fn kind_mismatches() {
    let src = format!(
        "{LIB}
pub fn f<const N: Int>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  f::<100nF>(h.P)  nc: h.Q }}"
    );
    assert!(
        check(&src).1.contains("E1401"),
        "unit literal for an Int parameter"
    );
    let src = format!(
        "{LIB}
pub fn g<V: Voltage>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  g::<3>(h.P)  nc: h.Q }}"
    );
    assert!(
        check(&src).1.contains("E113"),
        "bare number for a unit parameter stays E113"
    );
    let src = format!(
        "{LIB}
pub device D<const N: Int> {{ pins {{ A: 1 [passive] }} }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(
        check(&src).1.contains("E406"),
        "const generics on devices are rejected"
    );
    let src = format!(
        "{LIB}
pub fn f<const N: Int = 2mm>(p: Pin) {{ net _: p }}
design Board {{ inst h: HOST  net _: h.P, h.Q }}"
    );
    assert!(
        check(&src).1.contains("E406"),
        "Int default must be an integer literal"
    );
}

// ---------------------------------------------------------------------------
// Numeric identity vs literal spelling: two contracts. Identity is the exact
// femto count + unit type (1.5mm == 1.50mm); the spelling is what the source
// wrote and must survive pure forwarding (literal, parens, parameter/const
// chains) — only arithmetic composes a canonical text.
// ---------------------------------------------------------------------------

const SPACER_LIB: &str = r#"
pub trait Cap { designator_prefix: "C" }
pub device Spacer<W: Length> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { width: W }
}
impl Cap for Spacer {}
pub footprint FP {}
pub part SP15: Spacer<1.5mm> { primary { mfr: "m", mpn: "sp", footprint: FP } }
"#;

#[test]
fn equal_value_different_spelling_is_one_identity_no_e802() {
    // `1.5mm` and `1.50mm` under one MPN: same component, no E802.
    let src = format!(
        "{LIB}
pub device SpA<W: Length> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ width: W }}
}}
pub footprint FPA {{}}
pub part S1: SpA<1.5mm> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPA }} }}
pub part S2: SpA<1.50mm> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPA }} }}
design B {{
    inst a: S1  inst b: S2
    net _: a.A, b.A  net _: a.B, b.B
}}
"
    );
    let (c, r) = check(&src);
    assert!(!c.diags.has_errors(), "equal femto = one identity:\n{r}");
}

#[test]
fn different_values_under_one_mpn_still_e802() {
    let src = format!(
        "{LIB}
pub device SpB<W: Length> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ width: W }}
}}
pub footprint FPB {{}}
pub part S1: SpB<1.5mm> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPB }} }}
pub part S2: SpB<2.0mm> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPB }} }}
design B {{
    inst a: S1  inst b: S2
    net _: a.A, b.A  net _: a.B, b.B
}}
"
    );
    let (c, r) = check(&src);
    assert!(c.diags.has_errors(), "distinct values stay distinct");
    let e802: Vec<_> = c.diags.iter().filter(|d| d.code == "E802").collect();
    assert_eq!(e802.len(), 1, "exactly one E802:\n{r}");
    assert!(
        r.contains("shares manufacturer `m` + MPN `sp`"),
        "the AVL message:\n{r}"
    );
}

#[test]
fn default_and_equal_explicit_argument_share_identity() {
    let src = format!(
        "{LIB}
pub device SpC<W: Length = 1.5mm> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ width: W }}
}}
pub footprint FPC {{}}
pub part S1: SpC {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPC }} }}
pub part S2: SpC<1.50mm> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPC }} }}
design B {{
    inst a: S1  inst b: S2
    net _: a.A, b.A  net _: a.B, b.B
}}
"
    );
    let (c, r) = check(&src);
    assert!(
        !c.diags.has_errors(),
        "default `1.5mm` and explicit `1.50mm` are the same component:\n{r}"
    );
}

#[test]
fn spec_width_text_survives_pure_forwarding() {
    // The IR spec carries the source's literal spelling through every pure
    // forwarding form; arithmetic composes the canonical text instead.
    let src = |arg: &str| {
        format!(
            "{SPACER_LIB}
pub part PX: Spacer<{arg}> {{ primary {{ mfr: \"m\", mpn: \"px\", footprint: FP }} }}
design B {{
    inst x: PX
    net _: x.A  net _: x.B
}}
"
        )
    };
    // (literal, expected spec text)
    for (arg, want) in [
        ("1.50mm", "1.50mm"),
        ("(1.50mm)", "1.50mm"),
        ("((1.50mm))", "1.50mm"),
        ("1.50mm + 0mm", "1.5mm"),
        ("0mm + 1.50mm", "1.5mm"),
        ("-1.50mm", "-1.50mm"),
    ] {
        let mut c = check_files_in("board", &[("src/main.cohdl".to_string(), src(arg))], None)
            .expect("selection");
        let _ = build_artifacts(&mut c, &LockState::default());
        assert!(
            !c.diags.has_errors(),
            "`{arg}` must check cleanly:\n{}",
            c.diags.render(&c.sm)
        );
        let ir = c.ir.as_ref().unwrap();
        let width = ir.instances["B::x"].specs.get("width").unwrap();
        assert_eq!(
            width.text, want,
            "`{arg}`: spelling contract (femto {})",
            width.femto
        );
    }
}

#[test]
fn spec_width_text_survives_generic_and_const_chains() {
    // Pure forwarding through a generic parameter and a Length const keeps
    // the original literal; the value rides the substitution, the spelling
    // with it.
    let src = format!(
        "{SPACER_LIB}
pub device Wrap<W: Length> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ width: W }}
}}
pub part W1: Wrap<1.50mm> {{ primary {{ mfr: \"m\", mpn: \"w1\", footprint: FP }} }}
pub subdesign Carrier<T: Length> {{
    ports {{ required G: Pin }}
    inst s: Spacer<T>
    net _: G, s.A  net _: s.B
}}
design B {{
    const T: Length = 1.50mm
    inst w: W1
    subdesign c1: Carrier<1.50mm> {{ G: w.A }}
    subdesign c2: Carrier<T> {{ G: w.B }}
    net link: w.A, w.B
}}"
    );
    let (c, r) = check(&src);
    assert!(!c.diags.has_errors(), "{r}");
    let ir = c.ir.as_ref().unwrap();
    let w = ir.instances["B::w"].specs.get("width").unwrap();
    assert_eq!(w.text, "1.50mm");
    for path in ["B::c1::s", "B::c2::s"] {
        let s = ir.instances[path].specs.get("width").unwrap();
        assert_eq!(s.text, "1.50mm", "{path}: forwarded spelling");
    }
}

#[test]
fn bare_length_into_capacitance_is_exactly_one_e112() {
    // `1.5mm` where a Capacitance is expected: one E112 with the same shape
    // the pre-expression grammar produced — expected/found units, and the
    // primary label naming the literal.
    let src = format!(
        "{LIB}
pub device Cap2<C: Capacitance> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ capacitance: C }}
}}
pub footprint FPD {{}}
pub part C1: Cap2<1.5mm> {{ primary {{ mfr: \"m\", mpn: \"c1\", footprint: FPD }} }}
design B {{
    inst a: C1
    net _: a.A  net _: a.B
}}"
    );
    let (c, r) = check(&src);
    assert!(c.diags.has_errors(), "must be rejected");
    let e112: Vec<&cohdl::diag::Diagnostic> = c.diags.iter().filter(|d| d.code == "E112").collect();
    assert_eq!(e112.len(), 1, "exactly one E112:\n{r}");
    // Reject any additional error, including fixture resolution failures.
    let codes: Vec<&str> = c
        .diags
        .iter()
        .filter(|d| matches!(d.severity, cohdl::diag::Severity::Error))
        .map(|d| d.code)
        .collect();
    assert_eq!(codes, vec!["E112"], "the complete error set:\n{r}");
    let d = e112[0];
    assert!(matches!(d.severity, cohdl::diag::Severity::Error));
    assert_eq!(
        d.message,
        "generic argument for `C` has the wrong unit type: expected `Capacitance`, found `Length`"
    );
    let start = src.find("<1.5mm>").unwrap() + 1;
    assert_eq!(
        d.primary.span,
        cohdl::span::Span::new(cohdl::span::FileId(0), start as u32, (start + 5) as u32)
    );
    assert_eq!(c.sm.snippet(d.primary.span), "1.5mm");
    assert_eq!(d.primary.message, "`1.5mm` is a `Length`");
}

#[test]
fn wrong_unit_literal_matches_pre_expression_diag_shape() {
    // A non-Length unit literal in a Length slot keeps the exact historical
    // code/message/span/label — the pre-existing compatible path.
    let src = format!(
        "{LIB}
pub device SpE<W: Length> {{
    pins {{ A: 1 [passive], B: 2 [passive] }}
    spec {{ width: W }}
}}
pub footprint FPE {{}}
pub part S1: SpE<1.5uF> {{ primary {{ mfr: \"m\", mpn: \"sp\", footprint: FPE }} }}
design B {{
    inst a: S1
    net _: a.A  net _: a.B
}}"
    );
    let (c, r) = check(&src);
    assert!(c.diags.has_errors());
    let e112: Vec<&cohdl::diag::Diagnostic> = c.diags.iter().filter(|d| d.code == "E112").collect();
    assert_eq!(e112.len(), 1, "exactly one E112:\n{r}");
    assert_eq!(
        c.diags
            .iter()
            .filter(|d| d.severity == cohdl::diag::Severity::Error)
            .map(|d| d.code)
            .collect::<Vec<_>>(),
        vec!["E112"],
        "{r}"
    );
    let d = e112[0];
    assert_eq!(
        d.message,
        "generic argument for `W` has the wrong unit type: expected `Length`, found `Capacitance`"
    );
    let start = src.find("<1.5uF>").unwrap() + 1;
    assert_eq!(
        d.primary.span,
        cohdl::span::Span::new(cohdl::span::FileId(0), start as u32, (start + 5) as u32)
    );
    assert_eq!(c.sm.snippet(d.primary.span), "1.5uF");
    assert_eq!(d.primary.message, "`1.5uF` is a `Capacitance`");
}

// Every body is checked even when no expansion reaches it. When it does
// expand, both passes must describe the same static mistake identically.
const DIRECT_LIB: &str = r#"
pub device D<C: Capacitance> { pins { A: 1 [passive], B: 2 [passive] } spec { capacitance: C } }
pub footprint FPD {}
pub part P: D<1uF> { primary { mfr: "m", mpn: "c", footprint: FPD } }
"#;

fn assert_wrong_unit(src: &str, argument: &str, value: &str, unit: &str, codes: &[&str]) {
    use cohdl::diag::Severity;
    use cohdl::span::{FileId, Span};
    let (c, rendered) = check(src);
    let errors: Vec<_> = c
        .diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(
        errors.iter().map(|d| d.code).collect::<Vec<_>>(),
        codes,
        "{src}\n{rendered}"
    );
    let wrong: Vec<_> = errors.iter().filter(|d| d.code == "E112").collect();
    assert_eq!(wrong.len(), 1, "{rendered}");
    let d = wrong[0];
    let start = src.find(&format!("D<{argument}>")).expect("argument site") + 2;
    assert_eq!(
        d.primary.span,
        Span::new(FileId(0), start as u32, (start + argument.len()) as u32)
    );
    assert_eq!(c.sm.snippet(d.primary.span), argument);
    assert_eq!(d.message, format!("generic argument for `C` has the wrong unit type: expected `Capacitance`, found `{unit}`"));
    assert_eq!(d.primary.message, format!("`{value}` is a `{unit}`"));
    assert!(d.secondary.is_empty());
    assert!(d.help.is_empty());
}

fn wrong_unit_contexts(argument: &str, prefix: &str) -> Vec<String> {
    let body = format!("{prefix} inst x: D<{argument}> net _: x.A, x.B");
    vec![
        format!("{DIRECT_LIB}\ndesign B {{ {body} }}"),
        format!("{DIRECT_LIB}\npub fn f() {{ {body} }} design B {{}}"),
        format!("{DIRECT_LIB}\npub fn f() {{ {body} }} design B {{ f() }}"),
        format!("{DIRECT_LIB}\npub subdesign S {{ {body} }} design B {{}}"),
        format!("{DIRECT_LIB}\npub subdesign S {{ {body} }} design B {{ subdesign s: S {{}} }}"),
    ]
}

#[test]
fn non_length_wrong_unit_is_checked_once_in_every_body_context() {
    for src in wrong_unit_contexts("1.5V", "") {
        assert_wrong_unit(&src, "1.5V", "1.5V", "Voltage", &["E112"]);
    }
}

#[test]
fn bare_length_wrong_unit_is_checked_once_in_every_body_context() {
    for src in wrong_unit_contexts("1.50mm", "") {
        assert_wrong_unit(&src, "1.50mm", "1.50mm", "Length", &["E112"]);
    }
}

#[test]
fn parenthesized_length_wrong_unit_is_checked_once_in_every_body_context() {
    for src in wrong_unit_contexts("(1.50mm)", "") {
        assert_wrong_unit(&src, "(1.50mm)", "1.50mm", "Length", &["E112"]);
    }
}

#[test]
fn computed_length_wrong_unit_is_checked_once_in_every_body_context() {
    for src in wrong_unit_contexts("1.50mm + 0mm", "") {
        assert_wrong_unit(&src, "1.50mm + 0mm", "1.50mm + 0mm", "Length", &["E112"]);
    }
}

#[test]
fn const_length_expression_wrong_unit_uses_the_lexical_environment() {
    for src in wrong_unit_contexts("L + 0mm", "const L: Length = 1.50mm") {
        assert_wrong_unit(&src, "L + 0mm", "L + 0mm", "Length", &["E112"]);
    }
}

#[test]
fn empty_loop_keeps_the_static_wrong_unit_error() {
    // Physical inst declarations are independently forbidden in loops.
    // Even a zero-iteration loop must retain its unit diagnostic as well.
    for (argument, value, unit) in [("1.5V", "1.5V", "Voltage"), ("1.50mm", "1.50mm", "Length")] {
        let src = format!(
            "{DIRECT_LIB}\ndesign B {{ for empty: i in 0..0 {{ inst bad: D<{argument}> }} }}"
        );
        assert_wrong_unit(&src, argument, value, unit, &["E1406", "E112"]);
    }
}

#[test]
fn unknown_length_expression_wrong_unit_is_checked_in_unused_and_used_bodies() {
    for argument in ["L + 0mm", "(L)"] {
        let body = format!("inst x: D<{argument}> net _: x.A, x.B");
        let sources = [
            format!("{DIRECT_LIB}\npub fn f<L: Length>() {{ {body} }} design B {{}}"),
            format!("{DIRECT_LIB}\npub fn f<L: Length>() {{ {body} }} design B {{ f::<1.50mm>() }}"),
            format!("{DIRECT_LIB}\npub subdesign S<L: Length> {{ {body} }} design B {{}}"),
            format!("{DIRECT_LIB}\npub subdesign S<L: Length> {{ {body} }} design B {{ subdesign s: S<1.50mm> {{}} }}"),
        ];
        for src in sources {
            assert_wrong_unit(&src, argument, argument, "Length", &["E112"]);
        }
    }
}

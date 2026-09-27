//! Cross-feature regressions for the RFC-033 review fixes.
use cohdl::pipeline::check_files_in;

const LIB: &str = r#"
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub footprint FP {}
pub part P: D { primary { mfr: "m", mpn: "p", footprint: FP } }
"#;

fn diagnostics(body: &str) -> String {
    let files = vec![("src/main.cohdl".into(), format!("{LIB}\n{body}"))];
    let mut c = check_files_in("board", &files, None).unwrap();
    c.diags.sort(&c.sm);
    c.diags.render(&c.sm)
}

#[test]
fn constants_are_forward_declared_and_scoped_to_each_body() {
    let r = diagnostics(
        r#"
pub fn unused(p: Pin) {
    const A: Int = C + 1
    const C: Int = 2
    for left: i in 0..0 { const X: Int = i + A net _: p }
    for right: i in 0..0 { const X: Int = i + C net _: p }
}
design B { inst a: P net _: a.A, a.B
    layout { const X: Length = Y + 1mm const Y: Length = 2mm place a at (X, 0mm) }
}
"#,
    );
    assert!(!r.contains("error["), "{r}");
}

#[test]
fn empty_loops_check_concrete_pins_and_function_arity() {
    for (operation, code) in [("net _: a.MISSING", "E203"), ("f(a.A, a.B)", "E502")] {
        let r = diagnostics(&format!("pub fn f(p: Pin) {{ net _: p }} design B {{ inst a: P for empty: i in 0..0 {{ {operation} }} net _: a.A, a.B }}"));
        assert!(r.contains(code), "{operation}: {r}");
    }
}

#[test]
fn only_declared_arrays_have_a_length_in_unused_definitions() {
    for body in [
        "pub fn f(p: Pin) { const N: Int = p.len }",
        "pub fn f(p: Pin) { inst a: P const N: Int = a.len net _: p, a.A, a.B }",
    ] {
        let r = diagnostics(&format!("{body} design B {{ inst a: P net _: a.A, a.B }}"));
        assert!(r.contains("E1401"), "{r}");
    }
    let r = diagnostics("pub fn f<const N: Int>(p: Pin) { const M: Int = a.len inst a: [P; N] net _: p, a[0].A, a[0].B } design B { inst a: P net _: a.A, a.B }");
    assert!(
        !r.contains("error["),
        "unknown-valued array remains valid: {r}"
    );
}

#[test]
fn known_local_constants_expose_zero_divisors_even_uncalled() {
    let r = diagnostics("pub fn f(p: Pin) { const ZERO: Int = OTHER - 2 const OTHER: Int = 2 const BAD: Int = 1 / ZERO net _: p } design B { inst a: P net _: a.A, a.B }");
    assert!(r.contains("E1403"), "{r}");
    assert!(!r.contains("E202"), "{r}");
}

#[test]
fn constant_shadowing_and_scope_escape_are_rejected() {
    for definition in [
        "pub fn f<const N: Int>(p: Pin) { const N: Int = 2 net _: p }",
        "pub fn f(p: Pin) { for each: i in 0..0 { const p: Int = 1 } net _: p }",
        "pub fn f(p: Pin) { layout { const p: Length = 1mm } net _: p }",
    ] {
        let r = diagnostics(&format!(
            "{definition} design B {{ inst a: P net _: a.A, a.B }}"
        ));
        assert!(r.contains("E201"), "{r}");
    }
    let r = diagnostics("pub fn f(p: Pin) { for left: i in 0..0 { const X: Int = 1 } for right: i in 0..0 { const Y: Int = X } net _: p } design B { inst a: P net _: a.A, a.B }");
    assert!(r.contains("E202"), "loop-local X cannot escape: {r}");
}

#[test]
fn real_loop_binding_and_layout_forward_constants_share_the_environment() {
    let r = diagnostics("design B { inst a: [P; 2] for wire: i in 0..2 { const X: Int = i net _: a[X].A, a[X].B } layout { for place_each: i in 0..2 { const X: Length = Y + 1mm const Y: Length = i * 2mm place a[i] at (X, 0mm) } } }");
    assert!(!r.contains("error["), "{r}");
}

#[test]
fn bound_layout_checks_empty_descendants_without_specializing_skipped_calls() {
    let actual = diagnostics("pub subdesign S<const N: Int> { layout { for empty: i in 0..0 { const BAD: Int = 1 / N } } } design B { subdesign s: S<0> }");
    assert!(actual.contains("E1403"), "{actual}");
    let skipped = diagnostics("pub fn f<const N: Int>(p: Pin) { layout { for empty: i in 0..0 { const BAD: Int = 1 / N } } net _: p } design B { inst a: P for skip: i in 0..0 { f::<0>(a.A) } net _: a.A, a.B }");
    assert!(!skipped.contains("error["), "{skipped}");
}

#[test]
fn constant_cycles_are_reported_in_unused_and_layout_scopes() {
    for body in [
        "pub fn f(p: Pin) { const A: Int = B const B: Int = A net _: p } design B {}",
        "design B { layout { const X: Length = Y const Y: Length = X } }",
    ] {
        let r = diagnostics(body);
        assert!(r.contains("E1407"), "{r}");
        assert!(!r.contains("E202"), "cycles still have declared names: {r}");
    }
}

#[test]
fn invalid_bare_int_arguments_are_rejected_in_uncalled_helpers() {
    for number in ["1.5", "9223372036854775808"] {
        let r = diagnostics(&format!("pub fn f<const N: Int>(p: Pin) {{ net _: p }} pub fn unused(p: Pin) {{ f::<{number}>(p) }} design B {{}}"));
        assert!(
            r.contains(if number.contains('.') {
                "E1401"
            } else {
                "E1402"
            }),
            "{r}"
        );
    }
}

#[test]
fn generic_argument_types_are_checked_without_entering_the_callee() {
    for call in [
        "count::<L>()",
        "count::<L + 0mm>()",
        "count::<1mm>()",
        "pitch::<N>()",
        "pitch::<N + 0>()",
    ] {
        for body in [format!("pub fn unused() {{ const L: Length = 1mm const N: Int = 1 {call} }} design B {{}}"),
            format!("design B {{ const L: Length = 1mm const N: Int = 1 for skipped: i in 0..0 {{ {call} }} }}")]
        {
            let r = diagnostics(&format!("pub fn count<const N: Int>() {{}} pub fn pitch<PITCH: Length>() {{}} {body}"));
            assert!(r.contains("E1401"), "{call}: {r}");
        }
    }
    let r = diagnostics("pub fn f<const N: Int, PITCH: Length>() { const BAD: Int = 1 / N } pub fn unused<const M: Int, STEP: Length>() { for skipped: i in 0..0 { f::<M, STEP>() } } design B {}");
    assert!(
        !r.contains("error["),
        "unknown values remain unspecialized: {r}"
    );
}

#[test]
fn duplicate_declarations_report_only_the_later_site() {
    for body in [
        "design B { for clash: i in 0..0 {} for clash: j in 0..0 {} }",
        "pub fn unused() { for clash: i in 0..0 {} for clash: j in 0..0 {} } design B {}",
        "design B { layout { for clash: i in 0..0 {} for clash: j in 0..0 {} } }",
        "pub fn unused(p: Pin) { const clash: Int = 1 net clash: p } design B {}",
        "pub fn unused(p: Pin) { net clash: p const clash: Int = 1 } design B {}",
    ] {
        let source = format!("{LIB}\n{body}");
        let later_site = source.rfind("clash").unwrap();
        let files = vec![("src/main.cohdl".into(), source)];
        let mut c = check_files_in("board", &files, None).unwrap();
        c.diags.sort(&c.sm);
        let duplicates: Vec<_> = c.diags.iter().filter(|d| d.code == "E201").collect();
        assert_eq!(duplicates.len(), 1, "{body}: {}", c.diags.render(&c.sm));
        assert_eq!(duplicates[0].primary.span.start as usize, later_site);
    }
    let r = diagnostics("pub fn unused(p: Pin) { net same: p net same: p } design B {}");
    assert!(
        !r.contains("E201"),
        "repeated net declarations still merge: {r}"
    );
}

#[test]
fn called_pin_expressions_have_one_consistent_kind_error() {
    for expression in ["p + 1", "p.len"] {
        let r = diagnostics(&format!(
            "pub fn f(p: Pin) {{ const N: Int = {expression} net _: p }}
             design B {{ inst a: P f(a.A) nc: a.B }}"
        ));
        assert_eq!(r.matches("error[E1401]").count(), 1, "{r}");
        assert!(
            !r.contains("E202"),
            "a Pin parameter is known but not a value: {r}"
        );
    }
}

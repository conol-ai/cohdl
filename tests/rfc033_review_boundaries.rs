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
fn bare_int_arguments_never_fall_back_to_zero() {
    for value in [
        "9223372036854775808",
        "(9223372036854775808)",
        "1.5",
        "(1.5)",
    ] {
        let r = diagnostics(&format!(
            "pub fn f<const N: Int>(p: Pin) {{ for n: i in 0..N {{ net _: p }} }}
             design B {{ inst a: P f::<{value}>(a.A) net _: a.A, a.B }}"
        ));
        assert!(
            r.contains(if value.contains('.') {
                "E1401"
            } else {
                "E1402"
            }),
            "invalid Int {value}: {r}"
        );
    }
    for value in ["9223372036854775808", "1.5"] {
        let r = diagnostics(&format!(
            "pub fn f<const N: Int = {value}>(p: Pin) {{ net _: p }}
             design B {{ inst a: P f(a.A) nc: a.B }}"
        ));
        assert!(
            r.contains(if value.contains('.') {
                "E1401"
            } else {
                "E1402"
            }),
            "invalid default {value}: {r}"
        );
    }
    for value in ["0", "9223372036854775807", "-9223372036854775808"] {
        let r = diagnostics(&format!(
            "pub fn f<const N: Int>(p: Pin) {{ net _: p }}
             design B {{ inst a: P f::<{value}>(a.A) nc: a.B }}"
        ));
        assert!(!r.contains("error["), "valid Int {value}: {r}");
    }
}

#[test]
fn array_lengths_and_local_lengths_can_be_forwarded_together() {
    let r = diagnostics(
        r#"
pub fn wire<const N: Int, PITCH: Length>(p: Pin) {
    const WIDTH: Length = N * PITCH
    net _: p
}
design B {
    const STEP: Length = 2mm
    inst a: [P; 2]
    wire::<a.len, STEP>(a[0].A)
    net _: a[0].B, a[1].A, a[1].B
}
"#,
    );
    assert!(!r.contains("error["), "{r}");
}

#[test]
fn forward_array_length_dependency_reaches_subdesign_generic() {
    let r = diagnostics(
        r#"
pub subdesign S<const N: Int> { ports { optional IN: Pin } }
design B {
    const COUNT: Int = a.len
    subdesign s: S<COUNT>
    inst a: [P; 2]
    net _: a[0..=1].A, a[0..=1].B
}

"#,
    );
    assert!(!r.contains("error["), "{r}");
}

#[test]
fn sibling_layout_constants_are_isolated_and_may_reference_binders() {
    let r = diagnostics(
        r#"
design B {
    inst a: [P; 2]
    net _: a[0..=1].A, a[0..=1].B
    layout {
        for left: i in 0..1 { const X: Length = i * 2mm place a[i] at (X, 0mm) }
        for right: i in 1..2 { const X: Length = i * 2mm place a[i] at (X, 0mm) }
    }
}
"#,
    );
    assert!(!r.contains("error["), "{r}");
}

#[test]
fn empty_loop_checks_forward_constant_zero_without_specializing_callee() {
    let r = diagnostics(
        r#"
pub fn unused<const N: Int>(p: Pin) {
    for empty: i in 0..0 {
        const BAD: Int = N / ZERO
        const ZERO: Int = 0
        net _: p
    }
}
design B { inst a: P net _: a.A, a.B }
"#,
    );
    assert!(r.contains("E1403"), "known zero divisor: {r}");
    assert!(!r.contains("E202"), "forward name is visible: {r}");
}

#[test]
fn duplicate_sibling_labels_still_fail_after_scope_isolation() {
    let r = diagnostics(
        r#"
design B {
    inst a: P
    for repeated: i in 0..0 { const X: Int = i net _: a.A }
    for repeated: i in 0..0 { const X: Int = i net _: a.B }
    net _: a.A, a.B
}

"#,
    );
    assert!(
        r.contains("E201"),
        "duplicate loop labels remain invalid: {r}"
    );
}

#[test]
fn direct_array_lengths_are_visible_before_instance_expansion() {
    for argument in ["a.len", "(a.len)", "a.len + 1"] {
        let r = diagnostics(&format!(
            "pub subdesign S<const N: Int> {{ ports {{ optional IN: Pin }} }}
             design B {{ subdesign s: S<{argument}> inst a: [P; 2]
                         net _: a[0..=1].A, a[0..=1].B }}"
        ));
        assert!(!r.contains("error["), "{argument}: {r}");
    }
}

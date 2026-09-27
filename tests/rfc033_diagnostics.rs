use cohdl::diag::Severity;
use cohdl::pipeline::{check_files_in, Checked};
use cohdl::span::{FileId, Span};

const LIB: &str = "pub device D { pins { A: 1 [passive], B: 2 [passive] } }\npub footprint FP {}\npub part P: D { primary { mfr: \"m\", mpn: \"p\", footprint: FP } }\n";

fn check(body: &str) -> (Checked, String) {
    let source = format!("{LIB}{body}");
    let mut c = check_files_in("board", &[("main.cohdl".into(), source.clone())], None).unwrap();
    c.diags.sort(&c.sm);
    (c, source)
}

fn codes(c: &Checked, expected: &[&str]) {
    assert_eq!(
        c.diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect::<Vec<_>>(),
        expected,
        "{}",
        c.diags.render(&c.sm)
    );
}

fn span(source: &str, text: &str) -> Span {
    let start = source.rfind(text).unwrap();
    Span::new(FileId(0), start as u32, (start + text.len()) as u32)
}

#[test]
fn nonpositive_arrays_fail_once_before_materialization() {
    for value in ["0 - 1", "0"] {
        for declaration in ["inst a: [P; N]", "subdesign a: [S; N]"] {
            let (c, src) = check(&format!("pub subdesign S {{ ports {{ optional A: Pin }} }} design B {{ const N: Int = {value} {declaration} }}"));
            codes(&c, &["E211"]);
            let d = c.diags.iter().next().unwrap();
            assert_eq!(d.primary.span, {
                let s = span(&src, "N]");
                Span::new(s.file, s.start, s.end - 1)
            });
        }
    }
}

#[test]
fn static_expression_failure_is_independent_of_iteration_count() {
    for count in [0, 1, 50] {
        for loop_body in [
            format!("layout {{ for pos: i in 0..{count} {{ place a[i] at (10mm + i, 0mm) }} }}"),
            format!("for pos: i in 0..{count} {{ layout {{ place a[i] at (10mm + i, 0mm) }} }}"),
        ] {
            let (c, src) = check(&format!(
                "design B {{ inst a: [P; 50] net _: a[0..=49].A, a[0..=49].B {loop_body} }}"
            ));
            codes(&c, &["E1401"]);
            assert_eq!(
                c.diags.iter().next().unwrap().primary.span,
                span(&src, "10mm + i")
            );
        }
    }
}

#[test]
fn duplicate_placements_identify_each_target_and_iteration() {
    let (c, src) = check("design B { inst a: [P; 2] net _: a[0..=1].A, a[0..=1].B layout { for pos: i in 0..2 { place a[i] at (0mm, 0mm) place a[i] at (1mm, 0mm) } } }");
    codes(&c, &["E1007", "E1007"]);
    for (i, d) in c.diags.iter().enumerate() {
        assert_eq!(d.primary.span, span(&src, "a[i]"));
        assert_eq!(
            d.message,
            format!(
                "`a[i]` is placed more than once; target `B::a_{i}` — in B::__for_pos_{i}, i = {i}"
            )
        );
        assert_eq!(
            d.primary.message,
            format!("target `B::a_{i}` — in B::__for_pos_{i}, i = {i}")
        );
        assert!(d.help.is_empty());
        assert!(d.secondary.is_empty());
    }
}

#[test]
fn rotation_errors_identify_computed_angle_target_and_iteration() {
    let (c, src) = check("design B { inst a: [P; 4] net _: a[0..=3].A, a[0..=3].B layout { for pos: i in 0..4 { place a[i] at (0mm, 0mm) rotate 200 * i } } }");
    codes(&c, &["E1007", "E1007"]);
    for (i, d) in (2..4).zip(c.diags.iter()) {
        assert_eq!(d.primary.span, span(&src, "200 * i"));
        assert_eq!(d.message, format!("`rotate {}` is not a rotation — give a whole number of degrees in 0..=359 (counter-clockwise); target `B::a_{i}`, computed angle {} — in B::__for_pos_{i}, i = {i}", 200 * i, 200 * i));
        assert_eq!(
            d.primary.message,
            format!(
                "target `B::a_{i}`, computed angle {} — in B::__for_pos_{i}, i = {i}",
                200 * i
            )
        );
        assert!(d.help.is_empty());
        assert!(d.secondary.is_empty());
    }
}

#[test]
fn nested_layout_errors_carry_all_frames_in_main_message() {
    for rotate in [false, true] {
        let placements = if rotate {
            "place a[i] at (0mm, 0mm) rotate 200 * i"
        } else {
            "place a[i] at (0mm, 0mm) place a[i] at (1mm, 0mm)"
        };
        let (c, src) = check(&format!("design B {{ inst a: [P; 4] net _: a[0..=3].A, a[0..=3].B layout {{ for rows: row in 7..8 {{ for pos: i in 2..4 {{ {placements} }} }} }} }}"));
        codes(&c, &["E1007", "E1007"]);
        for (i, d) in (2..4).zip(c.diags.iter()) {
            let suffix = format!(" — in B::__for_rows_7::__for_pos_{i}, row = 7, i = {i}");
            let (message, label, site) = if rotate {
                (format!("`rotate {}` is not a rotation — give a whole number of degrees in 0..=359 (counter-clockwise)", 200 * i), format!("target `B::a_{i}`, computed angle {}{suffix}", 200 * i), "200 * i")
            } else {
                (
                    "`a[i]` is placed more than once".to_string(),
                    format!("target `B::a_{i}`{suffix}"),
                    "a[i]",
                )
            };
            assert_eq!(d.primary.span, span(&src, site));
            assert_eq!(d.message, format!("{message}; {label}"));
            assert_eq!(d.primary.message, label);
            assert!(d.help.is_empty());
            assert!(d.secondary.is_empty());
        }
    }
}

#[test]
fn nonloop_layout_errors_keep_legacy_messages() {
    for subdesign in [false, true] {
        for (placement, site, message) in [
            ("place a at (0mm, 0mm) place a at (1mm, 0mm)", "a at", "`a` is placed more than once"),
            ("place a at (0mm, 0mm) rotate 400", "400", "`rotate 400` is not a rotation — give a whole number of degrees in 0..=359 (counter-clockwise)"),
        ] {
            let body = format!("inst a: P net _: a.A, a.B layout {{ {placement} }}");
            let body = if subdesign {
                format!("pub subdesign S {{ {body} }} design B {{ subdesign s: S }}")
            } else {
                format!("design B {{ {body} }}")
            };
            let (c, src) = check(&body);
            codes(&c, &["E1007"]);
            let d = c.diags.iter().next().unwrap();
            let mut expected_span = span(&src, site);
            if site == "a at" {
                expected_span.end = expected_span.start + 1;
            }
            assert_eq!(d.primary.span, expected_span);
            assert_eq!(d.message, message);
            assert!(d.primary.message.is_empty());
            assert!(d.help.is_empty());
            assert!(d.secondary.is_empty());
        }
    }
}

#[test]
fn value_errors_survive_for_each_iteration() {
    let (c, src) = check("design B { inst a: [P; 2] net _: a[0..=1].A, a[0..=1].B layout { for pos: i in 0..2 { place a[i] at (1mm / (i - i), 0mm) } } }");
    codes(&c, &["E1403", "E1403"]);
    for (i, d) in c.diags.iter().enumerate() {
        assert_eq!(d.primary.span, span(&src, "1mm / (i - i)"));
        assert!(
            d.message
                .ends_with(&format!(" — in B::__for_pos_{i}, i = {i}")),
            "{}",
            d.message
        );
    }
    let (c, _) = check("design B { inst a: [P; 2] net _: a[0..=1].A, a[0..=1].B layout { for pos: i in 2..4 { place a[i] at (0mm, 0mm) } } }");
    codes(&c, &["E202", "E202"]);
    for (i, d) in (2..4).zip(c.diags.iter()) {
        assert!(
            d.message.contains(&format!("index {i} is out of bounds")),
            "{}",
            d.message
        );
        assert!(
            d.message
                .ends_with(&format!(" — in B::__for_pos_{i}, i = {i}")),
            "{}",
            d.message
        );
    }
}

#[test]
fn integer_literal_errors_have_one_precise_code() {
    for (literal, code) in [
        ("9223372036854775808", "E1402"),
        ("-9223372036854775809", "E1402"),
        ("1.5", "E1401"),
        ("-1.5", "E1401"),
    ] {
        for value in [literal.to_string(), format!("({literal})")] {
            for body in [format!("design B {{ const N: Int = {value} }}"), format!("pub fn f<const N: Int>(p: Pin) {{ net _: p }} design B {{ inst a: P f::<{value}>(a.A) net _: a.A, a.B }}")] {
                let (c, _) = check(&body);
                codes(&c, &[code]);
            }
        }
    }
}

#[test]
fn failed_activation_does_not_poison_another_binding() {
    for body in [
        "pub fn f<const N: Int>(p: Pin) { const BAD: Int = 1 / (N - N) net _: p } design B { inst a: P f::<1>(a.A) f::<2>(a.B) }",
        "pub subdesign S<const N: Int> { const BAD: Int = 1 / (N - N) } design B { subdesign a: S<1> subdesign b: S<2> }",
    ] {
        let (c, src) = check(body);
        codes(&c, &["E1403", "E1403"]);
        for (n, d) in (1..3).zip(c.diags.iter()) {
            assert_eq!(d.primary.span, span(&src, "1 / (N - N)"));
            assert!(d.message.contains(&format!("N = {n}")), "{}", d.message);
        }
    }
}

#[test]
fn static_failure_does_not_hide_independent_iteration_errors() {
    let (c, _) = check("design B { inst a: [P; 2] net _: a[0..=1].A, a[0..=1].B layout { for pos: i in 0..2 { place a[i] at (10mm + i, 0mm) place a[i] at (1mm / (i - i), 0mm) } } }");
    codes(&c, &["E1401", "E1403", "E1403"]);
}

#[test]
fn invalid_int_defaults_do_not_become_missing_arguments() {
    for (literal, code) in [
        ("9223372036854775808", "E1402"),
        ("-9223372036854775809", "E1402"),
        ("1.5", "E1401"),
        ("-1.5", "E1401"),
    ] {
        let (c, _) = check(&format!("pub fn f<const N: Int = {literal}>(p: Pin) {{ net _: p }} design B {{ inst a: P f(a.A) net _: a.A, a.B }}"));
        codes(&c, &[code]);
    }
}

#[test]
fn generic_array_failures_are_local_to_each_activation() {
    for declaration in ["inst a: [P; N]", "subdesign a: [Empty; N]"] {
        let (c, _) = check(&format!("pub subdesign Empty {{}} pub subdesign S<const N: Int> {{ {declaration} }} design B {{ subdesign x: S<-1> subdesign y: S<0> }}"));
        codes(&c, &["E211", "E211"]);
    }
}

#[test]
fn generic_argument_value_errors_keep_each_activation_and_binder() {
    for body in [
        "pub fn g<const K: Int>(p: Pin) { net _: p } pub fn f<const N: Int>(p: Pin) { g::<1 / (N - N)>(p) } design B { inst a: P net _: a.A,a.B f::<1>(a.A) f::<2>(a.B) }",
        "pub subdesign Leaf<const K: Int> {} pub subdesign S<const N: Int> { subdesign leaf: Leaf<1 / (N - N)> } design B { subdesign a: S<1> subdesign b: S<2> }",
        "pub fn g<const K: Int>(p: Pin) { net _: p } design B { inst a: P net _: a.A,a.B for calls: i in 0..2 { g::<1 / (i - i)>(a.A) } }",
    ] {
        let (c, _) = check(body);
        codes(&c, &["E1403", "E1403"]);
        let messages: Vec<_> = c.diags.iter().map(|d| d.message.as_str()).collect();
        assert_ne!(messages[0], messages[1]);
    }
}

#[test]
fn mixed_const_array_cycles_have_one_complete_chain() {
    for declaration in ["inst a: [P; N]", "subdesign a: [Empty; N]"] {
        let (c, _) = check(&format!(
            "pub subdesign Empty {{}} design B {{ const N: Int = a.len {declaration} }}"
        ));
        codes(&c, &["E1407"]);
        assert_eq!(
            c.diags.iter().next().unwrap().message,
            "cyclic dependency: `N` → `a`.len → `N`"
        );
    }
}

#[test]
fn large_fractional_literals_are_kind_errors_not_integer_overflow() {
    for value in ["999999999999999999999999.5", "-999999999999999999999999.5"] {
        for body in [format!("design B {{ const N: Int = {value} }}"), format!("pub fn f<const N: Int>(p: Pin) {{ net _: p }} design B {{ inst a: P f::<{value}>(a.A) net _: a.A, a.B }}")] {
            let (c, _) = check(&body);
            codes(&c, &["E1401"]);
        }
    }
}

#[test]
fn caller_loop_labels_do_not_enter_the_callee_lexical_scope() {
    let (c, _) = check("pub fn f(p: Pin) { for links: i in 0..1 { net _: p } } design B { inst a: P net _: a.A,a.B for links: i in 0..2 { f(a.A) } }");
    codes(&c, &[]);
}

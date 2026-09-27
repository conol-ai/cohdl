//! PR-43 re-review (2026-09-27) regression A: a signed Temperature literal
//! used as a bare generic argument must parse through the legacy
//! `GenericArg::Unit` path again (RFC-033 routed every leading `-` into the
//! expression grammar, turning `Td<-40C>` into E1401). Signed Length
//! expression semantics, Int negatives, wrong-unit E112 and rejection of
//! other negative physical quantities / real temperature arithmetic are
//! unchanged.

use cohdl::pipeline::check_files_in;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

const LIB: &str = r#"
pub device Td<T: Temperature> { pins { A: 1 [passive] } spec { tmin: T } }
pub footprint FP {}
"#;

#[test]
fn negative_temperature_bare_generic_arg_on_inst_resolves_exactly() {
    let src = format!(
        "{LIB}
design B {{
    inst t: Td<-40C>
    net _: t.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "signed Temperature bare arg must pass:\n{r}"
    );
    // resolved spec carries the exact signed value into the manufacturing IR
    let ir = chk.ir.as_ref().unwrap();
    let inst = ir.instances.get("B::t").expect("instance B::t");
    let tmin = inst.specs.get("tmin").expect("tmin spec resolved");
    assert_eq!(tmin.text, "-40C", "spelling preserved");
    assert_eq!(tmin.femto, -40_000_000_000_000_000i128, "exact femto value");
    // parser produced the legacy Unit argument with the full source span
    let design = chk.world.designs.values().next().unwrap();
    let inst_stmt = design
        .body
        .iter()
        .find_map(|s| match s {
            cohdl::ast::Stmt::Inst(i) => Some(i),
            _ => None,
        })
        .expect("inst stmt");
    match &inst_stmt.ty.generic_args[0] {
        cohdl::ast::GenericArg::Unit(_, span) => {
            assert_eq!(
                chk.sm.snippet(*span),
                "-40C",
                "span covers the signed literal"
            );
        }
        other => panic!("expected GenericArg::Unit, got {other:?}"),
    }
}

#[test]
fn unsigned_temperature_control_still_passes() {
    let src = format!(
        "{LIB}
design B {{
    inst t: Td<85C>
    net _: t.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(!chk.diags.has_errors(), "{r}");
    let ir = chk.ir.as_ref().unwrap();
    assert_eq!(
        ir.instances["B::t"].specs["tmin"].text, "85C",
        "unsigned control"
    );
}

#[test]
fn negative_temperature_on_part_binding_and_fn_call() {
    let src = format!(
        "{LIB}
pub part TP: Td<-40.00C> {{ primary {{ mfr: \"m\", mpn: \"t\", footprint: FP }} }}
fn sense<T: Temperature>() {{ }}
design B {{
    inst t: TP
    sense::<-40C>()
    net _: t.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "part-level + fn-call signed args:\n{r}"
    );
    let ir = chk.ir.as_ref().unwrap();
    let tmin = &ir.instances["B::t"].specs["tmin"];
    assert_eq!(tmin.text, "-40.00C", "part-level spelling preserved");
    assert_eq!(tmin.femto, -40_000_000_000_000_000i128);
}

#[test]
fn wrong_unit_and_other_negative_quantities_still_rejected() {
    // wrong unit kind for the Temperature parameter → E112
    let src = format!(
        "{LIB}
design B {{
    inst t: Td<100nF>
    net _: t.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E112"), "{r}");
    // negative resistance must not be smuggled through the signed path
    let lib2 = r#"
pub device Rq<R: Resistance> { pins { A: 1 [passive] } }
pub footprint FP {}
"#;
    let src2 = format!(
        "{lib2}
design B {{
    inst t: Rq<-1kohm>
    net _: t.A
}}"
    );
    let r2 = check(&src2).1;
    assert!(
        r2.contains("E105") || r2.contains("E1401"),
        "negative unsigned quantity still rejected:\n{r2}"
    );
    // real temperature arithmetic is still not a supported expression domain
    let src3 = format!(
        "{LIB}
design B {{
    inst t: Td<40C + 1C>
    net _: t.A
}}"
    );
    let r3 = check(&src3).1;
    assert!(r3.contains("E1401"), "{r3}");
}

#[test]
fn signed_length_literal_and_expression_path_untouched() {
    let lib = r#"
pub device Pd<L: Length> { pins { A: 1 [passive] } spec { w: L } }
pub footprint FP {}
"#;
    let src = format!(
        "{lib}
design B {{
    inst a: Pd<-1.5mm>
    inst b: Pd<0mm - 1.5mm>
    net _: a.A, b.A
}}"
    );
    let (chk, r) = check(&src);
    assert!(
        !chk.diags.has_errors(),
        "signed Length literal + expression:\n{r}"
    );
    let ir = chk.ir.as_ref().unwrap();
    assert_eq!(ir.instances["B::a"].specs["w"].text, "-1.5mm");
    assert_eq!(
        ir.instances["B::b"].specs["w"].femto,
        -1_500_000_000_000_000i128
    );
}

#[test]
fn negative_length_stays_expression_for_arithmetic() {
    // a leading '-' before a Length literal must NOT be truncated back to a
    // bare literal — full arithmetic expressions remain legal
    let lib = r#"
pub device Pd<L: Length> { pins { A: 1 [passive] } spec { w: L } }
pub footprint FP {}
"#;
    for (expr, femto) in [
        ("-1mm + 2mm", 1_000_000_000_000_000i128),
        ("-1mm * 2", -2_000_000_000_000_000i128),
        ("-1mm / 2", -500_000_000_000_000i128),
    ] {
        let src = format!(
            "{lib}
design B {{
    inst a: Pd<{expr}>
    net _: a.A
}}"
        );
        let (chk, r) = check(&src);
        assert!(
            !chk.diags.has_errors(),
            "{expr} must stay a legal expression:
{r}"
        );
        assert_eq!(
            chk.ir.as_ref().unwrap().instances["B::a"].specs["w"].femto,
            femto,
            "{expr} exact value"
        );
    }
}

#[test]
fn negative_temperature_arithmetic_still_rejected_in_expr() {
    let src = format!(
        "{LIB}
design B {{
    inst t: Td<-40C + 1C>
    net _: t.A
}}"
    );
    let r = check(&src).1;
    assert!(r.contains("E1401"), "{r}");
}

//! RFC-033 Task 5 — end-to-end evaluator coverage via the public API.
//!
//! Expressions are parsed from real source (`design X { const A: Int = … }`)
//! so spans/precedence/signed-literal assembly are the parser's real output,
//! then evaluated through `cohdl::check::eval`.

use std::collections::{BTreeMap, BTreeSet};

use cohdl::ast::{Expr, ItemKind, Stmt};
use cohdl::check::eval::{self, Env, NameKind, Ty, Value};
use cohdl::diag::Diagnostics;
use cohdl::span::SourceMap;

/// Parse `design D { const A: Int = <src> }` and return the const's value.
fn expr_of(src: &str) -> (Expr, String) {
    let full = format!("design D {{\n    const A: Int = {src}\n}}\n");
    let mut sm = SourceMap::new();
    let f = sm.add_file("main.cohdl", &full);
    let mut diags = Diagnostics::new();
    let tokens = cohdl::lex::lex(f, sm.text(f), &mut diags);
    let ast = cohdl::parse::parse(tokens, &mut diags);
    assert!(
        !diags.has_errors(),
        "parse failed for `{src}`:\n{}",
        diags.render(&sm)
    );
    let design = ast
        .items
        .iter()
        .find_map(|i| match &i.kind {
            ItemKind::Design(d) => Some(d),
            _ => None,
        })
        .expect("design item");
    match &design.body[0] {
        Stmt::Const(c) => (c.value.clone(), full),
        other => panic!("expected a const, got {other:?}"),
    }
}

fn ev(src: &str) -> Result<Value, String> {
    let (e, full) = expr_of(src);
    let mut diags = Diagnostics::new();
    let v = eval::eval(&e, &Env::empty(), &mut diags);
    match v {
        Some(v) if !diags.has_errors() => Ok(v),
        _ => {
            let mut sm = SourceMap::new();
            sm.add_file("main.cohdl", &full);
            diags.sort(&sm);
            Err(diags.render(&sm))
        }
    }
}

#[test]
fn precedence_associativity_and_truncation() {
    assert!(matches!(ev("2 + 3 * 4"), Ok(Value::Int(14))));
    assert!(matches!(ev("10 - 4 - 3"), Ok(Value::Int(3))));
    assert!(matches!(ev("7 / 2"), Ok(Value::Int(3))));
    assert!(matches!(ev("-7 / 2"), Ok(Value::Int(-3))));
    assert!(matches!(ev("-7 % 2"), Ok(Value::Int(-1))));
    assert!(matches!(ev("(1 + 2) * 3"), Ok(Value::Int(9))));
}

#[test]
fn int_overflow_and_zero_division() {
    assert!(ev("9223372036854775807 + 1").unwrap_err().contains("E1402"));
    assert!(ev("-9223372036854775808 / -1")
        .unwrap_err()
        .contains("E1402"));
    assert!(ev("1 / 0").unwrap_err().contains("E1403"));
    assert!(ev("1 % 0").unwrap_err().contains("E1403"));
    assert!(ev("-(-9223372036854775808)").unwrap_err().contains("E1402"));
}

#[test]
fn exact_length_arithmetic() {
    assert!(matches!(
        ev("10mm + 2 * 4mm").unwrap(),
        Value::Length(v) if v.femto == 18_000_000_000_000_000
    ));
    assert!(matches!(
        ev("1.00mm + 0mm").unwrap(),
        Value::Length(v) if v.femto == 1_000_000_000_000_000
    ));
    assert_eq!(eval::length_text(1_000_000_000_000_000), "1mm");
    assert_eq!(eval::length_text(-500_000_000_000_000), "-0.5mm");
    assert!(matches!(
        ev("1mm / 8").unwrap(),
        Value::Length(v) if v.femto == 125_000_000_000_000
    ));
    assert!(ev("1mm / 3").unwrap_err().contains("E1402"));
    assert!(ev("10mm + 2").unwrap_err().contains("E1401"));
    assert!(ev("1mm * 1mm").unwrap_err().contains("E1401"));
    assert!(ev("1mm / 0").unwrap_err().contains("E1403"));
}

#[test]
fn non_length_unit_in_expression_position_is_e1401() {
    // Task 3 keeps such nodes so legacy positions keep their own codes; the
    // evaluator judges by the ACTUAL unit type.
    assert!(ev("3V").unwrap_err().contains("E1401"));
    assert!(ev("2 * 100nF").unwrap_err().contains("E1401"));
}

#[test]
fn names_and_len_resolve_through_env() {
    let (e, full) = expr_of("N + 1");
    let names: BTreeMap<String, NameKind> = [("N".to_string(), NameKind::GenericInt(41))]
        .into_iter()
        .collect();
    let lens: BTreeMap<String, i64> = BTreeMap::new();
    let unknown: BTreeSet<String> = BTreeSet::new();
    let env = Env {
        names: &names,
        array_lens: &lens,
        unknown_arrays: &unknown,
    };
    let mut diags = Diagnostics::new();
    assert!(matches!(
        eval::eval(&e, &env, &mut diags),
        Some(Value::Int(42))
    ));
    assert!(!diags.has_errors(), "{}", {
        let mut sm = SourceMap::new();
        sm.add_file("main.cohdl", &full);
        diags.render(&sm)
    });

    let (e, _) = expr_of("arr.len");
    let names: BTreeMap<String, NameKind> = BTreeMap::new();
    let lens: BTreeMap<String, i64> = [("arr".to_string(), 7)].into_iter().collect();
    let env = Env {
        names: &names,
        array_lens: &lens,
        unknown_arrays: &unknown,
    };
    let mut diags = Diagnostics::new();
    assert!(matches!(
        eval::eval(&e, &env, &mut diags),
        Some(Value::Int(7))
    ));
}

#[test]
fn type_check_with_unknowns_and_invariants() {
    // Unknown-typed names type-check without values.
    let (e, _) = expr_of("(X + X) * Y");
    let names: BTreeMap<String, NameKind> = [
        ("X".to_string(), NameKind::Unknown(Ty::Length)),
        ("Y".to_string(), NameKind::Unknown(Ty::Int)),
    ]
    .into_iter()
    .collect();
    let lens: BTreeMap<String, i64> = BTreeMap::new();
    let unknown: BTreeSet<String> = BTreeSet::new();
    let env = Env {
        names: &names,
        array_lens: &lens,
        unknown_arrays: &unknown,
    };
    let mut diags = Diagnostics::new();
    assert!(matches!(
        eval::type_check(&e, &env, &mut diags),
        Some(Ty::Length)
    ));
    assert!(!diags.has_errors());

    // A known zero divisor surfaces even under an unknown-typed multiplier.
    let (e, _) = expr_of("(1 / 0) * X");
    let names: BTreeMap<String, NameKind> = [("X".to_string(), NameKind::Unknown(Ty::Int))]
        .into_iter()
        .collect();
    let env = Env {
        names: &names,
        array_lens: &lens,
        unknown_arrays: &unknown,
    };
    let mut diags = Diagnostics::new();
    let _ = eval::type_check(&e, &env, &mut diags);
    let mut sm = SourceMap::new();
    sm.add_file("main.cohdl", "design D { const A: Int = (1 / 0) * X }");
    diags.sort(&sm);
    assert!(diags.render(&sm).contains("E1403"));
}

#[test]
fn length_value_carries_canonical_text() {
    let v = eval::length_value(2_500_000_000_000_000);
    assert_eq!(v.text, "2.5mm");
    assert_eq!(v.femto, 2_500_000_000_000_000);
}

#[test]
fn length_minimum_is_valid_but_division_overflow_is_checked() {
    let min = "(0.000000000000001mm * -9223372036854775808 * -9223372036854775808 * -2)";
    assert!(matches!(ev(min).unwrap(), Value::Length(v) if v.femto == i128::MIN));
    for (expression, code) in [
        (format!("{min} / -1"), "E1402"),
        ("1mm / 3".into(), "E1402"),
        (format!("{min} / 0"), "E1403"),
    ] {
        let (e, _) = expr_of(&expression);
        let mut diags = Diagnostics::new();
        assert!(eval::eval(&e, &Env::empty(), &mut diags).is_none());
        assert_eq!(diags.iter().map(|d| d.code).collect::<Vec<_>>(), [code]);
    }
}

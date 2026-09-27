//! RFC-033 §2 — the compile-time Int/Length expression evaluator.
//!
//! Two domains, both exact: `Int` is signed 64-bit structural integer;
//! `Length` is femto-mm (10^-15 mm) in `i128`. No rounding anywhere — a
//! division that cannot be represented exactly is an error (E1402), never a
//! silently rounded value. The evaluator is used in three environments
//! (definition validation with unknown values, activation, iteration) — the
//! `Env` carries which names/arrays are visible and which are still unknown.
//!
//! NOTE (RFC-033 Task 3 deviation): an `Expr::Length` node may carry a
//! NON-Length `UnitValue` (the parser keeps the node so legacy positions can
//! report their own precise codes — e.g. `place … at (0mm, 3V)` reports
//! E1007 at check). Both `type_check` and `eval` judge by the `UnitValue`'s
//! ACTUAL unit type; a non-Length unit here is E1401, never a Length.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{expr_text, BinOp, Expr, UnaryOp};
use crate::diag::{Diagnostic, Diagnostics};
use crate::span::Span;
use crate::units::{UnitType, UnitValue};

/// A fully evaluated expression value. `Length` carries the full unit value:
/// exact femto-mm for identity, plus the source spelling when the value is a
/// literal or a pure forward of one (arithmetic composes the canonical text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Length(UnitValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Int,
    Length,
}

/// What a name means during evaluation/type-checking.
#[derive(Debug, Clone)]
pub enum NameKind {
    /// A visible pin, instance, net, or non-expression generic. Kept distinct
    /// from an unknown Int/Length so visibility never invents a value type.
    NonValue,
    Const(Value),
    Binder(i64),
    GenericInt(i64),
    GenericLength(UnitValue),
    Unknown(Ty),
}

pub struct Env<'a> {
    pub names: &'a BTreeMap<String, NameKind>,
    pub array_lens: &'a BTreeMap<String, i64>,
    pub unknown_arrays: &'a BTreeSet<String>,
}

impl<'a> Env<'a> {
    /// The all-unknown environment (definition validation shape): no names,
    /// no arrays.
    pub fn empty() -> Env<'static> {
        static EMPTY_NAMES: std::sync::OnceLock<BTreeMap<String, NameKind>> =
            std::sync::OnceLock::new();
        static EMPTY_LENS: std::sync::OnceLock<BTreeMap<String, i64>> = std::sync::OnceLock::new();
        static EMPTY_UNKNOWN: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
        Env {
            names: EMPTY_NAMES.get_or_init(Default::default),
            array_lens: EMPTY_LENS.get_or_init(Default::default),
            unknown_arrays: EMPTY_UNKNOWN.get_or_init(Default::default),
        }
    }
}

/// One lexical dependency: a typed const, or an array's Int length (`ty: None`).
#[derive(Clone)]
pub(crate) struct LocalExpr {
    pub name: crate::ast::Ident,
    pub value: Expr,
    pub ty: Option<Ty>,
    pub span: Span,
}

/// Resolve a body's dependency graph in either a type-only or bound environment.
/// Unknown inputs remain typed unknowns; known values propagate without
/// constructing any circuit objects or specializing a referenced callee.
pub(crate) fn resolve_locals(
    locals: &[LocalExpr],
    names: &mut BTreeMap<String, NameKind>,
    array_lens: &mut BTreeMap<String, i64>,
    unknown_arrays: &mut BTreeSet<String>,
    failed: &[Span],
    diags: &mut Diagnostics,
) {
    struct Resolver<'a> {
        pending: BTreeMap<String, LocalExpr>,
        failed: &'a [Span],
        names: &'a mut BTreeMap<String, NameKind>,
        lens: &'a mut BTreeMap<String, i64>,
        arrays: &'a mut BTreeSet<String>,
        done: BTreeSet<String>,
        stack: Vec<String>,
        diags: &'a mut Diagnostics,
    }
    fn references(e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Name(id) | Expr::Len(id, _) => out.push(id.name.clone()),
            Expr::Unary { rhs, .. } | Expr::Paren(rhs, _) => references(rhs, out),
            Expr::Binary { lhs, rhs, .. } => {
                references(lhs, out);
                references(rhs, out);
            }
            _ => {}
        }
    }
    impl Resolver<'_> {
        fn resolve(&mut self, name: &str) {
            if self.done.contains(name) {
                return;
            }
            if let Some(pos) = self.stack.iter().position(|n| n == name) {
                let mut cycle = self.stack[pos..].to_vec();
                cycle.push(name.to_string());
                let rendered = cycle
                    .iter()
                    .map(|n| {
                        if self.pending[n].ty.is_none() {
                            format!("`{n}`.len")
                        } else {
                            format!("`{n}`")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" → ");
                self.diags.push(Diagnostic::error(
                    "E1407",
                    self.pending[name].span,
                    format!("cyclic dependency: {rendered}"),
                ));
                self.done.extend(cycle);
                return;
            }
            let local = self.pending[name].clone();
            self.stack.push(name.to_string());
            let mut refs = Vec::new();
            references(&local.value, &mut refs);
            for dependency in refs {
                if self.pending.contains_key(&dependency) {
                    self.resolve(&dependency);
                }
            }
            self.stack.pop();
            if !self.done.insert(name.to_string()) {
                return;
            }
            if expression_failed(local.value.span(), self.failed) {
                return;
            }
            let env = Env {
                names: self.names,
                array_lens: self.lens,
                unknown_arrays: self.arrays,
            };
            let want = local.ty.unwrap_or(Ty::Int);
            let Some(actual) = type_check(&local.value, &env, self.diags) else {
                return;
            };
            if actual != want {
                self.diags.push(Diagnostic::error(
                    "E1401",
                    local.value.span(),
                    format!(
                        "{} `{}` must be {}, but `{}` is {}",
                        if local.ty.is_some() {
                            "const"
                        } else {
                            "array length"
                        },
                        name,
                        ty_name_ty(want),
                        expr_text(&local.value),
                        ty_name_ty(actual),
                    ),
                ));
                return;
            }
            if let Some(value) = eval_if_concrete(&local.value, &env) {
                if local.ty.is_some() {
                    self.names.insert(name.to_string(), NameKind::Const(value));
                } else if let Value::Int(n) = value {
                    if n < 1 {
                        self.diags.push(Diagnostic::error(
                            "E211",
                            local.value.span(),
                            format!("array length `{n}` must be 1 or more"),
                        ));
                    } else {
                        self.lens.insert(name.to_string(), n);
                        self.arrays.remove(name);
                    }
                }
            }
        }
    }
    for local in locals {
        if let Some(ty) = local.ty {
            names.insert(local.name.name.clone(), NameKind::Unknown(ty));
        } else {
            array_lens.remove(&local.name.name);
            unknown_arrays.insert(local.name.name.clone());
        }
    }
    let mut resolver = Resolver {
        failed,
        pending: locals
            .iter()
            .map(|l| (l.name.name.clone(), l.clone()))
            .collect(),
        names,
        lens: array_lens,
        arrays: unknown_arrays,
        done: BTreeSet::new(),
        stack: Vec::new(),
        diags,
    };
    let names: Vec<_> = resolver.pending.keys().cloned().collect();
    for name in names {
        resolver.resolve(&name);
    }
}

/// A failed subexpression prevents evaluating its enclosing expression. These
/// sites belong to one definition or activation, never a global dedup key.
pub(crate) fn expression_failed(span: Span, failed: &[Span]) -> bool {
    failed
        .iter()
        .any(|bad| bad.file == span.file && span.start <= bad.start && bad.end <= span.end)
}

pub(crate) fn expression_failures(diags: &Diagnostics) -> Vec<Span> {
    diags
        .iter()
        // Undefined expression names are invariant failures too. Dynamic
        // index bounds are checked during expansion, outside this validation
        // batch, and must still report independently for each activation.
        .filter(|d| {
            matches!(
                d.code,
                "E1401" | "E1402" | "E1403" | "E1407" | "E211" | "E202"
            )
        })
        .map(|d| d.primary.span)
        .collect()
}

fn ty_name(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "an Int",
        Value::Length(_) => "a Length",
    }
}

fn ty_name_ty(t: Ty) -> &'static str {
    match t {
        Ty::Int => "an Int",
        Ty::Length => "a Length",
    }
}

fn op_sym(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
    }
}

fn overflow(diags: &mut Diagnostics, span: Span, e: &Expr) -> Option<Value> {
    diags.push(Diagnostic::error(
        "E1402",
        span,
        format!("Int overflow in `{}`", expr_text(e)),
    ));
    None
}

fn overflow_len(diags: &mut Diagnostics, span: Span, e: &Expr) -> Option<Value> {
    diags.push(Diagnostic::error(
        "E1402",
        span,
        format!("Length overflow in `{}`", expr_text(e)),
    ));
    None
}

fn div_zero(diags: &mut Diagnostics, span: Span, e: &Expr) -> Option<Value> {
    diags.push(Diagnostic::error(
        "E1403",
        span,
        format!("division by zero in `{}`", expr_text(e)),
    ));
    None
}

fn kind_mismatch(
    diags: &mut Diagnostics,
    span: Span,
    e: &Expr,
    op: BinOp,
    l: &str,
    r: &str,
) -> Option<Value> {
    diags.push(Diagnostic::error(
        "E1401",
        span,
        format!(
            "`{}`: {} {} {} is not a supported operation — Int combines with Int; Length adds/subtracts Length, scales by Int, divides by Int",
            expr_text(e),
            l,
            op_sym(op),
            r
        ),
    ));
    None
}

fn binary(
    op: BinOp,
    l: Value,
    r: Value,
    span: Span,
    e: &Expr,
    diags: &mut Diagnostics,
) -> Option<Value> {
    use Value::*;
    match (op, l, r) {
        (BinOp::Add, Int(a), Int(b)) => a
            .checked_add(b)
            .map(Int)
            .or_else(|| overflow(diags, span, e)),
        (BinOp::Sub, Int(a), Int(b)) => a
            .checked_sub(b)
            .map(Int)
            .or_else(|| overflow(diags, span, e)),
        (BinOp::Mul, Int(a), Int(b)) => a
            .checked_mul(b)
            .map(Int)
            .or_else(|| overflow(diags, span, e)),
        (BinOp::Div, Int(_), Int(0)) | (BinOp::Rem, Int(_), Int(0)) => div_zero(diags, span, e),
        // `i64::MIN / -1` overflows; checked_div reports it as E1402.
        (BinOp::Div, Int(a), Int(b)) => a
            .checked_div(b)
            .map(Int)
            .or_else(|| overflow(diags, span, e)),
        (BinOp::Rem, Int(a), Int(b)) => a
            .checked_rem(b)
            .map(Int)
            .or_else(|| overflow(diags, span, e)),
        // Length arithmetic composes a CANONICAL text: only a literal (or a
        // pure forward of one) keeps its own spelling.
        (BinOp::Add, Length(a), Length(b)) => a
            .femto
            .checked_add(b.femto)
            .map(|f| Length(length_value(f)))
            .or_else(|| overflow_len(diags, span, e)),
        (BinOp::Sub, Length(a), Length(b)) => a
            .femto
            .checked_sub(b.femto)
            .map(|f| Length(length_value(f)))
            .or_else(|| overflow_len(diags, span, e)),
        (BinOp::Mul, Int(a), Length(b)) | (BinOp::Mul, Length(b), Int(a)) => {
            // Int-scaling keeps the femto domain exact.
            b.femto
                .checked_mul(a as i128)
                .map(|f| Length(length_value(f)))
                .or_else(|| overflow_len(diags, span, e))
        }
        (BinOp::Div, Length(_), Int(0)) => div_zero(diags, span, e),
        (BinOp::Div, Length(a), Int(b)) => {
            let b = b as i128;
            let Some(remainder) = a.femto.checked_rem(b) else {
                return overflow_len(diags, span, e);
            };
            if remainder != 0 {
                diags.push(Diagnostic::error(
                    "E1402",
                    span,
                    format!(
                        "`{}` is not exactly representable — Length division must be exact (no rounding)",
                        expr_text(e)
                    ),
                ));
                return None;
            }
            a.femto
                .checked_div(b)
                .map(|v| Length(length_value(v)))
                .or_else(|| overflow_len(diags, span, e))
        }
        (op, lv, rv) => kind_mismatch(diags, span, e, op, ty_name(&lv), ty_name(&rv)),
    }
}

/// The `Value` of an `Expr::Length` node — `None` (E1401) when the node
/// carries a non-Length unit (Task 3 keeps such nodes so legacy positions
/// can report their own codes; the evaluator itself judges by actual unit).
fn length_literal(v: &UnitValue, span: Span, diags: &mut Diagnostics) -> Option<UnitValue> {
    if v.unit == UnitType::Length {
        Some(v.clone())
    } else {
        diags.push(Diagnostic::error(
            "E1401",
            span,
            format!(
                "`{}` is a `{}` — expected an Int or Length expression",
                v.text,
                v.unit.type_name()
            ),
        ));
        None
    }
}

/// Evaluate ONLY when the expression is fully concrete (no unknown names or
/// arrays); `None` otherwise, with no diagnostics. The static type-checker
/// uses this per-binary-node so a known-zero divisor surfaces even under an
/// unknown sibling operand.
pub(crate) fn eval_if_concrete(e: &Expr, env: &Env) -> Option<Value> {
    fn concrete(e: &Expr, env: &Env) -> bool {
        match e {
            Expr::Int(_, _) | Expr::Length(_, _) => true,
            Expr::Paren(inner, _) => concrete(inner, env),
            Expr::Name(id) => match env.names.get(&id.name) {
                Some(NameKind::Const(_))
                | Some(NameKind::Binder(_))
                | Some(NameKind::GenericInt(_))
                | Some(NameKind::GenericLength(_)) => true,
                Some(NameKind::Unknown(_) | NameKind::NonValue) | None => false,
            },
            Expr::Len(id, _) => env.array_lens.contains_key(&id.name),
            Expr::Unary { rhs, .. } => concrete(rhs, env),
            Expr::Binary { lhs, rhs, .. } => concrete(lhs, env) && concrete(rhs, env),
        }
    }
    if concrete(e, env) {
        eval(e, env, &mut Diagnostics::new())
    } else {
        None
    }
}

pub fn eval(e: &Expr, env: &Env, diags: &mut Diagnostics) -> Option<Value> {
    match e {
        Expr::Int(n, _) => Some(Value::Int(*n)),
        // A literal keeps its own spelling — the literal IS the provenance.
        Expr::Length(v, span) => length_literal(v, *span, diags).map(Value::Length),
        Expr::Paren(inner, _) => eval(inner, env, diags),
        Expr::Name(id) => match env.names.get(&id.name) {
            Some(NameKind::Const(v)) => Some(v.clone()),
            Some(NameKind::Binder(i)) | Some(NameKind::GenericInt(i)) => Some(Value::Int(*i)),
            // A forwarded Length keeps the spelling it was bound with.
            Some(NameKind::GenericLength(u)) => {
                length_literal(u, id.span, diags).map(Value::Length)
            }
            // Caller decides (definition validation never calls eval on
            // unknowns — it type-checks instead).
            Some(NameKind::Unknown(_)) => None,
            Some(NameKind::NonValue) => {
                diags.push(Diagnostic::error(
                    "E1401",
                    id.span,
                    format!("`{}` is not a compile-time Int or Length value", id.name),
                ));
                None
            }
            None => {
                diags.push(Diagnostic::error(
                    "E202",
                    id.span,
                    format!(
                        "unknown name `{}` in this expression — expected a const, a loop variable or an Int/Length generic parameter",
                        id.name
                    ),
                ));
                None
            }
        },
        Expr::Len(id, span) => match env.array_lens.get(&id.name) {
            Some(n) => Some(Value::Int(*n)),
            None if env.unknown_arrays.contains(&id.name) => None,
            None => {
                let known = env.names.contains_key(&id.name);
                diags.push(Diagnostic::error(
                    if known { "E1401" } else { "E202" },
                    *span,
                    if known {
                        format!(
                            "`{}` is not an array — `.len` requires a visible array",
                            id.name
                        )
                    } else {
                        format!(
                            "unknown array `{}` — `.len` reads a visible array's declared length",
                            id.name
                        )
                    },
                ));
                None
            }
        },
        Expr::Unary { op, rhs, span } => {
            let v = eval(rhs, env, diags)?;
            match (op, v) {
                (UnaryOp::Plus, v) => Some(v),
                (UnaryOp::Neg, Value::Int(i)) => i
                    .checked_neg()
                    .map(Value::Int)
                    .or_else(|| overflow(diags, *span, e)),
                (UnaryOp::Neg, Value::Length(f)) => f
                    .femto
                    .checked_neg()
                    .map(|x| Value::Length(length_value(x)))
                    .or_else(|| overflow_len(diags, *span, e)),
            }
        }
        Expr::Binary { op, lhs, rhs, span } => {
            let l = eval(lhs, env, diags)?;
            let r = eval(rhs, env, diags)?;
            binary(*op, l, r, *span, e, diags)
        }
    }
}

/// Type-check with possibly-unknown values. Mirrors `binary` over `Ty`:
/// `Int op Int → Int`, `Length ± Length → Length`, `Int * Length → Length`,
/// `Length / Int → Length`, anything else E1401. When both operands are
/// concrete (literals/consts), also runs `binary` so invariant failures
/// (a known-zero divisor) surface even in a body that never activates.
pub fn type_check(e: &Expr, env: &Env, diags: &mut Diagnostics) -> Option<Ty> {
    match e {
        Expr::Int(_, _) => Some(Ty::Int),
        Expr::Length(v, span) => length_literal(v, *span, diags).map(|_| Ty::Length),
        Expr::Paren(inner, _) => type_check(inner, env, diags),
        Expr::Name(id) => match env.names.get(&id.name) {
            Some(NameKind::Const(v)) => Some(match v {
                Value::Int(_) => Ty::Int,
                Value::Length(_) => Ty::Length,
            }),
            Some(NameKind::Binder(_)) | Some(NameKind::GenericInt(_)) => Some(Ty::Int),
            Some(NameKind::GenericLength(u)) => {
                length_literal(u, id.span, diags).map(|_| Ty::Length)
            }
            Some(NameKind::Unknown(t)) => Some(*t),
            Some(NameKind::NonValue) => {
                diags.push(Diagnostic::error(
                    "E1401",
                    id.span,
                    format!("`{}` is not a compile-time Int or Length value", id.name),
                ));
                None
            }
            None => {
                diags.push(Diagnostic::error(
                    "E202",
                    id.span,
                    format!(
                        "unknown name `{}` in this expression — expected a const, a loop variable or an Int/Length generic parameter",
                        id.name
                    ),
                ));
                None
            }
        },
        // `.len` on a known array is an Int; on an unknown array (definition
        // validation) it is still an Int — just without a value.
        Expr::Len(id, span) => {
            if env.array_lens.contains_key(&id.name) || env.unknown_arrays.contains(&id.name) {
                Some(Ty::Int)
            } else {
                let known = env.names.contains_key(&id.name);
                diags.push(Diagnostic::error(
                    if known { "E1401" } else { "E202" },
                    *span,
                    if known {
                        format!(
                            "`{}` is not an array — `.len` requires a visible array",
                            id.name
                        )
                    } else {
                        format!(
                            "unknown array `{}` — `.len` reads a visible array's declared length",
                            id.name
                        )
                    },
                ));
                None
            }
        }
        Expr::Unary { op: _, rhs, span } => {
            // Unary preserves the operand's type; a concrete operand may
            // still overflow under negation (evaluated below).
            let t = type_check(rhs, env, diags)?;
            // Only surface the value-level error when the operand is fully
            // concrete (no unknown names/arrays — definition validation).
            if env.concrete(e, diags).is_some() {
                let _ = eval(e, env, diags);
            }
            let _ = span;
            Some(t)
        }
        Expr::Binary { op, lhs, rhs, span } => {
            let lt = type_check(lhs, env, diags)?;
            let rt = type_check(rhs, env, diags)?;
            let result = match (op, lt, rt) {
                (BinOp::Add, Ty::Int, Ty::Int)
                | (BinOp::Sub, Ty::Int, Ty::Int)
                | (BinOp::Mul, Ty::Int, Ty::Int)
                | (BinOp::Div, Ty::Int, Ty::Int)
                | (BinOp::Rem, Ty::Int, Ty::Int) => Some(Ty::Int),
                (BinOp::Add, Ty::Length, Ty::Length) | (BinOp::Sub, Ty::Length, Ty::Length) => {
                    Some(Ty::Length)
                }
                (BinOp::Mul, Ty::Int, Ty::Length) | (BinOp::Mul, Ty::Length, Ty::Int) => {
                    Some(Ty::Length)
                }
                (BinOp::Div, Ty::Length, Ty::Int) => Some(Ty::Length),
                _ => {
                    diags.push(Diagnostic::error(
                        "E1401",
                        *span,
                        format!(
                            "`{}`: {} {} {} is not a supported operation — Int combines with Int; Length adds/subtracts Length, scales by Int, divides by Int",
                            expr_text(e),
                            ty_name_ty(lt),
                            op_sym(*op),
                            ty_name_ty(rt)
                        ),
                    ));
                    None
                }
            };
            // Subexpression-concrete operands: surface invariant E1402/E1403
            // wherever BOTH operands of this node (recursively) are concrete
            // — and, for division/remainder, wherever the DIVISOR is
            // concretely zero (a known-zero divisor is an invariant failure
            // even under an unknown dividend). Unknown-typed names
            // type-check WITHOUT a value.
            if result.is_some() {
                if let (Some(lv), Some(rv)) =
                    (eval_if_concrete(lhs, env), eval_if_concrete(rhs, env))
                {
                    let _ = binary(*op, lv, rv, *span, e, diags);
                } else if matches!(op, BinOp::Div | BinOp::Rem) {
                    if let Some(Value::Int(0)) = eval_if_concrete(rhs, env) {
                        let _ = binary(*op, Value::Int(0), Value::Int(0), *span, e, diags);
                    }
                }
            }
            result
        }
    }
}

impl Env<'_> {
    /// Whether an expression is fully concrete (no unknown names/arrays).
    fn concrete(&self, e: &Expr, _diags: &mut Diagnostics) -> Option<()> {
        fn walk(e: &Expr, env: &Env) -> Option<()> {
            match e {
                Expr::Int(_, _) | Expr::Length(_, _) => Some(()),
                Expr::Paren(inner, _) => walk(inner, env),
                Expr::Name(id) => match env.names.get(&id.name) {
                    Some(NameKind::Const(_))
                    | Some(NameKind::Binder(_))
                    | Some(NameKind::GenericInt(_))
                    | Some(NameKind::GenericLength(_)) => Some(()),
                    Some(NameKind::Unknown(_) | NameKind::NonValue) | None => None,
                },
                Expr::Len(id, _) => env.array_lens.get(&id.name).map(|_| ()),
                Expr::Unary { rhs, .. } => walk(rhs, env),
                Expr::Binary { lhs, rhs, .. } => walk(lhs, env).and_then(|_| walk(rhs, env)),
            }
        }
        walk(e, self)
    }
}

/// `Value::Length` → canonical text `"<mm_femto>mm"`; a literal keeps its
/// own text (handled by callers).
pub fn length_text(femto: i128) -> String {
    format!("{}mm", crate::emit::geom::mm_femto(femto))
}

/// Length value → `UnitValue` with canonical text (for IR placements and
/// generic arguments).
pub fn length_value(femto: i128) -> UnitValue {
    UnitValue {
        unit: UnitType::Length,
        femto,
        text: length_text(femto),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::SourceMap;

    /// Parse `design D { const A: Int = <src> }` and return the const's
    /// value expression.
    fn e(src: &str) -> Expr {
        let full = format!("design D {{\n    const A: Int = {src}\n}}\n");
        let mut sm = SourceMap::new();
        let f = sm.add_file("t.cohdl", &full);
        let mut diags = Diagnostics::new();
        let tokens = crate::lex::lex(f, sm.text(f), &mut diags);
        let ast = crate::parse::parse(tokens, &mut diags);
        assert!(!diags.has_errors(), "parse failed:\n{}", diags.render(&sm));
        let design = ast
            .items
            .iter()
            .find_map(|i| match &i.kind {
                crate::ast::ItemKind::Design(d) => Some(d),
                _ => None,
            })
            .expect("design item");
        match &design.body[0] {
            crate::ast::Stmt::Const(c) => c.value.clone(),
            other => panic!("expected a const, got {other:?}"),
        }
    }

    fn ev(src: &str) -> Result<Value, String> {
        let full = format!("design D {{\n    const A: Int = {src}\n}}\n");
        let expr = {
            let mut sm = SourceMap::new();
            let f = sm.add_file("t.cohdl", full.clone());
            let mut diags = Diagnostics::new();
            let tokens = crate::lex::lex(f, sm.text(f), &mut diags);
            let ast = crate::parse::parse(tokens, &mut diags);
            assert!(!diags.has_errors(), "parse failed:\n{}", diags.render(&sm));
            let design = ast
                .items
                .iter()
                .find_map(|i| match &i.kind {
                    crate::ast::ItemKind::Design(d) => Some(d),
                    _ => None,
                })
                .expect("design item");
            match &design.body[0] {
                crate::ast::Stmt::Const(c) => c.value.clone(),
                other => panic!("expected a const, got {other:?}"),
            }
        };
        let mut diags = Diagnostics::new();
        let v = eval(&expr, &Env::empty(), &mut diags);
        match v {
            Some(v) if !diags.has_errors() => Ok(v),
            _ => {
                let mut sm = SourceMap::new();
                sm.add_file("t.cohdl", full);
                diags.sort(&sm);
                Err(diags.render(&sm))
            }
        }
    }

    #[test]
    fn precedence_and_assoc() {
        assert!(matches!(ev("2 + 3 * 4"), Ok(Value::Int(14))));
        assert!(matches!(ev("10 - 4 - 3"), Ok(Value::Int(3))));
        assert!(matches!(ev("7 / 2"), Ok(Value::Int(3))));
        assert!(matches!(ev("-7 / 2"), Ok(Value::Int(-3))));
        assert!(matches!(ev("-7 % 2"), Ok(Value::Int(-1))));
        assert!(matches!(ev("(1 + 2) * 3"), Ok(Value::Int(9))));
    }

    #[test]
    fn int_domain_errors() {
        assert!(ev("9223372036854775807 + 1").unwrap_err().contains("E1402"));
        assert!(ev("-9223372036854775808 / -1")
            .unwrap_err()
            .contains("E1402"));
        assert!(ev("1 / 0").unwrap_err().contains("E1403"));
        assert!(ev("1 % 0").unwrap_err().contains("E1403"));
        assert!(ev("-(-9223372036854775808)").unwrap_err().contains("E1402"));
    }

    #[test]
    fn length_domain() {
        assert!(matches!(
            ev("10mm + 2 * 4mm").unwrap(),
            Value::Length(v) if v.femto == 18_000_000_000_000_000
        ));
        assert!(matches!(
            ev("1.00mm + 0mm").unwrap(),
            Value::Length(v) if v.femto == 1_000_000_000_000_000
        ));
        assert_eq!(length_text(1_000_000_000_000_000), "1mm");
        assert_eq!(length_text(-500_000_000_000_000), "-0.5mm");
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
    fn non_length_unit_literal_is_e1401() {
        // Task 3 deviation: an Expr::Length node may carry a non-Length unit;
        // the evaluator judges by ACTUAL unit type.
        assert!(ev("3V").unwrap_err().contains("E1401"));
        assert!(ev("3V + 1mm").unwrap_err().contains("E1401"));
    }

    #[test]
    fn unknown_names_and_arrays() {
        let expr = e("MYSTERY + 1");
        let mut diags = Diagnostics::new();
        let v = eval(&expr, &Env::empty(), &mut diags);
        assert!(v.is_none());
        let mut sm = SourceMap::new();
        sm.add_file("t.cohdl", "design D { const A: Int = MYSTERY + 1 }");
        diags.sort(&sm);
        let r = diags.render(&sm);
        assert!(r.contains("E202") && r.contains("MYSTERY"), "{r}");

        let expr = e("arr.len");
        let mut diags = Diagnostics::new();
        let v = eval(&expr, &Env::empty(), &mut diags);
        assert!(v.is_none());
        let mut sm = SourceMap::new();
        sm.add_file("t.cohdl", "design D {\n    const A: Int = arr.len\n}");
        diags.sort(&sm);
        let r = diags.render(&sm);
        assert!(r.contains("E202") && r.contains("unknown array"), "{r}");
    }

    #[test]
    fn type_check_unknown_names_type_without_values() {
        let expr = e("(X + X) * Y");
        let names: BTreeMap<String, NameKind> = [
            ("X".to_string(), NameKind::Unknown(Ty::Length)),
            ("Y".to_string(), NameKind::Unknown(Ty::Int)),
        ]
        .into_iter()
        .collect();
        let unknown: BTreeSet<String> = BTreeSet::new();
        let lens: BTreeMap<String, i64> = BTreeMap::new();
        let env = Env {
            names: &names,
            array_lens: &lens,
            unknown_arrays: &unknown,
        };
        let mut diags = Diagnostics::new();
        let t = type_check(&expr, &env, &mut diags);
        let mut sm2 = crate::span::SourceMap::new();
        sm2.add_file("t.cohdl", "design D {\n    const A: Int = (X + X) * Y\n}\n");
        let rr = diags.render(&sm2);
        assert!(
            !diags.has_errors(),
            "unknown-typed names must type-check:\n{rr}"
        );
        assert!(matches!(t, Some(Ty::Length)));
    }

    #[test]
    fn type_check_surfaces_known_zero_divisor() {
        let expr = e("(1 / 0) * X");
        let names: BTreeMap<String, NameKind> = [("X".to_string(), NameKind::Unknown(Ty::Int))]
            .into_iter()
            .collect();
        let unknown: BTreeSet<String> = BTreeSet::new();
        let lens: BTreeMap<String, i64> = BTreeMap::new();
        let env = Env {
            names: &names,
            array_lens: &lens,
            unknown_arrays: &unknown,
        };
        let mut diags = Diagnostics::new();
        let _ = type_check(&expr, &env, &mut diags);
        let mut sm = SourceMap::new();
        sm.add_file("t.cohdl", "design D {\n    const A: Int = (1 / 0) * X\n}");
        diags.sort(&sm);
        let r = diags.render(&sm);
        assert!(
            r.contains("E1403"),
            "a known zero divisor must surface:\n{r}"
        );
    }

    #[test]
    fn length_value_helpers() {
        let v = length_value(-500_000_000_000_000);
        assert_eq!(v.text, "-0.5mm");
        assert_eq!(v.unit, UnitType::Length);
        assert_eq!(v.femto, -500_000_000_000_000);
    }
}

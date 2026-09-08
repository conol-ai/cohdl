//! RFC-032 declaration-time validation of every `subdesign`, independent of
//! whether a design ever uses it — the same discipline `bodies` applies to
//! uncalled fns.
//!
//! Two jobs:
//!
//! 1. **Structural body checking**, reusing `bodies`' statement machinery
//!    with ports as `Pin`-shaped reference bases, so an unused subdesign
//!    cannot hide an unknown device, a bad pin reference, or a nested use
//!    site of the wrong kind.
//! 2. **Containment-cycle detection** (E1304), naming the FULL cycle —
//!    mirroring RFC-006's cyclic-`fn`-call diagnostic exactly. Checked here,
//!    at declaration, so the cycle is reported even when no design
//!    instantiates it (expansion carries its own guard for reachable ones).

use crate::ast::{Stmt, SubdesignDef};
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::{short, World};
use std::collections::BTreeMap;

pub fn check_subdesigns(world: &World, diags: &mut Diagnostics) {
    for (fq, s) in &world.subdesigns {
        check_one(world, fq, s, diags);
    }
    check_containment_cycles(world, diags);
}

fn check_one(world: &World, fq: &str, s: &SubdesignDef, diags: &mut Diagnostics) {
    // Nested use sites must name real subdesigns; inst/net/nc/call/layout
    // statements reuse `bodies`' machinery below via the fn-shaped shim.
    for stmt in &s.body {
        if let Stmt::SubdesignUse(sub) = stmt {
            let ty = &sub.ty.name;
            if world.subdesigns.contains_key(&ty.name) {
                continue;
            }
            if let Some(sym) = world.symbols.get(&ty.name) {
                diags.push(Diagnostic::error(
                    "E205",
                    ty.span,
                    format!(
                        "`{}` is a {} — a `subdesign` use site requires a subdesign",
                        ty.name, sym.kind
                    ),
                ));
            }
            // Unresolved names were reported at the rewrite pass (E202).
        }
    }
    // Ports behave exactly like `Pin` parameters for reference checking, and
    // the generics carry over unchanged — `bodies` already knows how to check
    // that shape, so a subdesign body is checked AS IF it were a fn whose
    // parameters are its ports (RFC-032: ports reuse RFC-002 pin semantics).
    crate::check::bodies::check_subdesign_body(world, fq, s, diags);
}

/// E1304: direct or indirect recursive containment, reporting each cycle
/// once, at the first (deterministically smallest-named) participant, naming
/// the full cycle.
fn check_containment_cycles(world: &World, diags: &mut Diagnostics) {
    // fq → the fq subdesigns its body's use sites reference (with a span for
    // the report).
    let mut edges: BTreeMap<&str, Vec<(&str, crate::span::Span)>> = BTreeMap::new();
    for (fq, s) in &world.subdesigns {
        let mut out = Vec::new();
        for stmt in &s.body {
            if let Stmt::SubdesignUse(sub) = stmt {
                if world.subdesigns.contains_key(&sub.ty.name.name) {
                    out.push((sub.ty.name.name.as_str(), sub.span));
                }
            }
        }
        edges.insert(fq.as_str(), out);
    }
    // Three-color DFS: every back edge is one cycle, reported once, with the
    // full member chain. Deterministic: starts and edges iterate in BTreeMap
    // / declaration order.
    let mut state: BTreeMap<&str, u8> = BTreeMap::new(); // 1 = in stack, 2 = done
    let mut path: Vec<&str> = Vec::new();
    let starts: Vec<&str> = edges.keys().copied().collect();
    for start in starts {
        if !state.contains_key(start) {
            visit(start, &edges, &mut state, &mut path, diags);
        }
    }
}

fn visit<'a>(
    node: &'a str,
    edges: &BTreeMap<&'a str, Vec<(&'a str, crate::span::Span)>>,
    state: &mut BTreeMap<&'a str, u8>,
    path: &mut Vec<&'a str>,
    diags: &mut Diagnostics,
) {
    state.insert(node, 1);
    path.push(node);
    for (next, span) in &edges[node] {
        match state.get(next) {
            Some(1) => {
                let pos = path.iter().position(|n| n == next).unwrap();
                let mut cycle: Vec<&str> = path[pos..].to_vec();
                cycle.push(next);
                let display = cycle
                    .iter()
                    .map(|n| format!("`{}`", short(n)))
                    .collect::<Vec<_>>()
                    .join(" → ");
                diags.push(Diagnostic::error(
                    "E1304",
                    *span,
                    format!("recursive subdesign containment: {}", display),
                ));
            }
            Some(2) => {}
            _ => visit(next, edges, state, path, diags),
        }
    }
    path.pop();
    state.insert(node, 2);
}

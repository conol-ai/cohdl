//! RFC-005: collision-free designator allocation + `design.lock`.
//!
//! The allocator is a pure, total function over immutable inputs — never a
//! stateful "scan up from 1 until free" loop. Per prefix, the full reserved-
//! number set (prior assignments ∪ tombstones ∪ overrides) is computed once,
//! before any fresh number is chosen; fresh instances (sorted by hierarchical
//! path) then take the sorted sequence of missing positive integers. The
//! result is asserted injective as an explicit postcondition on every run.
//!
//! `design.lock` is TOML with two tables, written byte-stably:
//!
//! ```toml
//! [designators]
//! "Board::c1" = "C1"
//!
//! [tombstones]
//! "Board::old_cap" = "C2"
//! ```

use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LockState {
    pub designators: BTreeMap<String, String>,
    pub tombstones: BTreeMap<String, String>,
}

impl LockState {
    /// Parse the two-table TOML subset used by `design.lock`.
    pub fn parse(text: &str) -> Result<LockState, String> {
        let mut state = LockState::default();
        let mut section = String::new();
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_string();
                if section != "designators" && section != "tombstones" {
                    return Err(format!(
                        "design.lock line {}: unknown table `[{}]`",
                        lineno + 1,
                        section
                    ));
                }
                continue;
            }
            let Some((key_raw, value_raw)) = line.split_once('=') else {
                return Err(format!(
                    "design.lock line {}: expected `\"path\" = \"designator\"`",
                    lineno + 1
                ));
            };
            let key = unquote(key_raw.trim())
                .ok_or_else(|| format!("design.lock line {}: malformed key", lineno + 1))?;
            let value = unquote(value_raw.trim())
                .ok_or_else(|| format!("design.lock line {}: malformed value", lineno + 1))?;
            let table = match section.as_str() {
                "designators" => &mut state.designators,
                "tombstones" => &mut state.tombstones,
                _ => {
                    return Err(format!(
                        "design.lock line {}: entry outside a table",
                        lineno + 1
                    ))
                }
            };
            table.insert(key, value);
        }
        Ok(state)
    }

    /// Byte-stable canonical rendering.
    pub fn render(&self) -> String {
        let mut out = String::from("[designators]\n");
        for (path, d) in &self.designators {
            out.push_str(&format!("\"{}\" = \"{}\"\n", path, d));
        }
        out.push_str("\n[tombstones]\n");
        for (path, d) in &self.tombstones {
            out.push_str(&format!("\"{}\" = \"{}\"\n", path, d));
        }
        out
    }
}

fn unquote(s: &str) -> Option<String> {
    let inner = s.strip_prefix('"')?.strip_suffix('"')?;
    // Lock keys/values never contain quotes or escapes.
    if inner.contains('"') || inner.contains('\\') {
        return None;
    }
    Some(inner.to_string())
}

/// Split a designator into (prefix, number): `C17` → ("C", 17).
fn split_designator(d: &str) -> Option<(&str, u64)> {
    let prefix_len = d.chars().take_while(|c| c.is_ascii_uppercase()).count();
    if prefix_len == 0 || prefix_len == d.len() {
        return None;
    }
    let num: u64 = d[prefix_len..].parse().ok()?;
    Some((&d[..prefix_len], num))
}

/// Assign designators to every instance in `ir` (RFC-005's four steps),
/// returning the new lock state to persist. Overrides that collide report
/// E803; the checked injectivity postcondition is a hard assertion — its
/// failure is a compiler bug, never a shippable netlist.
pub fn assign_designators(
    world: &World,
    ir: &mut DesignIr,
    prior: &LockState,
    diags: &mut Diagnostics,
) -> LockState {
    // ---- Step 1: partition live instances (total, disjoint by construction).
    let mut kept: BTreeMap<String, String> = BTreeMap::new(); // path → designator
    let mut overridden: BTreeMap<String, (String, crate::span::Span)> = BTreeMap::new();
    let mut fresh: Vec<String> = Vec::new(); // paths

    for (path, inst) in &ir.instances {
        if let Some((d, span)) = &inst.designator_override {
            overridden.insert(path.clone(), (d.clone(), *span));
        } else if let Some(d) = prior.designators.get(path) {
            kept.insert(path.clone(), d.clone());
        } else {
            fresh.push(path.clone());
        }
    }

    // ---- Step 2: validate overrides (pure conflict detection).
    let mut claimed: BTreeMap<&str, &str> = BTreeMap::new(); // designator → path
    for (path, d) in &kept {
        claimed.insert(d.as_str(), path.as_str());
    }
    let mut override_ok: BTreeMap<String, String> = BTreeMap::new();
    for (path, (d, span)) in &overridden {
        if let Some(other) = claimed.get(d.as_str()) {
            diags.push(
                Diagnostic::error(
                    "E803",
                    *span,
                    format!(
                        "designator override `{}` on `{}` collides with the assignment of `{}`",
                        d, path, other
                    ),
                )
                .with_help(
                    "pick a different designator, or remove the stale entry from design.lock",
                ),
            );
            // Poison: fall back to fresh assignment so compilation can
            // continue reporting further errors deterministically.
            fresh.push(path.clone());
            continue;
        }
        claimed.insert(d.as_str(), path.as_str());
        override_ok.insert(path.clone(), d.clone());
    }
    fresh.sort();

    // ---- Step 3: fresh assignment via a total, injective numbering function.
    // Newly-tombstoned paths: in prior lock, not live now.
    let mut new_tombstones = prior.tombstones.clone();
    for (path, d) in &prior.designators {
        if !ir.instances.contains_key(path) {
            new_tombstones.insert(path.clone(), d.clone());
        }
    }

    // Reserved numbers per prefix, computed ONCE, immutably, before any fresh
    // assignment: kept ∪ overrides ∪ all tombstones.
    let mut reserved: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for d in kept
        .values()
        .chain(override_ok.values())
        .chain(new_tombstones.values())
    {
        if let Some((prefix, num)) = split_designator(d) {
            reserved.entry(prefix.to_string()).or_default().insert(num);
        }
    }

    // Group fresh instances by prefix, sorted by hierarchical path.
    let mut fresh_by_prefix: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in &fresh {
        let device = &ir.instances[path].device;
        let prefix = world.designator_prefix(device);
        fresh_by_prefix
            .entry(prefix)
            .or_default()
            .push(path.clone());
    }

    let mut assigned: BTreeMap<String, String> = BTreeMap::new();
    for (prefix, paths) in &fresh_by_prefix {
        let reserved_nums = reserved.get(prefix).cloned().unwrap_or_default();
        // The Nth fresh instance gets the Nth positive integer missing from
        // the reserved set — positions in one sorted sequence, not repeated
        // searches against mutating state.
        let mut missing = (1u64..).filter(|n| !reserved_nums.contains(n));
        for path in paths {
            let num = missing.next().expect("integers are unbounded");
            assigned.insert(path.clone(), format!("{}{}", prefix, num));
        }
    }

    // ---- Final map + Step 4: checked injectivity postcondition.
    let mut result: BTreeMap<String, String> = BTreeMap::new();
    for (path, d) in kept.iter().chain(override_ok.iter()).chain(assigned.iter()) {
        result.insert(path.clone(), d.clone());
    }
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for (path, d) in &result {
        if let Some(prev) = seen.insert(d.as_str(), path.as_str()) {
            // Overrides colliding with overrides/kept are already E803 above;
            // reaching here means the allocator itself broke its invariant.
            panic!(
                "COMPILER BUG (RFC-005 injectivity postcondition): designator `{}` assigned to both `{}` and `{}`",
                d, prev, path
            );
        }
    }

    for (path, inst) in ir.instances.iter_mut() {
        if let Some(d) = result.get(path) {
            inst.designator = Some(d.clone());
        }
    }

    LockState {
        designators: result,
        tombstones: new_tombstones,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_roundtrip() {
        let text = "[designators]\n\"Board::c1\" = \"C1\"\n\"Board::mcu\" = \"U1\"\n\n[tombstones]\n\"Board::old\" = \"C2\"\n";
        let state = LockState::parse(text).unwrap();
        assert_eq!(state.designators.len(), 2);
        assert_eq!(state.tombstones["Board::old"], "C2");
        assert_eq!(state.render(), text);
    }

    #[test]
    fn split() {
        assert_eq!(split_designator("C17"), Some(("C", 17)));
        assert_eq!(split_designator("MK2"), Some(("MK", 2)));
        assert_eq!(split_designator("C"), None);
        assert_eq!(split_designator("17"), None);
    }
}

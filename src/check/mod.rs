//! The type checker: the "resolves + type-checks" rungs of the verdict ladder.

pub mod bodies;
pub mod expand;
pub mod footprints;
pub mod generics;
pub mod impls;

use crate::ast::SourceFile;
use crate::diag::Diagnostics;
use crate::ir::DesignIr;
use crate::resolve::{build_world, World};

/// The declaration-level checks that don't need a design and share the
/// build-only RFC-018 boundary: impl satisfaction (RFC-003) and part
/// validation. Both entry points route through this ONE implementation
/// (review R6-5 — the two APIs must not diverge). NOTE: RFC-018 pad/device
/// consistency is NOT here — the RFC pins it to `cohdl build`, so it runs in
/// `pipeline::build_artifacts` (declaration-complete: it walks `world.parts`,
/// not the instantiated IR).
fn run_declaration_checks(world: &mut World, diags: &mut Diagnostics) {
    impls::check_impls(world, diags);
    generics::check_parts(world, diags);
    // Semantically validate every function body, called or not (R6-3).
    bodies::check_fn_bodies(world, diags);
}

/// Build the world and run every declaration-level check (everything that
/// doesn't need a design): resolution, impl satisfaction (RFC-003), part
/// validation.
pub fn check_declarations(files: Vec<SourceFile>, diags: &mut Diagnostics) -> World {
    let mut world = build_world(files, diags);
    run_declaration_checks(&mut world, diags);
    world
}

/// RFC-016 module-aware variant: `modules[i]` names file i's package and
/// module path.
pub fn check_declarations_in(
    files: Vec<SourceFile>,
    modules: &[crate::resolve::ModuleInfo],
    diags: &mut Diagnostics,
) -> World {
    let mut world = crate::resolve::build_world_in(files, modules, diags);
    run_declaration_checks(&mut world, diags);
    world
}

/// Expand and check one design end-to-end (RFC-006 expansion, RFC-002
/// exhaustiveness, net merging).
pub fn check_design(world: &World, design_name: &str, diags: &mut Diagnostics) -> Option<DesignIr> {
    let design = world.designs.get(design_name)?;
    Some(expand::expand_design(world, design, diags))
}

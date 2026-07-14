//! The type checker: the "resolves + type-checks" rungs of the verdict ladder.

pub mod expand;
pub mod generics;
pub mod impls;

use crate::ast::SourceFile;
use crate::diag::Diagnostics;
use crate::ir::DesignIr;
use crate::resolve::{build_world, World};

/// Build the world and run every declaration-level check (everything that
/// doesn't need a design): resolution, impl satisfaction (RFC-003), part
/// validation.
pub fn check_declarations(files: Vec<SourceFile>, diags: &mut Diagnostics) -> World {
    let mut world = build_world(files, diags);
    impls::check_impls(&mut world, diags);
    generics::check_parts(&world, diags);
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
    impls::check_impls(&mut world, diags);
    generics::check_parts(&world, diags);
    world
}

/// Expand and check one design end-to-end (RFC-006 expansion, RFC-002
/// exhaustiveness, net merging).
pub fn check_design(world: &World, design_name: &str, diags: &mut Diagnostics) -> Option<DesignIr> {
    let design = world.designs.get(design_name)?;
    Some(expand::expand_design(world, design, diags))
}

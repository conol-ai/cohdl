//! The type checker: the "resolves + type-checks" rungs of the verdict ladder.

pub mod expand;
pub mod footprints;
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
    // RFC-018 pad/device consistency is a pure declaration check (both the
    // footprint's pad list and the part's device are static), so it runs
    // here — over every declared part, not only instantiated ones (R5-4).
    footprints::check_pad_consistency(&world, diags);
    world
}

/// Expand and check one design end-to-end (RFC-006 expansion, RFC-002
/// exhaustiveness, net merging).
pub fn check_design(world: &World, design_name: &str, diags: &mut Diagnostics) -> Option<DesignIr> {
    let design = world.designs.get(design_name)?;
    Some(expand::expand_design(world, design, diags))
}

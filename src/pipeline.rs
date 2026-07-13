//! The full compilation pipeline, shared by the CLI and the fixture tests.
//!
//! Verdict ladder: parses ⊂ resolves ⊂ type-checks ⊂ connects ⊂ passes
//! residual DRC ⊂ emits netlist. `check` runs everything up to and including
//! residual DRC; `build` additionally assigns designators, binds parts, and
//! emits the netlist + BOM.

use crate::diag::Diagnostics;
use crate::ir::DesignIr;
use crate::lock::LockState;
use crate::resolve::World;
use crate::span::SourceMap;

pub struct Checked {
    pub sm: SourceMap,
    pub diags: Diagnostics,
    pub world: World,
    pub ir: Option<DesignIr>,
    /// The design that was compiled (None if selection failed).
    pub design_name: Option<String>,
}

/// Parse + resolve + type-check + expand + residual DRC.
///
/// `design` selection: explicit override > manifest top > the only design in
/// the project. Selection failures are returned as `Err(message)` — they are
/// project-level, not source-level, so they carry no span.
pub fn check_files(files: &[(String, String)], design: Option<&str>) -> Result<Checked, String> {
    let mut sm = SourceMap::new();
    let mut diags = Diagnostics::new();
    let mut parsed = Vec::new();
    for (name, content) in files {
        let file_id = sm.add_file(name.clone(), content.clone());
        let tokens = crate::lex::lex(file_id, sm.text(file_id), &mut diags);
        parsed.push(crate::parse::parse(tokens, &mut diags));
    }
    let world = crate::check::check_declarations(parsed, &mut diags);

    let design_name = match design {
        Some(d) => {
            if !world.designs.contains_key(d) {
                let available: Vec<&String> = world.designs.keys().collect();
                return Err(format!(
                    "no design named `{}` (available: {})",
                    d,
                    if available.is_empty() {
                        "none".to_string()
                    } else {
                        available
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ));
            }
            Some(d.to_string())
        }
        None => match world.designs.len() {
            0 => None, // declaration-only project: still checkable
            1 => Some(world.designs.keys().next().unwrap().clone()),
            _ => {
                return Err(format!(
                "project has {} designs ({}) — pass --design or set `[design] top` in cohdl.toml",
                world.designs.len(),
                world.designs.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
            }
        },
    };

    let ir = design_name
        .as_deref()
        .and_then(|d| crate::check::check_design(&world, d, &mut diags));

    if let Some(ir) = &ir {
        crate::drc::run_drc(&world, ir, &mut diags);
    }

    diags.sort(&sm);
    Ok(Checked {
        sm,
        diags,
        world,
        ir,
        design_name,
    })
}

pub struct BuildArtifacts {
    pub netlist: String,
    pub bom: String,
    pub lock: LockState,
}

/// The `build` half: designators (RFC-005), part binding, emitters.
/// Only call when `checked.diags` has no errors and `checked.ir` is Some.
pub fn build_artifacts(checked: &mut Checked, prior_lock: &LockState) -> Option<BuildArtifacts> {
    let ir = checked.ir.as_mut()?;
    let mut diags = Diagnostics::new();
    let lock = crate::lock::assign_designators(&checked.world, ir, prior_lock, &mut diags);
    crate::emit::bind_parts(&checked.world, ir, &mut diags);
    let failed = diags.has_errors();
    diags.sort(&checked.sm);
    checked.diags.extend(diags);
    if failed {
        return None;
    }
    let ir = checked.ir.as_ref().unwrap();
    Some(BuildArtifacts {
        netlist: crate::emit::kicad::emit_kicad_net(&checked.world, ir),
        bom: crate::emit::bom::emit_bom_csv(&checked.world, ir),
        lock,
    })
}

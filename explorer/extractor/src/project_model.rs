//! Load a CoHDL project (read-only), run the pipeline, project ExplorerModel.

use crate::model::*;
use cohdl::ast::PinRole;
use cohdl::diag::Severity;
use cohdl::resolve::World;
use cohdl::span::{SourceMap, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Rail threshold (spec R2): nets with more members than this render as
/// stubs even without a voltage/gnd annotation.
const RAIL_FANOUT: usize = 5;

pub fn extract(dir: &Path) -> Result<ExplorerModel, String> {
    let (manifest_path, manifest) = cohdl::project::peek_manifest(dir)?;
    let manifest_display = manifest_path.display().to_string();

    // RFC-029 dependency resolution, mirroring the CLI (read-only: the lock
    // file is verified but never rewritten here).
    let deps_raw = manifest.deps_raw.clone().unwrap_or_default();
    let entries = cohdl::deps::validate_deps(&manifest_display, &deps_raw)
        .map_err(|d| render_pkg_diags(&d))?;
    let registry = cohdl::deps::Registry {
        lib_root: find_lib_root_near(dir),
        project_deps: dir.join("deps"),
        cache_root: cohdl::registry::cache_root(),
    };
    let lock_path = dir.join("cohdl.lock");
    let prior_lock = std::fs::read_to_string(&lock_path).ok();
    let resolution = cohdl::deps::resolve(
        &manifest_display,
        &lock_path.display().to_string(),
        &entries,
        &registry,
        prior_lock.as_deref(),
        cohdl::deps::Update::No,
        // Offline like check/LSP: no fetch (E1102 says "run `cohdl install`"),
        // and no std override path here, so nothing is skipped transitively.
        cohdl::deps::ResolveOpts {
            skip_transitive: &[],
            fetch: None,
        },
    )
    .map_err(|d| render_pkg_diags(&d))?;

    let proj = cohdl::project::load_project_with_deps(dir, &resolution.deps)?;
    // namespace root (sanitized package name) -> package dir, for doc paths
    let mut pkg_dirs: BTreeMap<String, std::path::PathBuf> = BTreeMap::new();
    for (name, d) in &resolution.deps {
        pkg_dirs.insert(cohdl::pipeline::package_root(name), d.clone());
    }
    pkg_dirs.insert(cohdl::pipeline::package_root(&proj.name), dir.to_path_buf());
    let dep_names: Vec<String> = resolution.deps.iter().map(|(n, _)| n.clone()).collect();
    let checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )?;

    if let Some(sel) = &checked.selection_error {
        return Err(format!("design selection failed: {sel}"));
    }
    let mut checked = checked;
    let world = &checked.world;
    let sm = &checked.sm;
    let mut ir = checked
        .ir
        .take()
        .ok_or("no design IR (check did not reach expansion; run `cohdl check` for details)")?;

    // Designator assignment (RFC-005), read-only against the project's
    // design.lock: prior state is honoured but never rewritten here.
    let prior_text = std::fs::read_to_string(dir.join("design.lock")).unwrap_or_default();
    let prior = cohdl::lock::LockState::parse(&prior_text)
        .or_else(|_| cohdl::lock::LockState::parse(""))
        .map_err(|e| format!("design.lock parse: {e}"))?;
    cohdl::lock::assign_designators(world, &mut ir, &prior, &mut checked.diags);
    let ir = &ir;

    // Connectivity index: (instance path, logical pin) -> connected.
    let mut connected: BTreeSet<(String, String)> = BTreeSet::new();
    for net in &ir.nets {
        for (p, l) in &net.members {
            connected.insert((p.clone(), l.clone()));
        }
    }

    let mut instances = Vec::new();
    for (path, inst) in &ir.instances {
        let device = world
            .devices
            .get(&inst.device)
            .ok_or_else(|| format!("device `{}` missing from world", inst.device))?;
        let pins = device
            .pins_for(inst.variant.as_deref())
            .iter()
            .map(|p| Pin {
                logical: p.name.name.clone(),
                numbers: p.numbers.iter().map(|n| n.text.clone()).collect(),
                role: role_name(p.role_or_default()).to_string(),
                obligation: p.obligation.keyword().to_string(),
                connected: connected.contains(&(path.clone(), p.name.name.clone())),
                nc: ir.nc_pins.contains(&(path.clone(), p.name.name.clone())),
            })
            .collect();

        let part = inst.part.as_ref().map(|fq| {
            let (mut mfr, mut mpn, mut footprint) = (None, None, None);
            if let Some(pd) = world.parts.get(fq) {
                for f in &pd.primary.fields {
                    match f.name.name.as_str() {
                        "mfr" => mfr = Some(f.value.clone()),
                        "mpn" => mpn = Some(f.value.clone()),
                        _ => {}
                    }
                }
                footprint = pd.primary.footprint.as_ref().map(|i| i.name.clone());
            }
            PartRef {
                fq: fq.clone(),
                mfr,
                mpn,
                footprint,
            }
        });

        // #[doc] references: device-level plus part-level, source order,
        // resolved to absolute paths inside the owning package dir.
        let mut docs: Vec<DocRef> = Vec::new();
        for key in [Some(&inst.device), inst.part.as_ref()]
            .into_iter()
            .flatten()
        {
            let ns = key.split("::").next().unwrap_or("");
            if let Some(ds) = world.docs.get(key) {
                for d in ds {
                    if docs.iter().any(|x| x.name == *d) {
                        continue;
                    }
                    let abs = pkg_dirs
                        .get(ns)
                        .map(|root| root.join(d).to_string_lossy().to_string())
                        .unwrap_or_default();
                    docs.push(DocRef {
                        name: d.clone(),
                        abs,
                    });
                }
            }
        }

        instances.push(Instance {
            path: path.clone(),
            device_fq: inst.device.clone(),
            variant: inst.variant.clone(),
            designator: inst
                .designator
                .clone()
                .or_else(|| inst.designator_override.as_ref().map(|(d, _)| d.clone())),
            part,
            impl_traits: inst.impl_traits.iter().cloned().collect(),
            placement_hint: inst.placement_hint.clone(),
            specs: inst
                .specs
                .iter()
                .map(|(k, v)| SpecEntry {
                    name: k.clone(),
                    value: v.to_string(),
                })
                .collect(),
            span: src_span(sm, inst.span),
            docs,
            pins,
        });
    }

    // Net member physical expansion mirrors the KiCad emitter's lookup.
    let pin_numbers: BTreeMap<(String, String), Vec<String>> = ir
        .instances
        .iter()
        .flat_map(|(path, inst)| {
            let dev = world.devices.get(&inst.device);
            let pins: Vec<_> = dev
                .map(|d| d.pins_for(inst.variant.as_deref()).to_vec())
                .unwrap_or_default();
            pins.into_iter().map(move |p| {
                (
                    (path.clone(), p.name.name.clone()),
                    p.numbers.iter().map(|n| n.text.clone()).collect(),
                )
            })
        })
        .collect();

    let nets: Vec<Net> = ir
        .nets
        .iter()
        .map(|n| Net {
            name: n.name.clone(),
            voltage: n.voltage.as_ref().map(ToString::to_string),
            is_gnd: n.is_gnd,
            members: n
                .members
                .iter()
                .map(|(p, l)| NetMember {
                    instance_path: p.clone(),
                    logical_pin: l.clone(),
                    numbers: pin_numbers
                        .get(&(p.clone(), l.clone()))
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect(),
            span: src_span(sm, n.span),
        })
        .collect();

    let nc = ir
        .nc_pins
        .iter()
        .map(|(p, l)| NcPin {
            instance_path: p.clone(),
            logical_pin: l.clone(),
        })
        .collect();

    let diagnostics: Vec<Diag> = checked
        .diags
        .iter()
        .map(|d| Diag {
            code: d.code.to_string(),
            severity: match d.severity {
                Severity::Error => "error".to_string(),
                Severity::Warning => "warning".to_string(),
            },
            message: d.message.clone(),
            span: Some(src_span(sm, d.primary.span)),
        })
        .collect();
    let verdict = if diagnostics.iter().any(|d| d.severity == "error") {
        "fail"
    } else {
        "pass"
    };

    // Derived display hints.
    let two_terminal: Vec<String> = ir
        .instances
        .iter()
        .filter(|(_, i)| {
            i.impl_traits
                .iter()
                .any(|t| cohdl::resolve::short(t) == "TwoTerminal")
        })
        .map(|(p, _)| p.clone())
        .collect();
    let rails: Vec<String> = ir
        .nets
        .iter()
        .filter(|n| n.is_gnd || n.voltage.is_some() || n.members.len() > RAIL_FANOUT)
        .map(|n| n.name.clone())
        .collect();
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in ir.instances.keys() {
        let segs: Vec<&str> = path.split("::").collect();
        if segs.len() > 2 {
            groups
                .entry(segs[..segs.len() - 1].join("::"))
                .or_default()
                .push(path.clone());
        }
    }
    let fn_groups = groups
        .into_iter()
        .map(|(name, members)| FnGroup { name, members })
        .collect();
    let bypasses: Vec<Bypass> = ir
        .layout
        .bypasses
        .iter()
        .map(|b| Bypass {
            cap: b.cap_path.clone(),
            target: b.target_path.clone(),
        })
        .collect();

    // Footprint geometry for every content-bearing footprint a bound part's
    // primary AVL entry references (the same "used" definition as the
    // kicad_mod emitter, narrowed to primary — that is what the BOM ships).
    let mut footprints: BTreeMap<String, FootprintGeo> = BTreeMap::new();
    for inst in ir.instances.values() {
        let Some(pd) = inst.part.as_ref().and_then(|p| world.parts.get(p)) else {
            continue;
        };
        let Some(fp_ref) = pd.primary.footprint.as_ref() else {
            continue;
        };
        let fq = &fp_ref.name;
        if footprints.contains_key(fq) {
            continue;
        }
        let Some(fp) = world.footprints.get(fq) else {
            continue;
        };
        if cohdl::check::footprints::is_placeholder(fp) {
            continue;
        }
        footprints.insert(fq.clone(), footprint_geo(world, fp));
    }

    Ok(ExplorerModel {
        schema_version: 1,
        design: checked.design_name.clone().unwrap_or_default(),
        verdict: verdict.to_string(),
        instances,
        nets,
        nc,
        diagnostics,
        derived: Derived {
            two_terminal,
            rails,
            fn_groups,
            bypasses,
        },
        footprints,
    })
}

const FEMTO_PER_MM: f64 = 1e15;

fn femto_mm(v: &cohdl::units::UnitValue) -> f64 {
    v.femto as f64 / FEMTO_PER_MM
}

fn shape_geo(c: &cohdl::ast::Courtyard) -> FpShape {
    FpShape {
        shape: c.shape.0.name().to_string(),
        x: femto_mm(&c.at.0),
        y: femto_mm(&c.at.1),
        size: c.size.iter().map(femto_mm).collect(),
    }
}

fn footprint_geo(world: &World, fp: &cohdl::ast::FootprintDef) -> FootprintGeo {
    use cohdl::ast::{MountHoleGeom, MountHolePlating, PadDrill, PadPlating};
    let pads = fp
        .pads
        .iter()
        .map(|p| {
            let def = world.pads.get(&p.pad.name);
            let pth = def
                .and_then(|d| d.plating.as_ref())
                .is_some_and(|(pl, _)| *pl == PadPlating::PlatedThroughHole);
            FpPad {
                number: p.number.text.clone(),
                shape: def
                    .and_then(|d| d.shape)
                    .map_or_else(|| "rect".to_string(), |(s, _)| s.name().to_string()),
                x: femto_mm(&p.x),
                y: femto_mm(&p.y),
                size: def
                    .map(|d| d.size.iter().map(femto_mm).collect())
                    .unwrap_or_default(),
                rotate: p.rotate,
                drill: def
                    .and_then(|d| d.drill.as_ref())
                    .map(|(dr, _)| match dr {
                        PadDrill::Round(d) => vec![femto_mm(d)],
                        PadDrill::Slot(w, l) => vec![femto_mm(w), femto_mm(l)],
                    })
                    .unwrap_or_default(),
                pth,
            }
        })
        .collect();
    let mount_holes = fp
        .mount_holes
        .iter()
        .map(|h| FpHole {
            shape: h.shape_or_default().name().to_string(),
            x: femto_mm(&h.x),
            y: femto_mm(&h.y),
            size: match &h.geom {
                MountHoleGeom::Diameter(d) => vec![femto_mm(d)],
                MountHoleGeom::Size(s, _) => s.iter().map(femto_mm).collect(),
            },
            plated: h.plating == MountHolePlating::Plated,
        })
        .collect();
    FootprintGeo {
        pads,
        mount_holes,
        courtyard: fp.courtyard.as_ref().map(shape_geo),
        window: fp.window.as_deref().map(shape_geo),
    }
}

fn role_name(r: PinRole) -> &'static str {
    match r {
        PinRole::Input => "input",
        PinRole::Output => "output",
        PinRole::Bidirectional => "bidirectional",
        PinRole::Passive => "passive",
        PinRole::PowerIn => "power_in",
        PinRole::PowerOut => "power_out",
    }
}

fn src_span(sm: &SourceMap, span: Span) -> SrcSpan {
    let lc = sm.line_col(span.file, span.start);
    SrcSpan {
        file: sm.name(span.file).to_string(),
        line: lc.line,
        col: lc.col,
    }
}

/// The cohdl checkout's lib/ root, located relative to the target project
/// (walk up from the project dir looking for a `lib/` that is a library
/// root — works for `../cohdl/examples/*`), else COHDL_LIB env override.
fn find_lib_root_near(dir: &Path) -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("COHDL_LIB") {
        let p = std::path::PathBuf::from(p);
        if cohdl::deps::is_library_root(&p) {
            return Some(p);
        }
    }
    let abs = dir.canonicalize().ok()?;
    for anc in abs.ancestors() {
        let cand = anc.join("lib");
        if cohdl::deps::is_library_root(&cand) {
            return Some(cand);
        }
    }
    None
}

fn render_pkg_diags(diags: &[cohdl::deps::PackageDiag]) -> String {
    cohdl::deps::render_human(diags)
}

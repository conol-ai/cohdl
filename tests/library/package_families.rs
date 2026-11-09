//! W4 package-family lands, wave 1 (library plan item 3c): `sod`, `sot` and
//! `do214`.
//!
//! Each land is pinned exactly against its source table (see each package's
//! `docs/README.md`): pad numbers, centres, sizes, rotation, courtyard and the
//! polarity or pin-1 mark. The frame is part of the land: SOD and DO-214 put
//! pad 1 (cathode) at -x; SOT-23-3 uses `transistor::FP_JSCJ_SOT23`'s
//! assembler frame (pins 1/2 in the +x column, pin 1 at +y, pin 3 at -x).
//!
//! The same checker must reject the two polarity defect classes: a mirrored
//! SOT-23 (pins 1 and 2 swapped, the #48 class) and a diode land whose
//! cathode is pad 2. The defect fixtures are derived from the shipped source
//! by one textual change each, so they differ from it only in the defect.

use cohdl::ast::{PadLayer, PadPlating, PadShape, SilkItem};
use cohdl::resolve::World;

/// Femto-units (10^-15 mm) per micrometre.
const UM: i128 = 1_000_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    /// `polarity_marker cathode_pin N shape band`
    CathodeBand(&'static str),
    /// `pin_1_marker near pad N shape dot`
    Pin1Dot(&'static str),
}

struct Land {
    package: &'static str,
    /// Package source file under `lib/<package>/`.
    source: &'static str,
    footprint: &'static str,
    pad: &'static str,
    /// Declared pad size (before the placement's rotation), micrometres.
    size_um: (i128, i128),
    rotate: u16,
    /// `(number, x, y)` in micrometres, CoHDL frame (+y down).
    pads: &'static [(&'static str, i128, i128)],
    courtyard_um: (i128, i128),
    mark: Mark,
}

const LANDS: [Land; 4] = [
    // Hongjiacheng BZT52 Rev 2.1 p4: J 0.91, K 2.36, M 1.22 (one-sided limits).
    Land {
        package: "sod",
        source: "src/footprints.cohdl",
        footprint: "SOD3716X135N",
        pad: "P_SOD3716X135N_LEAD",
        size_um: (910, 1220),
        rotate: 0,
        pads: &[("1", -1635, 0), ("2", 1635, 0)],
        courtyard_um: (4700, 2400),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng B5819WS Rev 2.0 p3: X 0.70, Y 0.70, Z 2.15 (midpoints).
    Land {
        package: "sod",
        source: "src/footprints.cohdl",
        footprint: "SOD2613X110N",
        pad: "P_SOD2613X110N_LEAD",
        size_um: (700, 700),
        rotate: 0,
        pads: &[("1", -1075, 0), ("2", 1075, 0)],
        courtyard_um: (3400, 2000),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng MMBT5551 Rev 2.0 p3: J 0.80, K 0.90, M 2.00, N 1.90
    // (midpoints), on the FP_JSCJ_SOT23 assembler frame.
    Land {
        package: "sot",
        source: "src/footprints.cohdl",
        footprint: "SOT3P190X290X130N",
        pad: "P_SOT3P190X290X130N_LEAD",
        size_um: (800, 900),
        rotate: 270,
        pads: &[("1", 1000, 950), ("2", 1000, -950), ("3", -1000, 0)],
        courtyard_um: (3400, 3600),
        mark: Mark::Pin1Dot("1"),
    },
    // Hongjiacheng SS34 Rev 2.0 p3: M 1.70, J 2.10, K 2.30 (one-sided limits).
    Land {
        package: "do214",
        source: "src/footprints.cohdl",
        footprint: "DIOM5026X241N",
        pad: "P_DIOM5026X241N_LEAD",
        size_um: (2100, 1700),
        rotate: 0,
        pads: &[("1", -2200, 0), ("2", 2200, 0)],
        courtyard_um: (7000, 3400),
        mark: Mark::CathodeBand("1"),
    },
];

fn um(femto: i128) -> String {
    format!("{}um", femto as f64 / UM as f64)
}

/// Every way `world`'s copy of `land` differs from the source table.
fn mismatches(world: &World, land: &Land) -> Vec<String> {
    let root = cohdl::pipeline::package_root(land.package);
    let fq = format!("{root}::{}", land.footprint);
    let pad_fq = format!("{root}::{}", land.pad);
    let Some(fp) = world.footprints.get(&fq) else {
        return vec![format!("{fq} is not declared")];
    };
    let mut out = Vec::new();
    if fp.pads.len() != land.pads.len() {
        out.push(format!(
            "{fq}: {} pad placements, expected {}",
            fp.pads.len(),
            land.pads.len()
        ));
    }
    for (number, x, y) in land.pads {
        let places: Vec<_> = fp
            .pads
            .iter()
            .filter(|place| place.number.text == *number)
            .collect();
        let [place] = places.as_slice() else {
            out.push(format!(
                "{fq}: pad {number} placed {} times, expected once",
                places.len()
            ));
            continue;
        };
        if place.pad.name != pad_fq {
            out.push(format!(
                "{fq}: pad {number} uses {}, expected {pad_fq}",
                place.pad.name
            ));
        }
        if (place.x.femto, place.y.femto) != (x * UM, y * UM) {
            out.push(format!(
                "{fq}: pad {number} at ({}, {}), expected ({}, {})",
                um(place.x.femto),
                um(place.y.femto),
                um(x * UM),
                um(y * UM)
            ));
        }
        if place.rotate != land.rotate {
            out.push(format!(
                "{fq}: pad {number} rotated {}, expected {}",
                place.rotate, land.rotate
            ));
        }
    }
    match world.pads.get(&pad_fq) {
        None => out.push(format!("{pad_fq} is not declared")),
        Some(pad) => {
            if pad.shape.map(|(shape, _)| shape) != Some(PadShape::Rect) {
                out.push(format!("{pad_fq}: not a rectangular pad"));
            }
            let size: Vec<i128> = pad.size.iter().map(|value| value.femto).collect();
            if size != [land.size_um.0 * UM, land.size_um.1 * UM] {
                out.push(format!(
                    "{pad_fq}: size {:?}, expected {:?} um",
                    size.iter().map(|v| v / UM).collect::<Vec<_>>(),
                    land.size_um
                ));
            }
            if pad.layer.map(|(layer, _)| layer) != Some(PadLayer::TopCopper)
                || pad.plating.map(|(plating, _)| plating) != Some(PadPlating::Smd)
            {
                out.push(format!("{pad_fq}: not a top-copper SMD land"));
            }
        }
    }
    match &fp.courtyard {
        None => out.push(format!("{fq}: no courtyard")),
        Some(courtyard) => {
            let size: Vec<i128> = courtyard.size.iter().map(|value| value.femto).collect();
            if courtyard.shape.0 != PadShape::Rect
                || (courtyard.at.0.femto, courtyard.at.1.femto) != (0, 0)
                || size != [land.courtyard_um.0 * UM, land.courtyard_um.1 * UM]
            {
                out.push(format!(
                    "{fq}: courtyard is not the centred {:?} um rectangle",
                    land.courtyard_um
                ));
            }
        }
    }
    let marks: Vec<Mark> = fp
        .silkscreen
        .iter()
        .flat_map(|block| block.items.iter())
        .filter_map(|item| match item {
            SilkItem::PolarityMarker {
                cathode_pad,
                shape: cohdl::ast::PolarityShape::Band,
                ..
            } => Some(Mark::CathodeBand(if cathode_pad.text == "1" {
                "1"
            } else {
                "other"
            })),
            SilkItem::Pin1Marker {
                pad,
                shape: cohdl::ast::Pin1Shape::Dot,
                ..
            } => Some(Mark::Pin1Dot(if pad.text == "1" { "1" } else { "other" })),
            SilkItem::PolarityMarker { .. } | SilkItem::Pin1Marker { .. } => {
                Some(Mark::CathodeBand("unexpected shape"))
            }
            SilkItem::Graphic(..) => None,
        })
        .collect();
    if marks != [land.mark] {
        out.push(format!(
            "{fq}: polarity/pin-1 marks {marks:?}, expected [{:?}]",
            land.mark
        ));
    }
    out
}

fn shipped_source(land: &Land) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join("lib").join(land.package).join(land.source)).unwrap()
}

/// Check one package's source text (shipped or a defect fixture derived from
/// it) and return its world; a fixture must still be a valid declaration set
/// so the land checker, not the compiler, is what rejects it.
fn world_of(package: &str, source: &str) -> World {
    let files = vec![("src/footprints.cohdl".to_string(), source.to_string())];
    let mut checked = cohdl::pipeline::check_files_in(package, &files, None).unwrap();
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    checked.world
}

/// Apply `(from, to)` replacements, each of which must match exactly once.
fn derive(source: &str, edits: &[(&str, &str)]) -> String {
    let mut out = source.to_string();
    for (from, to) in edits {
        assert_eq!(
            out.matches(from).count(),
            1,
            "defect fixture edit `{from}` must match the shipped source exactly once"
        );
        out = out.replacen(from, to, 1);
    }
    out
}

#[test]
fn w4_wave1_lands_match_their_source_tables() {
    for land in &LANDS {
        let world = world_of(land.package, &shipped_source(land));
        let found = mismatches(&world, land);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }
}

#[test]
fn mirrored_sot23_is_rejected() {
    let land = &LANDS[2];
    assert_eq!(land.footprint, "SOT3P190X290X130N");
    // Pins 1 and 2 swap places: the land the #48 mirror defect produced.
    let mirrored = derive(
        &shipped_source(land),
        &[
            (
                "pad 1: P_SOT3P190X290X130N_LEAD at (1mm, 0.95mm)",
                "pad 1: P_SOT3P190X290X130N_LEAD at (1mm, -0.95mm)",
            ),
            (
                "pad 2: P_SOT3P190X290X130N_LEAD at (1mm, -0.95mm)",
                "pad 2: P_SOT3P190X290X130N_LEAD at (1mm, 0.95mm)",
            ),
        ],
    );
    let found = mismatches(&world_of(land.package, &mirrored), land);
    assert!(
        found
            .iter()
            .any(|m| m.contains("pad 1 at (1000um, -950um)"))
            && found.iter().any(|m| m.contains("pad 2 at (1000um, 950um)")),
        "a mirrored SOT-23 passed the exact-land check: {found:?}"
    );
}

#[test]
fn diode_lands_with_the_cathode_on_pad_2_are_rejected() {
    for land in LANDS
        .iter()
        .filter(|land| matches!(land.mark, Mark::CathodeBand(_)))
    {
        let source = shipped_source(land);
        let (x1, x2) = (land.pads[0].1, land.pads[1].1);
        assert!(
            x1 < 0 && x2 > 0,
            "{}: cathode must be at -x",
            land.footprint
        );
        let place = |number: &str, x: i128| {
            format!(
                "pad {number}: {} at ({}mm, 0mm)",
                land.pad,
                x as f64 / 1000.0
            )
        };
        // The band moved to pad 2: the copper is right, the mark is wrong.
        let band_on_pad_2 = derive_band(land, &source);
        let found = mismatches(&world_of(land.package, &band_on_pad_2), land);
        assert!(
            found.iter().any(|m| m.contains("polarity/pin-1 marks")),
            "{}: a band on pad 2 passed: {found:?}",
            land.footprint
        );
        // The numbers swapped: pad 2 (the anode) now sits on the band end.
        let swapped = derive(
            &source,
            &[
                (&place("1", x1), &place("1", x2)),
                (&place("2", x2), &place("2", x1)),
            ],
        );
        let found = mismatches(&world_of(land.package, &swapped), land);
        assert!(
            found.iter().any(|m| m.contains("pad 1 at"))
                && found.iter().any(|m| m.contains("pad 2 at")),
            "{}: a land with pad 2 at the cathode end passed: {found:?}",
            land.footprint
        );
    }
}

/// Move `land`'s cathode band to pad 2 inside its own footprint body only.
fn derive_band(land: &Land, source: &str) -> String {
    let start = source
        .find(&format!("pub footprint {} {{", land.footprint))
        .unwrap();
    let end = start + source[start..].find("\n}\n").unwrap();
    let body = &source[start..end];
    let edited = derive(
        body,
        &[(
            "polarity_marker cathode_pin 1 shape band",
            "polarity_marker cathode_pin 2 shape band",
        )],
    );
    format!("{}{}{}", &source[..start], edited, &source[end..])
}

/// The fixture consumer binds every wave-1 land through the packages'
/// qualified paths, so pad consistency and KiCad emission are exercised on
/// real parts rather than passing vacuously.
#[test]
fn wave1_fixture_consumer_binds_and_emits_every_land() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps: Vec<(String, std::path::PathBuf)> = ["std", "sod", "sot", "do214"]
        .iter()
        .map(|name| (name.to_string(), root.join("lib").join(name)))
        .collect();
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project = cohdl::project::load_project_with_deps(
        &root.join("tests/fixtures/package-families/wave1_consumer.cohdl"),
        &deps,
    )
    .unwrap();
    let mut checked =
        cohdl::pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap();
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let mut pad_diags = cohdl::diag::Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&checked.world, &mut pad_diags);
    assert!(!pad_diags.has_errors(), "{}", pad_diags.render(&checked.sm));
    for land in &LANDS {
        let found = mismatches(&checked.world, land);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    cohdl::pipeline::build_artifacts(&mut checked, &cohdl::lock::LockState::default()).unwrap();
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir).unwrap();
    let module = |name: &str| {
        mods.iter()
            .find(|(fq, _, _)| fq == name)
            .map(|(_, _, text)| text.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "no emitted module for {name}: {:?}",
                    mods.iter().map(|m| &m.0).collect::<Vec<_>>()
                )
            })
    };
    for (name, lines) in [
        (
            "sod::SOD3716X135N",
            &[
                "(pad \"1\" smd rect (at -1.635 0) (size 0.91 1.22)",
                "(pad \"2\" smd rect (at 1.635 0) (size 0.91 1.22)",
                // Cathode band outboard of pad 1.
                "(fp_line (start -2.39 -0.61) (end -2.39 0.61) (layer \"F.SilkS\") (stroke (width 0.3)",
                "(fp_rect (start -2.35 -1.2) (end 2.35 1.2) (layer \"F.CrtYd\")",
            ][..],
        ),
        (
            "sod::SOD2613X110N",
            &[
                "(pad \"1\" smd rect (at -1.075 0) (size 0.7 0.7)",
                "(pad \"2\" smd rect (at 1.075 0) (size 0.7 0.7)",
                "(fp_line (start -1.725 -0.35) (end -1.725 0.35) (layer \"F.SilkS\") (stroke (width 0.3)",
                "(fp_rect (start -1.7 -1) (end 1.7 1) (layer \"F.CrtYd\")",
            ][..],
        ),
        (
            "do214::DIOM5026X241N",
            &[
                "(pad \"1\" smd rect (at -2.2 0) (size 2.1 1.7)",
                "(pad \"2\" smd rect (at 2.2 0) (size 2.1 1.7)",
                "(fp_line (start -3.55 -0.85) (end -3.55 0.85) (layer \"F.SilkS\") (stroke (width 0.3)",
                "(fp_rect (start -3.5 -1.7) (end 3.5 1.7) (layer \"F.CrtYd\")",
            ][..],
        ),
        (
            "sot::SOT3P190X290X130N",
            &[
                "(pad \"1\" smd rect (at 1 0.95 270) (size 0.8 0.9)",
                "(pad \"2\" smd rect (at 1 -0.95 270) (size 0.8 0.9)",
                "(pad \"3\" smd rect (at -1 0 270) (size 0.8 0.9)",
                // Pin-1 dot beyond pad 1, on the +y side.
                "(fp_circle (center 1 1.85) (end 1.2 1.85) (layer \"F.SilkS\")",
                "(fp_rect (start -1.7 -1.8) (end 1.7 1.8) (layer \"F.CrtYd\")",
            ][..],
        ),
    ] {
        let text = module(name);
        for line in lines {
            assert!(text.contains(line), "{name} is missing `{line}`:\n{text}");
        }
    }
}

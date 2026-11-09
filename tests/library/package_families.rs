//! W4 package-family lands, waves 1 to 3 (library plan item 3c): `sod`,
//! `sot` and `do214`.
//!
//! Each land is pinned exactly against its source table (see each package's
//! `docs/README.md`): pad numbers, centres, sizes, rotation, courtyard and the
//! polarity or pin-1 mark. The frame is part of the land: SOD, DO-214 and
//! SMAF put pad 1 (cathode) at -x; every SOT land uses
//! `transistor::FP_JSCJ_SOT23`'s assembler frame (the pin-1 edge in the +x
//! column with pin 1 at +y; SOT-23 and SOT-323 pin 3, and the SOT-89 and
//! SOT-223 tab, at -x). The SOT-89 tab is a second placement of pad 2; the
//! SOT-223 tab is pad 4.
//!
//! The same checker must reject the polarity defect classes: a mirrored SOT
//! land (pin 1 on the wrong end of its row, the #48 class), a diode land
//! whose cathode is pad 2, and a tab numbered against the convention. The
//! defect fixtures are derived from the shipped source by one textual change
//! each, so they differ from it only in the defect.

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

/// A placement whose pad differs from the land's main lead pad: the SOT-89
/// lead-2 stem and tab, the SOT-223 tab.
struct Extra {
    number: &'static str,
    pad: &'static str,
    /// Declared pad size (before the placement's rotation), micrometres.
    size_um: (i128, i128),
    /// Centre in micrometres, CoHDL frame (+y down).
    x: i128,
    y: i128,
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
    /// Further placements with their own pad; same rotation as `pads`.
    extra: &'static [Extra],
    courtyard_um: (i128, i128),
    mark: Mark,
}

/// Wave 1: SOD-123, SOD-323, SOT-23-3 and SMA.
const WAVE1: [Land; 4] = [
    // Hongjiacheng BZT52 Rev 2.1 p4: J 0.91, K 2.36, M 1.22 (one-sided limits).
    Land {
        package: "sod",
        source: "src/footprints.cohdl",
        footprint: "SOD3716X135N",
        pad: "P_SOD3716X135N_LEAD",
        size_um: (910, 1220),
        rotate: 0,
        extra: &[],
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
        extra: &[],
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
        extra: &[],
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
        extra: &[],
        pads: &[("1", -2200, 0), ("2", 2200, 0)],
        courtyard_um: (7000, 3400),
        mark: Mark::CathodeBand("1"),
    },
];

/// Wave 2: SMB, SOD-123FL, SMC and SOD-523.
const WAVE2: [Land; 4] = [
    // Hongjiacheng SMBJ Rev 1.1 p2: M 2.26, J 2.10, K 2.74 (one-sided limits).
    Land {
        package: "do214",
        source: "src/footprints.cohdl",
        footprint: "DIOM5336X265N",
        pad: "P_DIOM5336X265N_LEAD",
        size_um: (2100, 2260),
        rotate: 0,
        extra: &[],
        pads: &[("1", -2420, 0), ("2", 2420, 0)],
        courtyard_um: (7500, 4500),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng SMF Rev 2.2 p2 has no land: Diodes SOD123F Rev. 2026-04-08
    // p1, X 1.34, Y 1.80, C 2.86.
    Land {
        package: "sod",
        source: "src/footprints.cohdl",
        footprint: "SODFL3818X120N",
        pad: "P_SODFL3818X120N_LEAD",
        size_um: (1340, 1800),
        rotate: 0,
        extra: &[],
        pads: &[("1", -1430, 0), ("2", 1430, 0)],
        courtyard_um: (4700, 2500),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng SMCJ Rev 2.2 p2 has no land: Diodes SMC Rev. 2017-03-14 p1,
    // X 2.50, Y 3.30, C 6.90.
    Land {
        package: "do214",
        source: "src/footprints.cohdl",
        footprint: "DIOM7959X261N",
        pad: "P_DIOM7959X261N_LEAD",
        size_um: (2500, 3300),
        rotate: 0,
        extra: &[],
        pads: &[("1", -3450, 0), ("2", 3450, 0)],
        courtyard_um: (9900, 6800),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng 1N4148WT Rev 2.0 p3: X 0.60, Y 0.70, Z 1.42 (midpoints).
    Land {
        package: "sod",
        source: "src/footprints.cohdl",
        footprint: "SOD1608X70N",
        pad: "P_SOD1608X70N_LEAD",
        size_um: (600, 700),
        rotate: 0,
        extra: &[],
        pads: &[("1", -710, 0), ("2", 710, 0)],
        courtyard_um: (2600, 1500),
        mark: Mark::CathodeBand("1"),
    },
];

/// Wave 3: SMAF, SOT-89-3, SOT-323 and SOT-223.
const WAVE3: [Land; 4] = [
    // Hongjiacheng SS22F-SS220F Rev 2.0 p3: M 1.80, J 1.60, K 2.2 (one-sided
    // limits).
    Land {
        package: "do214",
        source: "src/footprints.cohdl",
        footprint: "DIOM4726X120N",
        pad: "P_DIOM4726X120N_LEAD",
        size_um: (1600, 1800),
        rotate: 0,
        extra: &[],
        pads: &[("1", -1900, 0), ("2", 1900, 0)],
        courtyard_um: (5900, 3300),
        mark: Mark::CathodeBand("1"),
    },
    // Hongjiacheng D882 Rev 2.0 p3's land does not fit its outline: Diodes
    // SOT89 Rev. 2026-01-16 p1, X 0.580, Y 1.630, C 1.500, X1 0.760, X2
    // 1.933, Y1 3.030, Y4 4.530. Pad 2 is the lead-2 stem and the tab.
    Land {
        package: "sot",
        source: "src/footprints.cohdl",
        footprint: "SOT3P150X450X250N",
        pad: "P_SOT3P150X450X250N_LEAD",
        size_um: (580, 1630),
        rotate: 270,
        pads: &[("1", 1450, 1500), ("3", 1450, -1500)],
        extra: &[
            Extra {
                number: "2",
                pad: "P_SOT3P150X450X250N_STEM",
                size_um: (760, 1630),
                x: 1450,
                y: 0,
            },
            Extra {
                number: "2",
                pad: "P_SOT3P150X450X250N_TAB",
                size_um: (1933, 3030),
                x: -750,
                y: 0,
            },
        ],
        courtyard_um: (5100, 5200),
        mark: Mark::Pin1Dot("1"),
    },
    // Hongjiacheng 2N7002W Rev 2.0 p5: J 0.70, K 0.90, M 1.90, N 1.30
    // (midpoints), on the SOT-23-3 frame.
    Land {
        package: "sot",
        source: "src/footprints.cohdl",
        footprint: "SOT3P130X200X125N",
        pad: "P_SOT3P130X200X125N_LEAD",
        size_um: (700, 900),
        rotate: 270,
        extra: &[],
        pads: &[("1", 950, 650), ("2", 950, -650), ("3", -950, 0)],
        courtyard_um: (3300, 2800),
        mark: Mark::Pin1Dot("1"),
    },
    // AMS1117 p7 has no land: Diodes SOT223 Rev. 2017-03-06 p1, X 1.20,
    // Y 1.60, C 2.30, X1 3.30, Y1 1.60, C1 6.40. The tab is pad 4.
    Land {
        package: "sot",
        source: "src/footprints.cohdl",
        footprint: "SOT4P230X650X350N",
        pad: "P_SOT4P230X650X350N_LEAD",
        size_um: (1200, 1600),
        rotate: 270,
        pads: &[("1", 3200, 2300), ("2", 3200, 0), ("3", 3200, -2300)],
        extra: &[Extra {
            number: "4",
            pad: "P_SOT4P230X650X350N_TAB",
            size_um: (3300, 1600),
            x: -3200,
            y: 0,
        }],
        courtyard_um: (8500, 7300),
        mark: Mark::Pin1Dot("1"),
    },
];

/// Every W4 land, all three waves.
fn all_lands() -> impl Iterator<Item = &'static Land> {
    WAVE1.iter().chain(WAVE2.iter()).chain(WAVE3.iter())
}

fn um(femto: i128) -> String {
    format!("{}um", femto as f64 / UM as f64)
}

/// Every way `world`'s copy of `land` differs from the source table.
fn mismatches(world: &World, land: &Land) -> Vec<String> {
    let root = cohdl::pipeline::package_root(land.package);
    let fq = format!("{root}::{}", land.footprint);
    let Some(fp) = world.footprints.get(&fq) else {
        return vec![format!("{fq} is not declared")];
    };
    let mut out = Vec::new();
    // Every expected placement: (number, pad, x, y), main lead pad first.
    let expected: Vec<(&str, &str, i128, i128)> = land
        .pads
        .iter()
        .map(|(number, x, y)| (*number, land.pad, *x, *y))
        .chain(
            land.extra
                .iter()
                .map(|extra| (extra.number, extra.pad, extra.x, extra.y)),
        )
        .collect();
    if fp.pads.len() != expected.len() {
        out.push(format!(
            "{fq}: {} pad placements, expected {}",
            fp.pads.len(),
            expected.len()
        ));
    }
    for (number, pad, x, y) in &expected {
        let pad_fq = format!("{root}::{pad}");
        let places: Vec<_> = fp
            .pads
            .iter()
            .filter(|place| place.number.text == *number && place.pad.name == pad_fq)
            .collect();
        let [place] = places.as_slice() else {
            out.push(format!(
                "{fq}: pad {number} ({pad}) placed {} times, expected once",
                places.len()
            ));
            continue;
        };
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
    let pad_sizes = std::iter::once((land.pad, land.size_um))
        .chain(land.extra.iter().map(|extra| (extra.pad, extra.size_um)));
    for (pad, size_um) in pad_sizes {
        let pad_fq = format!("{root}::{pad}");
        match world.pads.get(&pad_fq) {
            None => out.push(format!("{pad_fq} is not declared")),
            Some(def) => {
                if def.shape.map(|(shape, _)| shape) != Some(PadShape::Rect) {
                    out.push(format!("{pad_fq}: not a rectangular pad"));
                }
                let size: Vec<i128> = def.size.iter().map(|value| value.femto).collect();
                if size != [size_um.0 * UM, size_um.1 * UM] {
                    out.push(format!(
                        "{pad_fq}: size {:?}, expected {:?} um",
                        size.iter().map(|v| v / UM).collect::<Vec<_>>(),
                        size_um
                    ));
                }
                if def.layer.map(|(layer, _)| layer) != Some(PadLayer::TopCopper)
                    || def.plating.map(|(plating, _)| plating) != Some(PadPlating::Smd)
                {
                    out.push(format!("{pad_fq}: not a top-copper SMD land"));
                }
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

fn assert_lands_match(lands: &[Land]) {
    for land in lands {
        let world = world_of(land.package, &shipped_source(land));
        let found = mismatches(&world, land);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }
}

#[test]
fn w4_wave1_lands_match_their_source_tables() {
    assert_lands_match(&WAVE1);
}

#[test]
fn w4_wave2_lands_match_their_source_tables() {
    assert_lands_match(&WAVE2);
}

#[test]
fn w4_wave3_lands_match_their_source_tables() {
    assert_lands_match(&WAVE3);
}

#[test]
fn mirrored_sot23_is_rejected() {
    let land = &WAVE1[2];
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

/// Mirror one wave-3 SOT land about the x axis inside the shipped source:
/// every placement off the axis moves to the other end of its row, so pin 1
/// lands where the last pin of the row should be.
fn mirrored(land: &Land) -> String {
    let place = |number: &str, pad: &str, x: i128, y: i128| {
        format!(
            "pad {number}: {pad} at ({}mm, {}mm)",
            x as f64 / 1000.0,
            y as f64 / 1000.0
        )
    };
    let placements: Vec<(&str, &str, i128, i128)> = land
        .pads
        .iter()
        .map(|(number, x, y)| (*number, land.pad, *x, *y))
        .chain(
            land.extra
                .iter()
                .map(|extra| (extra.number, extra.pad, extra.x, extra.y)),
        )
        .filter(|(.., y)| *y != 0)
        .collect();
    let edits: Vec<(String, String)> = placements
        .iter()
        .map(|(number, pad, x, y)| (place(number, pad, *x, *y), place(number, pad, *x, -y)))
        .collect();
    let edits: Vec<(&str, &str)> = edits
        .iter()
        .map(|(from, to)| (from.as_str(), to.as_str()))
        .collect();
    // Applied one at a time, each edit's target is unique when it runs (the
    // pin-1 line is rewritten before the pin it trades places with).
    edits.iter().fold(shipped_source(land), |source, edit| {
        derive(&source, std::slice::from_ref(edit))
    })
}

#[test]
fn mirrored_wave3_sot_lands_are_rejected() {
    // SOT-89 and SOT-223: pins 1 and 3 trade ends of the lead row with the
    // tab unchanged. SOT-323: pins 1 and 2 trade places, as on SOT-23.
    let sot_lands: Vec<&Land> = WAVE3
        .iter()
        .filter(|land| matches!(land.mark, Mark::Pin1Dot(_)))
        .collect();
    assert_eq!(
        sot_lands
            .iter()
            .map(|land| land.footprint)
            .collect::<Vec<_>>(),
        [
            "SOT3P150X450X250N",
            "SOT3P130X200X125N",
            "SOT4P230X650X350N"
        ]
    );
    for land in sot_lands {
        let (_, x1, y1) = land.pads[0];
        assert!(y1 > 0, "{}: pin 1 must sit at +y", land.footprint);
        let found = mismatches(&world_of(land.package, &mirrored(land)), land);
        let wrong_pin_1 = format!("pad 1 at ({}um, {}um)", x1, -y1);
        assert!(
            found.iter().any(|m| m.contains(&wrong_pin_1)),
            "{}: a mirrored land passed the exact-land check: {found:?}",
            land.footprint
        );
    }
}

/// Renumber the tab of one wave-3 land and return the package's rendered
/// diagnostics: a tab numbered against the convention changes the distinct
/// pad-number count, so RFC-021's name check (E809) must reject it.
fn renumbered_tab_diagnostics(land: &Land, from: &str, to: &str) -> String {
    let source = derive(&shipped_source(land), &[(from, to)]);
    let files = vec![("src/footprints.cohdl".to_string(), source)];
    let mut checked = cohdl::pipeline::check_files_in(land.package, &files, None).unwrap();
    checked.diags.sort(&checked.sm);
    assert!(checked.diags.has_errors(), "{from} -> {to} passed");
    checked.diags.render(&checked.sm)
}

#[test]
fn sot_tabs_numbered_against_the_convention_are_rejected() {
    let sot89 = &WAVE3[1];
    let sot223 = &WAVE3[3];
    assert_eq!(
        (sot89.footprint, sot223.footprint),
        ("SOT3P150X450X250N", "SOT4P230X650X350N")
    );
    // SOT-89 tab as a separate pad 4: four pad numbers under a 3-pin name.
    let message = renumbered_tab_diagnostics(
        sot89,
        "pad 2: P_SOT3P150X450X250N_TAB",
        "pad 4: P_SOT3P150X450X250N_TAB",
    );
    assert!(
        message.contains("E809")
            && message.contains("`SOT3P150X450X250N` names 3 pins, but declares 4"),
        "{message}"
    );
    // SOT-223 tab folded into pad 2: three pad numbers under a 4-pin name.
    let message = renumbered_tab_diagnostics(
        sot223,
        "pad 4: P_SOT4P230X650X350N_TAB",
        "pad 2: P_SOT4P230X650X350N_TAB",
    );
    assert!(
        message.contains("E809")
            && message.contains("`SOT4P230X650X350N` names 4 pins, but declares 3"),
        "{message}"
    );
}

#[test]
fn diode_lands_with_the_cathode_on_pad_2_are_rejected() {
    let diode_lands: Vec<&Land> = all_lands()
        .filter(|land| matches!(land.mark, Mark::CathodeBand(_)))
        .collect();
    // Eight diode lands: wave 1's three, all four of wave 2's and wave 3's
    // SMAF.
    assert_eq!(diode_lands.len(), 8);
    for land in diode_lands {
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

/// Check a fixture consumer under `tests/fixtures/package-families/`,
/// confirm every land in `lands` still matches its source table in the
/// consumer's world, emit the design to KiCad, and require each land to be
/// emitted (so bound) with every `expected` line in its module.
fn assert_consumer_binds_and_emits(fixture: &str, lands: &[Land], expected: &[(&str, &[&str])]) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let deps: Vec<(String, std::path::PathBuf)> = ["std", "sod", "sot", "do214"]
        .iter()
        .map(|name| (name.to_string(), root.join("lib").join(name)))
        .collect();
    let dep_names: Vec<String> = deps.iter().map(|(name, _)| name.clone()).collect();
    let project = cohdl::project::load_project_with_deps(
        &root.join("tests/fixtures/package-families").join(fixture),
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
    for land in lands {
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
    // Every land of the wave is emitted: none is left unbound by the fixture.
    for land in lands {
        let name = format!(
            "{}::{}",
            cohdl::pipeline::package_root(land.package),
            land.footprint
        );
        assert!(
            expected.iter().any(|(fq, _)| *fq == name),
            "{name} has no expected emission lines"
        );
    }
    for (name, lines) in expected {
        let text = module(name);
        for line in lines.iter() {
            assert!(text.contains(line), "{name} is missing `{line}`:\n{text}");
        }
    }
}

/// The fixture consumer binds every wave-1 land through the packages'
/// qualified paths, so pad consistency and KiCad emission are exercised on
/// real parts rather than passing vacuously.
#[test]
fn wave1_fixture_consumer_binds_and_emits_every_land() {
    assert_consumer_binds_and_emits(
        "wave1_consumer.cohdl",
        &WAVE1,
        &[
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
        ],
    );
}

/// The wave-2 fixture consumer does the same for SMB, SOD-123FL, SMC and
/// SOD-523.
#[test]
fn wave2_fixture_consumer_binds_and_emits_every_land() {
    assert_consumer_binds_and_emits(
        "wave2_consumer.cohdl",
        &WAVE2,
        &[
            (
                "do214::DIOM5336X265N",
                &[
                    "(pad \"1\" smd rect (at -2.42 0) (size 2.1 2.26)",
                    "(pad \"2\" smd rect (at 2.42 0) (size 2.1 2.26)",
                    // Cathode band outboard of pad 1.
                    "(fp_line (start -3.77 -1.13) (end -3.77 1.13) (layer \"F.SilkS\") (stroke (width 0.3)",
                    "(fp_rect (start -3.75 -2.25) (end 3.75 2.25) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "sod::SODFL3818X120N",
                &[
                    "(pad \"1\" smd rect (at -1.43 0) (size 1.34 1.8)",
                    "(pad \"2\" smd rect (at 1.43 0) (size 1.34 1.8)",
                    "(fp_line (start -2.4 -0.9) (end -2.4 0.9) (layer \"F.SilkS\") (stroke (width 0.3)",
                    "(fp_rect (start -2.35 -1.25) (end 2.35 1.25) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "do214::DIOM7959X261N",
                &[
                    "(pad \"1\" smd rect (at -3.45 0) (size 2.5 3.3)",
                    "(pad \"2\" smd rect (at 3.45 0) (size 2.5 3.3)",
                    "(fp_line (start -5 -1.65) (end -5 1.65) (layer \"F.SilkS\") (stroke (width 0.3)",
                    "(fp_rect (start -4.95 -3.4) (end 4.95 3.4) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "sod::SOD1608X70N",
                &[
                    "(pad \"1\" smd rect (at -0.71 0) (size 0.6 0.7)",
                    "(pad \"2\" smd rect (at 0.71 0) (size 0.6 0.7)",
                    "(fp_line (start -1.31 -0.35) (end -1.31 0.35) (layer \"F.SilkS\") (stroke (width 0.3)",
                    "(fp_rect (start -1.3 -0.75) (end 1.3 0.75) (layer \"F.CrtYd\")",
                ][..],
            ),
        ],
    );
}

/// The wave-3 fixture consumer does the same for SMAF, SOT-89-3, SOT-323
/// and SOT-223, with the SOT-223 tab (pad 4) mapped to the regulator output
/// alongside pad 2.
#[test]
fn wave3_fixture_consumer_binds_and_emits_every_land() {
    assert_consumer_binds_and_emits(
        "wave3_consumer.cohdl",
        &WAVE3,
        &[
            (
                "do214::DIOM4726X120N",
                &[
                    "(pad \"1\" smd rect (at -1.9 0) (size 1.6 1.8)",
                    "(pad \"2\" smd rect (at 1.9 0) (size 1.6 1.8)",
                    // Cathode band outboard of pad 1.
                    "(fp_line (start -3 -0.9) (end -3 0.9) (layer \"F.SilkS\") (stroke (width 0.3)",
                    "(fp_rect (start -2.95 -1.65) (end 2.95 1.65) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "sot::SOT3P150X450X250N",
                &[
                    "(pad \"1\" smd rect (at 1.45 1.5 270) (size 0.58 1.63)",
                    // Pad 2 twice: the lead-2 stem and the tab.
                    "(pad \"2\" smd rect (at 1.45 0 270) (size 0.76 1.63)",
                    "(pad \"2\" smd rect (at -0.75 0 270) (size 1.933 3.03)",
                    "(pad \"3\" smd rect (at 1.45 -1.5 270) (size 0.58 1.63)",
                    // Pin-1 dot beyond pad 1, on the +y side.
                    "(fp_circle (center 1.45 2.29) (end 1.65 2.29) (layer \"F.SilkS\")",
                    "(fp_rect (start -2.55 -2.6) (end 2.55 2.6) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "sot::SOT3P130X200X125N",
                &[
                    "(pad \"1\" smd rect (at 0.95 0.65 270) (size 0.7 0.9)",
                    "(pad \"2\" smd rect (at 0.95 -0.65 270) (size 0.7 0.9)",
                    "(pad \"3\" smd rect (at -0.95 0 270) (size 0.7 0.9)",
                    "(fp_circle (center 0.95 1.5) (end 1.15 1.5) (layer \"F.SilkS\")",
                    "(fp_rect (start -1.65 -1.4) (end 1.65 1.4) (layer \"F.CrtYd\")",
                ][..],
            ),
            (
                "sot::SOT4P230X650X350N",
                &[
                    "(pad \"1\" smd rect (at 3.2 2.3 270) (size 1.2 1.6)",
                    "(pad \"2\" smd rect (at 3.2 0 270) (size 1.2 1.6)",
                    "(pad \"3\" smd rect (at 3.2 -2.3 270) (size 1.2 1.6)",
                    // The tab is pad 4.
                    "(pad \"4\" smd rect (at -3.2 0 270) (size 3.3 1.6)",
                    "(fp_circle (center 3.2 3.4) (end 3.4 3.4) (layer \"F.SilkS\")",
                    "(fp_rect (start -4.25 -3.65) (end 4.25 3.65) (layer \"F.CrtYd\")",
                ][..],
            ),
        ],
    );
}

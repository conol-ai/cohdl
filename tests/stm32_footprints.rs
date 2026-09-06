//! Round-trip proof for the focused KiCad footprints used by the STM32 catalog.
//!
//! This deliberately proves the documented projection contract, not a claim
//! that every KiCad source graphic is losslessly imported. Electrical copper,
//! mask, circular paste, placements, and pad numbers are exact. Rectangular
//! courtyards remain exact; 13 stepped QFP/SO outlines become conservative
//! bounding rectangles. Pin-1 polygon vertices/fill are retained while the
//! source's 0.12-mm stroke uses CoHDL's documented 0.05-mm polygon hairline.

use cohdl::ast::{
    AvlEntry, Ident, PadLayer, PadPaste, PadPlating, PadShape, PartDef, SilkFill, SilkGraphic,
    SilkItem, TypeRef,
};
use cohdl::ir::{DesignIr, IrInstance, LayoutIr};
use cohdl::span::{FileId, Span};
use cohdl::{emit, pipeline};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SNAPSHOT_SHA256: &str = "173fe24d5e881ec4bfb4d5e9b50ee490ee577ac0509b619954714d74435109e8";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn object<'a>(value: &'a Value, context: &str) -> &'a Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context}: expected object, got {value}"))
}

fn array<'a>(value: &'a Value, context: &str) -> &'a [Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context}: expected array, got {value}"))
}

fn string<'a>(value: &'a Value, context: &str) -> &'a str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("{context}: expected string, got {value}"))
}

fn field<'a>(value: &'a Value, name: &str, context: &str) -> &'a Value {
    object(value, context)
        .get(name)
        .unwrap_or_else(|| panic!("{context}: missing `{name}`"))
}

fn str_field<'a>(value: &'a Value, name: &str, context: &str) -> &'a str {
    string(field(value, name, context), &format!("{context}.{name}"))
}

fn assert_keys(value: &Value, expected: &[&str], context: &str) {
    let actual = object(value, context)
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{context}: schema keys changed");
}

/// Parse the snapshot's canonical decimal millimetres into CoHDL femto-mm.
fn femto(text: &str) -> i128 {
    assert!(
        !text.is_empty() && !text.contains(['e', 'E']),
        "bad decimal {text}"
    );
    let (negative, unsigned) = text
        .strip_prefix('-')
        .map_or((false, text), |value| (true, value));
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    assert!(
        !whole.is_empty()
            && whole.bytes().all(|byte| byte.is_ascii_digit())
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
            && fraction.len() <= 15,
        "bad canonical decimal {text}"
    );
    let whole = whole.parse::<i128>().unwrap() * 1_000_000_000_000_000i128;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i128>().unwrap() * 10i128.pow(15 - fraction.len() as u32)
    };
    if negative {
        -(whole + fraction)
    } else {
        whole + fraction
    }
}

fn pair(value: &Value, context: &str) -> (i128, i128) {
    let values = array(value, context);
    assert_eq!(values.len(), 2, "{context}: expected pair");
    (
        femto(string(&values[0], context)),
        femto(string(&values[1], context)),
    )
}

fn library_package(kicad_name: &str) -> &'static str {
    match kicad_name.split_once(':').map(|pair| pair.0) {
        Some("Package_BGA") => "bga",
        Some("Package_CSP") => "csp",
        Some("Package_QFP") => "qfp",
        Some("Package_SO") => "soic",
        other => panic!("unexpected focused KiCad library {other:?}"),
    }
}

fn load_generated_world(package: &str) -> cohdl::resolve::World {
    let path = root()
        .join("lib")
        .join(package)
        .join("src/kicad_generated.cohdl");
    let files = vec![(
        "src/kicad_generated.cohdl".to_string(),
        std::fs::read_to_string(&path).unwrap(),
    )];
    let mut checked = pipeline::check_files_in(package, &files, None).expect("pipeline runs");
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "{} must parse/resolve/check:\n{}",
        path.display(),
        checked.diags.render(&checked.sm)
    );
    checked.world
}

fn insert_emitter_witnesses(
    package: &str,
    world: &mut cohdl::resolve::World,
    rows: &[&Value],
) -> DesignIr {
    let span = Span::new(FileId(0), 0, 0);
    let ident = |name: String| Ident { name, span };
    let mut instances = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let context = format!("{package} footprint {index}");
        let footprint = format!("{package}::{}", str_field(row, "public_name", &context));
        let part_name = format!("{package}::__STM32_FOOTPRINT_WITNESS_{index:03}");
        world.parts.insert(
            part_name.clone(),
            PartDef {
                name: ident(format!("__STM32_FOOTPRINT_WITNESS_{index:03}")),
                device: TypeRef {
                    name: ident("__EMITTER_ONLY".to_string()),
                    generic_args: Vec::new(),
                    variant: None,
                    span,
                },
                primary: AvlEntry {
                    fields: Vec::new(),
                    footprint: Some(ident(footprint)),
                    span,
                },
                alts: Vec::new(),
                span,
            },
        );
        let path = format!("Witness::{index:03}");
        instances.insert(
            path.clone(),
            IrInstance {
                path,
                device: "__EMITTER_ONLY".to_string(),
                variant: None,
                specs: BTreeMap::new(),
                part: Some(part_name),
                virtual_only: false,
                designator_override: None,
                designator: None,
                placement_hint: None,
                impl_traits: BTreeSet::new(),
                span,
            },
        );
    }
    DesignIr {
        name: "STM32FootprintWitness".to_string(),
        instances,
        nets: Vec::new(),
        nc_pins: BTreeSet::new(),
        layout: LayoutIr::default(),
    }
}

fn line_occurrences(text: &str, expected: &str) -> usize {
    text.lines().filter(|line| *line == expected).count()
}

#[test]
fn stm32_focused_footprints_round_trip_all_normalized_geometry() {
    let snapshot_path = root().join("tools/stm32_footprint_data/footprints.json");
    let bytes = std::fs::read(&snapshot_path).unwrap();
    assert_eq!(
        cohdl::hash::sha256_hex(&bytes),
        SNAPSHOT_SHA256,
        "the whole normalized snapshot is an explicit review boundary"
    );
    let snapshot: Value = serde_json::from_slice(&bytes).expect("valid normalized JSON");

    assert_keys(
        &snapshot,
        &[
            "coverage",
            "footprints",
            "projection_contract",
            "schema_version",
            "source",
        ],
        "snapshot",
    );
    assert_eq!(field(&snapshot, "schema_version", "snapshot"), 1);
    let coverage = field(&snapshot, "coverage", "snapshot");
    assert_keys(
        coverage,
        &["footprints", "footprints_by_library", "pads"],
        "coverage",
    );
    assert_eq!(field(coverage, "footprints", "coverage"), 103);
    assert_eq!(field(coverage, "pads", "coverage"), 9147);
    assert_eq!(
        field(
            field(coverage, "footprints_by_library", "coverage"),
            "Package_BGA",
            "coverage.by_library",
        ),
        20
    );
    assert_eq!(
        field(
            field(coverage, "footprints_by_library", "coverage"),
            "Package_CSP",
            "coverage.by_library",
        ),
        70
    );
    assert_eq!(
        field(
            field(coverage, "footprints_by_library", "coverage"),
            "Package_QFP",
            "coverage.by_library",
        ),
        10
    );
    assert_eq!(
        field(
            field(coverage, "footprints_by_library", "coverage"),
            "Package_SO",
            "coverage.by_library",
        ),
        3
    );
    let source = field(&snapshot, "source", "snapshot");
    assert_keys(
        source,
        &[
            "commit",
            "format_version",
            "license",
            "license_file",
            "license_sha256",
            "repository",
        ],
        "source",
    );
    assert_eq!(
        str_field(source, "commit", "source"),
        "819223b66f96508feaeaa305301b5e6bb5c1038b"
    );
    assert_eq!(str_field(source, "format_version", "source"), "20260206");
    assert_eq!(str_field(source, "license", "source"), "CC-BY-SA-4.0");

    let rows = array(field(&snapshot, "footprints", "snapshot"), "footprints");
    assert_eq!(rows.len(), 103);
    let mut worlds = BTreeMap::from([
        ("bga", load_generated_world("bga")),
        ("csp", load_generated_world("csp")),
        ("qfp", load_generated_world("qfp")),
        ("soic", load_generated_world("soic")),
    ]);
    let mut grouped: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();

    let mut pad_count = 0usize;
    let mut circle_paste_count = 0usize;
    let mut follow_copper_count = 0usize;
    let mut mask_count = 0usize;
    let mut roundrect_count = 0usize;
    let mut rotated_count = 0usize;
    let mut exact_courtyards = 0usize;
    let mut projected_courtyards = 0usize;
    let mut documented_silk_projections = 0usize;

    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("footprints[{row_index}]");
        assert_keys(
            row,
            &[
                "courtyard",
                "description",
                "generator",
                "kicad_name",
                "pads",
                "pin_1_polygon",
                "public_name",
                "reference_at",
                "source_path",
                "source_rules",
                "source_sha256",
                "tags",
                "version",
            ],
            &context,
        );
        assert_eq!(str_field(row, "version", &context), "20260206");
        assert_eq!(
            str_field(row, "generator", &context),
            "kicad-footprint-generator"
        );
        let source_path = str_field(row, "source_path", &context);
        assert!(source_path.ends_with(".kicad_mod"));
        let source_hash = str_field(row, "source_sha256", &context);
        assert_eq!(source_hash.len(), 64);
        assert!(source_hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let source_rules = field(row, "source_rules", &context);
        let allowed_rules = [
            "solder_mask_margin",
            "solder_paste_margin",
            "solder_paste_ratio",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert!(
            object(source_rules, &context)
                .keys()
                .all(|key| allowed_rules.contains(key.as_str())),
            "{context}: unsupported source rule"
        );
        for value in object(source_rules, &context).values() {
            let _ = femto(string(value, &context));
        }

        let kicad_name = str_field(row, "kicad_name", &context);
        let package = library_package(kicad_name);
        grouped.entry(package).or_default().push(row);
        let public_name = str_field(row, "public_name", &context);
        assert!(public_name.starts_with("KICAD_"));
        let fq = format!("{package}::{public_name}");
        let world = &worlds[package];
        let footprint = world
            .footprints
            .get(&fq)
            .unwrap_or_else(|| panic!("{context}: missing parsed/resolved `{fq}`"));
        let pads = array(field(row, "pads", &context), &format!("{context}.pads"));
        assert_eq!(footprint.pads.len(), pads.len(), "{context}: pad count");

        for (pad_index, (source_pad, placement)) in pads.iter().zip(&footprint.pads).enumerate() {
            let pad_context = format!("{context}.pads[{pad_index}]");
            let has_mask = object(source_pad, &pad_context).contains_key("mask_expansion");
            let mut pad_keys = vec!["at", "copper", "number", "paste", "rotation"];
            if has_mask {
                pad_keys.push("mask_expansion");
            }
            assert_keys(source_pad, &pad_keys, &pad_context);
            assert_eq!(
                placement.number.text,
                str_field(source_pad, "number", &pad_context)
            );
            let (x, y) = pair(field(source_pad, "at", &pad_context), &pad_context);
            assert_eq!(placement.x.femto, x, "{pad_context}: x");
            assert_eq!(placement.y.femto, y, "{pad_context}: y");
            let rotation = field(source_pad, "rotation", &pad_context)
                .as_u64()
                .expect("whole rotation") as u16;
            assert_eq!(placement.rotate, rotation, "{pad_context}: rotation");
            rotated_count += usize::from(rotation != 0);

            let pad = world
                .pads
                .get(&placement.pad.name)
                .unwrap_or_else(|| panic!("{pad_context}: unresolved pad symbol"));
            assert_eq!(pad.layer.map(|value| value.0), Some(PadLayer::TopCopper));
            assert_eq!(pad.plating.map(|value| value.0), Some(PadPlating::Smd));
            let copper = field(source_pad, "copper", &pad_context);
            let shape = str_field(copper, "shape", &pad_context);
            let has_radius = object(copper, &pad_context).contains_key("corner_radius");
            assert_keys(
                copper,
                if has_radius {
                    &["corner_radius", "shape", "size"]
                } else {
                    &["shape", "size"]
                },
                &format!("{pad_context}.copper"),
            );
            let size = array(field(copper, "size", &pad_context), &pad_context);
            assert_eq!(size.len(), 2, "{pad_context}: source copper is W/H");
            let width = femto(string(&size[0], &pad_context));
            let height = femto(string(&size[1], &pad_context));
            match shape {
                "circle" => {
                    assert_eq!(width, height, "{pad_context}: circle diameter");
                    assert_eq!(pad.shape.map(|value| value.0), Some(PadShape::Circle));
                    assert_eq!(pad.size.len(), 1);
                    assert_eq!(pad.size[0].femto, width);
                    assert!(pad.corner_radius.is_none());
                }
                "roundrect" => {
                    assert_eq!(pad.shape.map(|value| value.0), Some(PadShape::Rect));
                    assert_eq!(pad.size.len(), 2);
                    assert_eq!(pad.size[0].femto, width);
                    assert_eq!(pad.size[1].femto, height);
                    let radius = femto(str_field(copper, "corner_radius", &pad_context));
                    assert_eq!(
                        pad.corner_radius.as_ref().map(|value| value.0.femto),
                        Some(radius)
                    );
                    roundrect_count += 1;
                }
                other => panic!("{pad_context}: unsupported source shape {other}"),
            }
            if has_mask {
                let expected = femto(str_field(source_pad, "mask_expansion", &pad_context));
                assert_eq!(
                    pad.mask_expansion.as_ref().map(|value| value.0.femto),
                    Some(expected),
                    "{pad_context}: mask expansion"
                );
                mask_count += 1;
            } else {
                assert!(
                    pad.mask_expansion.is_none(),
                    "{pad_context}: unexpected mask"
                );
            }

            let paste = field(source_pad, "paste", &pad_context);
            match str_field(paste, "mode", &pad_context) {
                "follow_copper" => {
                    assert_keys(paste, &["mode"], &format!("{pad_context}.paste"));
                    assert!(
                        pad.paste.is_none(),
                        "{pad_context}: paste must follow copper"
                    );
                    follow_copper_count += 1;
                }
                "circle" => {
                    assert_keys(
                        paste,
                        &["diameter", "mode"],
                        &format!("{pad_context}.paste"),
                    );
                    let expected = femto(str_field(paste, "diameter", &pad_context));
                    match pad.paste.as_ref().map(|value| &value.0) {
                        Some(PadPaste::Circle(actual)) => assert_eq!(actual.femto, expected),
                        other => panic!("{pad_context}: expected circular paste, got {other:?}"),
                    }
                    circle_paste_count += 1;
                }
                other => panic!("{pad_context}: unsupported paste mode {other}"),
            }
            pad_count += 1;
        }

        let courtyard_source = field(row, "courtyard", &context);
        assert_keys(
            courtyard_source,
            &["projection", "rect", "source_graphics", "stroke_width"],
            &format!("{context}.courtyard"),
        );
        assert_eq!(
            str_field(courtyard_source, "stroke_width", &context),
            "0.05"
        );
        match str_field(courtyard_source, "projection", &context) {
            "exact_rect" => exact_courtyards += 1,
            "conservative_axis_aligned_bounding_rect" => projected_courtyards += 1,
            other => panic!("{context}: unsupported courtyard projection {other}"),
        }
        for (graphic_index, graphic) in array(
            field(courtyard_source, "source_graphics", &context),
            &context,
        )
        .iter()
        .enumerate()
        {
            let graphic_context = format!("{context}.courtyard.source_graphics[{graphic_index}]");
            assert_keys(graphic, &["end", "kind", "start"], &graphic_context);
            assert!(matches!(
                str_field(graphic, "kind", &graphic_context),
                "line" | "rect"
            ));
            let _ = pair(field(graphic, "start", &graphic_context), &graphic_context);
            let _ = pair(field(graphic, "end", &graphic_context), &graphic_context);
        }
        let courtyard = footprint.courtyard.as_ref().expect("generated courtyard");
        assert_eq!(courtyard.shape.0, PadShape::Rect);
        let rect = field(courtyard_source, "rect", &context);
        assert_keys(rect, &["at", "size"], &format!("{context}.courtyard.rect"));
        let (cx, cy) = pair(field(rect, "at", &context), &context);
        let (cw, ch) = pair(field(rect, "size", &context), &context);
        assert_eq!((courtyard.at.0.femto, courtyard.at.1.femto), (cx, cy));
        assert_eq!(courtyard.size.len(), 2);
        assert_eq!((courtyard.size[0].femto, courtyard.size[1].femto), (cw, ch));

        let reference = pair(field(row, "reference_at", &context), &context);
        let parsed_reference = footprint
            .silkscreen_ref
            .as_ref()
            .expect("generated silkscreen reference");
        assert_eq!(
            (parsed_reference.0.femto, parsed_reference.1.femto),
            reference
        );
        let pin_one = field(row, "pin_1_polygon", &context);
        assert_keys(
            pin_one,
            &["points", "source_stroke_width"],
            &format!("{context}.pin_1_polygon"),
        );
        assert_eq!(str_field(pin_one, "source_stroke_width", &context), "0.12");
        documented_silk_projections += 1;
        let expected_points = array(field(pin_one, "points", &context), &context)
            .iter()
            .map(|value| pair(value, &context))
            .collect::<Vec<_>>();
        let silk = footprint.silkscreen.as_ref().expect("generated pin-1 silk");
        assert_eq!(silk.items.len(), 1);
        match &silk.items[0] {
            SilkItem::Graphic(SilkGraphic::Polygon { points, fill }, _) => {
                assert_eq!(*fill, SilkFill::Solid);
                assert_eq!(
                    points
                        .iter()
                        .map(|point| (point.0.femto, point.1.femto))
                        .collect::<Vec<_>>(),
                    expected_points
                );
            }
            other => panic!("{context}: expected one filled pin-1 polygon, got {other:?}"),
        }
    }

    assert_eq!(pad_count, 9147);
    assert_eq!(circle_paste_count, 8045);
    assert_eq!(follow_copper_count, 1102);
    assert_eq!(mask_count, 8045);
    assert_eq!(roundrect_count, 1102);
    assert_eq!(
        rotated_count, 0,
        "the pinned sources currently use zero rotation"
    );
    assert_eq!(exact_courtyards, 90);
    assert_eq!(projected_courtyards, 13);
    assert_eq!(documented_silk_projections, 103);

    let mut emitted_by_fq = BTreeMap::new();
    for package in ["bga", "csp", "qfp", "soic"] {
        let world = worlds.get_mut(package).unwrap();
        let ir = insert_emitter_witnesses(package, world, &grouped[package]);
        let emitted = emit::kicad_mod::emit_kicad_mods(world, &ir);
        assert_eq!(emitted.len(), grouped[package].len());
        for (fq, _, text) in emitted {
            assert!(emitted_by_fq.insert(fq, text).is_none());
        }
    }
    assert_eq!(emitted_by_fq.len(), 103);

    let mut emitted_pad_lines = 0usize;
    let mut emitted_silk_hairlines = 0usize;
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("emitted footprint {row_index}");
        let package = library_package(str_field(row, "kicad_name", &context));
        let fq = format!("{package}::{}", str_field(row, "public_name", &context));
        let world = &worlds[package];
        let footprint = &world.footprints[&fq];
        let module = &emitted_by_fq[&fq];
        for placement in &footprint.pads {
            let pad = &world.pads[&placement.pad.name];
            let shape = pad.shape.unwrap().0;
            let (width, height) = match pad.size.as_slice() {
                [diameter] => (diameter, diameter),
                [width, height] => (width, height),
                other => panic!("{context}: bad pad size arity {}", other.len()),
            };
            let angle = if placement.rotate == 0 {
                String::new()
            } else {
                format!(" {}", placement.rotate)
            };
            let paste_override = pad.paste.is_some();
            let layers = if paste_override {
                "\"F.Cu\" \"F.Mask\""
            } else {
                "\"F.Cu\" \"F.Paste\" \"F.Mask\""
            };
            let (kshape, corner) = match shape {
                PadShape::Circle => ("circle", String::new()),
                PadShape::Rect if pad.corner_radius.is_some() => {
                    ("roundrect", " (roundrect_rratio 0.25)".to_string())
                }
                other => panic!("{context}: unprojected copper shape {other:?}"),
            };
            let mask = pad.mask_expansion.as_ref().map_or(String::new(), |value| {
                format!(" (solder_mask_margin {})", emit::geom::mm(&value.0))
            });
            let copper_line = format!(
                "  (pad \"{}\" smd {} (at {} {}{}) (size {} {}) (layers {}){}{})",
                placement.number.text,
                kshape,
                emit::geom::mm(&placement.x),
                emit::geom::mm(&placement.y),
                angle,
                emit::geom::mm(width),
                emit::geom::mm(height),
                layers,
                corner,
                mask,
            );
            assert_eq!(
                line_occurrences(module, &copper_line),
                1,
                "{context}: missing/duplicate emitted copper:\n{copper_line}"
            );
            emitted_pad_lines += 1;
            if let Some((PadPaste::Circle(diameter), _)) = &pad.paste {
                let paste_line = format!(
                    "  (pad \"\" smd circle (at {} {}{}) (size {} {}) (layers \"F.Paste\"))",
                    emit::geom::mm(&placement.x),
                    emit::geom::mm(&placement.y),
                    angle,
                    emit::geom::mm(diameter),
                    emit::geom::mm(diameter),
                );
                assert_eq!(
                    line_occurrences(module, &paste_line),
                    1,
                    "{context}: missing/duplicate emitted circular paste:\n{paste_line}"
                );
                emitted_pad_lines += 1;
            }
        }
        assert_eq!(
            module
                .lines()
                .filter(|line| line.starts_with("  (pad \""))
                .count(),
            footprint.pads.len()
                + footprint
                    .pads
                    .iter()
                    .filter(|placement| matches!(
                        world.pads[&placement.pad.name].paste,
                        Some((PadPaste::Circle(_), _))
                    ))
                    .count(),
            "{context}: no extra emitted padstack members"
        );

        let courtyard = footprint.courtyard.as_ref().unwrap();
        let courtyard_line = format!(
            "  (fp_rect (start {} {}) (end {} {}) (layer \"F.CrtYd\") (stroke (width 0.05) (type solid)) (fill none))",
            emit::geom::corner_lo(&courtyard.at.0, &courtyard.size[0]),
            emit::geom::corner_lo(&courtyard.at.1, &courtyard.size[1]),
            emit::geom::corner_hi(&courtyard.at.0, &courtyard.size[0]),
            emit::geom::corner_hi(&courtyard.at.1, &courtyard.size[1]),
        );
        assert_eq!(
            line_occurrences(module, &courtyard_line),
            1,
            "{context}: courtyard"
        );
        let reference = footprint.silkscreen_ref.as_ref().unwrap();
        let reference_line = format!(
            "  (fp_text reference \"REF**\" (at {} {}) (layer \"F.SilkS\"))",
            emit::geom::mm(&reference.0),
            emit::geom::mm(&reference.1),
        );
        assert_eq!(
            line_occurrences(module, &reference_line),
            1,
            "{context}: reference anchor"
        );

        let pin_one = field(row, "pin_1_polygon", &context);
        let points = array(field(pin_one, "points", &context), &context)
            .iter()
            .map(|value| {
                let values = array(value, &context);
                format!(
                    "(xy {} {})",
                    string(&values[0], &context),
                    string(&values[1], &context)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let silk_line = format!(
            "  (fp_poly (pts {points}) (layer \"F.SilkS\") (stroke (width 0.05) (type solid)) (fill solid))"
        );
        assert_eq!(
            str_field(pin_one, "source_stroke_width", &context),
            "0.12",
            "{context}: source side of the documented stroke projection"
        );
        assert_eq!(
            line_occurrences(module, &silk_line),
            1,
            "{context}: emitted side of the documented stroke projection"
        );
        emitted_silk_hairlines += 1;
    }
    assert_eq!(emitted_pad_lines, 9147 + 8045);
    assert_eq!(emitted_silk_hairlines, 103);
}

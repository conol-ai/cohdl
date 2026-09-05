//! Round-trip proof for the pinned Espressif footprint projection.
//!
//! The normalized snapshot is the review boundary.  Every generated pad stack
//! and placement is compared with the resolved CoHDL declarations.  The test
//! deliberately proves the documented projection rather than claiming that
//! CoHDL can express arbitrary source courtyard, keepout, fabrication, or
//! silkscreen primitives.

use cohdl::ast::{
    AvlEntry, AvlField, Ident, PadDrill, PadPaste, PartDef, Pin1Shape, SilkGraphic, SilkItem,
    TypeRef,
};
use cohdl::ir::{DesignIr, IrInstance, LayoutIr};
use cohdl::span::{FileId, Span};
use cohdl::{emit, pipeline};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SNAPSHOT_SHA256: &str = "fca1484784d12c8224bdbaefda665a38d1baae89544f76aaabc15d4b123101b1";

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

fn usize_field(value: &Value, name: &str, context: &str) -> usize {
    field(value, name, context)
        .as_u64()
        .unwrap_or_else(|| panic!("{context}.{name}: expected unsigned integer")) as usize
}

fn assert_keys(value: &Value, expected: &[&str], context: &str) {
    let actual = object(value, context)
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{context}: schema keys changed");
}

fn assert_sha256(value: &str, context: &str) {
    assert_eq!(value.len(), 64, "{context}: SHA-256 length");
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{context}: expected lowercase hexadecimal SHA-256"
    );
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
        "bad canonical millimetres {text}"
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

fn load_world() -> cohdl::resolve::World {
    let root = root();
    let deps = vec![
        ("std".to_string(), root.join("lib/std")),
        ("qfn".to_string(), root.join("lib/qfn")),
    ];
    let dep_names = deps
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let project = cohdl::project::load_project_with_deps(&root.join("lib/@espressif/esp32"), &deps)
        .expect("load generated ESP32 package and focused dependencies");
    let mut checked =
        pipeline::check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .expect("ESP32 package selection");
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "generated ESP32/qfn geometry must parse, resolve, and check:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked.world
}

fn fq_name(row: &Value, context: &str) -> String {
    format!(
        "{}::{}",
        pipeline::package_root(str_field(row, "owner", context)),
        str_field(row, "public_name", context)
    )
}

fn assert_pad_stack(stack: &Value, pad: &cohdl::ast::PadDef, context: &str) {
    let stack_obj = object(stack, context);
    let mut keys = vec!["layer", "plating", "shape", "size"];
    for optional in [
        "chamfer",
        "corner_radius",
        "drill",
        "mask_expansion",
        "paste",
    ] {
        if stack_obj.contains_key(optional) {
            keys.push(optional);
        }
    }
    assert_keys(stack, &keys, context);

    assert_eq!(
        pad.shape.map(|(shape, _)| shape.name()),
        Some(str_field(stack, "shape", context)),
        "{context}: copper shape"
    );
    let expected_size = array(field(stack, "size", context), context)
        .iter()
        .map(|value| femto(string(value, context)))
        .collect::<Vec<_>>();
    assert_eq!(
        pad.size.iter().map(|value| value.femto).collect::<Vec<_>>(),
        expected_size,
        "{context}: copper size"
    );
    assert_eq!(
        pad.layer.map(|(layer, _)| layer.name()),
        Some(str_field(stack, "layer", context)),
        "{context}: copper layer"
    );
    assert_eq!(
        pad.plating.map(|(plating, _)| plating.name()),
        Some(str_field(stack, "plating", context)),
        "{context}: plating"
    );

    match (
        stack_obj.get("drill"),
        pad.drill.as_ref().map(|(drill, _)| drill),
    ) {
        (None, None) => {}
        (Some(expected), Some(PadDrill::Round(actual))) => {
            let expected = array(expected, context);
            assert_eq!(expected.len(), 1, "{context}: round drill arity");
            assert_eq!(actual.femto, femto(string(&expected[0], context)));
        }
        (Some(expected), Some(PadDrill::Slot(actual_w, actual_h))) => {
            let expected = array(expected, context);
            assert_eq!(expected.len(), 2, "{context}: slot drill arity");
            assert_eq!(actual_w.femto, femto(string(&expected[0], context)));
            assert_eq!(actual_h.femto, femto(string(&expected[1], context)));
        }
        other => panic!("{context}: drill mismatch {other:?}"),
    }

    match (stack_obj.get("chamfer"), pad.chamfer.as_ref()) {
        (None, None) => {}
        (Some(expected), Some((corner, cut, _))) => {
            assert_keys(expected, &["corner", "cut"], &format!("{context}.chamfer"));
            assert_eq!(corner.name(), str_field(expected, "corner", context));
            assert_eq!(cut.femto, femto(str_field(expected, "cut", context)));
        }
        other => panic!("{context}: chamfer mismatch {other:?}"),
    }

    match (stack_obj.get("corner_radius"), pad.corner_radius.as_ref()) {
        (None, None) => {}
        (Some(expected), Some((actual, _))) => {
            assert_eq!(actual.femto, femto(string(expected, context)));
        }
        other => panic!("{context}: corner-radius mismatch {other:?}"),
    }
    match (stack_obj.get("mask_expansion"), pad.mask_expansion.as_ref()) {
        (None, None) => {}
        (Some(expected), Some((actual, _))) => {
            assert_eq!(actual.femto, femto(string(expected, context)));
        }
        other => panic!("{context}: mask-expansion mismatch {other:?}"),
    }

    let expected_paste = stack_obj.get("paste").map(|value| string(value, context));
    let actual_paste = pad.paste.as_ref().map(|(paste, _)| paste);
    match (
        expected_paste,
        pad.plating.map(|(value, _)| value),
        actual_paste,
    ) {
        (Some("follow_copper"), Some(cohdl::ast::PadPlating::Smd), None) => {}
        (Some("none"), Some(cohdl::ast::PadPlating::Smd), Some(PadPaste::None)) => {}
        // Through-hole pads intrinsically have no stencil layer, so the
        // normalized stack omits the inapplicable paste property.
        (None, Some(cohdl::ast::PadPlating::PlatedThroughHole), None) => {}
        other => panic!("{context}: paste mismatch {other:?}"),
    }
}

fn snapshot_bbox(pad: &Value, context: &str) -> (i128, i128, i128, i128) {
    let (x, y) = pair(field(pad, "at", context), context);
    let stack = field(pad, "stack", context);
    let size = array(field(stack, "size", context), context);
    let mut width = femto(string(&size[0], context));
    let mut height = if size.len() == 1 {
        width
    } else {
        femto(string(&size[1], context))
    };
    let rotation = field(pad, "rotation", context)
        .as_u64()
        .expect("whole pad rotation");
    assert_eq!(
        rotation % 90,
        0,
        "{context}: containment needs cardinal geometry"
    );
    if rotation % 180 == 90 {
        std::mem::swap(&mut width, &mut height);
    }
    (x - width / 2, y - height / 2, x + width / 2, y + height / 2)
}

#[test]
fn esp32_generated_footprints_round_trip_every_normalized_pad() {
    let snapshot_path = root().join("tools/esp32_footprint_data/footprints.json");
    let bytes = std::fs::read(&snapshot_path).unwrap();
    assert_eq!(
        cohdl::hash::sha256_hex(&bytes),
        SNAPSHOT_SHA256,
        "the whole normalized ESP32 geometry snapshot is a review boundary"
    );
    let snapshot: Value = serde_json::from_slice(&bytes).expect("valid normalized JSON");
    assert_keys(
        &snapshot,
        &[
            "coverage",
            "footprints",
            "projection_contract",
            "schema_version",
            "sources",
        ],
        "snapshot",
    );
    assert_eq!(field(&snapshot, "schema_version", "snapshot"), 1);

    let coverage = field(&snapshot, "coverage", "snapshot");
    assert_keys(
        coverage,
        &[
            "direct_cad_footprints",
            "direct_cad_evidence_files",
            "footprints",
            "footprints_by_owner",
            "module_footprints",
            "placements",
        ],
        "coverage",
    );
    assert_eq!(usize_field(coverage, "footprints", "coverage"), 55);
    assert_eq!(usize_field(coverage, "placements", "coverage"), 3237);
    assert_eq!(usize_field(coverage, "module_footprints", "coverage"), 36);
    assert_eq!(
        usize_field(coverage, "direct_cad_footprints", "coverage"),
        19
    );
    assert_eq!(
        usize_field(coverage, "direct_cad_evidence_files", "coverage"),
        21
    );
    let owners = field(coverage, "footprints_by_owner", "coverage");
    assert_keys(owners, &["@espressif/esp32", "qfn"], "coverage owners");
    assert_eq!(usize_field(owners, "@espressif/esp32", "owners"), 40);
    assert_eq!(usize_field(owners, "qfn", "owners"), 15);

    let projection = field(&snapshot, "projection_contract", "snapshot");
    assert_keys(
        projection,
        &[
            "coordinates",
            "copper",
            "courtyard",
            "mask",
            "paste",
            "silkscreen",
            "unnumbered",
        ],
        "projection contract",
    );
    assert!(str_field(projection, "copper", "projection").contains("rotation"));
    assert!(str_field(projection, "paste", "projection").contains("level 123"));
    assert!(str_field(projection, "paste", "projection").contains("containment-proved"));
    assert!(str_field(projection, "courtyard", "projection").contains("conservative"));
    assert!(str_field(projection, "silkscreen", "projection").contains("keepout"));

    let sources = field(&snapshot, "sources", "snapshot");
    assert_keys(
        sources,
        &[
            "espressif_direct_cad",
            "espressif_kicad",
            "kicad_footprints_secondary",
        ],
        "sources",
    );
    let esp = field(sources, "espressif_kicad", "sources");
    assert_keys(
        esp,
        &["commit", "license", "license_sha256", "repository"],
        "Espressif KiCad source",
    );
    assert_eq!(
        str_field(esp, "commit", "Espressif KiCad source"),
        "1dfc3110895c9cd62daf332f49c49ee0ee200831"
    );
    assert_eq!(
        str_field(esp, "license_sha256", "Espressif KiCad source"),
        "6eb43c2548ac6714db47ccbd62354bd194e918f606b071a5e9893680b941d75a"
    );
    let secondary = field(sources, "kicad_footprints_secondary", "sources");
    assert_keys(
        secondary,
        &[
            "commit",
            "files",
            "license",
            "license_sha256",
            "repository",
            "role",
        ],
        "secondary KiCad source",
    );
    assert_eq!(
        str_field(secondary, "commit", "secondary KiCad source"),
        "819223b66f96508feaeaa305301b5e6bb5c1038b"
    );
    assert_eq!(
        str_field(secondary, "license_sha256", "secondary KiCad source"),
        "45d2bce75e5a4208f5afb01b8fb2c406e700371c4fe2b5f5cd5c443d46db4d8f"
    );
    let secondary_files = array(field(secondary, "files", "secondary"), "secondary files");
    assert_eq!(secondary_files.len(), 8);
    for (index, source) in secondary_files.iter().enumerate() {
        let context = format!("secondary files[{index}]");
        assert_keys(source, &["path", "sha256"], &context);
        assert!(str_field(source, "path", &context).ends_with(".kicad_mod"));
        assert_sha256(str_field(source, "sha256", &context), &context);
    }
    let direct = field(sources, "espressif_direct_cad", "sources");
    assert_keys(
        direct,
        &[
            "archives",
            "coordinate_projection",
            "coordinate_unit",
            "redistribution",
            "source_page",
        ],
        "direct CAD source",
    );
    assert_eq!(
        str_field(direct, "coordinate_unit", "direct CAD source"),
        "1/1500000mm"
    );
    assert_eq!(
        str_field(direct, "coordinate_projection", "direct CAD source"),
        "ROUND_HALF_UP to 0.000000000000001mm"
    );
    assert_eq!(
        str_field(direct, "redistribution", "direct CAD source"),
        "normalized facts only; raw CAD is not bundled"
    );
    let archives = field(direct, "archives", "direct CAD source");
    assert_keys(
        archives,
        &["soc/ESP32-C6_Footprint_0.zip"],
        "direct archives",
    );
    assert_sha256(
        str_field(archives, "soc/ESP32-C6_Footprint_0.zip", "direct archives"),
        "C6 source archive",
    );

    let world = load_world();
    let rows = array(field(&snapshot, "footprints", "snapshot"), "footprints");
    assert_eq!(rows.len(), 55);
    let mut previous_key: Option<(String, String)> = None;
    let mut owner_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut source_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut shape_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut rotation_counts: BTreeMap<u64, usize> = BTreeMap::new();
    let mut placement_count = 0usize;
    let mut chamfer_count = 0usize;
    let mut corner_radius_count = 0usize;
    let mut paste_none_count = 0usize;
    let mut paste_overlay_count = 0usize;
    let mut thermal_via_count = 0usize;
    let mut repeated_groups = 0usize;
    let mut repeated_extra_placements = 0usize;
    let mut keepout_polygons = 0usize;
    let mut keepout_edges = 0usize;
    let mut exact_courtyards = 0usize;
    let mut projected_courtyards = 0usize;
    let mut synthesized_courtyards = 0usize;
    let mut duplicate_source_placements = 0usize;
    let mut unnumbered_source_pads = 0usize;
    let mut omitted_top_mask_polygons = 0usize;
    let mut omitted_bottom_mask_polygons = 0usize;
    let mut paste_containment_proofs = 0usize;
    let mut alternate_source_count = 0usize;

    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("footprints[{row_index}]");
        let mut row_keys = vec![
            "courtyard",
            "keepout_guides",
            "normalization",
            "owner",
            "pads",
            "pin_1_marker",
            "public_name",
            "reference_at",
            "source",
        ];
        if object(row, &context).contains_key("alternate_sources") {
            row_keys.push("alternate_sources");
        }
        assert_keys(row, &row_keys, &context);
        let owner = str_field(row, "owner", &context);
        assert!(matches!(owner, "qfn" | "@espressif/esp32"));
        *owner_counts.entry(owner.to_string()).or_default() += 1;
        let public_name = str_field(row, "public_name", &context);
        let key = (owner.to_string(), public_name.to_string());
        if let Some(previous) = &previous_key {
            assert!(previous < &key, "snapshot rows must be uniquely sorted");
        }
        previous_key = Some(key);

        let source = field(row, "source", &context);
        let source_kind = str_field(source, "kind", &context);
        *source_counts.entry(source_kind.to_string()).or_default() += 1;
        match source_kind {
            "espressif_kicad" => {
                assert_keys(source, &["kind", "name", "path", "sha256"], &context);
                assert!(str_field(source, "path", &context).ends_with(".kicad_mod"));
            }
            "espressif_direct_cad" => {
                let mut source_keys = vec!["kind", "member", "path", "sha256", "url"];
                if object(source, &context).contains_key("archive_sha256") {
                    source_keys.push("archive_sha256");
                    assert_sha256(str_field(source, "archive_sha256", &context), &context);
                }
                assert_keys(source, &source_keys, &context);
                assert!(
                    str_field(source, "url", &context).starts_with("https://www.espressif.com/")
                );
            }
            other => panic!("{context}: unsupported source kind {other}"),
        }
        assert_sha256(str_field(source, "sha256", &context), &context);
        if let Some(alternates) = object(row, &context).get("alternate_sources") {
            for (alternate_index, alternate) in array(alternates, &context).iter().enumerate() {
                let alternate_context = format!("{context}.alternate_sources[{alternate_index}]");
                assert_keys(
                    alternate,
                    &["kind", "member", "path", "sha256", "url"],
                    &alternate_context,
                );
                assert_eq!(
                    str_field(alternate, "kind", &alternate_context),
                    "espressif_direct_cad"
                );
                assert_sha256(
                    str_field(alternate, "sha256", &alternate_context),
                    &alternate_context,
                );
                assert!(str_field(alternate, "url", &alternate_context)
                    .starts_with("https://www.espressif.com/"));
                alternate_source_count += 1;
            }
        }

        let fq = fq_name(row, &context);
        let footprint = world
            .footprints
            .get(&fq)
            .unwrap_or_else(|| panic!("{context}: missing resolved `{fq}`"));
        assert!(
            footprint.mount_holes.is_empty(),
            "{context}: unexpected mount hole"
        );
        let pads = array(field(row, "pads", &context), &format!("{context}.pads"));
        assert_eq!(
            footprint.pads.len(),
            pads.len(),
            "{context}: placement count"
        );

        let snapshot_numbers = pads
            .iter()
            .map(|pad| str_field(pad, "number", &context))
            .collect::<BTreeSet<_>>();
        let parsed_numbers = footprint
            .pads
            .iter()
            .map(|pad| pad.number.text.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            parsed_numbers, snapshot_numbers,
            "{context}: pad-number set"
        );
        assert!(parsed_numbers.iter().all(|number| !number.is_empty()));

        let mut number_counts: BTreeMap<&str, usize> = BTreeMap::new();
        for (pad_index, (source_pad, placement)) in pads.iter().zip(&footprint.pads).enumerate() {
            let pad_context = format!("{context}.pads[{pad_index}]");
            let mut pad_keys = vec!["at", "number", "rotation", "stack"];
            if source_kind == "espressif_kicad" {
                pad_keys.push("source_number");
            }
            if object(source_pad, &pad_context).contains_key("projection_role") {
                pad_keys.push("projection_role");
            }
            assert_keys(source_pad, &pad_keys, &pad_context);
            let number = str_field(source_pad, "number", &pad_context);
            *number_counts.entry(number).or_default() += 1;
            assert_eq!(placement.number.text, number, "{pad_context}: number");
            let (x, y) = pair(field(source_pad, "at", &pad_context), &pad_context);
            assert_eq!((placement.x.femto, placement.y.femto), (x, y));
            let rotation = field(source_pad, "rotation", &pad_context)
                .as_u64()
                .expect("whole pad rotation");
            assert_eq!(
                u64::from(placement.rotate),
                rotation,
                "{pad_context}: rotation"
            );
            *rotation_counts.entry(rotation).or_default() += 1;

            let pad = world
                .pads
                .get(&placement.pad.name)
                .unwrap_or_else(|| panic!("{pad_context}: unresolved pad symbol"));
            let stack = field(source_pad, "stack", &pad_context);
            assert_pad_stack(stack, pad, &format!("{pad_context}.stack"));
            let shape = str_field(stack, "shape", &pad_context).to_string();
            *shape_counts.entry(shape).or_default() += 1;
            chamfer_count += usize::from(object(stack, &pad_context).contains_key("chamfer"));
            corner_radius_count +=
                usize::from(object(stack, &pad_context).contains_key("corner_radius"));
            paste_none_count += usize::from(
                object(stack, &pad_context)
                    .get("paste")
                    .is_some_and(|value| string(value, &pad_context) == "none"),
            );
            if let Some(role) = object(source_pad, &pad_context).get("projection_role") {
                match string(role, &pad_context) {
                    "contained_paste_overlay" => {
                        paste_overlay_count += 1;
                        assert_eq!(str_field(stack, "paste", &pad_context), "follow_copper");
                        let overlay = snapshot_bbox(source_pad, &pad_context);
                        let contained = pads.iter().any(|candidate| {
                            str_field(candidate, "number", &pad_context) == number
                                && object(field(candidate, "stack", &pad_context), &pad_context)
                                    .get("paste")
                                    .is_some_and(|value| string(value, &pad_context) == "none")
                                && {
                                    let container = snapshot_bbox(candidate, &pad_context);
                                    container.0 <= overlay.0
                                        && container.1 <= overlay.1
                                        && container.2 >= overlay.2
                                        && container.3 >= overlay.3
                                }
                        });
                        assert!(
                            contained,
                            "{pad_context}: paste overlay escapes source copper"
                        );
                    }
                    "thermal_via" => {
                        thermal_via_count += 1;
                        assert_eq!(
                            str_field(stack, "plating", &pad_context),
                            "plated_through_hole"
                        );
                        assert_eq!(str_field(stack, "layer", &pad_context), "through_all");
                    }
                    other => panic!("{pad_context}: unsupported projection role {other}"),
                }
            }
            placement_count += 1;
        }
        for count in number_counts.values().copied().filter(|count| *count > 1) {
            repeated_groups += 1;
            repeated_extra_placements += count - 1;
        }

        let courtyard_source = field(row, "courtyard", &context);
        let mut courtyard_keys = vec!["at", "projection", "size"];
        if object(courtyard_source, &context).contains_key("source_primitives") {
            courtyard_keys.push("source_primitives");
        }
        assert_keys(
            courtyard_source,
            &courtyard_keys,
            &format!("{context}.courtyard"),
        );
        match str_field(courtyard_source, "projection", &context) {
            "exact_rect" => exact_courtyards += 1,
            "conservative_axis_aligned_bounding_rect" => projected_courtyards += 1,
            "synthesized_pad_bounding_rect" => synthesized_courtyards += 1,
            other => panic!("{context}: unsupported courtyard projection {other}"),
        }
        let courtyard = footprint.courtyard.as_ref().expect("generated courtyard");
        assert_eq!(courtyard.shape.0, cohdl::ast::PadShape::Rect);
        assert_eq!(
            (courtyard.at.0.femto, courtyard.at.1.femto),
            pair(field(courtyard_source, "at", &context), &context)
        );
        assert_eq!(courtyard.size.len(), 2);
        assert_eq!(
            (courtyard.size[0].femto, courtyard.size[1].femto),
            pair(field(courtyard_source, "size", &context), &context)
        );

        let reference = footprint
            .silkscreen_ref
            .as_ref()
            .expect("generated reference anchor");
        assert_eq!(
            (reference.0.femto, reference.1.femto),
            pair(field(row, "reference_at", &context), &context)
        );

        let wants_pin_one = field(row, "pin_1_marker", &context)
            .as_bool()
            .expect("pin_1_marker boolean");
        let guides = array(field(row, "keepout_guides", &context), &context);
        keepout_polygons += guides.len();
        let silk = footprint.silkscreen.as_ref().expect("generated silk block");
        let mut item_index = 0usize;
        if wants_pin_one {
            match &silk.items[item_index] {
                SilkItem::Pin1Marker { pad, shape, .. } => {
                    assert_eq!(pad.text, "1");
                    assert_eq!(*shape, Pin1Shape::Dot);
                }
                other => panic!("{context}: expected semantic pin-1 marker, got {other:?}"),
            }
            item_index += 1;
        }
        for (guide_index, guide) in guides.iter().enumerate() {
            let points = array(guide, &context);
            assert!(points.len() >= 3);
            keepout_edges += points.len();
            for edge in 0..points.len() {
                let expected_from = pair(&points[edge], &context);
                let expected_to = pair(&points[(edge + 1) % points.len()], &context);
                match &silk.items[item_index] {
                    SilkItem::Graphic(SilkGraphic::Line { from, to, width }, _) => {
                        assert_eq!((from.0.femto, from.1.femto), expected_from);
                        assert_eq!((to.0.femto, to.1.femto), expected_to);
                        assert_eq!(width.femto, femto("0.12"));
                    }
                    other => panic!(
                        "{context}.keepout_guides[{guide_index}]: expected exact line, got {other:?}"
                    ),
                }
                item_index += 1;
            }
        }
        assert_eq!(
            silk.items.len(),
            item_index,
            "{context}: extra silk projection"
        );

        let normalization = field(row, "normalization", &context);
        match source_kind {
            "espressif_kicad" => {
                assert_keys(
                    normalization,
                    &["identical_duplicate_pads_removed", "unnumbered_source_pads"],
                    &context,
                );
                unnumbered_source_pads +=
                    usize_field(normalization, "unnumbered_source_pads", &context);
            }
            "espressif_direct_cad" => {
                assert_keys(
                    normalization,
                    &[
                        "decal_name",
                        "identical_duplicate_pads_removed",
                        "paste_containment_proofs",
                        "paste_overlays",
                        "source_mask_polygons_not_projected",
                        "thermal_vias",
                    ],
                    &context,
                );
                let masks = field(
                    normalization,
                    "source_mask_polygons_not_projected",
                    &context,
                );
                assert_keys(
                    masks,
                    &["bottom", "top"],
                    &format!("{context}.source masks"),
                );
                omitted_top_mask_polygons += usize_field(masks, "top", &context);
                omitted_bottom_mask_polygons += usize_field(masks, "bottom", &context);
                paste_containment_proofs +=
                    usize_field(normalization, "paste_containment_proofs", &context);
                assert_eq!(usize_field(normalization, "paste_overlays", &context), 0);
            }
            _ => unreachable!(),
        }
        duplicate_source_placements +=
            usize_field(normalization, "identical_duplicate_pads_removed", &context);
    }

    assert_eq!(
        owner_counts,
        BTreeMap::from([
            ("@espressif/esp32".to_string(), 40),
            ("qfn".to_string(), 15),
        ])
    );
    assert_eq!(
        source_counts,
        BTreeMap::from([
            ("espressif_direct_cad".to_string(), 19),
            ("espressif_kicad".to_string(), 36),
        ])
    );
    assert_eq!(
        shape_counts,
        BTreeMap::from([
            ("circle".to_string(), 241),
            ("oval".to_string(), 532),
            ("rect".to_string(), 2464),
        ])
    );
    assert_eq!(
        rotation_counts,
        BTreeMap::from([(0, 1718), (45, 17), (90, 1120), (180, 381), (315, 1),])
    );
    assert_eq!(placement_count, 3237);
    assert_eq!(chamfer_count, 19);
    assert_eq!(corner_radius_count, 89);
    assert_eq!(paste_none_count, 0);
    assert_eq!(paste_overlay_count, 0);
    assert_eq!(thermal_via_count, 235);
    assert_eq!(repeated_groups, 55);
    assert_eq!(repeated_extra_placements, 645);
    assert_eq!(keepout_polygons, 20);
    assert_eq!(keepout_edges, 80);
    assert_eq!(exact_courtyards, 1);
    assert_eq!(projected_courtyards, 53);
    assert_eq!(synthesized_courtyards, 1);
    assert_eq!(duplicate_source_placements, 12);
    assert_eq!(unnumbered_source_pads, 9);
    assert_eq!(omitted_top_mask_polygons, 23);
    assert_eq!(omitted_bottom_mask_polygons, 71);
    assert_eq!(paste_containment_proofs, 0);
    assert_eq!(alternate_source_count, 2);
}

fn ident(name: impl Into<String>, span: Span) -> Ident {
    Ident {
        name: name.into(),
        span,
    }
}

fn emitter_witness(world: &mut cohdl::resolve::World, footprint: &str, index: usize) -> DesignIr {
    let span = Span::new(FileId(0), 0, 0);
    let part_name = format!("__ESP32_FOOTPRINT_WITNESS_{index:02}");
    world.parts.insert(
        part_name.clone(),
        PartDef {
            name: ident(part_name.clone(), span),
            device: TypeRef {
                name: ident("__EMITTER_ONLY", span),
                generic_args: Vec::new(),
                variant: None,
                span,
            },
            primary: AvlEntry {
                fields: vec![
                    AvlField {
                        name: ident("mfr", span),
                        value: "Espressif".to_string(),
                        span,
                    },
                    AvlField {
                        name: ident("mpn", span),
                        value: format!("ESP32-FOOTPRINT-WITNESS-{index:02}"),
                        span,
                    },
                ],
                footprint: Some(ident(footprint, span)),
                span,
            },
            alts: Vec::new(),
            span,
        },
    );
    let path = format!("Witness::{index:02}");
    DesignIr {
        name: format!("ESP32FootprintWitness{index:02}"),
        instances: BTreeMap::from([(
            path.clone(),
            IrInstance {
                path,
                device: "__EMITTER_ONLY".to_string(),
                variant: None,
                specs: BTreeMap::new(),
                part: Some(part_name),
                virtual_only: false,
                designator_override: None,
                designator: Some(format!("U{}", index + 1)),
                placement_hint: None,
                impl_traits: BTreeSet::new(),
                span,
            },
        )]),
        nets: Vec::new(),
        nc_pins: BTreeSet::new(),
        layout: LayoutIr::default(),
    }
}

fn numbered_pad_lines(text: &str, indent: &str, number: &str) -> usize {
    let prefix = format!("{indent}(pad \"{number}\" ");
    text.lines()
        .filter(|line| line.starts_with(&prefix))
        .count()
}

#[test]
fn esp32_representative_geometry_is_consistent_across_all_board_emitters() {
    let mut world = load_world();
    let representatives = [
        "qfn::ESPRESSIF_QFN48_0P4_6_E4P7",
        "qfn::ESPRESSIF_QFN40_0P4_5_E3P3",
        "qfn::ESPRESSIF_QFN56_0P4_7B",
        "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_A",
        "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_B",
        "espressif_esp32::FP_ESP32_C3_MINI_1",
        "espressif_esp32::FP_ESP32_C6_WROOM_1U",
        "espressif_esp32::FP_ESP32_PICO_D4",
        // The pre-existing hand-audited source remains a public compatibility
        // footprint beside the generated catalog.
        "espressif_esp32::FP_ESP32_S3_WROOM_1",
    ];

    for (index, fq) in representatives.iter().enumerate() {
        assert!(
            world.footprints.contains_key(*fq),
            "missing representative `{fq}`"
        );
        let ir = emitter_witness(&mut world, fq, index);
        let modules = emit::kicad_mod::emit_kicad_mods(&world, &ir);
        assert_eq!(modules.len(), 1, "{fq}: one standalone footprint");
        assert_eq!(modules[0].0, *fq);
        let module = &modules[0].2;
        assert_eq!(modules, emit::kicad_mod::emit_kicad_mods(&world, &ir));
        let ipc = emit::ipc2581::emit_ipc2581(&world, &ir, "esp32-footprint-test");
        assert_eq!(
            ipc,
            emit::ipc2581::emit_ipc2581(&world, &ir, "esp32-footprint-test")
        );
        let board = emit::kicad_pcb::emit_kicad_pcb(&world, &ir, "esp32-footprint-test");
        assert_eq!(
            board,
            emit::kicad_pcb::emit_kicad_pcb(&world, &ir, "esp32-footprint-test")
        );

        let footprint = &world.footprints[*fq];
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for place in &footprint.pads {
            *counts.entry(place.number.text.as_str()).or_default() += 1;
        }
        assert_eq!(
            module
                .lines()
                .filter(|line| line.starts_with("  (pad \""))
                .count(),
            footprint.pads.len(),
            "{fq}: standalone KiCad pad-plan cardinality"
        );
        assert_eq!(
            board
                .lines()
                .filter(|line| line.starts_with("\t\t(pad \""))
                .count(),
            footprint.pads.len(),
            "{fq}: native-board pad-plan cardinality"
        );
        assert_eq!(
            ipc.matches("          <Pin number=\"").count(),
            counts.len(),
            "{fq}: IPC package has one logical Pin per distinct number"
        );
        for (number, expected_occurrences) in &counts {
            assert_eq!(
                numbered_pad_lines(module, "  ", number),
                *expected_occurrences
            );
            assert_eq!(
                numbered_pad_lines(&board, "\t\t", number),
                *expected_occurrences
            );
            assert_eq!(
                ipc.matches(&format!("<Pin number=\"{number}\" ")).count(),
                1,
                "{fq}: IPC logical pin {number}"
            );
            assert!(
                ipc.matches(&format!("pin=\"{number}\"/>")).count() >= expected_occurrences * 2,
                "{fq}: IPC physical features retain every occurrence of {number}"
            );
        }

        let rotations = footprint
            .pads
            .iter()
            .filter(|place| place.rotate != 0)
            .map(|place| place.rotate)
            .collect::<BTreeSet<_>>();
        for rotation in rotations {
            assert!(
                module.contains(&format!(" {rotation}) (size")),
                "{fq}: KiCad rotation {rotation}"
            );
            assert!(
                board.contains(&format!(" {rotation})\n")),
                "{fq}: board rotation {rotation}"
            );
            assert!(
                ipc.contains(&format!("<Xform rotation=\"{rotation}\"/>")),
                "{fq}: IPC rotation {rotation}"
            );
        }

        let has_chamfer = footprint
            .pads
            .iter()
            .any(|place| world.pads[&place.pad.name].chamfer.is_some());
        if has_chamfer {
            assert!(
                module.contains("(chamfer ") || module.contains("(gr_poly"),
                "{fq}: standalone KiCad chamfer"
            );
            assert!(
                board.contains("(chamfer ") || board.contains("(gr_poly"),
                "{fq}: native-board chamfer"
            );
            assert!(
                ipc.contains("<Contour><Polygon>"),
                "{fq}: IPC chamfer contour"
            );
        }
        if counts.values().any(|count| *count > 1) {
            assert!(
                counts.iter().any(|(number, count)| {
                    *count > 1
                        && numbered_pad_lines(module, "  ", number) == *count
                        && numbered_pad_lines(&board, "\t\t", number) == *count
                }),
                "{fq}: repeated electrical number survives both KiCad dialects"
            );
        }
        let has_suppressed_paste = footprint
            .pads
            .iter()
            .any(|place| matches!(world.pads[&place.pad.name].paste, Some((PadPaste::None, _))));
        if has_suppressed_paste {
            assert!(module.contains("\"F.Cu\" \"F.Mask\""));
            assert!(board.contains("\"F.Cu\" \"F.Mask\""));
            assert!(ipc.contains("<LayerFeature layerRef=\"F.Paste\">"));
        }
        let has_thermal_via = footprint.pads.iter().any(|place| {
            matches!(
                world.pads[&place.pad.name].plating,
                Some((cohdl::ast::PadPlating::PlatedThroughHole, _))
            )
        });
        if has_thermal_via {
            assert!(module.contains(" thru_hole "));
            assert!(board.contains(" thru_hole "));
            assert!(ipc.contains("<PadstackHoleDef"));
            assert!(ipc.contains("<Hole"));
        }
        assert!(module.contains("F.SilkS"));
        assert!(board.contains("F.SilkS"));
        assert!(ipc.contains("F.Silkscreen"));
    }
}

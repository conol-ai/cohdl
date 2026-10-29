//! Library-wide supplier sourcing-data conformance (RFC-034, library plan A4).
//!
//! The audit deliberately accepts arbitrary checked worlds and JSON paths so
//! fixtures exercise exactly the same code as the shipped library.

use cohdl::ast::{AvlEntry, PartDef, SupplierBinding};
use cohdl::resolve::World;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

const FEMTO_PER_MM: f64 = 1e15;

pub struct CheckedPackage<'a> {
    pub name: &'a str,
    pub package_versions: &'a BTreeMap<String, String>,
    pub world: &'a World,
}

impl<'a> CheckedPackage<'a> {
    pub fn new(
        name: &'a str,
        package_versions: &'a BTreeMap<String, String>,
        world: &'a World,
    ) -> Self {
        Self {
            name,
            package_versions,
            world,
        }
    }
}

#[derive(Debug)]
struct MapRow {
    mfr: String,
    mpn: String,
    pn: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Mapping {
    dx_mm: f64,
    dy_mm: f64,
    rotate_deg: f64,
}

#[derive(Debug)]
struct FrameRow {
    package: String,
    package_version: String,
    footprint: String,
    supplier: String,
    pn: String,
    mapping: Mapping,
}

fn error(context: &str, message: impl AsRef<str>, errors: &mut Vec<String>) {
    errors.push(format!("{context}: {}", message.as_ref()));
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match object.get(field) {
        Some(Value::String(value)) if !value.trim().is_empty() => Some(value),
        Some(Value::String(_)) => {
            error(
                context,
                format!("`{field}` must be a non-empty string"),
                errors,
            );
            None
        }
        Some(_) => {
            error(context, format!("`{field}` must be a string"), errors);
            None
        }
        None => {
            error(context, format!("missing required `{field}`"), errors);
            None
        }
    }
}

fn schema_version(object: &Map<String, Value>, context: &str, errors: &mut Vec<String>) {
    match object.get("schema_version") {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => {}
        Some(_) => error(context, "`schema_version` must be the integer 1", errors),
        None => error(context, "missing required `schema_version`", errors),
    }
}

fn valid_date(value: &str) -> bool {
    let mut parts = value.split('-');
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    if year.len() != 4
        || month.len() != 2
        || day.len() != 2
        || !year
            .bytes()
            .chain(month.bytes())
            .chain(day.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn date_field(object: &Map<String, Value>, field: &str, context: &str, errors: &mut Vec<String>) {
    if let Some(value) = string_field(object, field, context, errors) {
        if !valid_date(value) {
            error(
                context,
                format!("`{field}` must be a real YYYY-MM-DD date, got `{value}`"),
                errors,
            );
        }
    }
}

fn map_context(object: &Map<String, Value>, index: usize) -> String {
    let identity = ["mfr", "mpn", "pn"]
        .iter()
        .filter_map(|field| object.get(*field).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" / ");
    if identity.is_empty() {
        format!("lcsc_map.json rows[{index}]")
    } else {
        format!("lcsc_map.json rows[{index}] ({identity})")
    }
}

fn load_map(path: &Path, errors: &mut Vec<String>) -> Vec<MapRow> {
    let value = match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => value,
            Err(e) => {
                error(
                    &path.display().to_string(),
                    format!("invalid JSON: {e}"),
                    errors,
                );
                return Vec::new();
            }
        },
        Err(e) => {
            error(
                &path.display().to_string(),
                format!("cannot read file: {e}"),
                errors,
            );
            return Vec::new();
        }
    };
    let context = path.display().to_string();
    let Some(object) = value.as_object() else {
        error(&context, "top level must be an object", errors);
        return Vec::new();
    };
    schema_version(object, &context, errors);
    if string_field(object, "supplier", &context, errors) != Some("lcsc") {
        error(&context, "`supplier` must be `lcsc`", errors);
    }
    let Some(rows) = object.get("rows").and_then(Value::as_array) else {
        error(&context, "missing required array `rows`", errors);
        return Vec::new();
    };
    let mut parsed = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(object) = row.as_object() else {
            error(
                &format!("lcsc_map.json rows[{index}]"),
                "row must be an object",
                errors,
            );
            continue;
        };
        let context = map_context(object, index);
        let before = errors.len();
        let mfr = string_field(object, "mfr", &context, errors);
        let mpn = string_field(object, "mpn", &context, errors);
        let pn = string_field(object, "pn", &context, errors);
        if let Some(rule) = string_field(object, "rule", &context, errors) {
            if !matches!(rule, "exact" | "alias" | "manual") {
                error(&context, "`rule` must be exact, alias, or manual", errors);
            }
        }
        if let Some(category) = string_field(object, "category", &context, errors) {
            if !matches!(category, "basic" | "preferred" | "extended") {
                error(
                    &context,
                    "`category` must be basic, preferred, or extended",
                    errors,
                );
            }
        }
        if let Some(sha) = string_field(object, "snapshot_sha256", &context, errors) {
            if sha.len() != 64
                || !sha
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                error(
                    &context,
                    "`snapshot_sha256` must be 64 lowercase hexadecimal characters",
                    errors,
                );
            }
        }
        date_field(object, "date", &context, errors);
        if errors.len() == before {
            parsed.push(MapRow {
                mfr: mfr.unwrap().to_string(),
                mpn: mpn.unwrap().to_string(),
                pn: pn.unwrap().to_string(),
            });
        }
    }
    parsed
}

fn frame_context(object: &Map<String, Value>, index: usize) -> String {
    let id = object.get("id").and_then(Value::as_str).unwrap_or("?");
    let package = object.get("package").and_then(Value::as_str).unwrap_or("?");
    let footprint = object
        .get("footprint")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let supplier = object
        .get("supplier")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let pn = object.get("pn").and_then(Value::as_str).unwrap_or("?");
    format!("jlc_frames.json rows[{index}] id `{id}` ({package}::{footprint}, {supplier} {pn})")
}

fn number_field(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<f64> {
    match object.get(field).and_then(Value::as_f64) {
        Some(value) if value.is_finite() => Some(value),
        _ => {
            error(
                context,
                format!("`{field}` must be a finite number"),
                errors,
            );
            None
        }
    }
}

fn mapping_value(
    value: Option<&Value>,
    field: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<Mapping> {
    let Some(object) = value.and_then(Value::as_object) else {
        error(context, format!("`{field}` must be an object"), errors);
        return None;
    };
    let before = errors.len();
    let dx_mm = number_field(object, "dx_mm", context, errors);
    let dy_mm = number_field(object, "dy_mm", context, errors);
    let rotate_deg = number_field(object, "rotate_deg", context, errors);
    if let Some(angle) = rotate_deg {
        if angle.fract() != 0.0 || !(0.0..360.0).contains(&angle) {
            error(
                context,
                "`rotate_deg` must be an integer in the RFC-034 range 0..360",
                errors,
            );
        }
    }
    (errors.len() == before).then(|| Mapping {
        dx_mm: dx_mm.unwrap(),
        dy_mm: dy_mm.unwrap(),
        rotate_deg: rotate_deg.unwrap(),
    })
}

fn audit_evidence_record(repo_root: &Path, record: &str, context: &str, errors: &mut Vec<String>) {
    let path = Path::new(record);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        error(
            context,
            format!("evidence record `{record}` must stay inside the repository"),
            errors,
        );
        return;
    }
    let candidate = repo_root.join(path);
    let inside = candidate
        .canonicalize()
        .ok()
        .zip(repo_root.canonicalize().ok())
        .is_some_and(|(candidate, root)| candidate.starts_with(root));
    if !inside || !candidate.is_file() {
        error(
            context,
            format!("evidence record `{record}` does not name an existing repository file"),
            errors,
        );
    }
}

fn load_frames(path: &Path, repo_root: &Path, errors: &mut Vec<String>) -> Vec<FrameRow> {
    let value = match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => value,
            Err(e) => {
                error(
                    &path.display().to_string(),
                    format!("invalid JSON: {e}"),
                    errors,
                );
                return Vec::new();
            }
        },
        Err(e) => {
            error(
                &path.display().to_string(),
                format!("cannot read file: {e}"),
                errors,
            );
            return Vec::new();
        }
    };
    let context = path.display().to_string();
    let Some(object) = value.as_object() else {
        error(&context, "top level must be an object", errors);
        return Vec::new();
    };
    schema_version(object, &context, errors);
    let Some(rows) = object.get("rows").and_then(Value::as_array) else {
        error(&context, "missing required array `rows`", errors);
        return Vec::new();
    };
    let mut parsed = Vec::new();
    let mut ids = BTreeSet::new();
    let mut evidence = BTreeMap::new();
    let mut references = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(object) = row.as_object() else {
            error(
                &format!("jlc_frames.json rows[{index}]"),
                "row must be an object",
                errors,
            );
            continue;
        };
        let context = frame_context(object, index);
        let before = errors.len();
        let id = string_field(object, "id", &context, errors);
        if let Some(id) = id {
            if !ids.insert(id.to_string()) {
                error(&context, format!("duplicate frame id `{id}`"), errors);
            }
        }
        let package = string_field(object, "package", &context, errors);
        let package_version = string_field(object, "package_version", &context, errors);
        let footprint = string_field(object, "footprint", &context, errors);
        let supplier = string_field(object, "supplier", &context, errors);
        let pn = string_field(object, "pn", &context, errors);
        let mapping = mapping_value(object.get("mapping"), "mapping", &context, errors);
        if let Some(measured) = object.get("measured") {
            if !measured.is_null() {
                mapping_value(Some(measured), "measured", &context, errors);
            }
        } else {
            error(&context, "missing required `measured`", errors);
        }

        let nonzero = mapping.as_ref().is_some_and(|mapping| {
            mapping.dx_mm != 0.0 || mapping.dy_mm != 0.0 || mapping.rotate_deg != 0.0
        });
        match object.get("exception") {
            Some(Value::String(reason)) if !reason.trim().is_empty() && nonzero => {}
            Some(Value::Null) if !nonzero => {}
            Some(Value::String(_)) if nonzero => error(
                &context,
                "non-zero mapping requires a non-empty `exception`",
                errors,
            ),
            Some(Value::String(reason)) => error(
                &context,
                format!("zero mapping requires `exception: null`, got `{reason}`"),
                errors,
            ),
            Some(Value::Null) => error(
                &context,
                "non-zero mapping requires a non-empty `exception`; null is forbidden",
                errors,
            ),
            Some(_) => error(&context, "`exception` must be null or a string", errors),
            None => error(&context, "missing required `exception`", errors),
        }

        let has_evidence = object.get("evidence").is_some_and(|value| !value.is_null());
        let has_reference = object
            .get("evidence_ref")
            .is_some_and(|value| !value.is_null());
        if has_evidence == has_reference {
            error(
                &context,
                "exactly one of `evidence` and `evidence_ref` must be non-null",
                errors,
            );
        }
        match object.get("evidence") {
            Some(Value::Object(evidence)) => {
                if let Some(method) = string_field(evidence, "method", &context, errors) {
                    if !matches!(method, "preview" | "product_page") {
                        error(
                            &context,
                            "evidence `method` must be preview or product_page",
                            errors,
                        );
                    }
                }
                date_field(evidence, "date", &context, errors);
                if let Some(record) = string_field(evidence, "record", &context, errors) {
                    audit_evidence_record(repo_root, record, &context, errors);
                }
            }
            Some(Value::Null) => {}
            Some(_) => error(&context, "`evidence` must be an object or null", errors),
            None => error(&context, "missing required `evidence`", errors),
        }
        let mut evidence_ref = None;
        match object.get("evidence_ref") {
            Some(Value::Object(reference)) => {
                evidence_ref = string_field(reference, "row", &context, errors).map(str::to_string);
                if let Some(reason) = string_field(reference, "reason", &context, errors) {
                    if !matches!(reason, "same_supplier_footprint" | "same_preview_board") {
                        error(
                            &context,
                            "evidence_ref `reason` must be same_supplier_footprint or same_preview_board",
                            errors,
                        );
                    }
                }
                string_field(reference, "note", &context, errors);
            }
            Some(Value::Null) => {}
            Some(_) => error(&context, "`evidence_ref` must be an object or null", errors),
            None => error(&context, "missing required `evidence_ref`", errors),
        }

        if let (Some(id), Some(has_evidence)) = (id, Some(has_evidence)) {
            evidence.insert(id.to_string(), has_evidence);
        }
        if let (Some(id), Some(target)) = (id, evidence_ref.clone()) {
            references.push((id.to_string(), target, context.clone()));
        }
        if errors.len() == before {
            parsed.push(FrameRow {
                package: package.unwrap().to_string(),
                package_version: package_version.unwrap().to_string(),
                footprint: footprint.unwrap().to_string(),
                supplier: supplier.unwrap().to_string(),
                pn: pn.unwrap().to_string(),
                mapping: mapping.unwrap(),
            });
        }
    }
    for (id, target, context) in references {
        match evidence.get(&target) {
            Some(true) => {}
            Some(false) => error(
                &context,
                format!("frame `{id}` references `{target}`, which does not have its own evidence"),
                errors,
            ),
            None => error(
                &context,
                format!("frame `{id}` references missing evidence row `{target}`"),
                errors,
            ),
        }
    }
    parsed
}

fn entries(part: &PartDef) -> impl Iterator<Item = &AvlEntry> {
    std::iter::once(&part.primary).chain(part.alts.iter())
}

fn mapping_of(binding: &SupplierBinding) -> Mapping {
    let coordinate = |value: &Option<cohdl::units::UnitValue>| {
        value
            .as_ref()
            .map_or(0.0, |value| value.femto as f64 / FEMTO_PER_MM)
    };
    let (dx_mm, dy_mm) = binding.at.as_ref().map_or((0.0, 0.0), |(x, y)| {
        (coordinate(&x.value), coordinate(&y.value))
    });
    let rotate_deg = binding
        .rotate
        .as_ref()
        .and_then(|(text, _)| text.parse::<f64>().ok())
        .unwrap_or(0.0);
    Mapping {
        dx_mm,
        dy_mm,
        rotate_deg,
    }
}

fn mapping_matches(actual: &Mapping, expected: &Mapping) -> bool {
    const TOLERANCE_MM: f64 = 1e-9;
    (actual.dx_mm - expected.dx_mm).abs() <= TOLERANCE_MM
        && (actual.dy_mm - expected.dy_mm).abs() <= TOLERANCE_MM
        && actual.rotate_deg == expected.rotate_deg
}

/// Audit all supplier bindings owned by the supplied checked package worlds.
/// JSON syntax/schema/evidence are checked for every row, including unused rows.
pub fn audit_packages(
    repo_root: &Path,
    map_path: &Path,
    frames_path: &Path,
    packages: &[CheckedPackage<'_>],
) -> Result<(), String> {
    let mut errors = Vec::new();
    let map = load_map(map_path, &mut errors);
    let frames = load_frames(frames_path, repo_root, &mut errors);
    let mut identities: BTreeMap<(String, String), (String, String, String)> = BTreeMap::new();

    for package in packages {
        let prefix = format!("{}::", cohdl::pipeline::package_root(package.name));
        for (part_name, part) in &package.world.parts {
            if !part_name.starts_with(&prefix) {
                continue;
            }
            for entry in entries(part) {
                let mfr = entry.field("mfr").map_or("", |field| field.value.as_str());
                let mpn = entry.field("mpn").map_or("", |field| field.value.as_str());
                let effective_footprint = entry.effective_footprint(&part.primary);
                let footprint =
                    effective_footprint.map_or("<none>", |fp| cohdl::resolve::short(&fp.name));
                let frame_package = effective_footprint.and_then(|fp| {
                    let root = fp.name.split("::").next()?;
                    package.world.package_names.get(root)
                });
                let frame_version = frame_package
                    .and_then(|name| package.package_versions.get(name))
                    .map(String::as_str);
                for binding in &entry.suppliers {
                    let supplier = binding.supplier.name.as_str();
                    let pn = binding.part_number.as_str();
                    let context = format!(
                        "package `{}` part `{part_name}` footprint `{footprint}`, {supplier} {pn}",
                        package.name
                    );
                    let identity = (mfr.to_string(), mpn.to_string(), context.clone());
                    if let Some(previous) =
                        identities.insert((supplier.to_string(), pn.to_string()), identity.clone())
                    {
                        if previous.0 != identity.0 || previous.1 != identity.1 {
                            error(
                                &context,
                                format!(
                                    "global ({supplier}, {pn}) identity conflict: `{} / `{}` in {} versus `{} / `{}` in {}",
                                    previous.0, previous.1, previous.2, identity.0, identity.1, identity.2
                                ),
                                &mut errors,
                            );
                        }
                    }
                    if supplier != "lcsc" {
                        continue;
                    }
                    if !map
                        .iter()
                        .any(|row| row.pn == pn && row.mfr == mfr && row.mpn == mpn)
                    {
                        error(
                            &context,
                            format!("lcsc_map.json has no row for lcsc {pn} with mfr `{mfr}` and mpn `{mpn}`"),
                            &mut errors,
                        );
                    }
                    let exact = frames
                        .iter()
                        .filter(|row| {
                            frame_package.is_some_and(|name| row.package == *name)
                                && frame_version
                                    .is_some_and(|version| row.package_version == version)
                                && row.footprint == footprint
                                && row.supplier == supplier
                                && row.pn == pn
                        })
                        .collect::<Vec<_>>();
                    if exact.is_empty() {
                        error(
                            &context,
                            format!(
                                "jlc_frames.json has no exact (footprint package `{}`, its current package_version `{}`, footprint, supplier, pn) row for lcsc {pn}",
                                frame_package.map_or("<unresolved>", String::as_str),
                                frame_version.unwrap_or("<unresolved>")
                            ),
                            &mut errors,
                        );
                        continue;
                    }
                    let expected = mapping_of(binding);
                    if !exact
                        .iter()
                        .any(|row| mapping_matches(&row.mapping, &expected))
                    {
                        error(
                            &context,
                            format!(
                                "mapping mismatch for lcsc {pn}: binding requires dx_mm={}, dy_mm={}, rotate_deg={}",
                                expected.dx_mm, expected.dy_mm, expected.rotate_deg
                            ),
                            &mut errors,
                        );
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "supplier data audit failed:\n - {}",
            errors.join("\n - ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl::pipeline::check_files_in_with_deps;
    use std::path::{Path, PathBuf};

    struct OwnedPackage {
        name: String,
        package_versions: BTreeMap<String, String>,
        world: World,
    }

    fn packages_under(dir: &Path, out: &mut Vec<PathBuf>) {
        if dir.join("cohdl.toml").is_file() {
            out.push(dir.to_path_buf());
            return;
        }
        let mut children = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        children.sort();
        for child in children {
            packages_under(&child, out);
        }
    }

    fn load_package(dir: &Path, lib_root: &Path) -> OwnedPackage {
        let (_, manifest) = cohdl::project::peek_manifest(dir).unwrap();
        let mut deps = Vec::new();
        for (name, raw_version, _) in manifest.deps_raw.unwrap_or_default() {
            let wanted = cohdl::deps::parse_exact_version(&raw_version).unwrap();
            let (_, dep_dir) = cohdl::deps::available_versions(&lib_root.join(&name), &name)
                .unwrap()
                .into_iter()
                .find(|(version, _)| *version == wanted)
                .unwrap_or_else(|| {
                    panic!("{} lacks dependency {name} {raw_version}", dir.display())
                });
            deps.push((name, dep_dir));
        }
        deps.sort_by_key(|(name, _)| if name == "std" { 0 } else { 1 });
        let dep_names = deps
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let project = cohdl::project::load_project_with_deps(dir, &deps).unwrap();
        let mut checked = check_files_in_with_deps(&project.name, &dep_names, &project.files, None)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        checked.diags.sort(&checked.sm);
        assert!(
            !checked.diags.has_errors(),
            "{} declaration check failed:\n{}",
            dir.display(),
            checked.diags.render(&checked.sm)
        );
        checked.world.package_names = project.package_names;
        checked.world.package_versions = project.package_versions.clone();
        OwnedPackage {
            name: project.name,
            package_versions: project.package_versions,
            world: checked.world,
        }
    }

    fn borrowed(packages: &[OwnedPackage]) -> Vec<CheckedPackage<'_>> {
        packages
            .iter()
            .map(|package| {
                CheckedPackage::new(&package.name, &package.package_versions, &package.world)
            })
            .collect()
    }

    #[test]
    fn shipped_library_matches_empty_sourcing_data() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let lib = root.join("lib");
        let mut dirs = Vec::new();
        packages_under(&lib, &mut dirs);
        // std is still checked as a dependency of every ordinary library; the
        // existing library conformance test separately rejects it as a root.
        dirs.retain(|dir| *dir != lib.join("std"));
        let packages = dirs
            .iter()
            .map(|dir| load_package(dir, &lib))
            .collect::<Vec<_>>();
        audit_packages(
            root,
            &root.join("tools/sourcing_data/lcsc_map.json"),
            &root.join("tools/sourcing_data/jlc_frames.json"),
            &borrowed(&packages),
        )
        .unwrap();
    }

    fn fixture(case: &str) -> (PathBuf, Vec<OwnedPackage>) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/library-ci")
            .join(case);
        let lib = root.join("packages");
        let mut dirs = Vec::new();
        packages_under(&lib, &mut dirs);
        let packages = dirs
            .iter()
            .map(|dir| load_package(dir, &lib))
            .collect::<Vec<_>>();
        (root, packages)
    }

    fn audit_fixture(case: &str) -> Result<(), String> {
        let (root, packages) = fixture(case);
        audit_packages(
            &root,
            &root.join("lcsc_map.json"),
            &root.join("jlc_frames.json"),
            &borrowed(&packages),
        )
    }

    fn assert_bad_fixture(case: &str, expected: &[&str]) {
        let message = audit_fixture(case).unwrap_err();
        for expected in expected {
            assert!(
                message.contains(expected),
                "{case} error did not contain `{expected}`:\n{message}"
            );
        }
    }

    #[test]
    fn valid_fixture_passes_with_direct_and_reused_evidence() {
        audit_fixture("valid").unwrap();
    }

    #[test]
    fn frames_key_on_the_footprint_owner_not_the_binding_package() {
        let (root, packages) = fixture("g-footprint-owner");
        audit_packages(
            &root,
            &root.join("lcsc_map.json"),
            &root.join("jlc_frames.json"),
            &borrowed(&packages),
        )
        .unwrap();

        let message = audit_packages(
            &root,
            &root.join("lcsc_map.json"),
            &root.join("jlc_frames.binding-owner.json"),
            &borrowed(&packages),
        )
        .unwrap_err();
        for expected in [
            "package `ci_g`",
            "FP_SHARED",
            "lcsc C_G_OWNER",
            "footprint package `ci_fp`",
            "current package_version `2.4.6`",
        ] {
            assert!(
                message.contains(expected),
                "wrong-owner error did not contain `{expected}`:\n{message}"
            );
        }
    }

    #[test]
    fn every_evidence_rule_has_an_independent_json_counterexample() {
        let cases = [
            (
                "d-chain",
                "frame `bad` references `indirect`, which does not have its own evidence",
            ),
            (
                "d-missing-record",
                "evidence record `does-not-exist.txt` does not name an existing repository file",
            ),
            (
                "d-escape-record",
                "evidence record `../outside.txt` must stay inside the repository",
            ),
            (
                "d-bad-reason",
                "evidence_ref `reason` must be same_supplier_footprint or same_preview_board",
            ),
            ("d-empty-note", "`note` must be a non-empty string"),
            (
                "d-missing-exception",
                "non-zero mapping requires a non-empty `exception`; null is forbidden",
            ),
            (
                "d-zero-exception",
                "zero mapping requires `exception: null`, got `zero mapping must not carry this`",
            ),
            ("d-duplicate-id", "duplicate frame id `direct`"),
        ];
        for (case, expected) in cases {
            let message = audit_fixture(case).unwrap_err();
            assert!(
                message.contains(expected),
                "{case} error did not contain `{expected}`:\n{message}"
            );
            assert!(
                message.contains("C_D_RULE"),
                "{case} error did not name its supplier part number:\n{message}"
            );
        }
    }

    #[test]
    fn bad_a_map_names_package_footprint_and_part_number() {
        assert_bad_fixture(
            "a-map",
            &["package `ci_a`", "FP_CI_A", "lcsc C_A_MAP", "mfr `A Maker`"],
        );
    }

    #[test]
    fn bad_b_frame_version_names_current_package_version() {
        assert_bad_fixture(
            "b-frame-version",
            &[
                "package `ci_b`",
                "FP_CI_B",
                "lcsc C_B_FRAME",
                "current package_version `1.2.3`",
            ],
        );
    }

    #[test]
    fn bad_c_mapping_names_binding_and_part_number() {
        assert_bad_fixture(
            "c-mapping",
            &[
                "package `ci_c`",
                "FP_CI_C",
                "lcsc C_C_MAPPING",
                "rotate_deg=90",
            ],
        );
    }

    #[test]
    fn bad_d_evidence_names_frame_footprint_and_part_number() {
        assert_bad_fixture(
            "d-evidence",
            &[
                "ci_d::FP_CI_D",
                "lcsc C_D_EVIDENCE",
                "exactly one of `evidence` and `evidence_ref`",
            ],
        );
    }

    #[test]
    fn bad_e_global_identity_names_both_packages_and_part_number() {
        assert_bad_fixture(
            "e-uniqueness",
            &[
                "global (lcsc, C_E_SHARED)",
                "package `ci_e1`",
                "package `ci_e2`",
                "Maker One",
                "Maker Two",
            ],
        );
    }

    #[test]
    fn bad_f_schema_names_malformed_frame_identity() {
        assert_bad_fixture(
            "f-schema",
            &[
                "ci_f::FP_CI_F",
                "lcsc C_F_SCHEMA",
                "`rotate_deg` must be a finite number",
            ],
        );
    }
}

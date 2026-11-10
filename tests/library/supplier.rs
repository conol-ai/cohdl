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

/// What the W3 checks need to know about any frame row.
struct FrameInfo {
    direct: bool,
    key: (String, String, String, String),
    zero: bool,
}

/// A row that claims W3 sampled inference, checked against its group record
/// once every row is loaded.
struct SampledRow {
    id: String,
    group: String,
    record: String,
    context: String,
    key: (String, String, String, String),
    zero: bool,
}

/// The W3 rule Tony accepted on 2026-11-09 for `passive` chip resistors and MLCCs
/// (https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188), with his
/// 2026-11-10 decision that a sample not drawn does not block its group and the
/// numbers proposed in `docs/library-supplier-w3-sampling.md` §5. It must equal
/// `RULE` in `tools/sourcing/w3_rule.py`; a group record carrying any other rule
/// is rejected, so a record cannot loosen its own rule.
fn w3_rule() -> Value {
    serde_json::json!({
        "version": "w3-2026-11-10b",
        "package": "passive",
        "types": ["ChipResistor", "MLCC"],
        "min_sample": 3,
        "fraction": 0.05,
        "min_stock": 30,
        "angle_limit_deg": 10.0,
        "offset_raw_fraction": 1.0,
        "offset_residual_fraction": 0.5,
        "offset_bias": "board_median",
        "infer_zero_stock": false,
        "infer_not_drawn": false
    })
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
    // The generators bind an identity only when it names one part number, and
    // RFC-034 lets a part number name one identity: the map must agree.
    let mut by_identity: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    let mut by_pn: BTreeMap<&str, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for row in &parsed {
        by_identity
            .entry((&row.mfr, &row.mpn))
            .or_default()
            .insert(&row.pn);
        by_pn
            .entry(&row.pn)
            .or_default()
            .insert((&row.mfr, &row.mpn));
    }
    for ((mfr, mpn), pns) in &by_identity {
        if pns.len() > 1 {
            let pns = pns.iter().copied().collect::<Vec<_>>().join(", ");
            error(
                "lcsc_map.json",
                format!("`{mfr}` / `{mpn}` names more than one part number: {pns}"),
                errors,
            );
        }
    }
    for (pn, identities) in &by_pn {
        if identities.len() > 1 {
            let identities = identities
                .iter()
                .map(|(mfr, mpn)| format!("`{mfr}` / `{mpn}`"))
                .collect::<Vec<_>>()
                .join(", ");
            error(
                "lcsc_map.json",
                format!("lcsc {pn} names more than one identity: {identities}"),
                errors,
            );
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
    let mut infos = BTreeMap::new();
    let mut sampled_rows = Vec::new();
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
        // `sampled` is optional: a row without it reads as `sampled: null`.
        let has_sampled = object.get("sampled").is_some_and(|value| !value.is_null());
        if [has_evidence, has_reference, has_sampled]
            .iter()
            .filter(|present| **present)
            .count()
            != 1
        {
            error(
                &context,
                "exactly one of `evidence`, `evidence_ref` and `sampled` must be non-null",
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
                if evidence.contains_key("note") {
                    string_field(evidence, "note", &context, errors);
                }
                for key in evidence.keys() {
                    if !matches!(key.as_str(), "method" | "date" | "record" | "note") {
                        error(
                            &context,
                            format!("evidence has unknown field `{key}`"),
                            errors,
                        );
                    }
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
        let mut sampled = None;
        match object.get("sampled") {
            Some(Value::Object(claim)) => {
                let group = string_field(claim, "group", &context, errors);
                let record = string_field(claim, "record", &context, errors);
                if let Some(record) = record {
                    audit_evidence_record(repo_root, record, &context, errors);
                }
                string_field(claim, "note", &context, errors);
                for key in claim.keys() {
                    if !matches!(key.as_str(), "group" | "record" | "note") {
                        error(
                            &context,
                            format!("sampled has unknown field `{key}`"),
                            errors,
                        );
                    }
                }
                sampled = group.zip(record);
            }
            Some(Value::Null) | None => {}
            Some(_) => error(&context, "`sampled` must be an object or null", errors),
        }
        let key = (
            package.unwrap_or("?").to_string(),
            package_version.unwrap_or("?").to_string(),
            footprint.unwrap_or("?").to_string(),
            pn.unwrap_or("?").to_string(),
        );
        let zero = mapping
            .as_ref()
            .is_some_and(|mapping| !nonzero && mapping.rotate_deg == 0.0);
        if let Some(id) = id {
            infos.insert(
                id.to_string(),
                FrameInfo {
                    direct: has_evidence && !has_reference && !has_sampled,
                    key: key.clone(),
                    zero,
                },
            );
            if let Some((group, record)) = sampled {
                sampled_rows.push(SampledRow {
                    id: id.to_string(),
                    group: group.to_string(),
                    record: record.to_string(),
                    context: context.clone(),
                    key,
                    zero,
                });
            }
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
    audit_sampled(repo_root, &sampled_rows, &infos, errors);
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

fn json_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn json_strings(value: &Value, field: &str) -> Option<Vec<String>> {
    value
        .get(field)?
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_string))
        .collect()
}

/// Every check a W3 group record makes of a row inferred from it: the record
/// carries the accepted rule; its group is active, in scope and keyed on the
/// row's own (package, version, footprint); the land files it names still hash
/// as recorded; the row's part number is a member, not a sample and, unless the
/// rule allows it, neither out of stock nor left undrawn on a preview; and every
/// sample is a member with its own zero-mapping evidence row under the same key
/// and a reading within the rule: its angle, its raw offset and its residual once
/// the view's bias is taken off each within their limits, and the raw size and
/// residual what its vectors give.
fn audit_sampled(
    repo_root: &Path,
    rows: &[SampledRow],
    infos: &BTreeMap<String, FrameInfo>,
    errors: &mut Vec<String>,
) {
    let rule = w3_rule();
    let number = |field: &str| rule[field].as_f64().unwrap();
    let (min_sample, fraction) = (number("min_sample") as usize, number("fraction"));
    let angle_limit = number("angle_limit_deg");
    let (raw_fraction, residual_fraction) = (
        number("offset_raw_fraction"),
        number("offset_residual_fraction"),
    );
    let infer_zero_stock = rule["infer_zero_stock"].as_bool().unwrap();
    let infer_not_drawn = rule["infer_not_drawn"].as_bool().unwrap();
    let mut records: BTreeMap<String, Option<Value>> = BTreeMap::new();
    for row in rows {
        let context = &row.context;
        let record = records.entry(row.record.clone()).or_insert_with(|| {
            std::fs::read(repo_root.join(&row.record))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        });
        let Some(record) = record.as_ref() else {
            error(
                context,
                format!("W3 record `{}` is not readable JSON", row.record),
                errors,
            );
            continue;
        };
        if record.get("schema_version") != Some(&Value::from(1)) {
            error(
                context,
                format!("W3 record `{}` must have schema_version 1", row.record),
                errors,
            );
        }
        if record.get("rule") != Some(&rule) {
            error(
                context,
                format!(
                    "W3 record `{}` does not carry the accepted rule {rule}",
                    row.record
                ),
                errors,
            );
            continue;
        }
        let groups = record
            .get("groups")
            .and_then(Value::as_array)
            .map(|groups| {
                groups
                    .iter()
                    .filter(|group| json_str(group, "id") == Some(row.group.as_str()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let [group] = groups.as_slice() else {
            error(
                context,
                format!(
                    "W3 record `{}` must hold exactly one group `{}`",
                    row.record, row.group
                ),
                errors,
            );
            continue;
        };
        let (package, version, footprint, pn) = &row.key;
        if json_str(group, "status") != Some("active") {
            error(
                context,
                format!("W3 group `{}` is not active", row.group),
                errors,
            );
        }
        let in_scope = json_str(group, "package") == Some("passive")
            && json_str(group, "type").is_some_and(|kind| matches!(kind, "ChipResistor" | "MLCC"));
        if !in_scope {
            error(
                context,
                format!(
                    "W3 group `{}` is outside the rule's scope (passive ChipResistor or MLCC)",
                    row.group
                ),
                errors,
            );
        }
        if json_str(group, "package") != Some(package.as_str())
            || json_str(group, "package_version") != Some(version.as_str())
            || json_str(group, "footprint") != Some(footprint.as_str())
        {
            error(
                context,
                format!(
                    "W3 group `{}` is not keyed on the row's package, version and footprint",
                    row.group
                ),
                errors,
            );
        }
        match group.get("land").and_then(Value::as_object) {
            Some(land) if !land.is_empty() => {
                for (file, digest) in land {
                    let actual = std::fs::read(repo_root.join(file))
                        .ok()
                        .map(|bytes| cohdl::hash::sha256_hex(&bytes));
                    if actual.as_deref() != digest.as_str() {
                        error(
                            context,
                            format!("W3 group `{}` land file `{file}` no longer has its recorded SHA-256", row.group),
                            errors,
                        );
                    }
                }
            }
            _ => error(
                context,
                format!("W3 group `{}` must name its land files", row.group),
                errors,
            ),
        }
        let members = json_strings(group, "members").unwrap_or_default();
        let member_set = members.iter().collect::<BTreeSet<_>>();
        if member_set.len() != members.len() {
            error(
                context,
                format!("W3 group `{}` lists a member twice", row.group),
                errors,
            );
        }
        let zero_stock = json_strings(group, "zero_stock").unwrap_or_default();
        let not_drawn = json_strings(group, "not_drawn");
        if not_drawn.is_none() {
            error(
                context,
                format!("W3 group `{}` must list its members not drawn", row.group),
                errors,
            );
        }
        let not_drawn = not_drawn.unwrap_or_default();
        let samples = group
            .get("samples")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let sample_pns = samples
            .iter()
            .filter_map(|sample| json_str(sample, "pn"))
            .collect::<BTreeSet<_>>();
        if !member_set.contains(pn) {
            error(
                context,
                format!("lcsc {pn} is not a member of W3 group `{}`", row.group),
                errors,
            );
        }
        if sample_pns.contains(pn.as_str()) {
            error(
                context,
                format!(
                    "lcsc {pn} is a sample of W3 group `{}`, not inferred",
                    row.group
                ),
                errors,
            );
        }
        if !infer_zero_stock && zero_stock.contains(pn) {
            error(
                context,
                format!("lcsc {pn} has no stock, and the rule infers no zero-stock member"),
                errors,
            );
        }
        if !infer_not_drawn && not_drawn.contains(pn) {
            error(
                context,
                format!(
                    "lcsc {pn} was not drawn on a preview, and the rule infers no member not drawn"
                ),
                errors,
            );
        }
        if !row.zero {
            error(
                context,
                "a W3 inferred row must have a zero mapping",
                errors,
            );
        }
        let required = min_sample.max((fraction * members.len() as f64).ceil() as usize);
        if sample_pns.len() != samples.len() || sample_pns.len() < required {
            error(
                context,
                format!(
                    "W3 group `{}` has {} distinct samples; the rule requires {required} of {} members",
                    row.group,
                    sample_pns.len(),
                    members.len()
                ),
                errors,
            );
        }
        for sample in &samples {
            let sample_pn = json_str(sample, "pn").unwrap_or("?");
            let target = json_str(sample, "row").unwrap_or("?");
            let own = infos.get(target).filter(|info| {
                info.direct
                    && info.zero
                    && info.key
                        == (
                            package.clone(),
                            version.clone(),
                            footprint.clone(),
                            sample_pn.to_string(),
                        )
                    && target != row.id
            });
            if own.is_none() || !member_set.contains(&sample_pn.to_string()) {
                error(
                    context,
                    format!(
                        "W3 sample lcsc {sample_pn} must be a member with its own zero-mapping evidence row `{target}` under the group's key"
                    ),
                    errors,
                );
            }
            for message in
                sample_reading_errors(sample, angle_limit, raw_fraction, residual_fraction)
            {
                error(
                    context,
                    format!("W3 sample lcsc {sample_pn} {message}"),
                    errors,
                );
            }
        }
    }
}

/// What is wrong with one sample's reading under the rule, if anything (the
/// checks of `sample_ok` in `tools/sourcing/w3_rule.py`).
fn sample_reading_errors(
    sample: &Value,
    angle_limit: f64,
    raw_fraction: f64,
    residual_fraction: f64,
) -> Vec<String> {
    // A record's offsets are rounded to the micrometre.
    const ROUNDING_MM: f64 = 0.001;
    let number = |field: &str| sample.get(field).and_then(Value::as_f64);
    let vector = |field: &str| match sample.get(field).and_then(Value::as_array) {
        Some(items) if items.len() == 2 => items[0].as_f64().zip(items[1].as_f64()),
        _ => None,
    };
    let (Some(angle), Some(limit), Some(raw), Some(raw_xy), Some(bias_xy), Some(residual)) = (
        number("angle_deg"),
        number("offset_limit_mm"),
        number("raw_offset_mm"),
        vector("raw_offset_xy_mm"),
        vector("bias_xy_mm"),
        number("residual_offset_mm"),
    ) else {
        return vec![
            "reading must give angle_deg, offset_limit_mm, raw_offset_xy_mm, raw_offset_mm, bias_xy_mm and residual_offset_mm".to_string(),
        ];
    };
    let mut problems = Vec::new();
    if angle.abs() > angle_limit {
        problems.push(format!(
            "angle {angle} deg is outside the rule's {angle_limit} deg"
        ));
    }
    let measured = raw_xy.0.hypot(raw_xy.1);
    let left = (raw_xy.0 - bias_xy.0).hypot(raw_xy.1 - bias_xy.1);
    if (raw - measured).abs() > ROUNDING_MM || (residual - left).abs() > ROUNDING_MM {
        problems.push(format!(
            "offsets do not add up: raw {raw} mm and residual {residual} mm, but its vectors give {measured:.3} and {left:.3}"
        ));
    }
    if raw > raw_fraction * limit {
        problems.push(format!(
            "raw offset {raw} mm is above {raw_fraction} of its {limit} mm limit"
        ));
    }
    if residual > residual_fraction * limit {
        problems.push(format!(
            "residual offset {residual} mm is above {residual_fraction} of its {limit} mm limit"
        ));
    }
    problems
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
    fn shipped_library_matches_its_sourcing_data() {
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
            ("d-evidence-note", "`note` must be a non-empty string"),
            ("d-evidence-key", "evidence has unknown field `by`"),
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

    fn audit_sampled_variant(frames: &str) -> Result<(), String> {
        let (root, packages) = fixture("s-sampled");
        audit_packages(
            &root,
            &root.join("lcsc_map.json"),
            &root.join(frames),
            &borrowed(&packages),
        )
    }

    #[test]
    fn a_w3_inferred_row_passes_when_its_group_record_holds() {
        audit_sampled_variant("jlc_frames.json").unwrap();
    }

    #[test]
    fn every_w3_sampled_rule_has_an_independent_counterexample() {
        let cases = [
            ("rule", "does not carry the accepted rule"),
            ("paused", "W3 group `g1` is not active"),
            ("scope", "outside the rule's scope"),
            (
                "key",
                "not keyed on the row's package, version and footprint",
            ),
            (
                "land",
                "land file `land.txt` no longer has its recorded SHA-256",
            ),
            ("member", "lcsc C_S_OTHER is not a member of W3 group `g1`"),
            ("zero-stock", "lcsc C_S_ZERO has no stock"),
            ("sample", "lcsc C_S1 is a sample of W3 group `g1`"),
            (
                "few",
                "has 2 distinct samples; the rule requires 3 of 6 members",
            ),
            (
                "own-evidence",
                "W3 sample lcsc C_S3 must be a member with its own zero-mapping evidence row `s3`",
            ),
            (
                "reading",
                "W3 sample lcsc C_S2 angle 12 deg is outside the rule's 10 deg",
            ),
            (
                "offset",
                "W3 sample lcsc C_S2 residual offset 0.07 mm is above 0.5 of its 0.127 mm limit",
            ),
            (
                "raw",
                "W3 sample lcsc C_S1 raw offset 0.13 mm is above 1 of its 0.127 mm limit",
            ),
            ("arithmetic", "W3 sample lcsc C_S2 offsets do not add up"),
            ("not-drawn", "lcsc C_S_UNDRAWN was not drawn on a preview"),
            ("mapping", "a W3 inferred row must have a zero mapping"),
            (
                "two-kinds",
                "exactly one of `evidence`, `evidence_ref` and `sampled` must be non-null",
            ),
            ("field", "sampled has unknown field `by`"),
        ];
        for (case, expected) in cases {
            let message = audit_sampled_variant(&format!("jlc_frames.{case}.json")).unwrap_err();
            assert!(
                message.contains(expected),
                "{case} error did not contain `{expected}`:\n{message}"
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
                "exactly one of `evidence`, `evidence_ref` and `sampled`",
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
    fn bad_h_map_identity_names_both_sides_of_each_conflict() {
        assert_bad_fixture(
            "h-map-identity",
            &[
                "lcsc C_A_MAP names more than one identity: `A Maker` / `A-MPN`, `A Maker` / `A-MPN-OTHER`",
                "`B Maker` / `B-MPN` names more than one part number: C_H_ONE, C_H_TWO",
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

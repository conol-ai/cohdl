//! RFC-034 §5 online supplier audit. Never called by `check` or `build`: the
//! CLI enters this module only after the ordinary offline check has passed.

use crate::ast::AvlEntry;
use crate::deps::PackageDiag;
use crate::ir::DesignIr;
use crate::registry::{self, AuditCatalogRow, SupplierPartResult, SUPPLIER_AUDIT_BATCH};
use crate::resolve::World;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStatus {
    Ok,
    Unknown,
    Mismatch,
    OutOfStock,
    Unbound,
}

impl AuditStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unknown => "unknown",
            Self::Mismatch => "mismatch",
            Self::OutOfStock => "out-of-stock",
            Self::Unbound => "unbound",
        }
    }

    fn changes_exit_code(self) -> bool {
        matches!(self, Self::Unknown | Self::Mismatch)
    }
}

#[derive(Debug)]
struct SourceEntry {
    part: String,
    entry: String,
    mfr: String,
    mpn: String,
    bindings: BTreeMap<String, String>,
}

#[derive(Debug)]
struct AuditEntry {
    part: String,
    entry: String,
    mfr: String,
    mpn: String,
    pn: Option<String>,
    status: AuditStatus,
    mismatches: Vec<&'static str>,
    catalog: Option<AuditCatalogRow>,
    suggestions: Vec<AuditSuggestion>,
}

#[derive(Debug)]
struct AuditSuggestion {
    catalog: AuditCatalogRow,
    manufacturer_match: Option<bool>,
}

#[derive(Debug)]
struct SupplierReport {
    supplier: String,
    snapshot_date: Option<String>,
    entries: Vec<AuditEntry>,
}

#[derive(Debug)]
pub struct AuditReport {
    design: String,
    suppliers: Vec<SupplierReport>,
}

fn catalog_json(row: &AuditCatalogRow, indent: &str) -> String {
    let string = crate::emit::json::json_str;
    let mut out = String::from("{\n");
    for (name, value) in [
        ("pn", &row.pn),
        ("mpn", &row.mpn),
        ("mpn_key", &row.mpn_key),
        ("manufacturer", &row.manufacturer),
        ("manufacturer_key", &row.manufacturer_key),
        ("package", &row.package),
        ("class", &row.class),
        ("fetched_at", &row.fetched_at),
    ] {
        let _ = writeln!(out, "{indent}  \"{name}\": {},", string(value));
    }
    let _ = writeln!(out, "{indent}  \"stock\": {},", row.stock);
    let _ = writeln!(out, "{indent}  \"links\": [");
    for (index, link) in row.links.iter().enumerate() {
        let comma = if index + 1 == row.links.len() {
            ""
        } else {
            ","
        };
        let _ = writeln!(
            out,
            "{indent}    {{\"label\": {}, \"url\": {}}}{comma}",
            string(&link.label),
            string(&link.url)
        );
    }
    let _ = writeln!(out, "{indent}  ]");
    let _ = write!(out, "{indent}}}");
    out
}

impl AuditReport {
    pub fn has_binding_problems(&self) -> bool {
        self.suppliers
            .iter()
            .flat_map(|supplier| &supplier.entries)
            .any(|entry| entry.status.changes_exit_code())
    }

    /// Independent, complete report shape; this is not RFC-010 diagnostics.
    pub fn render_json(&self) -> String {
        let string = crate::emit::json::json_str;
        let nullable = |value: Option<&String>| {
            value.map_or_else(|| "null".to_string(), |value| string(value))
        };
        let mut out = String::from("{\n  \"schema_version\": 1,\n");
        let _ = writeln!(out, "  \"design\": {},", string(&self.design));
        out.push_str("  \"suppliers\": [");
        if self.suppliers.is_empty() {
            out.push_str("]\n}\n");
            return out;
        }
        out.push('\n');
        for (supplier_index, supplier) in self.suppliers.iter().enumerate() {
            let _ = writeln!(out, "    {{");
            let _ = writeln!(out, "      \"supplier\": {},", string(&supplier.supplier));
            let _ = writeln!(
                out,
                "      \"snapshot_date\": {},",
                nullable(supplier.snapshot_date.as_ref())
            );
            out.push_str("      \"entries\": [");
            if supplier.entries.is_empty() {
                out.push_str("]\n    }");
            } else {
                out.push('\n');
                for (entry_index, entry) in supplier.entries.iter().enumerate() {
                    let _ = writeln!(out, "        {{");
                    for (key, value) in [
                        ("part", &entry.part),
                        ("entry", &entry.entry),
                        ("mfr", &entry.mfr),
                        ("mpn", &entry.mpn),
                    ] {
                        let _ = writeln!(out, "          \"{key}\": {},", string(value));
                    }
                    let _ = writeln!(out, "          \"pn\": {},", nullable(entry.pn.as_ref()));
                    let _ = writeln!(
                        out,
                        "          \"status\": {},",
                        string(entry.status.as_str())
                    );
                    out.push_str("          \"mismatches\": [");
                    for (index, field) in entry.mismatches.iter().enumerate() {
                        if index != 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&string(field));
                    }
                    out.push_str("],\n");
                    let catalog = entry
                        .catalog
                        .as_ref()
                        .map_or_else(|| "null".to_string(), |row| catalog_json(row, "          "));
                    let _ = writeln!(out, "          \"catalog\": {catalog},");
                    out.push_str("          \"suggestions\": [");
                    if entry.suggestions.is_empty() {
                        out.push_str("]\n        }");
                    } else {
                        out.push('\n');
                        for (suggestion_index, suggestion) in entry.suggestions.iter().enumerate() {
                            let _ = writeln!(out, "            {{");
                            let _ = writeln!(
                                out,
                                "              \"catalog\": {},",
                                catalog_json(&suggestion.catalog, "              ")
                            );
                            let _ = writeln!(
                                out,
                                "              \"manufacturer_match\": {}",
                                suggestion
                                    .manufacturer_match
                                    .map_or_else(|| "null".to_string(), |value| value.to_string())
                            );
                            out.push_str("            }");
                            if suggestion_index + 1 != entry.suggestions.len() {
                                out.push(',');
                            }
                            out.push('\n');
                        }
                        out.push_str("          ]\n        }");
                    }
                    if entry_index + 1 != supplier.entries.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str("      ]\n    }");
            }
            if supplier_index + 1 != self.suppliers.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        out
    }

    pub fn render_human(&self) -> String {
        let mut out = String::new();
        let text = |value: &str| {
            value
                .chars()
                .filter(|ch| {
                    !ch.is_control()
                        && !matches!(
                            ch,
                            '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
                        )
                })
                .collect::<String>()
        };
        for supplier in &self.suppliers {
            for entry in &supplier.entries {
                let _ = writeln!(
                    out,
                    "{} {} {} {} pn={} mfr={:?} mpn={:?}",
                    text(&supplier.supplier),
                    text(&entry.part),
                    text(&entry.entry),
                    entry.status.as_str(),
                    text(entry.pn.as_deref().unwrap_or("-")),
                    text(&entry.mfr),
                    text(&entry.mpn)
                );
                if !entry.mismatches.is_empty() {
                    let _ = writeln!(out, "  mismatch: {}", entry.mismatches.join(", "));
                }
                if let Some(catalog) = &entry.catalog {
                    let _ = writeln!(
                        out,
                        "  catalog: mfr={:?} mpn={:?} stock={} class={:?}",
                        text(&catalog.manufacturer),
                        text(&catalog.mpn),
                        catalog.stock,
                        text(&catalog.class)
                    );
                }
                for suggestion in &entry.suggestions {
                    let agreement = match suggestion.manufacturer_match {
                        Some(true) => "yes",
                        Some(false) => "no",
                        None => "unknown",
                    };
                    let catalog = &suggestion.catalog;
                    let _ = writeln!(
                        out,
                        "  suggestion: pn={} class={:?} stock={} mfr={:?} manufacturer-match={}",
                        text(&catalog.pn),
                        text(&catalog.class),
                        catalog.stock,
                        text(&catalog.manufacturer),
                        agreement
                    );
                }
            }
        }
        out
    }
}

fn field(entry: &AvlEntry, name: &str) -> String {
    entry
        .field(name)
        .map(|field| field.value.clone())
        .unwrap_or_default()
}

/// E814's scope: actual instantiated parts only, but every primary/alternate
/// entry of each such part (not merely the entry a build would select).
fn source_entries(world: &World, ir: &DesignIr) -> Vec<SourceEntry> {
    let parts: BTreeSet<&str> = ir
        .instances
        .values()
        .filter_map(|instance| instance.part.as_deref())
        .collect();
    let mut entries = Vec::new();
    for part_name in parts {
        let Some(part) = world.parts.get(part_name) else {
            continue;
        };
        for (index, entry) in std::iter::once(&part.primary)
            .chain(part.alts.iter())
            .enumerate()
        {
            entries.push(SourceEntry {
                part: part_name.to_string(),
                entry: if index == 0 {
                    "primary".to_string()
                } else {
                    format!("alt[{}]", index - 1)
                },
                mfr: field(entry, "mfr"),
                mpn: field(entry, "mpn"),
                bindings: entry
                    .suppliers
                    .iter()
                    .map(|binding| (binding.supplier.name.clone(), binding.part_number.clone()))
                    .collect(),
            });
        }
    }
    entries
}

fn check_snapshot(
    seen: &mut bool,
    current: &mut Option<String>,
    next: Option<String>,
) -> Result<(), PackageDiag> {
    if *seen && *current != next {
        return Err(registry::unreachable(
            "supplier audit snapshot date changed between request batches".to_string(),
        ));
    }
    *seen = true;
    *current = next;
    Ok(())
}

pub fn audit(
    world: &World,
    ir: &DesignIr,
    requested: &[String],
) -> Result<AuditReport, PackageDiag> {
    let entries = source_entries(world, ir);
    let suppliers: BTreeSet<String> = if requested.is_empty() {
        entries
            .iter()
            .flat_map(|entry| entry.bindings.keys().cloned())
            .collect()
    } else {
        requested.iter().cloned().collect()
    };
    let mut reports = Vec::new();
    for supplier in suppliers {
        let bound: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.bindings.contains_key(&supplier))
            .map(|(index, _)| index)
            .collect();
        let pn_values: Vec<String> = bound
            .iter()
            .map(|index| entries[*index].bindings[&supplier].clone())
            .collect();
        let mut catalogs = BTreeMap::new();
        let mut snapshot_seen = false;
        let mut snapshot_date = None;
        for (batch_index, batch) in pn_values.chunks(SUPPLIER_AUDIT_BATCH).enumerate() {
            let response = registry::supplier_parts(&supplier, "pn", batch)?;
            check_snapshot(
                &mut snapshot_seen,
                &mut snapshot_date,
                response.snapshot_date,
            )?;
            for (index, part) in response.parts.into_iter().enumerate() {
                if let SupplierPartResult::Known(row) = part.result {
                    catalogs.insert(bound[batch_index * SUPPLIER_AUDIT_BATCH + index], row);
                }
            }
        }

        // No implicit unbound audit/suggestions without an explicit --supplier.
        // Empty source MPN is still reported unbound, but is never queried.
        let unbound: Vec<usize> = if requested.is_empty() {
            Vec::new()
        } else {
            entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.bindings.contains_key(&supplier))
                .map(|(index, _)| index)
                .collect()
        };
        let suggestion_entries: Vec<usize> = unbound
            .iter()
            .copied()
            .filter(|index| !entries[*index].mpn.is_empty())
            .collect();
        let suggestion_values: Vec<String> = suggestion_entries
            .iter()
            .map(|index| entries[*index].mpn.clone())
            .collect();
        let mut suggestions = BTreeMap::new();
        for (batch_index, batch) in suggestion_values.chunks(SUPPLIER_AUDIT_BATCH).enumerate() {
            // The registry requires normalized MPN query values to be unique.
            // Repeated AVL entries share the response for their one query row.
            let mut unique_values = Vec::new();
            let mut seen = BTreeSet::new();
            for value in batch {
                if seen.insert(value.as_str()) {
                    unique_values.push(value.clone());
                }
            }
            let response = registry::supplier_parts(&supplier, "mpn", &unique_values)?;
            check_snapshot(
                &mut snapshot_seen,
                &mut snapshot_date,
                response.snapshot_date,
            )?;
            let mut rows_by_mpn = BTreeMap::new();
            for part in response.parts {
                let SupplierPartResult::Suggestions(rows) = part.result else {
                    return Err(registry::unreachable(
                        "supplier audit MPN response was not a suggestion array".to_string(),
                    ));
                };
                rows_by_mpn.insert(part.requested, rows);
            }
            for (index, value) in batch.iter().enumerate() {
                suggestions.insert(
                    suggestion_entries[batch_index * SUPPLIER_AUDIT_BATCH + index],
                    rows_by_mpn.get(value).cloned().unwrap(),
                );
            }
        }

        // Only source values are normalized here. Catalog rows already carry
        // both registry-normalized keys. Empty fields are neither compared nor
        // sent (the normalize endpoint rejects them with HTTP 400).
        let audited: BTreeSet<usize> = bound.iter().chain(&unbound).copied().collect();
        let mut mfr_owners = Vec::new();
        let mut mpn_owners = Vec::new();
        let mut mfr_values = Vec::new();
        let mut mpn_values = Vec::new();
        for index in audited {
            let entry = &entries[index];
            if !entry.mfr.is_empty() {
                mfr_owners.push(index);
                mfr_values.push(entry.mfr.clone());
            }
            if !entry.mpn.is_empty() {
                mpn_owners.push(index);
                mpn_values.push(entry.mpn.clone());
            }
        }
        let mut mfr_keys = Vec::new();
        let mut mpn_keys = Vec::new();
        let mut mfr_offset = 0;
        let mut mpn_offset = 0;
        while mfr_offset < mfr_values.len() || mpn_offset < mpn_values.len() {
            let mfr_end = (mfr_offset + SUPPLIER_AUDIT_BATCH).min(mfr_values.len());
            let mpn_end = (mpn_offset + SUPPLIER_AUDIT_BATCH).min(mpn_values.len());
            let response = registry::normalize_supplier_fields(
                &mfr_values[mfr_offset..mfr_end],
                &mpn_values[mpn_offset..mpn_end],
            )?;
            mfr_keys.extend(response.mfr.into_iter().map(|(_, key)| key));
            mpn_keys.extend(response.mpn.into_iter().map(|(_, key)| key));
            mfr_offset = mfr_end;
            mpn_offset = mpn_end;
        }
        let source_mfr: BTreeMap<usize, String> = mfr_owners.into_iter().zip(mfr_keys).collect();
        let source_mpn: BTreeMap<usize, String> = mpn_owners.into_iter().zip(mpn_keys).collect();

        let mut report_entries = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            if let Some(pn) = entry.bindings.get(&supplier) {
                if let Some(row) = catalogs.remove(&index) {
                    let mut mismatches = Vec::new();
                    if source_mfr
                        .get(&index)
                        .is_some_and(|key| *key != row.manufacturer_key)
                    {
                        mismatches.push("mfr");
                    }
                    if source_mpn
                        .get(&index)
                        .is_some_and(|key| *key != row.mpn_key)
                    {
                        mismatches.push("mpn");
                    }
                    let status = if mismatches.is_empty() {
                        if row.stock == 0 {
                            AuditStatus::OutOfStock
                        } else {
                            AuditStatus::Ok
                        }
                    } else {
                        AuditStatus::Mismatch
                    };
                    report_entries.push(AuditEntry {
                        part: entry.part.clone(),
                        entry: entry.entry.clone(),
                        mfr: entry.mfr.clone(),
                        mpn: entry.mpn.clone(),
                        pn: Some(pn.clone()),
                        status,
                        mismatches,
                        catalog: Some(row),
                        suggestions: Vec::new(),
                    });
                } else {
                    report_entries.push(AuditEntry {
                        part: entry.part.clone(),
                        entry: entry.entry.clone(),
                        mfr: entry.mfr.clone(),
                        mpn: entry.mpn.clone(),
                        pn: Some(pn.clone()),
                        status: AuditStatus::Unknown,
                        mismatches: Vec::new(),
                        catalog: None,
                        suggestions: Vec::new(),
                    });
                }
            } else if !requested.is_empty() {
                let mut entry_suggestions: Vec<AuditSuggestion> = suggestions
                    .remove(&index)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|row| AuditSuggestion {
                        manufacturer_match: source_mfr
                            .get(&index)
                            .map(|key| *key == row.manufacturer_key),
                        catalog: row,
                    })
                    .collect();
                // Stable partition: matching manufacturers first, retaining
                // registry class/stock rank within each group. No mapping.
                entry_suggestions
                    .sort_by_key(|suggestion| suggestion.manufacturer_match != Some(true));
                report_entries.push(AuditEntry {
                    part: entry.part.clone(),
                    entry: entry.entry.clone(),
                    mfr: entry.mfr.clone(),
                    mpn: entry.mpn.clone(),
                    pn: None,
                    status: AuditStatus::Unbound,
                    mismatches: Vec::new(),
                    catalog: None,
                    suggestions: entry_suggestions,
                });
            }
        }
        reports.push(SupplierReport {
            supplier,
            snapshot_date,
            entries: report_entries,
        });
    }
    Ok(AuditReport {
        design: ir.name.clone(),
        suppliers: reports,
    })
}

use crate::proposal_files::{hash, manifest, relative};
use crate::proposal_schema::{object, strings, text};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn nullable_text(value: &Value) -> bool {
    value.is_null() || text(value)
}
fn nullable_hash(value: &Value) -> bool {
    value.is_null() || hash(value)
}
fn page(value: &Value) -> bool {
    value.is_null()
        || value
            .as_u64()
            .is_some_and(|p| p > 0 && p <= u32::MAX as u64)
}

pub fn shape(value: &Value) -> bool {
    object(
        value,
        &[
            "id",
            "title",
            "kind",
            "path",
            "revision",
            "sha256",
            "actual_sha256",
            "pages",
            "review",
            "uncertainty",
            "applicability",
            "status",
            "issues",
            "excerpts",
        ],
    ) && text(&value["id"])
        && text(&value["title"])
        && ["pdf", "markdown"].contains(&value["kind"].as_str().unwrap_or(""))
        && (value["path"].is_null() || value["path"].as_str().is_some_and(relative))
        && nullable_text(&value["revision"])
        && nullable_hash(&value["sha256"])
        && nullable_hash(&value["actual_sha256"])
        && page(&value["pages"])
        && ["reviewed", "unreviewed", "rejected"].contains(&value["review"].as_str().unwrap_or(""))
        && nullable_text(&value["uncertainty"])
        && ["ready", "missing", "changed", "invalid"]
            .contains(&value["status"].as_str().unwrap_or(""))
        && strings(&value["issues"], 10000)
        && value["applicability"].as_array().is_some_and(|a| {
            a.len() <= 4096
                && a.iter().all(|a| {
                    a.is_object()
                        && strings(&a["instances"], 10000)
                        && a["groups"].as_array().is_some_and(|g| {
                            g.len() <= 512
                                && g.iter().all(|g| {
                                    object(g, &["view", "region"])
                                        && text(&g["view"])
                                        && text(&g["region"])
                                })
                        })
                        && ["exact", "related"].contains(&a["relation"].as_str().unwrap_or(""))
                })
        })
        && value["excerpts"].as_array().is_some_and(|a| {
            a.len() <= 512
                && a.iter().all(|e| {
                    object(e, &["page", "text"]) && page(&e["page"]) && e["text"].is_string()
                })
        })
}

pub fn bindings(bundle: &Value, texts: &BTreeMap<&str, &str>) -> Result<(), String> {
    let entries = manifest(&bundle["snapshot"])?;
    let hashes: BTreeMap<_, _> = entries
        .iter()
        .map(|e| (e.path.as_str(), e.sha256.as_deref()))
        .collect();
    for (path, field) in [
        ("explorer/documents.json".to_owned(), "registry"),
        (
            format!(
                "views/{}.view.json",
                bundle["design"].as_str().unwrap_or("")
            ),
            "views",
        ),
    ] {
        let parsed = texts
            .get(path.as_str())
            .map(|t| serde_json::from_str::<Value>(t))
            .transpose()
            .map_err(|_| "Bundle input JSON is malformed")?;
        if parsed.as_ref().unwrap_or(&Value::Null) != &bundle[field] {
            return Err("Bundle parsed registry/view differs from exact input text".into());
        }
    }
    let mut ids = BTreeSet::new();
    let registry = bundle["registry"]["documents"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let document_paths: BTreeSet<_> = registry.iter().filter_map(|d| d["path"].as_str()).collect();
    for document in registry {
        if let Some(path) = document["path"].as_str() {
            if !hashes.contains_key(path) {
                return Err("Bundle snapshot omits a registered document".into());
            }
        }
    }
    for entry in &entries {
        if entry.sha256.is_some()
            && !document_paths.contains(entry.path.as_str())
            && !texts.contains_key(entry.path.as_str())
        {
            return Err("Bundle omits an exact compiler/config/view/registry text".into());
        }
    }
    for document in bundle["documents"].as_array().into_iter().flatten() {
        let id = document["id"].as_str().ok_or("Invalid document ID")?;
        let Some(original) = registry.iter().find(|d| d["id"] == id) else {
            return Err("Bundle document is absent from registry".into());
        };
        if !ids.insert(id) {
            return Err("Duplicate bundle document ID".into());
        }
        for key in [
            "title",
            "kind",
            "path",
            "revision",
            "sha256",
            "pages",
            "review",
            "uncertainty",
        ] {
            if document[key] != original[key] {
                return Err("Bundle document differs from snapshotted registry".into());
            }
        }
        for application in document["applicability"].as_array().into_iter().flatten() {
            if !original["applicability"]
                .as_array()
                .is_some_and(|a| a.contains(application))
            {
                return Err("Bundle document applicability is absent from registry".into());
            }
            let selection = &bundle["selection"];
            let all = selection["instances"].as_array().is_some_and(Vec::is_empty)
                && selection["groups"].as_array().is_some_and(Vec::is_empty);
            let matched = application["instances"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|i| {
                    selection["instances"]
                        .as_array()
                        .is_some_and(|a| a.contains(i))
                })
                || application["groups"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|g| {
                        selection["groups"]
                            .as_array()
                            .is_some_and(|a| a.contains(g))
                    });
            if !all && !matched {
                return Err("Bundle document is not applicable to selection".into());
            }
        }
        if let Some(path) = document["path"].as_str() {
            if hashes.get(path) != Some(&document["actual_sha256"].as_str()) {
                return Err("Bundle document actual hash differs from snapshot".into());
            }
        }
    }
    Ok(())
}

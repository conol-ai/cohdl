use crate::proposal_files::{manifest, Entry};
use crate::proposal_schema as schema;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn sources(bundle: &Value) -> BTreeMap<&str, &str> {
    bundle["sources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| Some((s["path"].as_str()?, s["text"].as_str()?)))
        .collect()
}

pub fn bundle(bundle: &Value, project: &Path) -> Result<(), String> {
    let canonical = project.canonicalize().map_err(|_| "Project unavailable")?;
    if bundle["project"].as_str() != canonical.to_str()
        || bundle["design"] != bundle["model"]["design"]
    {
        return Err("Bundle project/design does not match this project".into());
    }
    let snapshot = manifest(&bundle["snapshot"])?;
    let hashes: BTreeMap<_, _> = snapshot
        .iter()
        .map(|e| (e.path.as_str(), e.sha256.as_deref()))
        .collect();
    let design = bundle["design"]
        .as_str()
        .ok_or("Bundle design is missing")?;
    for path in [
        "cohdl.toml",
        "cohdl.lock",
        "design.lock",
        "explorer/documents.json",
        &format!("views/{design}.view.json"),
    ] {
        if !hashes.contains_key(path) {
            return Err("Bundle snapshot omits a mandatory input".into());
        }
    }
    let mut seen = BTreeSet::new();
    for source in bundle["sources"].as_array().into_iter().flatten() {
        let path = source["path"].as_str().ok_or("Invalid bundle source")?;
        let text = source["text"].as_str().ok_or("Invalid source text")?;
        let hash = cohdl::hash::sha256_hex(text.as_bytes());
        if !seen.insert(path)
            || source["sha256"] != hash
            || hashes.get(path) != Some(&Some(hash.as_str()))
        {
            return Err("Bundle source bytes/hash do not match snapshot".into());
        }
    }
    crate::proposal_documents::bindings(bundle, &sources(bundle))?;
    let instances: BTreeSet<_> = bundle["model"]["instances"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["path"].as_str())
        .collect();
    if bundle["selection"]["instances"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|i| !i.as_str().is_some_and(|s| instances.contains(s)))
    {
        return Err("Bundle selection names an unknown instance".into());
    }
    for group in bundle["selection"]["groups"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let valid = bundle["views"]["views"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|v| {
                v["name"] == group["view"]
                    && (group["region"] == "Other"
                        || v["regions"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .any(|r| r["name"] == group["region"]))
            });
        if !valid {
            return Err("Bundle selection names an unknown group".into());
        }
    }
    Ok(())
}

fn source_citation(citation: &Value, texts: &BTreeMap<&str, &str>) -> bool {
    let source = &citation["source"];
    if source.is_null() {
        return true;
    }
    let Some(text) = source["path"].as_str().and_then(|p| texts.get(p)) else {
        return false;
    };
    source["sha256"] == cohdl::hash::sha256_hex(text.as_bytes())
        && source["line"]
            .as_u64()
            .is_some_and(|line| line <= text.lines().count() as u64)
}

fn document_citation(citation: &Value, bundle: &Value) -> bool {
    if citation["document"].is_null() {
        return true;
    }
    bundle["documents"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|document| {
            if document["id"] != citation["document"]
                || document["status"] != "ready"
                || document["review"] != "reviewed"
                || !crate::proposal_files::hash(&document["sha256"])
                || document["sha256"] != document["actual_sha256"]
                || !document["path"].is_string()
            {
                return false;
            }
            let applicable = document["applicability"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|a| a["relation"] == "exact");
            let page = citation["page"].as_u64();
            applicable
                && if document["kind"] == "pdf" {
                    page.is_some_and(|page| {
                        document["pages"]
                            .as_u64()
                            .is_some_and(|total| page <= total)
                    })
                } else {
                    document["kind"] == "markdown" && page.is_none()
                }
        })
}

pub fn proposal(proposal: &Value, bundle: &Value, run: &Value) -> Result<Vec<Entry>, String> {
    if proposal["kind"] != run["kind"] {
        return Err("Proposal kind differs from run".into());
    }
    let texts = sources(bundle);
    for citation in proposal["citations"].as_array().into_iter().flatten() {
        if (!citation["source"].is_null()
            && !citation["source"]["path"]
                .as_str()
                .is_some_and(|p| p.ends_with(".cohdl")))
            || !source_citation(citation, &texts)
            || !document_citation(citation, bundle)
        {
            return Err("Proposal citation is not verified by the bundle".into());
        }
    }
    let mut after = manifest(&bundle["snapshot"])?;
    let mut paths = BTreeSet::new();
    let design = bundle["design"].as_str().unwrap_or("");
    let changes = proposal["changes"]
        .as_array()
        .ok_or("Proposal changes missing")?;
    if (proposal["kind"] == "plan") != changes.is_empty() {
        return Err("Plan is read-only; group/connect require edits".into());
    }
    for change in proposal["changes"].as_array().into_iter().flatten() {
        let path = change["path"].as_str().ok_or("Invalid change path")?;
        let allowed = match proposal["kind"].as_str() {
            Some("group") => path == format!("views/{design}.view.json"),
            Some("connect") => path.starts_with("src/") && path.ends_with(".cohdl"),
            _ => false,
        };
        if !allowed || !paths.insert(path) {
            return Err("Proposal edits duplicate or forbidden paths".into());
        }
        let entry = after
            .iter_mut()
            .find(|e| e.path == path)
            .ok_or("Change path is absent from snapshot")?;
        if json!(entry.sha256) != change["before_sha256"] {
            return Err("Change before hash differs from snapshot".into());
        }
        let old = change["old"].as_str().ok_or("Invalid old text")?;
        let new = change["new"].as_str().ok_or("Invalid new text")?;
        if new.len() > 1024 * 1024 {
            return Err("Replacement text exceeds limit".into());
        }
        let content = match &entry.sha256 {
            None if old.is_empty() && proposal["kind"] == "group" => new.to_owned(),
            Some(_) if !old.is_empty() => {
                let before = texts.get(path).ok_or("Bundle omits exact edited text")?;
                if before.match_indices(old).count() != 1 {
                    return Err("Edit old text must occur exactly once".into());
                }
                before.replacen(old, new, 1)
            }
            _ => return Err("Invalid existing/new edit precondition".into()),
        };
        if content.len() > crate::proposal_files::JSON_LIMIT as usize
            || texts.get(path).is_some_and(|t| **t == content)
        {
            return Err("Edit is empty or exceeds output limit".into());
        }
        if proposal["kind"] == "group" {
            crate::proposal_views::validate(content.as_bytes(), bundle)?;
        }
        entry.sha256 = Some(cohdl::hash::sha256_hex(content.as_bytes()));
    }
    Ok(after)
}

pub fn validation(validation: &Value, bundle: &Value, after: &[Entry]) -> Result<(), String> {
    if validation["decision"] != "PASS" {
        return Ok(());
    }
    if !schema::validation_pass(validation)
        || validation["before"] != bundle["snapshot"]
        || manifest(&validation["after"])? != after
    {
        return Err(
            "Validation PASS lacks matching snapshots or successful compiler evidence".into(),
        );
    }
    Ok(())
}

pub fn receipt(
    receipt: &Value,
    validation: &Value,
    bundle: &Value,
    after: &[Entry],
    proposal: &Value,
) -> Result<(), String> {
    if !schema::validation_pass(validation) {
        return Err("Receipt requires original compiler PASS".into());
    }
    let applied = receipt["apply"]["outcome"] == "applied";
    if !receipt["apply_validation"].is_null() {
        self::validation(&receipt["apply_validation"], bundle, after)?;
    }
    if (applied && !schema::validation_pass(&receipt["apply_validation"]))
        || receipt["before"] != bundle["snapshot"]
        || manifest(&receipt["after"])? != after
    {
        return Err("Receipt snapshots or fresh compiler evidence do not match".into());
    }
    let files = receipt["files"].as_array().ok_or("Receipt files missing")?;
    let changes = proposal["changes"]
        .as_array()
        .ok_or("Proposal changes missing")?;
    if files.len() != changes.len() {
        return Err("Receipt file count differs from proposal".into());
    }
    let mut seen = BTreeSet::new();
    for file in files {
        let path = file["path"].as_str().ok_or("Invalid receipt path")?;
        let change = changes
            .iter()
            .find(|c| c["path"] == path)
            .ok_or("Receipt has an unrelated file")?;
        let before = file["before"].as_str();
        let content = file["after"].as_str().ok_or("Invalid receipt after text")?;
        if !seen.insert(path)
            || json!(before.map(|b| cohdl::hash::sha256_hex(b.as_bytes()))) != file["before_sha256"]
            || file["before_sha256"] != change["before_sha256"]
            || file["after_sha256"] != cohdl::hash::sha256_hex(content.as_bytes())
            || after
                .iter()
                .find(|e| e.path == path)
                .map(|e| json!(e.sha256))
                != Some(file["after_sha256"].clone())
        {
            return Err("Receipt file bytes/hashes do not match proposal".into());
        }
        let expected = before.map_or_else(
            || change["new"].as_str().unwrap_or("").to_owned(),
            |b| {
                b.replacen(
                    change["old"].as_str().unwrap_or(""),
                    change["new"].as_str().unwrap_or(""),
                    1,
                )
            },
        );
        if content != expected {
            return Err("Receipt bytes do not implement the exact proposal edit".into());
        }
    }
    Ok(())
}

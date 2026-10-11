use crate::document_api::Response;
use crate::proposal_evidence as evidence;
use crate::proposal_files::{self as files, Entry};
use crate::proposal_schema as schema;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Prepared,
    Pending,
    Failed,
    Proposed,
    Rejected,
    Validated,
    Applied,
    Reverted,
    Interrupted,
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Freshness {
    Fresh,
    Stale,
    Unknown,
}
type Snapshots = BTreeMap<String, Result<Vec<Entry>, String>>;

fn json(code: u16, value: Value) -> Response {
    Response {
        code,
        content_type: "application/json",
        body: value.to_string().into_bytes(),
    }
}

fn record(
    project: &Path,
    id: &str,
    name: &str,
    expected: Option<&Value>,
    issues: &mut Vec<String>,
) -> Option<(Value, String)> {
    let relative = format!("explorer/proposals/{id}/{name}");
    let bytes = match files::bytes(project, &relative, files::JSON_LIMIT) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return None,
        Err(error) => {
            issues.push(format!("{name}: {error}"));
            return None;
        }
    };
    let hash = cohdl::hash::sha256_hex(&bytes);
    if expected.is_some_and(|expected| expected != &hash) {
        issues.push(format!("{name}: exact-byte SHA-256 does not match"));
        return None;
    }
    match serde_json::from_slice(&bytes) {
        Ok(value) => Some((value, hash)),
        Err(_) => {
            issues.push(format!("{name}: malformed JSON"));
            None
        }
    }
}

fn checked(
    value: Option<(Value, String)>,
    predicate: impl FnOnce(&Value) -> bool,
    name: &str,
    issues: &mut Vec<String>,
) -> Option<(Value, String)> {
    value.filter(|(value, _)| {
        let valid = predicate(value);
        if !valid {
            issues.push(format!("{name}: invalid schema or evidence bindings"));
        }
        valid
    })
}

fn provider_status(provider: &Value) -> Status {
    if !provider["finished_at"].is_null() {
        return if schema::succeeded(provider) {
            Status::Proposed
        } else {
            Status::Failed
        };
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let pending = schema::instant(&provider["heartbeat_at"])
        .and_then(|t| u128::try_from(t).ok())
        .and_then(|t| now.checked_sub(t))
        .is_some_and(|age| age <= 20_000);
    if pending {
        Status::Pending
    } else {
        Status::Interrupted
    }
}

fn completed(operation: &Value, outcome: &str) -> bool {
    operation["outcome"] == outcome
        && schema::timestamp(&operation["finished_at"])
        && operation["error"].is_null()
        && schema::instant(&operation["finished_at"]) >= schema::instant(&operation["started_at"])
}

fn lifecycle(
    run: &Value,
    proposal: &Value,
    validation: &Value,
    receipt: &Value,
    invalid: bool,
    bundle: &Value,
) -> Status {
    if run.is_null() {
        return if !invalid && !bundle.is_null() {
            Status::Prepared
        } else {
            Status::Rejected
        };
    }
    let provider = provider_status(&run["provider"]);
    if provider == Status::Failed {
        return provider;
    }
    if invalid {
        return Status::Rejected;
    }
    if provider != Status::Proposed {
        return provider;
    }
    if proposal.is_null() {
        return Status::Rejected;
    }
    if !receipt.is_null() {
        if !completed(&receipt["apply"], "applied") {
            return Status::Interrupted;
        }
        if !receipt["revert"].is_null() {
            return if completed(&receipt["revert"], "reverted") {
                Status::Reverted
            } else {
                Status::Interrupted
            };
        }
        return Status::Applied;
    }
    if validation.is_null() {
        Status::Proposed
    } else if schema::validation_pass(validation) {
        Status::Validated
    } else {
        Status::Rejected
    }
}

fn freshness(
    project: &Path,
    bundle: &Value,
    receipt: &Value,
    status: Status,
    cache: &mut Snapshots,
    issues: &mut Vec<String>,
) -> Freshness {
    let Some(design) = bundle["design"].as_str() else {
        return Freshness::Unknown;
    };
    if status == Status::Interrupted && !receipt.is_null() {
        return Freshness::Unknown;
    }
    let expected = if status == Status::Applied {
        &receipt["after"]
    } else if status == Status::Reverted {
        &receipt["before"]
    } else {
        &bundle["snapshot"]
    };
    let current = cache
        .entry(design.into())
        .or_insert_with(|| files::snapshot(project, design));
    match (current, files::manifest(expected)) {
        (Ok(current), Ok(expected)) if current == &expected => Freshness::Fresh,
        (Ok(_), Ok(_)) => {
            issues.push(
                "Current source/document input snapshot differs from the retained evidence".into(),
            );
            Freshness::Stale
        }
        (Err(error), _) => {
            issues.push(format!("Cannot establish current freshness: {error}"));
            Freshness::Unknown
        }
        _ => Freshness::Unknown,
    }
}

fn artifacts(project: &Path, id: &str) -> Vec<Value> {
    files::LOG_NAMES
        .into_iter()
        .filter_map(|name| {
            let path = files::confined(project, &format!("explorer/proposals/{id}/{name}")).ok()?;
            let meta = path.metadata().ok()?;
            (meta.is_file() && meta.len() <= files::LOG_LIMIT)
                .then(|| json!({"name":name,"url":format!("/api/proposals/{id}/artifacts/{name}")}))
        })
        .collect()
}

fn detail(project: &Path, id: &str, cache: &mut Snapshots) -> Value {
    let mut issues = Vec::new();
    let raw = record(project, id, "run.json", None, &mut issues);
    let run = checked(raw, |v| schema::run(v, id), "run.json", &mut issues)
        .map(|x| x.0)
        .unwrap_or(Value::Null);
    let expected = (!run.is_null()).then_some(&run["bundle_sha256"]);
    let raw = record(project, id, "bundle.json", expected, &mut issues);
    let mut bundle = checked(raw, schema::bundle, "bundle.json", &mut issues)
        .map(|x| x.0)
        .unwrap_or(Value::Null);
    if bundle.is_null() {
        issues.push("Immutable bundle is missing or invalid".into());
    } else if let Err(error) = evidence::bundle(&bundle, project) {
        issues.push(error);
        bundle = Value::Null;
    }
    let expected =
        (!run.is_null() && !run["proposal_sha256"].is_null()).then_some(&run["proposal_sha256"]);
    let raw = record(project, id, "proposal.json", expected, &mut issues);
    let proposal = checked(raw, schema::proposal, "proposal.json", &mut issues)
        .map(|x| x.0)
        .unwrap_or(Value::Null);
    if expected.is_some() && proposal.is_null() {
        issues.push("Referenced proposal is missing or invalid".into());
    }
    if !proposal.is_null() && expected.is_none() {
        issues.push("Proposal has no authoritative run hash".into());
    }
    let after = if !proposal.is_null() && !bundle.is_null() {
        match evidence::proposal(&proposal, &bundle, &run) {
            Ok(after) => Some(after),
            Err(error) => {
                issues.push(error);
                None
            }
        }
    } else {
        None
    };
    let raw = record(project, id, "validation.json", None, &mut issues);
    let validation = checked(
        raw,
        |v| schema::validation(v, &run),
        "validation.json",
        &mut issues,
    );
    let validation_hash = validation.as_ref().map(|x| x.1.clone());
    let validation = validation.map(|x| x.0).unwrap_or(Value::Null);
    if let Some(after) = &after {
        if !validation.is_null() {
            if let Err(error) = evidence::validation(&validation, &bundle, after) {
                issues.push(error);
            }
        }
    }
    let raw = record(project, id, "receipt.json", None, &mut issues);
    let receipt = checked(
        raw,
        |v| schema::receipt(v, &run),
        "receipt.json",
        &mut issues,
    )
    .map(|x| x.0)
    .unwrap_or(Value::Null);
    if !receipt.is_null() {
        if validation_hash.is_none_or(|hash| receipt["validation_sha256"] != hash) {
            issues.push("Receipt validation hash does not match exact original report".into());
        }
        if let Some(after) = &after {
            if let Err(error) = evidence::receipt(&receipt, &validation, &bundle, after, &proposal)
            {
                issues.push(error);
            }
        } else {
            issues.push("Receipt lacks a verified proposal".into());
        }
    }
    if !run.is_null() && provider_status(&run["provider"]) == Status::Proposed && proposal.is_null()
    {
        issues.push("Finished provider has no verified structured proposal".into());
    }
    let status = lifecycle(
        &run,
        &proposal,
        &validation,
        &receipt,
        !issues.is_empty(),
        &bundle,
    );
    let freshness = freshness(project, &bundle, &receipt, status, cache, &mut issues);
    let bundle_summary = if bundle.is_null() {
        Value::Null
    } else {
        json!({"design":bundle["design"],"selection":bundle["selection"],"snapshot":bundle["snapshot"]})
    };
    let mut hydrated = run;
    if hydrated.is_null() {
        hydrated = json!({"id":id});
    }
    hydrated["proposal"] = proposal;
    hydrated["bundle"] = bundle_summary;
    hydrated["validation"] = validation;
    hydrated["receipt"] = receipt;
    json!({"schema":1,"run":hydrated,"status":status,"freshness":freshness,"issues":issues,"artifacts":artifacts(project,id)})
}

pub fn handle(path: &str, project: &Path) -> Option<Response> {
    if path == "/api/proposals" {
        let ids = match files::run_ids(project) {
            Ok(ids) => ids,
            Err(error) => return Some(json(422, json!({"error":error}))),
        };
        let mut cache = Snapshots::new();
        let runs: Vec<_> = ids.iter().map(|id| {
            let detail = detail(project, id, &mut cache);
            json!({"id":id,"title":detail["run"]["proposal"]["title"].as_str().unwrap_or(id),"kind":detail["run"]["kind"],"status":detail["status"],"freshness":detail["freshness"],"issues":detail["issues"]})
        }).collect();
        return Some(json(200, json!({"schema":1,"runs":runs})));
    }
    let tail = path.strip_prefix("/api/proposals/")?;
    let segments: Vec<_> = tail.split('/').collect();
    let id = segments[0];
    if !files::identifier(id) {
        return Some(json(404, json!({"error":"Proposal not found"})));
    }
    let directory = files::confined(project, &format!("explorer/proposals/{id}"));
    if !directory.is_ok_and(|p| p.is_dir()) {
        return Some(json(404, json!({"error":"Proposal not found"})));
    }
    if segments.len() == 1 {
        return Some(json(200, detail(project, id, &mut Snapshots::new())));
    }
    if segments.len() == 3 && segments[1] == "artifacts" && files::LOG_NAMES.contains(&segments[2])
    {
        let bytes = files::bytes(
            project,
            &format!("explorer/proposals/{id}/{}", segments[2]),
            files::LOG_LIMIT,
        );
        if let Ok(Some(body)) = bytes {
            return Some(Response {
                code: 200,
                content_type: "text/plain; charset=utf-8",
                body,
            });
        }
    }
    Some(json(
        404,
        json!({"error":"Proposal artifact or endpoint not found"}),
    ))
}

use crate::proposal_files::{hash, manifest, relative};
use serde_json::Value;

fn exact_integer(value: &serde_json::Number) -> Option<i128> {
    value
        .as_i64()
        .map(i128::from)
        .or_else(|| value.as_u64().map(i128::from))
        .or_else(|| {
            value
                .as_f64()
                .filter(|n| n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= u64::MAX as f64)
                .map(|n| n as i128)
        })
}

pub fn model_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            a == b || exact_integer(a).is_some_and(|n| Some(n) == exact_integer(b))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| model_equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| model_equal(a, b)))
        }
        _ => left == right,
    }
}

pub fn object(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
}
pub fn text(value: &Value) -> bool {
    value.as_str().is_some_and(|s| !s.trim().is_empty())
}
fn nullable_text(value: &Value) -> bool {
    value.is_null() || text(value)
}
pub fn strings(value: &Value, limit: usize) -> bool {
    value
        .as_array()
        .is_some_and(|a| a.len() <= limit && a.iter().all(|s| s.is_string()))
}
pub fn timestamp(value: &Value) -> bool {
    instant(value).is_some()
}
pub fn instant(value: &Value) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value.as_str()?)
        .ok()
        .map(|t| t.timestamp_millis())
}
pub fn process(value: &Value, provider: bool) -> bool {
    let extra = if provider {
        vec!["engine", "heartbeat_at"]
    } else {
        vec!["result"]
    };
    let mut keys = vec![
        "command",
        "cwd",
        "started_at",
        "finished_at",
        "exit_code",
        "signal",
        "error",
    ];
    keys.extend(extra);
    object(value, &keys)
        && strings(&value["command"], 256)
        && value["command"]
            .as_array()
            .is_some_and(|a| !a.is_empty() && a.iter().all(text))
        && text(&value["cwd"])
        && timestamp(&value["started_at"])
        && (value["finished_at"].is_null() || timestamp(&value["finished_at"]))
        && (value["exit_code"].is_null() || value["exit_code"].as_i64().is_some())
        && nullable_text(&value["signal"])
        && nullable_text(&value["error"])
        && (!provider
            || (value["engine"] == "codex"
                && (value["heartbeat_at"].is_null() || timestamp(&value["heartbeat_at"]))))
        && (provider
            || value["result"].is_null()
            || (object(
                &value["result"],
                &["schema_version", "verdict", "diagnostics"],
            ) && value["result"]["schema_version"] == 1
                && ["pass", "fail"].contains(&value["result"]["verdict"].as_str().unwrap_or(""))
                && value["result"]["diagnostics"]
                    .as_array()
                    .is_some_and(|a| a.len() <= 10000 && a.iter().all(Value::is_object))))
}
pub fn succeeded(value: &Value) -> bool {
    timestamp(&value["finished_at"])
        && value["exit_code"] == 0
        && value["signal"].is_null()
        && value["error"].is_null()
        && instant(&value["finished_at"]) >= instant(&value["started_at"])
}
pub fn kind(value: &Value) -> bool {
    ["group", "plan", "connect"].contains(&value.as_str().unwrap_or(""))
}
pub fn run(value: &Value, id: &str) -> bool {
    object(
        value,
        &[
            "schema",
            "id",
            "kind",
            "created_at",
            "bundle_sha256",
            "proposal_sha256",
            "provider",
        ],
    ) && value["schema"] == 1
        && value["id"] == id
        && kind(&value["kind"])
        && timestamp(&value["created_at"])
        && hash(&value["bundle_sha256"])
        && (value["proposal_sha256"].is_null() || hash(&value["proposal_sha256"]))
        && process(&value["provider"], true)
}
fn selection(value: &Value) -> bool {
    object(value, &["schema", "instances", "groups"])
        && value["schema"] == 1
        && strings(&value["instances"], 2048)
        && value["groups"].as_array().is_some_and(|a| {
            a.len() <= 512
                && a.iter().all(|g| {
                    object(g, &["view", "region"]) && text(&g["view"]) && text(&g["region"])
                })
        })
}
pub fn bundle(value: &Value) -> bool {
    object(
        value,
        &[
            "schema",
            "project",
            "design",
            "selection",
            "snapshot",
            "sources",
            "model",
            "views",
            "registry",
            "documents",
            "constraints",
            "omissions",
        ],
    ) && value["schema"] == 1
        && text(&value["project"])
        && text(&value["design"])
        && selection(&value["selection"])
        && manifest(&value["snapshot"]).is_ok()
        && value["model"].is_object()
        && value["model"]["schema_version"] == 1
        && value["model"]["verdict"] == "pass"
        && (value["views"].is_null() || value["views"].is_object())
        && (value["registry"].is_null() || value["registry"].is_object())
        && strings(&value["omissions"], 4096)
        && value["sources"].as_array().is_some_and(|a| {
            a.len() <= 2048
                && a.iter().all(|s| {
                    object(s, &["path", "sha256", "text"])
                        && s["path"].as_str().is_some_and(relative)
                        && hash(&s["sha256"])
                        && s["text"].is_string()
                })
        })
        && value["documents"]
            .as_array()
            .is_some_and(|a| a.len() <= 512 && a.iter().all(crate::proposal_documents::shape))
        && value["constraints"]
            .as_array()
            .is_some_and(|a| a.len() <= 4096 && a.iter().all(Value::is_object))
}
pub fn proposal(value: &Value) -> bool {
    object(
        value,
        &[
            "schema",
            "kind",
            "title",
            "rationale",
            "uncertainty",
            "citations",
            "changes",
        ],
    ) && value["schema"] == 1
        && kind(&value["kind"])
        && text(&value["title"])
        && text(&value["rationale"])
        && text(&value["uncertainty"])
        && value["citations"]
            .as_array()
            .is_some_and(|a| !a.is_empty() && a.len() <= 256 && a.iter().all(citation))
        && value["changes"].as_array().is_some_and(|a| {
            a.len() <= 64
                && a.iter().all(|c| {
                    object(c, &["path", "before_sha256", "old", "new"])
                        && c["path"].as_str().is_some_and(relative)
                        && (c["before_sha256"].is_null() || hash(&c["before_sha256"]))
                        && c["old"].is_string()
                        && c["new"].is_string()
                })
        })
}
fn citation(value: &Value) -> bool {
    object(value, &["document", "page", "section", "source"])
        && nullable_text(&value["document"])
        && (value["page"].is_null()
            || value["page"]
                .as_u64()
                .is_some_and(|n| n > 0 && n <= u32::MAX as u64))
        && nullable_text(&value["section"])
        && (!value["document"].is_null() || (value["page"].is_null() && value["section"].is_null()))
        && (!value["document"].is_null() || !value["source"].is_null())
        && (value["source"].is_null()
            || (object(&value["source"], &["path", "line", "sha256"])
                && value["source"]["path"].as_str().is_some_and(relative)
                && value["source"]["line"].as_u64().is_some_and(|n| n > 0)
                && hash(&value["source"]["sha256"])))
}
pub fn validation(value: &Value, run: &Value) -> bool {
    object(
        value,
        &[
            "schema",
            "id",
            "proposal_sha256",
            "bundle_sha256",
            "started_at",
            "finished_at",
            "decision",
            "issues",
            "before",
            "after",
            "compiler",
        ],
    ) && bindings(value, run)
        && timestamp(&value["started_at"])
        && timestamp(&value["finished_at"])
        && instant(&value["finished_at"]) >= instant(&value["started_at"])
        && ["PASS", "FAIL"].contains(&value["decision"].as_str().unwrap_or(""))
        && strings(&value["issues"], 10000)
        && manifest(&value["before"]).is_ok()
        && manifest(&value["after"]).is_ok()
        && (value["compiler"].is_null() || process(&value["compiler"], false))
}
pub fn bindings(value: &Value, run: &Value) -> bool {
    value["schema"] == 1
        && value["id"] == run["id"]
        && hash(&value["proposal_sha256"])
        && value["proposal_sha256"] == run["proposal_sha256"]
        && value["bundle_sha256"] == run["bundle_sha256"]
}
pub fn validation_pass(value: &Value) -> bool {
    value["decision"] == "PASS"
        && value["issues"].as_array().is_some_and(Vec::is_empty)
        && succeeded(&value["compiler"])
        && value["compiler"]["result"]["schema_version"] == 1
        && value["compiler"]["result"]["verdict"] == "pass"
}
fn operation(value: &Value, outcomes: &[&str]) -> bool {
    object(value, &["started_at", "finished_at", "outcome", "error"])
        && timestamp(&value["started_at"])
        && (value["finished_at"].is_null() || timestamp(&value["finished_at"]))
        && nullable_text(&value["error"])
        && outcomes.contains(&value["outcome"].as_str().unwrap_or(""))
}
pub fn receipt(value: &Value, run: &Value) -> bool {
    object(
        value,
        &[
            "schema",
            "id",
            "proposal_sha256",
            "bundle_sha256",
            "validation_sha256",
            "apply_validation",
            "before",
            "after",
            "files",
            "apply",
            "revert",
        ],
    ) && bindings(value, run)
        && hash(&value["validation_sha256"])
        && (value["apply_validation"].is_null() || validation(&value["apply_validation"], run))
        && manifest(&value["before"]).is_ok()
        && manifest(&value["after"]).is_ok()
        && operation(
            &value["apply"],
            &["applying", "applied", "rolled_back", "interrupted"],
        )
        && (value["revert"].is_null()
            || operation(&value["revert"], &["reverting", "reverted", "interrupted"]))
        && value["files"].as_array().is_some_and(|a| {
            a.len() <= 64
                && a.iter().all(|f| {
                    object(
                        f,
                        &["path", "before_sha256", "after_sha256", "before", "after"],
                    ) && f["path"].as_str().is_some_and(relative)
                        && (f["before_sha256"].is_null() || hash(&f["before_sha256"]))
                        && hash(&f["after_sha256"])
                        && (f["before"].is_null() || f["before"].is_string())
                        && f["after"].is_string()
                })
        })
}

#[cfg(test)]
mod tests {
    use super::model_equal;
    use serde_json::json;

    #[test]
    fn numeric_equality_is_exact_without_large_integer_rounding() {
        assert!(model_equal(&json!(1), &json!(1.0)));
        assert!(!model_equal(&json!(1), &json!(1.01)));
        assert!(!model_equal(
            &json!(9_007_199_254_740_993u64),
            &json!(9_007_199_254_740_992.0f64)
        ));
        assert!(!model_equal(&json!(u64::MAX), &json!(u64::MAX as f64)));
        assert!(model_equal(&json!(i64::MIN), &json!(i64::MIN as f64)));
    }
}

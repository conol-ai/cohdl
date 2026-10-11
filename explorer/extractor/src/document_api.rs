use crate::document_registry;
use crate::document_types::Kind;
use crate::http_files::percent_decode;
use std::collections::BTreeMap;
use std::path::Path;

const SOURCE_LIMIT: u64 = 1024 * 1024;
const SNIPPET_LIMIT: usize = 64 * 1024;

pub struct Response {
    pub code: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

fn json(code: u16, value: serde_json::Value) -> Response {
    Response {
        code,
        content_type: "application/json",
        body: value.to_string().into_bytes(),
    }
}

fn query(full: &str) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    for pair in full
        .split_once('?')
        .map_or("", |p| p.1)
        .split('&')
        .filter(|s| !s.is_empty())
    {
        let (key, value) = pair.split_once('=').ok_or("Expected query key=value")?;
        let key = percent_decode(key, true).ok_or("Invalid query encoding")?;
        let value = percent_decode(value, true).ok_or("Invalid query encoding")?;
        if result.insert(key, value).is_some() {
            return Err("Duplicate query parameter".into());
        }
    }
    Ok(result)
}

fn source(project: &Path, full: &str) -> Result<serde_json::Value, String> {
    let parameters = query(full)?;
    if parameters
        .keys()
        .any(|k| !["path", "line", "count"].contains(&k.as_str()))
    {
        return Err("Unknown source query parameter".into());
    }
    let relative = parameters.get("path").ok_or("Source path is required")?;
    if !document_registry::safe_relative(relative) || !relative.ends_with(".cohdl") {
        return Err("Source must be a project-local .cohdl file".into());
    }
    let line = parameters
        .get("line")
        .map(|v| v.parse::<usize>())
        .transpose()
        .map_err(|_| "Invalid source line")?
        .unwrap_or(1);
    let count = parameters
        .get("count")
        .map(|v| v.parse::<usize>())
        .transpose()
        .map_err(|_| "Invalid source count")?
        .unwrap_or(20);
    if line == 0 || count == 0 || count > 200 {
        return Err("Source requires line >= 1 and count between 1 and 200".into());
    }
    let file = document_registry::safe_file(project, relative, "cohdl")
        .ok_or("Source is missing or not an allowed project-local .cohdl file")?;
    let bytes = document_registry::read_bounded(&file, SOURCE_LIMIT)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "Source is not UTF-8")?;
    let total = text.lines().count();
    if line > total {
        return Err("Source line is outside the file".into());
    }
    let selected: Vec<_> = text.lines().skip(line - 1).take(count).collect();
    let snippet = selected.join("\n");
    if snippet.len() > SNIPPET_LIMIT {
        return Err("Source excerpt exceeds supported size".into());
    }
    Ok(
        serde_json::json!({ "path": relative, "line": line, "count": selected.len(), "total_lines": total, "sha256": cohdl::hash::sha256_hex(&bytes), "text": snippet }),
    )
}

pub fn handle(
    path: &str,
    full: &str,
    project: &Path,
    dist: &Path,
    model: &str,
) -> Option<Response> {
    if path == "/api/source" {
        return Some(match source(project, full) {
            Ok(value) => json(200, value),
            Err(error) => json(422, serde_json::json!({"error": error})),
        });
    }
    if path != "/api/documents" && !path.starts_with("/api/documents/") {
        return None;
    }
    let registry = match document_registry::load(project, dist, model) {
        Ok(registry) => registry,
        Err((code, error)) => return Some(json(code, serde_json::json!({"error": error}))),
    };
    if path == "/api/documents" {
        return Some(json(200, serde_json::to_value(registry).unwrap()));
    }
    let id = path.strip_prefix("/api/documents/").unwrap_or("");
    let document = registry
        .documents
        .iter()
        .find(|d| d.document.id == id && d.open_url.is_some());
    let Some(document) = document else {
        return Some(json(
            404,
            serde_json::json!({"error":"Document is unavailable or invalid"}),
        ));
    };
    match document_registry::document_bytes(project, &document.document) {
        Ok(bytes)
            if document.actual_sha256.as_deref()
                == Some(cohdl::hash::sha256_hex(&bytes).as_str()) =>
        {
            Some(Response {
                code: 200,
                content_type: if document.document.kind == Kind::Pdf {
                    "application/pdf"
                } else {
                    "text/plain; charset=utf-8"
                },
                body: bytes,
            })
        }
        _ => Some(json(
            404,
            serde_json::json!({"error":"Document changed or is unavailable"}),
        )),
    }
}

pub type WatchSnapshot = Vec<(String, Option<(std::time::SystemTime, u64)>)>;

pub fn watch_snapshot(project: &Path) -> WatchSnapshot {
    let mut snapshot = Vec::new();
    let metadata = |relative: &str, extension: &str| {
        document_registry::safe_file(project, relative, extension)
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok().map(|time| (time, m.len())))
    };
    snapshot.push((
        "explorer/documents.json".into(),
        metadata("explorer/documents.json", "json"),
    ));
    if let Ok(registry) = document_registry::read(project) {
        for document in registry.documents {
            let stamp = document
                .path
                .as_deref()
                .filter(|p| document_registry::safe_relative(p))
                .and_then(|path| {
                    metadata(
                        path,
                        if document.kind == Kind::Pdf {
                            "pdf"
                        } else {
                            "md"
                        },
                    )
                });
            snapshot.push((document.id, stamp));
        }
    }
    snapshot
}

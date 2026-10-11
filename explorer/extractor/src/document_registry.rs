use crate::document_types::*;
use crate::http_files::{confined_file, view_file};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Component, Path};

pub const REGISTRY_LIMIT: u64 = 4 * 1024 * 1024;
pub const PDF_LIMIT: u64 = 64 * 1024 * 1024;
pub const TEXT_LIMIT: u64 = 4 * 1024 * 1024;

pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|_| "File is not readable")?;
    if !file
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() <= limit)
    {
        return Err("File exceeds the supported size".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "File is not readable")?;
    if bytes.len() as u64 > limit {
        return Err("File exceeds the supported size".into());
    }
    Ok(bytes)
}

pub fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\\', '\0'])
        && Path::new(value).components().all(
            |c| matches!(c, Component::Normal(name) if !name.to_string_lossy().starts_with('.')),
        )
}

pub fn read(project: &Path) -> Result<Registry, (u16, String)> {
    let relative = "explorer/documents.json";
    let exists = std::fs::symlink_metadata(project.join(relative)).is_ok();
    let file = confined_file(project, relative).ok_or((
        if exists { 422 } else { 404 },
        "Document registry is unavailable".into(),
    ))?;
    let bytes = read_bounded(&file, REGISTRY_LIMIT).map_err(|e| (422, e))?;
    let registry: Registry =
        serde_json::from_slice(&bytes).map_err(|_| (422, "Malformed document registry".into()))?;
    if registry.schema != 1 || registry.documents.len() > 512 || registry.constraints.len() > 4096 {
        return Err((
            422,
            "Document registry requires schema 1 and supported record counts".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    if registry.documents.iter().any(|d| {
        d.id.is_empty()
            || d.id.len() > 128
            || !d
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !ids.insert(&d.id)
    }) {
        return Err((422, "Document IDs must be unique ASCII identifiers".into()));
    }
    Ok(registry)
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096
}

fn citation_issues(
    document: &Document,
    page: Option<u32>,
    section: Option<&str>,
    issues: &mut Vec<String>,
) {
    if page.is_some_and(|p| p == 0 || document.pages.is_some_and(|total| p > total)) {
        issues.push("Citation page is outside the declared PDF pages".into());
    }
    if document.kind == Kind::Markdown && page.is_some() {
        issues.push("Markdown citations use sections, not PDF pages".into());
    }
    if section.is_some_and(|s| !nonempty(s)) {
        issues.push("Citation section must be nonempty".into())
    }
}

struct Targets {
    instances: BTreeSet<String>,
    groups: BTreeSet<(String, String)>,
}

impl Targets {
    fn load(model: &str, project: &Path, dist: &Path) -> Self {
        let model: serde_json::Value = serde_json::from_str(model).unwrap_or_default();
        let instances = model["instances"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v["path"].as_str().map(str::to_owned))
            .collect();
        let design = model["design"].as_str().unwrap_or("");
        let config = view_file(project, dist, &format!("views/{design}.view.json"))
            .and_then(|p| read_bounded(&p, REGISTRY_LIMIT).ok())
            .filter(|b| crate::view_config::validate(b, design).is_ok())
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
            .unwrap_or_default();
        let mut groups = BTreeSet::new();
        for view in config["views"].as_array().into_iter().flatten() {
            let Some(name) = view["name"].as_str() else {
                continue;
            };
            groups.insert((name.to_owned(), "Other".into()));
            for region in view["regions"].as_array().into_iter().flatten() {
                if let Some(region) = region["name"].as_str() {
                    groups.insert((name.to_owned(), region.to_owned()));
                }
            }
        }
        Self { instances, groups }
    }

    fn validate(&self, instances: &[String], groups: &[Group], issues: &mut Vec<String>) {
        if instances.is_empty() && groups.is_empty() {
            issues.push("Explicit applicability is required".into())
        }
        if instances.iter().any(|i| !self.instances.contains(i)) {
            issues.push("Applicability names an unknown instance".into())
        }
        if groups
            .iter()
            .any(|g| !self.groups.contains(&(g.view.clone(), g.region.clone())))
        {
            issues.push("Applicability names an unknown view/group".into())
        }
        if instances.len() > 10000 || groups.len() > 512 {
            issues.push("Applicability exceeds supported target counts".into())
        }
    }
}

fn document_issues(document: &Document, targets: &Targets) -> Vec<String> {
    let mut issues = Vec::new();
    if !nonempty(&document.title) {
        issues.push("Document title must be nonempty".into())
    }
    if matches!(document.review, Review::Rejected) {
        issues.push("Document review is rejected".into())
    }
    if document.revision.as_ref().is_some_and(|r| !nonempty(r)) {
        issues.push("Revision must be nonempty or null for unknown".into())
    }
    if document.pages == Some(0) || (document.kind == Kind::Markdown && document.pages.is_some()) {
        issues.push("PDF pages must be positive; Markdown pages must be null".into())
    }
    if document.sha256.as_ref().is_some_and(|h| {
        h.len() != 64
            || !h
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }) {
        issues.push("SHA-256 must contain 64 lowercase hexadecimal digits".into())
    }
    if document.source_url.as_ref().is_some_and(|url| {
        url.strip_prefix("https://").is_none_or(|rest| {
            rest.split('/')
                .next()
                .is_none_or(|host| host.is_empty() || host.starts_with(['?', '#', ':']))
        }) || url.chars().any(char::is_whitespace)
    }) {
        issues.push("Source URL must be an absolute HTTPS URL".into())
    }
    if document.applicability.is_empty() || document.applicability.len() > 4096 {
        issues.push("Document requires between 1 and 4096 applicability records".into())
    }
    for applicability in &document.applicability {
        targets.validate(&applicability.instances, &applicability.groups, &mut issues);
        citation_issues(
            document,
            applicability.page,
            applicability.section.as_deref(),
            &mut issues,
        );
        if matches!(applicability.relation, Relation::Related)
            && applicability
                .uncertainty
                .as_ref()
                .or(document.uncertainty.as_ref())
                .is_none_or(|u| !nonempty(u))
        {
            issues.push("Related-part applicability requires explicit uncertainty".into());
        }
    }
    issues
}

pub fn document_bytes(project: &Path, document: &Document) -> Result<Vec<u8>, String> {
    let relative = document
        .path
        .as_deref()
        .ok_or("Local original is unavailable")?;
    let extension = Path::new(relative)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let allowed = match document.kind {
        Kind::Pdf => extension.eq_ignore_ascii_case("pdf"),
        Kind::Markdown => extension.eq_ignore_ascii_case("md"),
    };
    if !safe_relative(relative) || !allowed {
        return Err("Document path is not an allowed project-local file".into());
    }
    let file = confined_file(project, relative)
        .ok_or("Local document is missing or outside the project")?;
    let bytes = read_bounded(
        &file,
        if document.kind == Kind::Pdf {
            PDF_LIMIT
        } else {
            TEXT_LIMIT
        },
    )?;
    match document.kind {
        Kind::Pdf if !bytes.starts_with(b"%PDF-") => return Err("Document is not a PDF".into()),
        Kind::Markdown if std::str::from_utf8(&bytes).is_err() => {
            return Err("Markdown is not UTF-8".into())
        }
        _ => {}
    }
    Ok(bytes)
}

pub fn load(project: &Path, dist: &Path, model: &str) -> Result<RegistryResult, (u16, String)> {
    let registry = read(project)?;
    let targets = Targets::load(model, project, dist);
    let documents = registry
        .documents
        .iter()
        .map(|document| {
            let mut issues = document_issues(document, &targets);
            let bytes = document_bytes(project, document);
            let actual_sha256 = bytes.as_ref().ok().map(|b| cohdl::hash::sha256_hex(b));
            if document.path.is_some() && document.sha256.is_none() {
                issues.push("Expected local hash is unknown".into())
            }
            if let (Some(expected), Some(actual)) = (&document.sha256, &actual_sha256) {
                if expected != actual {
                    issues.push("Local document hash differs from the expected revision".into())
                }
            }
            let missing = document.path.as_deref().is_none_or(|relative| {
                safe_relative(relative)
                    && std::fs::symlink_metadata(project.join(relative))
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
            });
            if let Err(error) = bytes {
                issues.push(error);
            }
            let status = if missing {
                "missing"
            } else if issues.is_empty() {
                "ready"
            } else {
                "invalid"
            };
            let open_url = (status == "ready").then(|| format!("/api/documents/{}", document.id));
            DocumentResult {
                document: document.clone(),
                status,
                actual_sha256,
                issues,
                open_url,
            }
        })
        .collect();
    let constraints = registry
        .constraints
        .into_iter()
        .map(|constraint| {
            let mut issues = Vec::new();
            if !nonempty(&constraint.text) {
                issues.push("Constraint text must be nonempty".into())
            }
            targets.validate(&constraint.instances, &constraint.groups, &mut issues);
            if constraint.citations.is_empty() {
                issues.push("Constraint requires source citations".into())
            }
            for citation in &constraint.citations {
                match registry
                    .documents
                    .iter()
                    .find(|d| d.id == citation.document)
                {
                    Some(document) => citation_issues(
                        document,
                        citation.page,
                        citation.section.as_deref(),
                        &mut issues,
                    ),
                    None => issues.push("Constraint cites an unknown document".into()),
                }
            }
            ConstraintResult { constraint, issues }
        })
        .collect();
    Ok(RegistryResult {
        schema: 1,
        documents,
        constraints,
    })
}

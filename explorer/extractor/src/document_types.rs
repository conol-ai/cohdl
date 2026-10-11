use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema: u32,
    pub documents: Vec<Document>,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub kind: Kind,
    pub path: Option<String>,
    pub source_url: Option<String>,
    pub revision: Option<String>,
    pub sha256: Option<String>,
    pub pages: Option<u32>,
    pub review: Review,
    pub uncertainty: Option<String>,
    pub applicability: Vec<Applicability>,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Pdf,
    Markdown,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Review {
    Unreviewed,
    Reviewed,
    Rejected,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub view: String,
    pub region: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Relation {
    Exact,
    Related,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub instances: Vec<String>,
    pub groups: Vec<Group>,
    pub relation: Relation,
    pub page: Option<u32>,
    pub section: Option<String>,
    pub facts: Option<BTreeMap<String, String>>,
    pub uncertainty: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub document: String,
    pub page: Option<u32>,
    pub section: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub text: String,
    pub instances: Vec<String>,
    pub groups: Vec<Group>,
    pub citations: Vec<Citation>,
    pub uncertainty: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentResult {
    #[serde(flatten)]
    pub document: Document,
    pub status: &'static str,
    pub actual_sha256: Option<String>,
    pub issues: Vec<String>,
    pub open_url: Option<String>,
}

#[derive(Serialize)]
pub struct ConstraintResult {
    #[serde(flatten)]
    pub constraint: Constraint,
    pub issues: Vec<String>,
}

#[derive(Serialize)]
pub struct RegistryResult {
    pub schema: u32,
    pub documents: Vec<DocumentResult>,
    pub constraints: Vec<ConstraintResult>,
}

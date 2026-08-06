//! ExplorerModel v1 — the stable JSON projection of `Checked{ir, world}`.
//!
//! Contract: versioned via `schema_version`; the frontend consumes only this,
//! never compiler internals. Collections are sorted for byte-stable output.

use serde::Serialize;

#[derive(Serialize)]
pub struct ExplorerModel {
    pub schema_version: u32,
    pub design: String,
    pub verdict: String,
    pub instances: Vec<Instance>,
    pub nets: Vec<Net>,
    pub nc: Vec<NcPin>,
    pub diagnostics: Vec<Diag>,
    pub derived: Derived,
    /// Real pad geometry for every content-bearing footprint referenced by a
    /// bound part, keyed by footprint fq name (RFC-018 declarations projected
    /// to plain mm floats — display-only, never byte-stability-critical).
    pub footprints: std::collections::BTreeMap<String, FootprintGeo>,
}

#[derive(Serialize)]
pub struct FootprintGeo {
    pub pads: Vec<FpPad>,
    pub mount_holes: Vec<FpHole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub courtyard: Option<FpShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<FpShape>,
}

#[derive(Serialize)]
pub struct FpPad {
    pub number: String,
    /// "rect" | "circle" | "oval"
    pub shape: String,
    pub x: f64,
    pub y: f64,
    /// (w, h) for rect/oval; (d) for circle. mm.
    pub size: Vec<f64>,
    pub rotate: u16,
    /// Round drill diameter or slot (w, l). Present only on PTH pads. mm.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drill: Vec<f64>,
    pub pth: bool,
}

#[derive(Serialize)]
pub struct FpHole {
    pub shape: String,
    pub x: f64,
    pub y: f64,
    pub size: Vec<f64>,
    pub plated: bool,
}

#[derive(Serialize)]
pub struct FpShape {
    pub shape: String,
    pub x: f64,
    pub y: f64,
    pub size: Vec<f64>,
}

#[derive(Serialize)]
pub struct Instance {
    pub path: String,
    pub device_fq: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<PartRef>,
    pub impl_traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_hint: Option<String>,
    pub specs: Vec<SpecEntry>,
    pub span: SrcSpan,
    pub docs: Vec<DocRef>,
    pub pins: Vec<Pin>,
}

#[derive(Serialize)]
pub struct DocRef {
    pub name: String,
    /// Absolute on-disk path, served via /api/file (allow-listed roots).
    pub abs: String,
}

#[derive(Serialize)]
pub struct SpecEntry {
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct PartRef {
    pub fq: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footprint: Option<String>,
}

#[derive(Serialize)]
pub struct Pin {
    pub logical: String,
    pub numbers: Vec<String>,
    pub role: String,
    pub obligation: String,
    pub connected: bool,
    pub nc: bool,
}

#[derive(Serialize)]
pub struct Net {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voltage: Option<String>,
    pub is_gnd: bool,
    pub members: Vec<NetMember>,
    pub span: SrcSpan,
}

#[derive(Serialize)]
pub struct NetMember {
    pub instance_path: String,
    pub logical_pin: String,
    pub numbers: Vec<String>,
}

#[derive(Serialize)]
pub struct NcPin {
    pub instance_path: String,
    pub logical_pin: String,
}

#[derive(Serialize)]
pub struct Diag {
    pub code: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SrcSpan>,
}

#[derive(Serialize)]
pub struct SrcSpan {
    pub file: String,
    pub line: u32,
    pub col: u32,
}

/// Pre-computed display hints (spec R1/R2 and grouping seeds for G2).
#[derive(Serialize)]
pub struct Derived {
    /// Instance paths implementing TwoTerminal (R1 inline candidates).
    pub two_terminal: Vec<String>,
    /// Net names to render as rail stubs (R2): is_gnd, voltage-annotated,
    /// or fan-out above the threshold.
    pub rails: Vec<String>,
    /// fn-expansion groups: instances sharing a call-chain path prefix.
    pub fn_groups: Vec<FnGroup>,
    /// RFC-027 #[bypass] facts: decoupling cap -> its target IC.
    pub bypasses: Vec<Bypass>,
}

#[derive(Serialize)]
pub struct Bypass {
    pub cap: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct FnGroup {
    pub name: String,
    pub members: Vec<String>,
}

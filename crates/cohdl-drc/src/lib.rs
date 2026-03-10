//! Built-in DRC (Design Rule Check) engine operating on [`ConnectivityIR`].
//!
//! Each rule is a struct implementing the [`DrcRule`] trait. [`DrcRunner`]
//! collects all built-in rules and executes them, returning a flat list of
//! [`DrcDiagnostic`]s. Diagnostics whose `rule_id` matches an
//! `#[allow(rule_name)]` annotation on the owning instance are suppressed.

pub mod rules;
pub mod user_rules;

use cohdl_sema::connectivity::ConnectivityIR;
use cohdl_syntax::ast::Span;

// ── Diagnostic types ─────────────────────────────────────────────────────────

/// Severity level for a DRC diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

/// A single diagnostic emitted by a DRC rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrcDiagnostic {
    /// Machine-readable rule identifier (e.g. `"E001"`, `"W002"`).
    pub rule_id: String,
    /// Severity.
    pub level: DiagnosticLevel,
    /// Source span associated with the offending construct.
    pub span: Span,
    /// Hierarchical instance path (empty for net-level diagnostics).
    pub instance_path: String,
    /// Human-readable message.
    pub message: String,
}

// ── DrcRule trait ─────────────────────────────────────────────────────────────

/// A single, stateless DRC rule.
pub trait DrcRule {
    /// Execute the rule against the given IR, returning zero or more diagnostics.
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic>;
}

// ── DrcRunner ────────────────────────────────────────────────────────────────

/// Collects all built-in rules and runs them, optionally filtering out
/// suppressed diagnostics.
pub struct DrcRunner {
    rules: Vec<Box<dyn DrcRule>>,
    /// Set of `(instance_path, rule_id)` pairs that should be suppressed.
    suppressed: Vec<(String, String)>,
}

impl DrcRunner {
    /// Create a runner pre-loaded with every built-in rule.
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(rules::VoltageExceed),
                Box::new(rules::PolarityMismatch),
                Box::new(rules::SpecNotSatisfied),
                Box::new(rules::TraitNotImpl),
                Box::new(rules::MissingSpecField),
                Box::new(rules::UnconnectedPin),
                Box::new(rules::FloatingNet),
                Box::new(rules::SingleDriver),
                Box::new(rules::MultiDriver),
            ],
            suppressed: Vec::new(),
        }
    }

    /// Add a [`UserDefinedRuleSet`] containing user-defined rules from `trait`
    /// and `device` definitions.  User-defined diagnostics appear alongside
    /// built-in ones in the output.
    pub fn add_user_rules(&mut self, rule_set: user_rules::UserDefinedRuleSet) {
        self.rules.push(Box::new(rule_set));
    }

    /// Register an `#[allow(rule_name)]` suppression for a given instance path.
    pub fn allow(&mut self, instance_path: &str, rule_id: &str) {
        self.suppressed
            .push((instance_path.to_string(), rule_id.to_string()));
    }

    /// Run every rule and return the (possibly filtered) diagnostics.
    pub fn run(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut diags: Vec<DrcDiagnostic> = Vec::new();
        for rule in &self.rules {
            diags.extend(rule.check(ir));
        }
        // Filter out suppressed diagnostics.
        diags.retain(|d| {
            !self
                .suppressed
                .iter()
                .any(|(path, id)| d.instance_path == *path && d.rule_id == *id)
        });
        diags
    }
}

impl Default for DrcRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

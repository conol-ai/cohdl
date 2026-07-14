//! Diagnostics: stable code + severity + precise span + message.
//!
//! Diagnostics are the primary AI-facing product surface — the repair loop
//! feeds the rendered text back to the model verbatim. Message quality and
//! specificity are load-bearing (Constitution: every diagnostic carries a
//! precise source span and a stable, documented error code; RFC tooling
//! sections: name the exact pin/trait/unit, never a bare "type mismatch").
//!
//! The code registry is informal-but-stable for MVP (RFC-011 will formalize
//! it): see docs/error-codes.md.

use crate::span::{SourceMap, Span};
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    /// Primary label; rendered with the caret line. Secondary labels point at
    /// related spans (e.g. "the earlier impl is here").
    pub primary: Label,
    pub secondary: Vec<Label>,
    /// `help:` lines — the machine-actionable suggestion, when one exists.
    pub help: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Error, span, message)
    }

    pub fn warning(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Warning, span, message)
    }

    fn new(code: &'static str, severity: Severity, span: Span, message: impl Into<String>) -> Self {
        let message = message.into();
        Diagnostic {
            code,
            severity,
            primary: Label {
                span,
                message: String::new(),
            },
            message,
            secondary: Vec::new(),
            help: Vec::new(),
        }
    }

    pub fn with_primary_label(mut self, message: impl Into<String>) -> Self {
        self.primary.message = message.into();
        self
    }

    pub fn with_secondary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.secondary.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

/// Collects diagnostics across the pipeline.
#[derive(Debug, Default)]
pub struct Diagnostics {
    diags: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, d: Diagnostic) {
        self.diags.push(d);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.diags.extend(other.diags);
    }

    pub fn has_errors(&self) -> bool {
        self.diags.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diags
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.diags.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diags.iter()
    }

    /// Deterministic presentation order: by file, then span start, then code.
    /// Exact duplicates (same span, code, and message) are collapsed — two
    /// passes can independently diagnose the same construct (e.g. an
    /// unresolved reference caught at both the rewrite pass and expansion),
    /// and a construct should be reported once.
    pub fn sort(&mut self, _sm: &SourceMap) {
        self.diags.sort_by_key(|d| {
            (
                d.primary.span.file,
                d.primary.span.start,
                d.primary.span.end,
                d.code,
            )
        });
        self.diags.dedup_by(|a, b| {
            a.code == b.code
                && a.primary.span == b.primary.span
                && a.message == b.message
                && a.primary.message == b.primary.message
        });
    }

    /// Render all diagnostics in rustc-like plain-text form (no color; the
    /// output is consumed verbatim by the repair loop and by tests).
    pub fn render(&self, sm: &SourceMap) -> String {
        let mut out = String::new();
        for d in &self.diags {
            render_one(&mut out, d, sm);
            out.push('\n');
        }
        if !self.diags.is_empty() {
            let e = self.error_count();
            let w = self.warning_count();
            let mut parts = Vec::new();
            if e > 0 {
                parts.push(format!("{} error{}", e, if e == 1 { "" } else { "s" }));
            }
            if w > 0 {
                parts.push(format!("{} warning{}", w, if w == 1 { "" } else { "s" }));
            }
            let _ = writeln!(out, "{} emitted", parts.join(", "));
        }
        out
    }
}

fn render_one(out: &mut String, d: &Diagnostic, sm: &SourceMap) {
    let sev = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    let _ = writeln!(out, "{}[{}]: {}", sev, d.code, d.message);

    let mut labels: Vec<(&Label, bool)> = Vec::with_capacity(1 + d.secondary.len());
    labels.push((&d.primary, true));
    for l in &d.secondary {
        labels.push((l, false));
    }

    for (i, (label, is_primary)) in labels.iter().enumerate() {
        let span = label.span;
        let lc = sm.line_col(span.file, span.start);
        let arrow = if i == 0 { " -->" } else { "  ::" };
        let _ = writeln!(
            out,
            "{} {}:{}:{}",
            arrow,
            sm.name(span.file),
            lc.line,
            lc.col
        );
        let line_text = sm.line_text(span.file, lc.line);
        let gutter = format!("{}", lc.line);
        let pad = " ".repeat(gutter.len());
        let _ = writeln!(out, "{} |", pad);
        let _ = writeln!(out, "{} | {}", gutter, line_text);
        // Caret width: span length clamped to the rest of the line, min 1.
        let end_lc = sm.line_col(span.file, span.end);
        let width = if end_lc.line == lc.line && end_lc.col > lc.col {
            (end_lc.col - lc.col) as usize
        } else {
            1
        };
        let marker = if *is_primary { "^" } else { "-" }.repeat(width.max(1));
        let _ = writeln!(
            out,
            "{} | {}{}{}{}",
            pad,
            " ".repeat((lc.col - 1) as usize),
            marker,
            if label.message.is_empty() { "" } else { " " },
            label.message
        );
    }
    for h in &d.help {
        let _ = writeln!(out, "  = help: {}", h);
    }
}

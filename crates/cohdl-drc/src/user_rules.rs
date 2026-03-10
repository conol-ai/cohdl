//! User-defined DRC rules declared in `trait` and `device` definitions.
//!
//! A user-defined rule has the form:
//!
//! ```hdl
//! rule voltage_derating(level: Warning) {
//!   assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
//!   message: "voltage {voltage}V exceeds 80% derating of {voltage_rating}V"
//! }
//! ```
//!
//! This module provides:
//! - [`RuleExpr`]: a simple expression IR for rule assertions.
//! - [`UserDefinedRule`]: the definition of a single user-defined rule.
//! - [`UserDefinedRuleSet`]: a collection that handles trait/device rule override.
//! - Evaluation against [`ConnectivityIR`] at DRC time.

use std::collections::HashMap;

use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net};
use cohdl_sema::typeck::{InstanceId, EXTERNAL_INSTANCE};
use cohdl_syntax::ast::Span;

use crate::{DiagnosticLevel, DrcDiagnostic, DrcRule};

// ── Expression IR ────────────────────────────────────────────────────────────

/// A simple expression IR for rule assertion evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleExpr {
    /// A float literal (e.g. `0.8`, `3.3`).
    Float(f64),
    /// Reference to `self.spec.<field>` — resolved from instance
    /// `generic_substitutions`.
    SpecField(String),
    /// `net_voltage(pin_a, pin_b)` — the voltage on the net(s) connecting the
    /// given pins of the current instance.
    NetVoltage { pin_a: String, pin_b: String },
    /// A binary operation.
    Binary {
        op: RuleBinOp,
        lhs: Box<RuleExpr>,
        rhs: Box<RuleExpr>,
    },
}

/// Binary operators supported in rule expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleBinOp {
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// `<`
    Lt,
    /// `>`
    Gt,
}

// ── Evaluation ───────────────────────────────────────────────────────────────

/// Result of evaluating a [`RuleExpr`].
#[derive(Debug, Clone, Copy)]
enum Value {
    Float(f64),
    Bool(bool),
}

/// Context needed for expression evaluation.
struct EvalCtx<'a> {
    /// The instance being checked.
    inst: &'a Instance,
    /// All nets in the design.
    nets: &'a [Net],
    /// Map from InstanceId → &Instance.
    imap: &'a HashMap<InstanceId, &'a Instance>,
}

/// Evaluate a [`RuleExpr`] in the given context.
fn eval(expr: &RuleExpr, ctx: &EvalCtx<'_>) -> Option<Value> {
    match expr {
        RuleExpr::Float(v) => Some(Value::Float(*v)),

        RuleExpr::SpecField(field) => {
            let raw = ctx.inst.generic_substitutions.get(field)?;
            parse_numeric(raw).map(Value::Float)
        }

        RuleExpr::NetVoltage { pin_a, pin_b } => {
            let va = pin_net_voltage(ctx.inst.id, pin_a, ctx.nets, ctx.imap);
            let vb = pin_net_voltage(ctx.inst.id, pin_b, ctx.nets, ctx.imap);
            // If both pins have voltage annotations, return the absolute
            // difference.  If only one does, use that (the other is assumed 0,
            // e.g. GND).  If neither has one, the expression is unevaluable.
            match (va, vb) {
                (Some(a), Some(b)) => Some(Value::Float((a - b).abs())),
                (Some(a), None) => Some(Value::Float(a)),
                (None, Some(b)) => Some(Value::Float(b)),
                (None, None) => None,
            }
        }

        RuleExpr::Binary { op, lhs, rhs } => {
            let l = eval(lhs, ctx)?;
            let r = eval(rhs, ctx)?;
            match (l, r) {
                (Value::Float(a), Value::Float(b)) => {
                    match op {
                        RuleBinOp::Mul => Some(Value::Float(a * b)),
                        RuleBinOp::Div => {
                            if b == 0.0 {
                                None
                            } else {
                                Some(Value::Float(a / b))
                            }
                        }
                        RuleBinOp::Add => Some(Value::Float(a + b)),
                        RuleBinOp::Sub => Some(Value::Float(a - b)),
                        RuleBinOp::Le => Some(Value::Bool(a <= b)),
                        RuleBinOp::Ge => Some(Value::Bool(a >= b)),
                        RuleBinOp::Lt => Some(Value::Bool(a < b)),
                        RuleBinOp::Gt => Some(Value::Bool(a > b)),
                    }
                }
                // Type mismatch — skip silently.
                _ => None,
            }
        }
    }
}

/// Determine the voltage on the net connected to a specific pin of an instance.
fn pin_net_voltage(
    inst_id: InstanceId,
    pin_name: &str,
    nets: &[Net],
    imap: &HashMap<InstanceId, &Instance>,
) -> Option<f64> {
    // Find the net containing (inst_id, pin_name).
    let net = nets.iter().find(|n| {
        n.pins
            .iter()
            .any(|p| p.instance_id == inst_id && p.pin == pin_name)
    })?;

    // Try to derive voltage from other instances on the net (look for a
    // `voltage` generic substitution).
    for pin in &net.pins {
        if pin.instance_id == EXTERNAL_INSTANCE || pin.instance_id == inst_id {
            continue;
        }
        if let Some(other) = imap.get(&pin.instance_id) {
            if let Some(v_str) = other.generic_substitutions.get("voltage") {
                if let Some(v) = parse_voltage_str(v_str) {
                    return Some(v);
                }
            }
        }
    }

    // Fall back to net-name heuristics (e.g. "3V3" → 3.3, "5V" → 5.0).
    parse_net_name_voltage(&net.name)
}

// ── Numeric helpers ──────────────────────────────────────────────────────────

/// Parse a numeric string that may carry an engineering suffix (e.g. `"3.3V"`,
/// `"100nF"`, `"10k"`).  Returns the bare numeric value in base units.
fn parse_numeric(s: &str) -> Option<f64> {
    let s = s.trim();

    // Try direct float parse first.
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }

    // Strip known suffixes and try again.
    let stripped = s
        .trim_end_matches('V')
        .trim_end_matches('v')
        .trim_end_matches('F')
        .trim_end_matches('f')
        .trim_end_matches('A')
        .trim_end_matches('a')
        .trim_end_matches("ohm")
        .trim_end_matches('R');

    // Handle SI multiplier prefixes on what remains.
    if stripped.is_empty() {
        return None;
    }

    let last = stripped.as_bytes()[stripped.len() - 1];
    let (num_part, multiplier) = match last {
        b'p' => (&stripped[..stripped.len() - 1], 1e-12),
        b'n' => (&stripped[..stripped.len() - 1], 1e-9),
        b'u' => (&stripped[..stripped.len() - 1], 1e-6),
        b'm' => (&stripped[..stripped.len() - 1], 1e-3),
        b'k' | b'K' => (&stripped[..stripped.len() - 1], 1e3),
        b'M' => (&stripped[..stripped.len() - 1], 1e6),
        b'G' => (&stripped[..stripped.len() - 1], 1e9),
        _ => (stripped, 1.0),
    };

    num_part.parse::<f64>().ok().map(|v| v * multiplier)
}

/// Parse a voltage string like `"3.3V"` or `"5V"` into an f64.
fn parse_voltage_str(s: &str) -> Option<f64> {
    let numeric = s
        .trim()
        .trim_end_matches('V')
        .trim_end_matches('v');
    numeric.parse::<f64>().ok()
}

/// Extract voltage from a net name such as `"3V3"` → 3.3, `"5V"` → 5.0.
fn parse_net_name_voltage(name: &str) -> Option<f64> {
    if let Some(pos) = name.find('V') {
        let before = &name[..pos];
        let after = &name[pos + 1..];
        if let Ok(int) = before.parse::<u32>() {
            if after.is_empty() {
                return Some(int as f64);
            }
            if let Ok(frac) = after.parse::<u32>() {
                let denom = 10f64.powi(after.len() as i32);
                return Some(int as f64 + frac as f64 / denom);
            }
        }
    }
    None
}

// ── Message template interpolation ───────────────────────────────────────────

/// Interpolate `{field}` placeholders in a message template.
///
/// Supported placeholders:
/// - `{field}` — looks up `field` in `generic_substitutions`.
/// - `{net_voltage}` — uses the pre-computed net voltage (if available).
fn interpolate_message(
    template: &str,
    inst: &Instance,
    net_voltage_val: Option<f64>,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            if name == "net_voltage" {
                if let Some(v) = net_voltage_val {
                    result.push_str(&format!("{v}"));
                } else {
                    result.push_str("?");
                }
            } else if let Some(val) = inst.generic_substitutions.get(&name) {
                result.push_str(val);
            } else {
                result.push_str("?");
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// ── UserDefinedRule ──────────────────────────────────────────────────────────

/// Describes which instances a user-defined rule applies to.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleAppliesTo {
    /// The rule applies to instances of a specific device.
    Device(String),
    /// The rule applies to instances implementing a specific trait.
    Trait(String),
}

/// A single user-defined DRC rule parsed from a `rule` block in a `trait` or
/// `device` definition.
#[derive(Debug, Clone, PartialEq)]
pub struct UserDefinedRule {
    /// Rule name (used as `rule_id` in diagnostics, e.g. `"voltage_derating"`).
    pub name: String,
    /// Severity level.
    pub level: DiagnosticLevel,
    /// The assertion expression that must hold.  When the assertion evaluates
    /// to `false`, a diagnostic is emitted.
    pub assertion: RuleExpr,
    /// Message template with `{field}` interpolation.
    pub message_template: String,
    /// Which instances this rule applies to.
    pub applies_to: RuleAppliesTo,
}

/// Check whether a rule applies to a given instance.
fn rule_matches(rule: &UserDefinedRule, inst: &Instance) -> bool {
    match &rule.applies_to {
        RuleAppliesTo::Device(device) => inst.device == *device,
        RuleAppliesTo::Trait(trait_name) => inst
            .generic_substitutions
            .get("impl_traits")
            .map(|traits| {
                traits
                    .split(',')
                    .any(|t| t.trim() == trait_name.as_str())
            })
            .unwrap_or(false),
    }
}

// ── UserDefinedRuleSet ───────────────────────────────────────────────────────

/// A collection of user-defined rules that handles device-level overrides of
/// trait-level rules.
///
/// When a device defines a rule with the same name as one inherited from a
/// parent trait, the device's version takes precedence for instances of that
/// device.
#[derive(Debug, Clone)]
pub struct UserDefinedRuleSet {
    rules: Vec<UserDefinedRule>,
}

impl UserDefinedRuleSet {
    /// Create an empty set.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the set.
    pub fn add(&mut self, rule: UserDefinedRule) {
        self.rules.push(rule);
    }

    /// Resolve the effective rules for a given instance, applying override
    /// semantics: if both a trait rule and a device rule share the same name,
    /// the device rule wins.
    fn effective_rules(&self, inst: &Instance) -> Vec<&UserDefinedRule> {
        let mut by_name: HashMap<&str, &UserDefinedRule> = HashMap::new();

        // First pass: collect trait-level rules that match.
        for rule in &self.rules {
            if matches!(&rule.applies_to, RuleAppliesTo::Trait(_)) && rule_matches(rule, inst) {
                by_name.insert(&rule.name, rule);
            }
        }

        // Second pass: device-level rules override trait-level ones.
        for rule in &self.rules {
            if matches!(&rule.applies_to, RuleAppliesTo::Device(_)) && rule_matches(rule, inst) {
                by_name.insert(&rule.name, rule);
            }
        }

        by_name.into_values().collect()
    }
}

impl Default for UserDefinedRuleSet {
    fn default() -> Self {
        Self::new()
    }
}

impl DrcRule for UserDefinedRuleSet {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let imap: HashMap<InstanceId, &Instance> =
            ir.instances.iter().map(|i| (i.id, i)).collect();

        let mut out = Vec::new();

        for inst in &ir.instances {
            let effective = self.effective_rules(inst);

            for rule in effective {
                let ctx = EvalCtx {
                    inst,
                    nets: &ir.nets,
                    imap: &imap,
                };

                // Compute net_voltage for message interpolation.
                let nv = compute_net_voltage_for_rule(&rule.assertion, &ctx);

                match eval(&rule.assertion, &ctx) {
                    Some(Value::Bool(true)) => {
                        // Assertion holds — no diagnostic.
                    }
                    Some(Value::Bool(false)) => {
                        let message =
                            interpolate_message(&rule.message_template, inst, nv);
                        out.push(DrcDiagnostic {
                            rule_id: rule.name.clone(),
                            level: rule.level,
                            span: Span { start: 0, end: 0 },
                            instance_path: inst.hierarchical_path.clone(),
                            message,
                        });
                    }
                    // Could not evaluate (missing data) — skip silently.
                    _ => {}
                }
            }
        }

        out
    }
}

/// Walk the expression tree to find a `NetVoltage` node and compute its value
/// for use in message interpolation.
fn compute_net_voltage_for_rule(expr: &RuleExpr, ctx: &EvalCtx<'_>) -> Option<f64> {
    match expr {
        RuleExpr::NetVoltage { .. } => {
            if let Some(Value::Float(v)) = eval(expr, ctx) {
                Some(v)
            } else {
                None
            }
        }
        RuleExpr::Binary { lhs, rhs, .. } => {
            compute_net_voltage_for_rule(lhs, ctx)
                .or_else(|| compute_net_voltage_for_rule(rhs, ctx))
        }
        _ => None,
    }
}

//! Lowering pass: converts a pest CST (`Pairs`) into the typed AST from
//! `cohdl_syntax`.

use cohdl_syntax::ast;
use pest::Parser;

use crate::{CohdlParser, Rule};

// ── ParseError ─────────────────────────────────────────────────────────────

/// An error produced during CST → AST lowering (or during pest parsing).
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Human-readable description of the problem.
    pub message: String,
    /// Source location where the error was detected.
    pub span: ast::Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// ── Convenience helpers ────────────────────────────────────────────────────

type Pair<'i> = pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

fn span_of(pair: &Pair<'_>) -> ast::Span {
    let s = pair.as_span();
    ast::Span {
        start: s.start(),
        end: s.end(),
    }
}

fn err(span: ast::Span, msg: impl Into<String>) -> ParseError {
    ParseError {
        message: msg.into(),
        span,
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

/// Parse a cohdl source string into a typed [`ast::SourceFile`].
///
/// Returns all collected errors if lowering (or initial PEG parsing) fails.
pub fn parse_source_file(src: &str) -> Result<ast::SourceFile, Vec<ParseError>> {
    let pairs = CohdlParser::parse(Rule::file, src).map_err(|e| {
        let (start, end) = match e.location {
            pest::error::InputLocation::Pos(p) => (p, p),
            pest::error::InputLocation::Span((s, e)) => (s, e),
        };
        vec![ParseError {
            message: e.to_string(),
            span: ast::Span { start, end },
        }]
    })?;

    let mut ctx = LowerCtx::new();
    let file = ctx.lower_file(pairs);

    if ctx.errors.is_empty() {
        Ok(file)
    } else {
        Err(ctx.errors)
    }
}

// ── Lowering context ───────────────────────────────────────────────────────

struct LowerCtx {
    errors: Vec<ParseError>,
}

impl LowerCtx {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn error(&mut self, span: ast::Span, msg: impl Into<String>) {
        self.errors.push(err(span, msg));
    }

    // ── File ───────────────────────────────────────────────────────────────

    fn lower_file(&mut self, pairs: Pairs<'_>) -> ast::SourceFile {
        let mut items = Vec::new();
        let mut file_span = ast::Span { start: 0, end: 0 };

        // The top-level `pairs` contains a single `file` pair; drill into it.
        for pair in pairs {
            if pair.as_rule() == Rule::file {
                file_span = span_of(&pair);
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::top_level_item => {
                            items.push(self.lower_top_level_item(inner));
                        }
                        Rule::EOI => {}
                        _ => {}
                    }
                }
            }
        }

        ast::SourceFile {
            items,
            span: file_span,
        }
    }

    // ── Top-level item ─────────────────────────────────────────────────────

    fn lower_top_level_item(&mut self, pair: Pair<'_>) -> ast::TopLevelItem {
        let item_span = span_of(&pair);
        let mut attributes = Vec::new();
        let mut visibility = None;
        let mut kind = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::attribute => {
                    attributes.push(self.lower_attribute(inner));
                }
                Rule::visibility => {
                    visibility = Some(ast::Visibility {
                        span: span_of(&inner),
                    });
                }
                Rule::trait_def => {
                    kind = Some(ast::TopLevelItemKind::Trait(self.lower_trait_def(inner)));
                }
                Rule::device_def => {
                    kind = Some(ast::TopLevelItemKind::Device(self.lower_device_def(inner)));
                }
                Rule::part_def => {
                    kind = Some(ast::TopLevelItemKind::Part(self.lower_part_def(inner)));
                }
                Rule::type_def => {
                    kind = Some(ast::TopLevelItemKind::TypeAlias(
                        self.lower_type_alias(inner),
                    ));
                }
                Rule::fn_def => {
                    kind = Some(ast::TopLevelItemKind::Fn(self.lower_fn_def(inner)));
                }
                Rule::module_def => {
                    kind = Some(ast::TopLevelItemKind::Module(self.lower_module_def(inner)));
                }
                Rule::design_def => {
                    kind = Some(ast::TopLevelItemKind::Design(self.lower_design_def(inner)));
                }
                Rule::use_decl => {
                    kind = Some(ast::TopLevelItemKind::Use(self.lower_use_decl(inner)));
                }
                Rule::mod_decl => {
                    kind = Some(ast::TopLevelItemKind::Mod(self.lower_mod_decl(inner)));
                }
                _ => {}
            }
        }

        let kind = kind.unwrap_or_else(|| {
            self.error(item_span, "expected a top-level item declaration");
            // Return a dummy mod decl to keep going
            ast::TopLevelItemKind::Mod(ast::ModDecl {
                name: ast::Ident {
                    name: String::new(),
                    span: item_span,
                },
                span: item_span,
            })
        });

        ast::TopLevelItem {
            attributes,
            visibility,
            kind,
            span: item_span,
        }
    }

    // ── Attribute ──────────────────────────────────────────────────────────

    fn lower_attribute(&mut self, pair: Pair<'_>) -> ast::Attribute {
        let attr_span = span_of(&pair);
        let inner = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::attribute_inner)
            .expect("attribute must have attribute_inner");

        let mut children = inner.into_inner();
        let name_pair = children.next().expect("attribute must have a name");
        let name = self.lower_ident(name_pair);

        let args = children.next().map(|args_pair| {
            // attribute_args
            let mut strings = Vec::new();
            let mut idents = Vec::new();
            for child in args_pair.into_inner() {
                match child.as_rule() {
                    Rule::string_literal => {
                        strings.push(self.lower_string_literal(child));
                    }
                    Rule::ident => {
                        idents.push(self.lower_ident(child));
                    }
                    _ => {}
                }
            }
            if !strings.is_empty() {
                ast::AttributeArgs::Strings(strings)
            } else {
                ast::AttributeArgs::Idents(idents)
            }
        });

        ast::Attribute {
            name,
            args,
            span: attr_span,
        }
    }

    // ── Ident / Path / DotPath ─────────────────────────────────────────────

    fn lower_ident(&mut self, pair: Pair<'_>) -> ast::Ident {
        ast::Ident {
            name: pair.as_str().to_string(),
            span: span_of(&pair),
        }
    }

    /// Lower an `ident` or `scoped_path` pair into a `Path`.
    fn lower_path_from_pair(&mut self, pair: Pair<'_>) -> ast::Path {
        let path_span = span_of(&pair);
        match pair.as_rule() {
            Rule::scoped_path => {
                let segments: Vec<ast::Ident> =
                    pair.into_inner().map(|p| self.lower_ident(p)).collect();
                ast::Path {
                    segments,
                    span: path_span,
                }
            }
            Rule::ident => ast::Path {
                segments: vec![self.lower_ident(pair)],
                span: path_span,
            },
            _ => {
                self.error(path_span, "expected identifier or scoped path");
                ast::Path {
                    segments: Vec::new(),
                    span: path_span,
                }
            }
        }
    }

    /// Lower `dot_path` rule: `(scoped_path | ident) ~ ("." ~ ident)+`
    fn lower_dot_path(&mut self, pair: Pair<'_>) -> ast::DotPath {
        let dp_span = span_of(&pair);
        let mut children = pair.into_inner();

        let first = children.next().expect("dot_path must have a root");
        let root = self.lower_path_from_pair(first);

        let fields: Vec<ast::Ident> = children.map(|p| self.lower_ident(p)).collect();

        ast::DotPath {
            root,
            fields,
            span: dp_span,
        }
    }

    // ── String literal ─────────────────────────────────────────────────────

    fn lower_string_literal(&mut self, pair: Pair<'_>) -> ast::StringLiteral {
        let sl_span = span_of(&pair);
        let raw = pair.as_str();
        // Strip surrounding quotes
        let value = if raw.len() >= 2 {
            raw[1..raw.len() - 1].to_string()
        } else {
            String::new()
        };
        ast::StringLiteral {
            value,
            span: sl_span,
        }
    }

    // ── Interpolated string ────────────────────────────────────────────────

    fn lower_interpolated_string(&mut self, pair: Pair<'_>) -> ast::InterpolatedString {
        let sp = span_of(&pair);
        let raw_text = pair.as_str();
        let value = if raw_text.len() >= 2 {
            raw_text[1..raw_text.len() - 1].to_string()
        } else {
            String::new()
        };
        ast::InterpolatedString {
            raw: value,
            span: sp,
        }
    }

    // ── Type expression ────────────────────────────────────────────────────

    fn lower_type_expr(&mut self, pair: Pair<'_>) -> ast::TypeExpr {
        let te_span = span_of(&pair);
        let mut path = None;
        let mut generic_args = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::scoped_path | Rule::ident => {
                    path = Some(self.lower_path_from_pair(child));
                }
                Rule::generic_args => {
                    generic_args = Some(self.lower_generic_args(child));
                }
                _ => {}
            }
        }

        let path = path.unwrap_or_else(|| {
            self.error(te_span, "type expression missing path");
            ast::Path {
                segments: Vec::new(),
                span: te_span,
            }
        });

        ast::TypeExpr {
            path,
            generic_args,
            span: te_span,
        }
    }

    // ── Generics ───────────────────────────────────────────────────────────

    fn lower_generic_params(&mut self, pair: Pair<'_>) -> ast::GenericParams {
        let gp_span = span_of(&pair);
        let params: Vec<ast::GenericParam> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::generic_param_def)
            .map(|p| self.lower_generic_param_def(p))
            .collect();

        ast::GenericParams {
            params,
            span: gp_span,
        }
    }

    fn lower_generic_param_def(&mut self, pair: Pair<'_>) -> ast::GenericParam {
        let gp_span = span_of(&pair);
        let mut name = None;
        let mut kind = None;
        let mut default = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::impl_constraint => {
                    kind = Some(ast::GenericParamKind::ImplConstraint(
                        self.lower_impl_constraint(child),
                    ));
                }
                Rule::type_expr if kind.is_none() => {
                    kind = Some(ast::GenericParamKind::Type(self.lower_type_expr(child)));
                }
                Rule::value_expr => {
                    default = Some(self.lower_value_expr(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(gp_span, "generic parameter missing name");
            ast::Ident {
                name: String::new(),
                span: gp_span,
            }
        });

        let kind = kind.unwrap_or_else(|| {
            self.error(gp_span, "generic parameter missing type or constraint");
            ast::GenericParamKind::Type(ast::TypeExpr {
                path: ast::Path {
                    segments: Vec::new(),
                    span: gp_span,
                },
                generic_args: None,
                span: gp_span,
            })
        });

        ast::GenericParam {
            name,
            kind,
            default,
            span: gp_span,
        }
    }

    fn lower_impl_constraint(&mut self, pair: Pair<'_>) -> ast::TraitBound {
        // impl_constraint = { "impl" ~ trait_bound }
        let child = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::trait_bound)
            .expect("impl_constraint must have trait_bound");
        self.lower_trait_bound(child)
    }

    fn lower_trait_bound(&mut self, pair: Pair<'_>) -> ast::TraitBound {
        let tb_span = span_of(&pair);
        let bounds: Vec<ast::TypeExpr> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::type_expr)
            .map(|p| self.lower_type_expr(p))
            .collect();

        ast::TraitBound {
            bounds,
            span: tb_span,
        }
    }

    fn lower_generic_args(&mut self, pair: Pair<'_>) -> ast::GenericArgs {
        let ga_span = span_of(&pair);
        let args: Vec<ast::GenericArg> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::generic_arg)
            .map(|p| self.lower_generic_arg(p))
            .collect();

        ast::GenericArgs {
            args,
            span: ga_span,
        }
    }

    fn lower_generic_arg(&mut self, pair: Pair<'_>) -> ast::GenericArg {
        let ga_span = span_of(&pair);
        let mut children = pair.into_inner();

        let name = self.lower_ident(children.next().expect("generic_arg must have name"));
        let value_pair = children.next().expect("generic_arg must have value");
        let value = self.lower_value_expr(value_pair);

        ast::GenericArg {
            name,
            value,
            span: ga_span,
        }
    }

    // ── Expressions ────────────────────────────────────────────────────────

    fn lower_value_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        // value_expr = { expr }
        let child = pair.into_inner().next().expect("value_expr must have expr");
        self.lower_expr(child)
    }

    fn lower_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        // expr = { comparison_expr }
        let child = pair
            .into_inner()
            .next()
            .expect("expr must have comparison_expr");
        self.lower_comparison_expr(child)
    }

    fn lower_comparison_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        let ce_span = span_of(&pair);
        let mut children = pair.into_inner();
        let lhs_pair = children.next().expect("comparison_expr must have lhs");
        let mut lhs = self.lower_additive_expr(lhs_pair);

        if let Some(op_pair) = children.next() {
            let op = match op_pair.as_str() {
                "<=" => ast::BinaryOp::Le,
                ">=" => ast::BinaryOp::Ge,
                "==" => ast::BinaryOp::Eq,
                "!=" => ast::BinaryOp::Ne,
                other => {
                    self.error(span_of(&op_pair), format!("unknown comparison op: {other}"));
                    ast::BinaryOp::Eq
                }
            };
            let rhs_pair = children.next().expect("comparison must have rhs");
            let rhs = self.lower_additive_expr(rhs_pair);
            lhs = ast::Expr {
                kind: ast::ExprKind::Binary(ast::BinaryExpr {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                }),
                span: ce_span,
            };
        }

        lhs
    }

    fn lower_additive_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        let ae_span = span_of(&pair);
        let mut children = pair.into_inner();
        let first = children.next().expect("additive_expr must have operand");
        let mut result = self.lower_multiplicative_expr(first);

        while let Some(op_pair) = children.next() {
            let op = match op_pair.as_str() {
                "+" => ast::BinaryOp::Add,
                "-" => ast::BinaryOp::Sub,
                other => {
                    self.error(span_of(&op_pair), format!("unknown additive op: {other}"));
                    ast::BinaryOp::Add
                }
            };
            let rhs_pair = children.next().expect("additive must have rhs");
            let rhs = self.lower_multiplicative_expr(rhs_pair);
            result = ast::Expr {
                kind: ast::ExprKind::Binary(ast::BinaryExpr {
                    lhs: Box::new(result),
                    op,
                    rhs: Box::new(rhs),
                }),
                span: ae_span,
            };
        }

        result
    }

    fn lower_multiplicative_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        let me_span = span_of(&pair);
        let mut children = pair.into_inner();
        let first = children
            .next()
            .expect("multiplicative_expr must have operand");
        let mut result = self.lower_unary_expr(first);

        while let Some(op_pair) = children.next() {
            let op = match op_pair.as_str() {
                "*" => ast::BinaryOp::Mul,
                "/" => ast::BinaryOp::Div,
                other => {
                    self.error(
                        span_of(&op_pair),
                        format!("unknown multiplicative op: {other}"),
                    );
                    ast::BinaryOp::Mul
                }
            };
            let rhs_pair = children.next().expect("multiplicative must have rhs");
            let rhs = self.lower_unary_expr(rhs_pair);
            result = ast::Expr {
                kind: ast::ExprKind::Binary(ast::BinaryExpr {
                    lhs: Box::new(result),
                    op,
                    rhs: Box::new(rhs),
                }),
                span: me_span,
            };
        }

        result
    }

    fn lower_unary_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        let ue_span = span_of(&pair);
        let mut children = pair.into_inner();
        let first = children.next().expect("unary_expr must have content");

        if first.as_rule() == Rule::unary_op {
            let op = match first.as_str() {
                "-" => ast::UnaryOp::Neg,
                "!" => ast::UnaryOp::Not,
                other => {
                    self.error(span_of(&first), format!("unknown unary op: {other}"));
                    ast::UnaryOp::Neg
                }
            };
            let operand_pair = children.next().expect("unary must have operand");
            let operand = self.lower_atom_expr(operand_pair);
            ast::Expr {
                kind: ast::ExprKind::Unary(ast::UnaryExpr {
                    op,
                    operand: Box::new(operand),
                }),
                span: ue_span,
            }
        } else {
            // No unary op, it's just atom_expr
            self.lower_atom_expr(first)
        }
    }

    fn lower_atom_expr(&mut self, pair: Pair<'_>) -> ast::Expr {
        let atom_span = span_of(&pair);
        let inner = pair
            .into_inner()
            .next()
            .expect("atom_expr must have content");

        match inner.as_rule() {
            Rule::expr => {
                // Parenthesized expression: "(" ~ expr ~ ")"
                let inner_expr = self.lower_expr(inner);
                ast::Expr {
                    kind: ast::ExprKind::Paren(Box::new(inner_expr)),
                    span: atom_span,
                }
            }
            Rule::boolean => {
                let val = inner.as_str() == "true";
                ast::Expr {
                    kind: ast::ExprKind::Bool(val),
                    span: atom_span,
                }
            }
            Rule::eng_number => {
                let en = self.lower_eng_number(inner);
                ast::Expr {
                    kind: ast::ExprKind::EngineeringNumber(en),
                    span: atom_span,
                }
            }
            Rule::string_literal => {
                let sl = self.lower_string_literal(inner);
                ast::Expr {
                    kind: ast::ExprKind::String(sl),
                    span: atom_span,
                }
            }
            Rule::fn_call_expr => {
                let fc = self.lower_fn_call_expr(inner);
                ast::Expr {
                    kind: ast::ExprKind::FnCall(fc),
                    span: atom_span,
                }
            }
            Rule::dot_path => {
                let dp = self.lower_dot_path(inner);
                ast::Expr {
                    kind: ast::ExprKind::DotPath(dp),
                    span: atom_span,
                }
            }
            Rule::type_expr => {
                // Could be an integer (plain number), or a path/type
                let te = self.lower_type_expr(inner);
                // Check if this looks like a plain integer (single ident that's all digits)
                if te.generic_args.is_none()
                    && te.path.segments.len() == 1
                    && te.path.segments[0].name.chars().all(|c| c.is_ascii_digit())
                {
                    if let Ok(n) = te.path.segments[0].name.parse::<u64>() {
                        return ast::Expr {
                            kind: ast::ExprKind::Integer(n),
                            span: atom_span,
                        };
                    }
                }
                ast::Expr {
                    kind: ast::ExprKind::Type(te),
                    span: atom_span,
                }
            }
            _ => {
                self.error(
                    atom_span,
                    format!("unexpected atom expression: {:?}", inner.as_rule()),
                );
                ast::Expr {
                    kind: ast::ExprKind::Bool(false),
                    span: atom_span,
                }
            }
        }
    }

    fn lower_eng_number(&mut self, pair: Pair<'_>) -> ast::EngineeringNumber {
        let en_span = span_of(&pair);
        let text = pair.as_str();

        // Split into numeric part and optional suffix
        let split_idx = text
            .find(|c: char| c.is_ascii_alphabetic())
            .unwrap_or(text.len());

        let number = text[..split_idx].to_string();
        let suffix = if split_idx < text.len() {
            Some(text[split_idx..].to_string())
        } else {
            None
        };

        ast::EngineeringNumber {
            number,
            suffix,
            span: en_span,
        }
    }

    fn lower_fn_call_expr(&mut self, pair: Pair<'_>) -> ast::FnCallExpr {
        let fc_span = span_of(&pair);
        let mut path = None;
        let mut args = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::scoped_path | Rule::ident => {
                    path = Some(self.lower_path_from_pair(child));
                }
                Rule::fn_call_expr_arg => {
                    let expr = child
                        .into_inner()
                        .next()
                        .expect("fn_call_expr_arg must have expr");
                    args.push(self.lower_expr(expr));
                }
                _ => {}
            }
        }

        let path = path.unwrap_or_else(|| {
            self.error(fc_span, "function call missing path");
            ast::Path {
                segments: Vec::new(),
                span: fc_span,
            }
        });

        ast::FnCallExpr {
            path,
            args,
            span: fc_span,
        }
    }

    // ── Use / Mod ──────────────────────────────────────────────────────────

    fn lower_use_decl(&mut self, pair: Pair<'_>) -> ast::UseDecl {
        let ud_span = span_of(&pair);
        let use_path_pair = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::use_path)
            .expect("use_decl must have use_path");

        let tree = self.lower_use_path(use_path_pair);

        ast::UseDecl {
            tree,
            span: ud_span,
        }
    }

    fn lower_use_path(&mut self, pair: Pair<'_>) -> ast::UseTree {
        let ut_span = span_of(&pair);
        let mut prefix = Vec::new();
        let mut group = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident => {
                    prefix.push(self.lower_ident(child));
                }
                Rule::use_group => {
                    let idents: Vec<ast::Ident> = child
                        .into_inner()
                        .filter(|p| p.as_rule() == Rule::ident)
                        .map(|p| self.lower_ident(p))
                        .collect();
                    group = Some(idents);
                }
                _ => {}
            }
        }

        ast::UseTree {
            prefix,
            group,
            span: ut_span,
        }
    }

    fn lower_mod_decl(&mut self, pair: Pair<'_>) -> ast::ModDecl {
        let md_span = span_of(&pair);
        let name = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::ident)
            .map(|p| self.lower_ident(p))
            .unwrap_or_else(|| {
                self.error(md_span, "mod declaration missing name");
                ast::Ident {
                    name: String::new(),
                    span: md_span,
                }
            });

        ast::ModDecl {
            name,
            span: md_span,
        }
    }

    // ── Trait ──────────────────────────────────────────────────────────────

    fn lower_trait_def(&mut self, pair: Pair<'_>) -> ast::TraitDecl {
        let td_span = span_of(&pair);
        let mut name = None;
        let mut parents = None;
        let mut body = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::trait_parents => {
                    parents = Some(self.lower_trait_parents(child));
                }
                Rule::trait_body_item => {
                    if let Some(item) = self.lower_trait_body_item(child) {
                        body.push(item);
                    }
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(td_span, "trait missing name");
            ast::Ident {
                name: String::new(),
                span: td_span,
            }
        });

        ast::TraitDecl {
            name,
            parents,
            body,
            span: td_span,
        }
    }

    fn lower_trait_parents(&mut self, pair: Pair<'_>) -> ast::TraitBound {
        let tp_span = span_of(&pair);
        let bounds: Vec<ast::TypeExpr> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::type_expr)
            .map(|p| self.lower_type_expr(p))
            .collect();

        ast::TraitBound {
            bounds,
            span: tp_span,
        }
    }

    fn lower_trait_body_item(&mut self, pair: Pair<'_>) -> Option<ast::TraitBodyItem> {
        let inner = pair.into_inner().next()?;
        match inner.as_rule() {
            Rule::pins_block => Some(ast::TraitBodyItem::Pins(self.lower_pins_block(inner))),
            Rule::spec_block => Some(ast::TraitBodyItem::Spec(self.lower_spec_block(inner))),
            Rule::rule_block => Some(ast::TraitBodyItem::Rule(self.lower_rule_block(inner))),
            Rule::designator_prefix => Some(ast::TraitBodyItem::DesignatorPrefix(
                self.lower_designator_prefix(inner),
            )),
            _ => None,
        }
    }

    fn lower_designator_prefix(&mut self, pair: Pair<'_>) -> ast::DesignatorPrefix {
        let dp_span = span_of(&pair);
        let sl = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::string_literal)
            .map(|p| self.lower_string_literal(p))
            .unwrap_or_else(|| {
                self.error(dp_span, "designator_prefix missing value");
                ast::StringLiteral {
                    value: String::new(),
                    span: dp_span,
                }
            });

        ast::DesignatorPrefix {
            prefix: sl,
            span: dp_span,
        }
    }

    // ── Pins block ─────────────────────────────────────────────────────────

    fn lower_pins_block(&mut self, pair: Pair<'_>) -> ast::PinsBlock {
        let pb_span = span_of(&pair);
        let mut qualifier = None;
        let mut entries = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::pins_qualifier => {
                    qualifier = child
                        .into_inner()
                        .find(|p| p.as_rule() == Rule::ident)
                        .map(|p| self.lower_ident(p));
                }
                Rule::pin_entry => {
                    entries.push(self.lower_pin_entry(child));
                }
                _ => {}
            }
        }

        ast::PinsBlock {
            qualifier,
            entries,
            span: pb_span,
        }
    }

    fn lower_pin_entry(&mut self, pair: Pair<'_>) -> ast::PinEntry {
        let pe_span = span_of(&pair);
        let mut children = pair.into_inner();
        let first = children.next().expect("pin_entry must have content");

        let kind = match first.as_rule() {
            Rule::pin_bus_call => {
                // pin_bus!(name, start, count)
                let mut bus_children = first.into_inner();
                let name = self.lower_ident(bus_children.next().expect("pin_bus must have name"));
                let start_pin = bus_children
                    .next()
                    .expect("pin_bus must have start")
                    .as_str()
                    .parse::<u64>()
                    .unwrap_or(0);
                let count = bus_children
                    .next()
                    .expect("pin_bus must have count")
                    .as_str()
                    .parse::<u64>()
                    .unwrap_or(0);
                ast::PinEntryKind::BusMacro {
                    name,
                    start_pin,
                    count,
                }
            }
            Rule::ident => {
                // ident : pin_value_or_type
                let name = self.lower_ident(first);
                let pvt = children
                    .next()
                    .expect("pin entry must have pin_value_or_type");
                self.lower_pin_value_or_type(name, pvt)
            }
            _ => {
                self.error(pe_span, "unexpected pin entry");
                ast::PinEntryKind::Single {
                    name: ast::Ident {
                        name: String::new(),
                        span: pe_span,
                    },
                    number: 0,
                }
            }
        };

        ast::PinEntry {
            kind,
            span: pe_span,
        }
    }

    fn lower_pin_value_or_type(&mut self, name: ast::Ident, pair: Pair<'_>) -> ast::PinEntryKind {
        // pin_value_or_type = { pin_value | ident }
        let inner = pair
            .into_inner()
            .next()
            .expect("pin_value_or_type must have content");

        match inner.as_rule() {
            Rule::pin_value => self.lower_pin_value(name, inner),
            Rule::ident => {
                let ty = self.lower_ident(inner);
                ast::PinEntryKind::Typed { name, ty }
            }
            _ => {
                let sp = span_of(&inner);
                self.error(sp, "expected pin value or type");
                ast::PinEntryKind::Single { name, number: 0 }
            }
        }
    }

    fn lower_pin_value(&mut self, name: ast::Ident, pair: Pair<'_>) -> ast::PinEntryKind {
        // pin_value = { pin_range | pin_list | integer }
        let inner = pair
            .into_inner()
            .next()
            .expect("pin_value must have content");

        match inner.as_rule() {
            Rule::pin_range => {
                let ints: Vec<u64> = inner
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::integer)
                    .map(|p| p.as_str().parse::<u64>().unwrap_or(0))
                    .collect();
                ast::PinEntryKind::Range {
                    name,
                    start: *ints.first().unwrap_or(&0),
                    end: *ints.get(1).unwrap_or(&0),
                }
            }
            Rule::pin_list => {
                let numbers: Vec<u64> = inner
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::integer)
                    .map(|p| p.as_str().parse::<u64>().unwrap_or(0))
                    .collect();
                ast::PinEntryKind::List { name, numbers }
            }
            Rule::integer => {
                let number = inner.as_str().parse::<u64>().unwrap_or(0);
                ast::PinEntryKind::Single { name, number }
            }
            _ => {
                let sp = span_of(&inner);
                self.error(sp, "expected pin value");
                ast::PinEntryKind::Single { name, number: 0 }
            }
        }
    }

    // ── Spec block ─────────────────────────────────────────────────────────

    fn lower_spec_block(&mut self, pair: Pair<'_>) -> ast::SpecBlock {
        let sb_span = span_of(&pair);
        let entries: Vec<ast::SpecEntry> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::spec_entry)
            .map(|p| self.lower_spec_entry(p))
            .collect();

        ast::SpecBlock {
            entries,
            span: sb_span,
        }
    }

    fn lower_spec_entry(&mut self, pair: Pair<'_>) -> ast::SpecEntry {
        let se_span = span_of(&pair);
        let mut children = pair.into_inner();
        let name = self.lower_ident(children.next().expect("spec_entry must have name"));
        let value_pair = children.next().expect("spec_entry must have value");
        let value = self.lower_value_expr(value_pair);

        ast::SpecEntry {
            name,
            value,
            span: se_span,
        }
    }

    // ── Rule block ─────────────────────────────────────────────────────────

    fn lower_rule_block(&mut self, pair: Pair<'_>) -> ast::RuleBlock {
        let rb_span = span_of(&pair);
        let mut name = None;
        let mut level = ast::RuleLevel::Error;
        let mut assertion = None;
        let mut message = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::rule_level => {
                    level = self.lower_rule_level(child);
                }
                Rule::rule_body => {
                    for body_child in child.into_inner() {
                        match body_child.as_rule() {
                            Rule::rule_assert => {
                                let expr_pair = body_child
                                    .into_inner()
                                    .find(|p| p.as_rule() == Rule::expr)
                                    .expect("rule_assert must have expr");
                                assertion = Some(self.lower_expr(expr_pair));
                            }
                            Rule::rule_message => {
                                let is_pair = body_child
                                    .into_inner()
                                    .find(|p| p.as_rule() == Rule::interpolated_string)
                                    .expect("rule_message must have interpolated_string");
                                message = Some(self.lower_interpolated_string(is_pair));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(rb_span, "rule block missing name");
            ast::Ident {
                name: String::new(),
                span: rb_span,
            }
        });

        let assertion = assertion.unwrap_or_else(|| {
            self.error(rb_span, "rule block missing assertion");
            ast::Expr {
                kind: ast::ExprKind::Bool(false),
                span: rb_span,
            }
        });

        let message = message.unwrap_or_else(|| {
            self.error(rb_span, "rule block missing message");
            ast::InterpolatedString {
                raw: String::new(),
                span: rb_span,
            }
        });

        ast::RuleBlock {
            name,
            level,
            assertion,
            message,
            span: rb_span,
        }
    }

    fn lower_rule_level(&mut self, pair: Pair<'_>) -> ast::RuleLevel {
        // rule_level = { "level" ~ ":" ~ ident }
        let ident = pair
            .into_inner()
            .find(|p| p.as_rule() == Rule::ident)
            .expect("rule_level must have ident");
        match ident.as_str() {
            "Error" => ast::RuleLevel::Error,
            "Warning" => ast::RuleLevel::Warning,
            other => {
                self.error(
                    span_of(&ident),
                    format!("unknown rule level: {other}, expected Error or Warning"),
                );
                ast::RuleLevel::Error
            }
        }
    }

    // ── Device ─────────────────────────────────────────────────────────────

    fn lower_device_def(&mut self, pair: Pair<'_>) -> ast::DeviceDecl {
        let dd_span = span_of(&pair);
        let mut name = None;
        let mut generic_params = None;
        let mut impl_traits = None;
        let mut body = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::generic_params => {
                    generic_params = Some(self.lower_generic_params(child));
                }
                Rule::impl_clause => {
                    let tb = child
                        .into_inner()
                        .find(|p| p.as_rule() == Rule::trait_bound)
                        .expect("impl_clause must have trait_bound");
                    impl_traits = Some(self.lower_trait_bound(tb));
                }
                Rule::device_body_item => {
                    if let Some(item) = self.lower_device_body_item(child) {
                        body.push(item);
                    }
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(dd_span, "device missing name");
            ast::Ident {
                name: String::new(),
                span: dd_span,
            }
        });

        ast::DeviceDecl {
            name,
            generic_params,
            impl_traits,
            body,
            span: dd_span,
        }
    }

    fn lower_device_body_item(&mut self, pair: Pair<'_>) -> Option<ast::DeviceBodyItem> {
        let inner = pair.into_inner().next()?;
        match inner.as_rule() {
            Rule::package_decl => {
                Some(ast::DeviceBodyItem::Package(self.lower_package_decl(inner)))
            }
            Rule::pins_block => Some(ast::DeviceBodyItem::Pins(self.lower_pins_block(inner))),
            Rule::spec_block => Some(ast::DeviceBodyItem::Spec(self.lower_spec_block(inner))),
            Rule::rule_block => Some(ast::DeviceBodyItem::Rule(self.lower_rule_block(inner))),
            _ => None,
        }
    }

    fn lower_package_decl(&mut self, pair: Pair<'_>) -> ast::PackageDecl {
        let pd_span = span_of(&pair);
        let mut path = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::scoped_path | Rule::ident => {
                    path = Some(self.lower_path_from_pair(child));
                }
                _ => {}
            }
        }

        let path = path.unwrap_or_else(|| {
            self.error(pd_span, "package declaration missing path");
            ast::Path {
                segments: Vec::new(),
                span: pd_span,
            }
        });

        ast::PackageDecl {
            path,
            span: pd_span,
        }
    }

    // ── Part ───────────────────────────────────────────────────────────────

    fn lower_part_def(&mut self, pair: Pair<'_>) -> ast::PartDecl {
        let pd_span = span_of(&pair);
        let mut name = None;
        let mut device_type = None;
        let mut avl_entries = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::type_expr if device_type.is_none() => {
                    device_type = Some(self.lower_type_expr(child));
                }
                Rule::avl_entry => {
                    avl_entries.push(self.lower_avl_entry(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(pd_span, "part missing name");
            ast::Ident {
                name: String::new(),
                span: pd_span,
            }
        });

        let device_type = device_type.unwrap_or_else(|| {
            self.error(pd_span, "part missing device type");
            ast::TypeExpr {
                path: ast::Path {
                    segments: Vec::new(),
                    span: pd_span,
                },
                generic_args: None,
                span: pd_span,
            }
        });

        ast::PartDecl {
            name,
            device_type,
            avl_entries,
            span: pd_span,
        }
    }

    fn lower_avl_entry(&mut self, pair: Pair<'_>) -> ast::AvlEntry {
        let ae_span = span_of(&pair);
        let mut kind = ast::AvlKind::Primary;
        let mut fields = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::avl_kind => {
                    kind = match child.as_str() {
                        "primary" => ast::AvlKind::Primary,
                        "alt" => ast::AvlKind::Alt,
                        _ => ast::AvlKind::Primary,
                    };
                }
                Rule::avl_field => {
                    fields.push(self.lower_avl_field(child));
                }
                _ => {}
            }
        }

        ast::AvlEntry {
            kind,
            fields,
            span: ae_span,
        }
    }

    fn lower_avl_field(&mut self, pair: Pair<'_>) -> ast::AvlField {
        let af_span = span_of(&pair);
        let mut children = pair.into_inner();
        let name = self.lower_ident(children.next().expect("avl_field must have name"));
        let value = self.lower_string_literal(children.next().expect("avl_field must have value"));

        ast::AvlField {
            name,
            value,
            span: af_span,
        }
    }

    // ── Type alias ─────────────────────────────────────────────────────────

    fn lower_type_alias(&mut self, pair: Pair<'_>) -> ast::TypeAlias {
        let ta_span = span_of(&pair);
        let mut name = None;
        let mut generic_params = None;
        let mut target = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::generic_params => {
                    generic_params = Some(self.lower_generic_params(child));
                }
                Rule::type_expr => {
                    target = Some(self.lower_type_expr(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(ta_span, "type alias missing name");
            ast::Ident {
                name: String::new(),
                span: ta_span,
            }
        });

        let target = target.unwrap_or_else(|| {
            self.error(ta_span, "type alias missing target type");
            ast::TypeExpr {
                path: ast::Path {
                    segments: Vec::new(),
                    span: ta_span,
                },
                generic_args: None,
                span: ta_span,
            }
        });

        ast::TypeAlias {
            name,
            generic_params,
            target,
            span: ta_span,
        }
    }

    // ── Function ───────────────────────────────────────────────────────────

    fn lower_fn_def(&mut self, pair: Pair<'_>) -> ast::FnDecl {
        let fd_span = span_of(&pair);
        let mut name = None;
        let mut generic_params = None;
        let mut params = Vec::new();
        let mut body = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::generic_params => {
                    generic_params = Some(self.lower_generic_params(child));
                }
                Rule::fn_param => {
                    params.push(self.lower_fn_param(child));
                }
                Rule::fn_body_stmt => {
                    body.push(self.lower_fn_body_stmt(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(fd_span, "function missing name");
            ast::Ident {
                name: String::new(),
                span: fd_span,
            }
        });

        ast::FnDecl {
            name,
            generic_params,
            params,
            body,
            span: fd_span,
        }
    }

    fn lower_fn_param(&mut self, pair: Pair<'_>) -> ast::FnParam {
        let fp_span = span_of(&pair);
        let mut name = None;
        let mut kind = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::impl_constraint => {
                    kind = Some(ast::FnParamKind::ImplConstraint(
                        self.lower_impl_constraint(child),
                    ));
                }
                Rule::type_expr if kind.is_none() => {
                    kind = Some(ast::FnParamKind::Type(self.lower_type_expr(child)));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(fp_span, "function parameter missing name");
            ast::Ident {
                name: String::new(),
                span: fp_span,
            }
        });

        let kind = kind.unwrap_or_else(|| {
            self.error(fp_span, "function parameter missing type");
            ast::FnParamKind::Type(ast::TypeExpr {
                path: ast::Path {
                    segments: Vec::new(),
                    span: fp_span,
                },
                generic_args: None,
                span: fp_span,
            })
        });

        ast::FnParam {
            name,
            kind,
            span: fp_span,
        }
    }

    fn lower_fn_body_stmt(&mut self, pair: Pair<'_>) -> ast::FnBodyStmt {
        let fbs_span = span_of(&pair);
        let mut attributes = Vec::new();
        let mut kind = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::attribute => {
                    attributes.push(self.lower_attribute(child));
                }
                Rule::inst_stmt => {
                    kind = Some(ast::FnBodyStmtKind::Inst(self.lower_inst_stmt(child)));
                }
                Rule::net_stmt => {
                    kind = Some(ast::FnBodyStmtKind::Net(self.lower_net_stmt(child)));
                }
                Rule::call_stmt => {
                    kind = Some(ast::FnBodyStmtKind::Call(self.lower_call_stmt(child)));
                }
                _ => {}
            }
        }

        let kind = kind.unwrap_or_else(|| {
            self.error(fbs_span, "expected inst, net, or call statement");
            ast::FnBodyStmtKind::Inst(ast::InstStmt {
                name: ast::Ident {
                    name: String::new(),
                    span: fbs_span,
                },
                ty: ast::TypeExpr {
                    path: ast::Path {
                        segments: Vec::new(),
                        span: fbs_span,
                    },
                    generic_args: None,
                    span: fbs_span,
                },
                avl_entries: None,
                span: fbs_span,
            })
        });

        ast::FnBodyStmt {
            attributes,
            kind,
            span: fbs_span,
        }
    }

    // ── Module ─────────────────────────────────────────────────────────────

    fn lower_module_def(&mut self, pair: Pair<'_>) -> ast::ModuleDecl {
        let md_span = span_of(&pair);
        let mut name = None;
        let mut items = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::top_level_item => {
                    items.push(self.lower_top_level_item(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(md_span, "module missing name");
            ast::Ident {
                name: String::new(),
                span: md_span,
            }
        });

        ast::ModuleDecl {
            name,
            items,
            span: md_span,
        }
    }

    // ── Design ─────────────────────────────────────────────────────────────

    fn lower_design_def(&mut self, pair: Pair<'_>) -> ast::DesignDecl {
        let dd_span = span_of(&pair);
        let mut name = None;
        let mut body = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::design_body_stmt => {
                    body.push(self.lower_design_body_stmt(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(dd_span, "design missing name");
            ast::Ident {
                name: String::new(),
                span: dd_span,
            }
        });

        ast::DesignDecl {
            name,
            body,
            span: dd_span,
        }
    }

    fn lower_design_body_stmt(&mut self, pair: Pair<'_>) -> ast::DesignBodyStmt {
        let dbs_span = span_of(&pair);
        let mut attributes = Vec::new();
        let mut kind = None;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::attribute => {
                    attributes.push(self.lower_attribute(child));
                }
                Rule::inst_stmt => {
                    kind = Some(ast::DesignBodyStmtKind::Inst(self.lower_inst_stmt(child)));
                }
                Rule::net_stmt => {
                    kind = Some(ast::DesignBodyStmtKind::Net(self.lower_net_stmt(child)));
                }
                Rule::call_stmt => {
                    kind = Some(ast::DesignBodyStmtKind::Call(self.lower_call_stmt(child)));
                }
                _ => {}
            }
        }

        let kind = kind.unwrap_or_else(|| {
            self.error(dbs_span, "expected inst, net, or call statement");
            ast::DesignBodyStmtKind::Inst(ast::InstStmt {
                name: ast::Ident {
                    name: String::new(),
                    span: dbs_span,
                },
                ty: ast::TypeExpr {
                    path: ast::Path {
                        segments: Vec::new(),
                        span: dbs_span,
                    },
                    generic_args: None,
                    span: dbs_span,
                },
                avl_entries: None,
                span: dbs_span,
            })
        });

        ast::DesignBodyStmt {
            attributes,
            kind,
            span: dbs_span,
        }
    }

    // ── Statements ─────────────────────────────────────────────────────────

    fn lower_inst_stmt(&mut self, pair: Pair<'_>) -> ast::InstStmt {
        let is_span = span_of(&pair);
        let mut name = None;
        let mut ty = None;
        let mut avl_entries = Vec::new();
        let mut has_avl = false;

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::ident if name.is_none() => {
                    name = Some(self.lower_ident(child));
                }
                Rule::type_expr if ty.is_none() => {
                    ty = Some(self.lower_type_expr(child));
                }
                Rule::avl_entry => {
                    has_avl = true;
                    avl_entries.push(self.lower_avl_entry(child));
                }
                _ => {}
            }
        }

        let name = name.unwrap_or_else(|| {
            self.error(is_span, "inst statement missing name");
            ast::Ident {
                name: String::new(),
                span: is_span,
            }
        });

        let ty = ty.unwrap_or_else(|| {
            self.error(is_span, "inst statement missing type");
            ast::TypeExpr {
                path: ast::Path {
                    segments: Vec::new(),
                    span: is_span,
                },
                generic_args: None,
                span: is_span,
            }
        });

        ast::InstStmt {
            name,
            ty,
            avl_entries: if has_avl { Some(avl_entries) } else { None },
            span: is_span,
        }
    }

    fn lower_net_stmt(&mut self, pair: Pair<'_>) -> ast::NetStmt {
        let ns_span = span_of(&pair);
        let mut target = None;
        let mut endpoints = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::net_target => {
                    target = Some(self.lower_net_endpoint_inner(child));
                }
                Rule::net_endpoint => {
                    endpoints.push(self.lower_net_endpoint_inner(child));
                }
                _ => {}
            }
        }

        let target = target.unwrap_or_else(|| {
            self.error(ns_span, "net statement missing target");
            ast::NetEndpoint {
                kind: ast::NetEndpointKind::Ident(ast::Ident {
                    name: String::new(),
                    span: ns_span,
                }),
                span: ns_span,
            }
        });

        ast::NetStmt {
            target,
            endpoints,
            span: ns_span,
        }
    }

    fn lower_net_endpoint_inner(&mut self, pair: Pair<'_>) -> ast::NetEndpoint {
        let ne_span = span_of(&pair);
        let inner = pair
            .into_inner()
            .next()
            .expect("net_target/net_endpoint must have content");

        let kind = match inner.as_rule() {
            Rule::dot_path => ast::NetEndpointKind::DotPath(self.lower_dot_path(inner)),
            Rule::ident => ast::NetEndpointKind::Ident(self.lower_ident(inner)),
            _ => {
                self.error(ne_span, "expected identifier or dot-path in net endpoint");
                ast::NetEndpointKind::Ident(ast::Ident {
                    name: String::new(),
                    span: ne_span,
                })
            }
        };

        ast::NetEndpoint {
            kind,
            span: ne_span,
        }
    }

    fn lower_call_stmt(&mut self, pair: Pair<'_>) -> ast::CallStmt {
        let cs_span = span_of(&pair);
        let mut path = None;
        let mut args = Vec::new();

        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::scoped_path | Rule::ident => {
                    path = Some(self.lower_path_from_pair(child));
                }
                Rule::call_arg => {
                    args.push(self.lower_call_arg(child));
                }
                _ => {}
            }
        }

        let path = path.unwrap_or_else(|| {
            self.error(cs_span, "call statement missing function path");
            ast::Path {
                segments: Vec::new(),
                span: cs_span,
            }
        });

        ast::CallStmt {
            path,
            args,
            span: cs_span,
        }
    }

    fn lower_call_arg(&mut self, pair: Pair<'_>) -> ast::CallArg {
        let ca_span = span_of(&pair);
        let mut children = pair.into_inner();
        let name = self.lower_ident(children.next().expect("call_arg must have name"));
        let value_pair = children.next().expect("call_arg must have value");
        let value = self.lower_value_expr(value_pair);

        ast::CallArg {
            name,
            value,
            span: ca_span,
        }
    }
}

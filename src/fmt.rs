//! RFC-009: `cohdl fmt` canonical form.
//!
//! A pure AST-to-text serializer: parse the source with the same lexer/parser
//! `check`/`build` use, then re-serialize deterministically. This is what makes
//! `fmt` idempotent (`fmt(fmt(x)) == fmt(x)`) and semantically inert (same
//! parse tree → same type-check verdict → same netlist bytes) *by
//! construction*, not by testing — two spellings of one construct parse to the
//! same AST node, and one AST node always serializes one way.
//!
//! Comments and author-placed blank lines are not in the AST, so they are
//! re-interleaved from their source line numbers: as the serializer walks the
//! AST it advances a line cursor, flushing full-line comments and preserved
//! blanks that precede each construct and re-attaching trailing comments to the
//! line of the construct they followed.
//!
//! `fmt` requires already-parsing source — it is a serializer, not a repair
//! tool. A pin missing its `[role]` bracket stays a parse error (RFC-008/E901);
//! `fmt` never papers over missing required syntax.
//!
//! Canonical form (the choices this file commits to, per RFC-009):
//! - 4-space indent; no tabs; no trailing whitespace; one trailing newline.
//! - `pins {}` / `spec {}` blocks: one declaration per line (the RFC's stated
//!   general rule). Pin obligation is always explicit (`required`/`optional`).
//! - `variants {}` and AVL `primary {}`/`alt {}`: inline, comma-space-separated.
//! - Empty `impl … {}`: one line.
//! - `net`/`nc` member lists wrap onto continuation lines aligned under the
//!   first member when the single line would exceed 100 columns.

use crate::ast::*;
use crate::diag::Diagnostics;
use crate::span::{FileId, SourceMap, Span};
use std::collections::{BTreeMap, BTreeSet};

const INDENT: &str = "    ";
const WIDTH: usize = 100;

/// Format one source file into canonical form. Returns `Err` (the rendered
/// parse diagnostics) when the input does not parse — `fmt` never operates on
/// broken source.
pub fn format_source(name: &str, text: &str) -> Result<String, String> {
    let mut sm = SourceMap::new();
    let file = sm.add_file(name, text);
    let mut diags = Diagnostics::new();
    let tokens = crate::lex::lex(file, sm.text(file), &mut diags);
    let ast = crate::parse::parse(tokens, &mut diags);
    if diags.has_errors() {
        diags.sort(&sm);
        return Err(diags.render(&sm));
    }
    let comments = scan_comments(text);
    let mut f = Formatter {
        sm: &sm,
        file,
        c: comments,
        out: Vec::new(),
        cursor: 1,
    };
    f.file(&ast);
    Ok(f.finish())
}

// ---------------------------------------------------------------------------
// Comment / blank-line scan (the only text-level pass — it recovers what the
// AST discards, keyed by source line).

struct Comments {
    /// line → a comment that is the only content on its line.
    full_line: BTreeMap<u32, String>,
    /// line → a comment that trails code on its line.
    trailing: BTreeMap<u32, String>,
    /// lines that are entirely whitespace.
    blank: BTreeSet<u32>,
    max_line: u32,
}

fn scan_comments(text: &str) -> Comments {
    let mut full_line = BTreeMap::new();
    let mut trailing = BTreeMap::new();
    let mut blank = BTreeSet::new();
    let mut line_no = 0u32;
    for raw in text.split('\n') {
        line_no += 1;
        // Find `//` that is not inside a string literal. CoHDL strings cannot
        // contain a newline or an escaped quote (see the lexer), so a bare `"`
        // toggle is a faithful scanner.
        let bytes = raw.as_bytes();
        let mut in_str = false;
        let mut comment_at = None;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'"' => in_str = !in_str,
                b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => {
                    comment_at = Some(i);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        match comment_at {
            Some(idx) => {
                let comment = raw[idx..].trim_end().to_string();
                if raw[..idx].trim().is_empty() {
                    full_line.insert(line_no, comment);
                } else {
                    trailing.insert(line_no, comment);
                }
            }
            None if raw.trim().is_empty() => {
                blank.insert(line_no);
            }
            None => {}
        }
    }
    Comments {
        full_line,
        trailing,
        blank,
        max_line: line_no,
    }
}

// ---------------------------------------------------------------------------
// The serializer.

struct Formatter<'a> {
    sm: &'a SourceMap,
    file: FileId,
    c: Comments,
    out: Vec<String>,
    /// The next source line not yet accounted for (1-based).
    cursor: u32,
}

impl Formatter<'_> {
    fn line_start(&self, span: Span) -> u32 {
        self.sm.line_col(self.file, span.start).line
    }
    fn line_end(&self, span: Span) -> u32 {
        self.sm.line_col(self.file, span.end).line
    }

    fn push(&mut self, indent: usize, text: impl AsRef<str>) {
        let mut s = INDENT.repeat(indent);
        s.push_str(text.as_ref());
        while s.ends_with(' ') {
            s.pop();
        }
        self.out.push(s);
    }

    /// Emit one blank line, collapsing runs and refusing a blank at file start
    /// or immediately after an opening brace.
    fn push_blank(&mut self) {
        match self.out.last() {
            None => {}
            Some(l) if l.is_empty() => {}
            Some(l) if l.ends_with('{') => {}
            _ => self.out.push(String::new()),
        }
    }

    /// Emit full-line comments and preserved blank lines occupying source lines
    /// `[cursor, before)`, then advance the cursor to `before`.
    fn flush_leading(&mut self, before: u32, indent: usize) {
        if before <= self.cursor {
            return;
        }
        let mut l = self.cursor;
        while l < before {
            if let Some(cmt) = self.c.full_line.get(&l).cloned() {
                self.push(indent, cmt);
            } else if self.c.blank.contains(&l) {
                self.push_blank();
            }
            l += 1;
        }
        self.cursor = before;
    }

    /// Re-attach a trailing comment sitting on source line `end_line` to the
    /// last emitted output line, and advance the cursor past it.
    fn attach_trailing(&mut self, end_line: u32) {
        if let Some(cmt) = self.c.trailing.get(&end_line).cloned() {
            if let Some(last) = self.out.last_mut() {
                if !last.is_empty() {
                    last.push(' ');
                    last.push_str(&cmt);
                }
            }
        }
        self.cursor = self.cursor.max(end_line + 1);
    }

    /// Close out a leaf construct spanning source lines `[start, end]` (a
    /// statement, pin, or spec field): attach a trailing comment on its last
    /// line, then rescue any comment stranded on an *interior* line — a
    /// full-line comment inside a wrapped member list, or a trailing comment on
    /// a non-final line — by re-emitting it as a full-line comment (RFC-009:
    /// comments are never dropped; a reflowed statement cannot keep a comment
    /// mid-list, so it moves to its own line rather than vanishing). No-op for
    /// the common single-line construct (`start == end`).
    fn finish_construct(&mut self, start: u32, end: u32, indent: usize) {
        self.attach_trailing(end);
        for l in start..end {
            if let Some(c) = self.c.full_line.get(&l).cloned() {
                self.push(indent, c);
            } else if let Some(c) = self.c.trailing.get(&l).cloned() {
                self.push(indent, c);
            }
        }
        self.cursor = self.cursor.max(end + 1);
    }

    fn finish(mut self) -> String {
        while self.out.last().is_some_and(|l| l.is_empty()) {
            self.out.pop();
        }
        let mut s = self.out.join("\n");
        s.push('\n');
        s
    }

    // -- top level -----------------------------------------------------------

    fn file(&mut self, ast: &SourceFile) {
        for item in &ast.items {
            self.flush_leading(self.line_start(item.span), 0);
            self.item(item);
            // A trailing comment on the item's own line (e.g. `impl X for Y {}
            // // note`); the item's body already handled its interior comments.
            self.attach_trailing(self.line_end(item.span));
            self.cursor = self.cursor.max(self.line_end(item.span) + 1);
        }
        // Any comments trailing the last item.
        self.flush_leading(self.c.max_line + 1, 0);
    }

    /// RFC-012: emit `#[intent("...")]` on its own line, preceding the
    /// declaration it annotates (same placement as `#[designator]`).
    fn emit_intent(&mut self, intent: &Option<String>, indent: usize) {
        if let Some(text) = intent {
            self.push(indent, format!("#[intent({})]", str_lit(text)));
        }
    }

    fn item(&mut self, item: &Item) {
        self.emit_intent(&item.intent, 0);
        let vis = if item.is_pub { "pub " } else { "" };
        match &item.kind {
            ItemKind::Trait(t) => self.trait_def(vis, t),
            ItemKind::Device(d) => self.device_def(vis, item, d),
            ItemKind::Impl(i) => self.impl_def(i),
            ItemKind::Fn(f) => self.fn_def(vis, item, f),
            ItemKind::Part(p) => self.part_def(vis, item, p),
            ItemKind::Design(d) => self.design_def(vis, item, d),
        }
    }

    // -- traits --------------------------------------------------------------

    fn trait_def(&mut self, vis: &str, t: &TraitDef) {
        let mut header = format!("{}trait {}", vis, t.name.name);
        if !t.super_traits.is_empty() {
            header.push_str(": ");
            header.push_str(&join(t.super_traits.iter().map(|s| s.name.clone()), " + "));
        }
        let has_body = t.designator_prefix.is_some() || !t.pins.is_empty() || !t.specs.is_empty();
        if !has_body {
            self.push(0, format!("{} {{}}", header));
            return;
        }
        self.push(0, format!("{} {{", header));
        if let Some((prefix, _)) = &t.designator_prefix {
            self.push(1, format!("designator_prefix: {}", str_lit(prefix)));
        }
        if !t.pins.is_empty() {
            self.push(1, "pins {");
            for p in &t.pins {
                self.push(
                    2,
                    format!("{} {}: pin", p.obligation.keyword(), p.name.name),
                );
            }
            self.push(1, "}");
        }
        if !t.specs.is_empty() {
            self.push(1, "spec {");
            for s in &t.specs {
                self.push(2, format!("{}: {}", s.name.name, s.ty.unit.type_name()));
            }
            self.push(1, "}");
        }
        self.push(0, "}");
    }

    // -- devices -------------------------------------------------------------

    fn device_def(&mut self, vis: &str, item: &Item, d: &DeviceDef) {
        let mut header = format!("{}device {}", vis, d.name.name);
        if !d.generics.is_empty() {
            header.push_str(&generic_params(&d.generics));
        }
        self.push(0, format!("{} {{", header));
        self.cursor = self.cursor.max(self.line_start(item.span) + 1);

        // Device-body members, emitted in source order so author blank lines
        // and comments between them are preserved.
        enum Member {
            Variants,
            Pins(usize),
            Spec(usize),
        }
        let mut members: Vec<(u32, Member)> = Vec::new();
        if let Some(vs) = d.variants_span {
            members.push((self.line_start(vs), Member::Variants));
        }
        for (i, pb) in d.pin_blocks.iter().enumerate() {
            members.push((self.line_start(pb.span), Member::Pins(i)));
        }
        for (i, sb) in d.spec_blocks.iter().enumerate() {
            members.push((self.line_start(sb.span), Member::Spec(i)));
        }
        members.sort_by_key(|(l, _)| *l);

        for (start, m) in members {
            self.flush_leading(start, 1);
            match m {
                Member::Variants => {
                    let names = join(d.variants.iter().map(|v| v.name.clone()), ", ");
                    self.push(1, format!("variants {{ {} }}", names));
                    if let Some(vs) = d.variants_span {
                        self.cursor = self.cursor.max(self.line_end(vs) + 1);
                    }
                }
                Member::Pins(i) => self.pin_block(&d.pin_blocks[i], 1),
                Member::Spec(i) => self.spec_block(&d.spec_blocks[i], 1),
            }
        }
        self.flush_leading(self.line_end(item.span), 1);
        self.push(0, "}");
    }

    fn pin_block(&mut self, pb: &PinBlock, indent: usize) {
        let open = match &pb.variant {
            Some(v) => format!("pins[{}] {{", v.name),
            None => "pins {".to_string(),
        };
        self.push(indent, open);
        self.cursor = self.cursor.max(self.line_start(pb.span) + 1);
        for pin in &pb.pins {
            self.flush_leading(self.line_start(pin.span), indent + 1);
            self.push(indent + 1, pin_text(pin));
            self.finish_construct(
                self.line_start(pin.span),
                self.line_end(pin.span),
                indent + 1,
            );
        }
        self.flush_leading(self.line_end(pb.span), indent + 1);
        self.push(indent, "}");
        self.cursor = self.cursor.max(self.line_end(pb.span) + 1);
    }

    fn spec_block(&mut self, sb: &SpecBlock, indent: usize) {
        let open = match &sb.variant {
            Some(v) => format!("spec[{}] {{", v.name),
            None => "spec {".to_string(),
        };
        self.push(indent, open);
        self.cursor = self.cursor.max(self.line_start(sb.span) + 1);
        for field in &sb.fields {
            self.flush_leading(self.line_start(field.span), indent + 1);
            self.push(indent + 1, spec_field_text(field));
            self.finish_construct(
                self.line_start(field.span),
                self.line_end(field.span),
                indent + 1,
            );
        }
        self.flush_leading(self.line_end(sb.span), indent + 1);
        self.push(indent, "}");
        self.cursor = self.cursor.max(self.line_end(sb.span) + 1);
    }

    // -- impls ---------------------------------------------------------------

    fn impl_def(&mut self, i: &ImplDef) {
        let header = format!("impl {} for {}", i.trait_name.name, i.device_name.name);
        if i.pin_map.is_empty() && i.spec_map.is_empty() {
            self.push(0, format!("{} {{}}", header));
            return;
        }
        self.push(0, format!("{} {{", header));
        if !i.pin_map.is_empty() {
            self.push(1, "pins {");
            for e in &i.pin_map {
                self.push(2, format!("{}: {}", e.role.name, e.target.name));
            }
            self.push(1, "}");
        }
        if !i.spec_map.is_empty() {
            self.push(1, "spec {");
            for e in &i.spec_map {
                self.push(2, format!("{}: {}", e.role.name, e.target.name));
            }
            self.push(1, "}");
        }
        self.push(0, "}");
    }

    // -- fns & designs -------------------------------------------------------

    fn fn_def(&mut self, vis: &str, item: &Item, f: &FnDef) {
        let mut header = format!("{}fn {}", vis, f.name.name);
        if !f.generics.is_empty() {
            header.push_str(&generic_params(&f.generics));
        }
        let params = join(f.params.iter().map(param_text), ", ");
        self.push(0, format!("{}({}) {{", header, params));
        self.body(&f.body, item);
        self.push(0, "}");
    }

    fn design_def(&mut self, vis: &str, item: &Item, d: &DesignDef) {
        self.push(0, format!("{}design {} {{", vis, d.name.name));
        self.body(&d.body, item);
        self.push(0, "}");
    }

    fn body(&mut self, stmts: &[Stmt], item: &Item) {
        self.cursor = self.cursor.max(self.line_start(item.span) + 1);
        for stmt in stmts {
            self.flush_leading(self.line_start(stmt.span()), 1);
            self.stmt(stmt, 1);
            self.finish_construct(self.line_start(stmt.span()), self.line_end(stmt.span()), 1);
        }
        self.flush_leading(self.line_end(item.span), 1);
    }

    fn stmt(&mut self, stmt: &Stmt, indent: usize) {
        match stmt {
            Stmt::Inst(s) => {
                self.emit_intent(&s.intent, indent);
                for attr in &s.attrs {
                    self.push(indent, attr_text(attr));
                }
                self.push(
                    indent,
                    format!("inst {}: {}", s.name.name, type_ref_text(&s.ty)),
                );
            }
            Stmt::Net(s) => {
                self.emit_intent(&s.intent, indent);
                let name = s.name.as_ref().map_or("_".to_string(), |n| n.name.clone());
                let ann = match &s.annotation {
                    Some(NetAnnotation::Voltage(v, _)) => format!(" [{}]", v.text),
                    Some(NetAnnotation::Gnd(_)) => " [gnd]".to_string(),
                    None => String::new(),
                };
                let prefix = format!("net {}{}: ", name, ann);
                let members: Vec<String> = s.members.iter().map(|m| m.to_string()).collect();
                self.wrapped(indent, &prefix, &members);
            }
            Stmt::Nc(s) => {
                self.emit_intent(&s.intent, indent);
                let members: Vec<String> = s.members.iter().map(|m| m.to_string()).collect();
                self.wrapped(indent, "nc: ", &members);
            }
            Stmt::Call(s) => {
                self.emit_intent(&s.intent, indent);
                let generics = if s.generic_args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "::<{}>",
                        join(s.generic_args.iter().map(generic_arg_text), ", ")
                    )
                };
                let args = join(s.args.iter().map(|a| a.to_string()), ", ");
                self.push(indent, format!("{}{}({})", s.callee.name, generics, args));
            }
        }
    }

    /// Emit a `prefix`-led member list, wrapping onto continuation lines
    /// aligned under the first member when the single line exceeds `WIDTH`.
    fn wrapped(&mut self, indent: usize, prefix: &str, members: &[String]) {
        let base = indent * INDENT.len();
        let oneline = format!("{}{}", prefix, members.join(", "));
        if base + oneline.len() <= WIDTH || members.len() <= 1 {
            self.push(indent, oneline);
            return;
        }
        let align = " ".repeat(prefix.len());
        let mut line = prefix.to_string();
        let mut first_on_line = true;
        for (i, m) in members.iter().enumerate() {
            let piece = if i + 1 < members.len() {
                format!("{},", m)
            } else {
                m.clone()
            };
            let extra = if first_on_line { 0 } else { 1 };
            if !first_on_line && base + line.len() + extra + piece.len() > WIDTH {
                self.push(indent, &line);
                line = align.clone();
                first_on_line = true;
            }
            if !first_on_line {
                line.push(' ');
            }
            line.push_str(&piece);
            first_on_line = false;
        }
        self.push(indent, &line);
    }

    // -- parts ---------------------------------------------------------------

    fn part_def(&mut self, vis: &str, _item: &Item, p: &PartDef) {
        self.push(
            0,
            format!(
                "{}part {}: {} {{",
                vis,
                p.name.name,
                type_ref_text(&p.device)
            ),
        );
        self.push(1, avl_text("primary", &p.primary));
        for alt in &p.alts {
            self.push(1, avl_text("alt", alt));
        }
        self.push(0, "}");
    }
}

// ---------------------------------------------------------------------------
// Text builders (pure functions of the AST — no layout state).

fn pin_text(pin: &DevicePin) -> String {
    let nums = join(pin.numbers.iter().map(|n| n.text.clone()), ", ");
    let role = match pin.role {
        Some((r, _)) => format!(" [{}]", r.name()),
        None => String::new(),
    };
    format!(
        "{} {}: {}{}",
        pin.obligation.keyword(),
        pin.name.name,
        nums,
        role
    )
}

fn spec_field_text(field: &DeviceSpecField) -> String {
    let value = match &field.value {
        SpecValue::Lit(v, _) => v.text.clone(),
        SpecValue::GenericRef(id) => id.name.clone(),
    };
    format!("{}: {}", field.name.name, value)
}

fn attr_text(attr: &Attr) -> String {
    if attr.args.is_empty() {
        format!("#[{}]", attr.name.name)
    } else {
        let args = join(attr.args.iter().map(|(s, _)| str_lit(s)), ", ");
        format!("#[{}({})]", attr.name.name, args)
    }
}

fn type_ref_text(ty: &TypeRef) -> String {
    let mut s = ty.name.name.clone();
    if !ty.generic_args.is_empty() {
        s.push('<');
        s.push_str(&join(ty.generic_args.iter().map(generic_arg_text), ", "));
        s.push('>');
    }
    if let Some(v) = &ty.variant {
        s.push('[');
        s.push_str(&v.name);
        s.push(']');
    }
    s
}

fn generic_arg_text(arg: &GenericArg) -> String {
    match arg {
        GenericArg::Unit(v, _) => v.text.clone(),
        GenericArg::Name(id) => id.name.clone(),
        GenericArg::Number(n, _) => n.clone(),
    }
}

fn generic_params(params: &[GenericParam]) -> String {
    let inner = join(
        params.iter().map(|p| {
            let bound = match &p.bound {
                GenericBound::Unit(u) => u.unit.type_name().to_string(),
                GenericBound::Traits(ts) => join(ts.iter().map(|t| t.name.clone()), " + "),
            };
            let default = match &p.default {
                Some((v, _)) => format!(" = {}", v.text),
                None => String::new(),
            };
            format!("{}: {}{}", p.name.name, bound, default)
        }),
        ", ",
    );
    format!("<{}>", inner)
}

fn param_text(p: &FnParam) -> String {
    let ty = match &p.ty {
        FnParamTy::Pin(_) => "Pin".to_string(),
        FnParamTy::Generic(id) => id.name.clone(),
        FnParamTy::ImplTrait(ts, _) => {
            format!("impl {}", join(ts.iter().map(|t| t.name.clone()), " + "))
        }
    };
    format!("{}: {}", p.name.name, ty)
}

fn avl_text(keyword: &str, entry: &AvlEntry) -> String {
    let fields = join(
        entry
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name.name, str_lit(&f.value))),
        ", ",
    );
    format!("{} {{ {} }}", keyword, fields)
}

/// A double-quoted string literal, emitted verbatim. The CoHDL grammar has no
/// string escapes: the lexer reads the raw bytes between quotes and never
/// unescapes (src/lex.rs), so a value can contain neither `"` nor a newline
/// (either would terminate the literal). Introducing a `\"`/`\\` escape here
/// would therefore NOT round-trip — the escape would be re-read as literal
/// backslashes and grow on every `fmt` pass, breaking idempotence and semantic
/// inertness (RFC-009). So the value is emitted exactly as parsed.
fn str_lit(s: &str) -> String {
    format!("\"{}\"", s)
}

fn join(items: impl Iterator<Item = String>, sep: &str) -> String {
    items.collect::<Vec<_>>().join(sep)
}

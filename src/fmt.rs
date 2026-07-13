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

    /// Emit one blank line, collapsing runs (RFC-009: an author-placed blank is
    /// preserved — including immediately after an opening brace — and 2+
    /// consecutive blanks collapse to 1; only file-leading blanks are dropped).
    fn push_blank(&mut self) {
        match self.out.last() {
            None => {}
            Some(l) if l.is_empty() => {}
            _ => self.out.push(String::new()),
        }
    }

    /// Emit full-line comments and preserved blank lines occupying source lines
    /// `[cursor, before)`, then advance the cursor to `before`. Comments are
    /// consumed from the map — each source comment is emitted exactly once, by
    /// construction.
    fn flush_leading(&mut self, before: u32, indent: usize) {
        if before <= self.cursor {
            return;
        }
        let mut l = self.cursor;
        while l < before {
            if let Some(cmt) = self.c.full_line.remove(&l) {
                self.push(indent, cmt);
            } else if self.c.blank.contains(&l) {
                self.push_blank();
            }
            l += 1;
        }
        self.cursor = before;
    }

    /// Re-attach (and consume) a trailing comment sitting on source line
    /// `end_line` to the last emitted output line, and advance the cursor.
    fn attach_trailing(&mut self, end_line: u32) {
        if let Some(cmt) = self.c.trailing.remove(&end_line) {
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
        self.finish_construct_ext(start, end, indent, true);
    }

    /// `finish_construct` with control over the end-line trailing attach —
    /// deferred when a sibling construct continues on the same source line.
    fn finish_construct_ext(&mut self, start: u32, end: u32, indent: usize, attach_end: bool) {
        if attach_end {
            self.attach_trailing(end);
        }
        for l in start..end {
            if let Some(c) = self.c.full_line.remove(&l) {
                self.push(indent, c);
            } else if let Some(c) = self.c.trailing.remove(&l) {
                self.push(indent, c);
            }
        }
        let past = if attach_end { end + 1 } else { end };
        self.cursor = self.cursor.max(past);
    }

    fn finish(mut self) -> String {
        // RFC-009 "never dropped" backstop: any comment not consumed by a
        // structural emitter (e.g. trailing a brace on its own line) is
        // appended at the end rather than lost. The emitters above handle
        // positioning for every common shape; this guarantees totality.
        let mut leftovers: Vec<(u32, String)> = self
            .c
            .full_line
            .iter()
            .map(|(l, c)| (*l, c.clone()))
            .chain(self.c.trailing.iter().map(|(l, c)| (*l, c.clone())))
            .collect();
        leftovers.sort_by_key(|(l, _)| *l);
        for (_, c) in leftovers {
            self.push(0, c);
        }
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

    /// Emit a single-string opaque attribute (`#[NAME("...")]`) on its own line,
    /// preceding the declaration it annotates — RFC-012 `#[intent]` and RFC-013
    /// `#[placement_hint]`, same placement as `#[designator]`. Comment-aware:
    /// leading comments before the attribute stay before it, and a trailing
    /// comment on the attribute's own line survives.
    fn emit_string_attr(&mut self, name: &str, value: &Option<(String, Span)>, indent: usize) {
        if let Some((text, span)) = value {
            self.flush_leading(self.line_start(*span), indent);
            self.push(indent, format!("#[{}({})]", name, str_lit(text)));
            self.attach_trailing(self.line_end(*span));
        }
    }

    fn item(&mut self, item: &Item) {
        self.emit_string_attr("intent", &item.intent, 0);
        // Comments between the attribute and the declaration stay between.
        self.flush_leading(self.line_start(item.decl_span), 0);
        let vis = if item.is_pub { "pub " } else { "" };
        match &item.kind {
            ItemKind::Trait(t) => self.trait_def(vis, item, t),
            ItemKind::Device(d) => self.device_def(vis, item, d),
            ItemKind::Impl(i) => self.impl_def(i),
            ItemKind::Fn(f) => self.fn_def(vis, item, f),
            ItemKind::Part(p) => self.part_def(vis, item, p),
            ItemKind::Design(d) => self.design_def(vis, item, d),
        }
    }

    // -- traits --------------------------------------------------------------

    fn trait_def(&mut self, vis: &str, item: &Item, t: &TraitDef) {
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
        self.attach_trailing(self.line_start(item.decl_span));

        // Body members in source order, so interleaved comments/blanks keep
        // their author-chosen positions.
        enum Member {
            Prefix,
            Pins,
            Spec,
        }
        let mut members: Vec<(u32, Member)> = Vec::new();
        if let Some((_, span)) = &t.designator_prefix {
            members.push((self.line_start(*span), Member::Prefix));
        }
        if let Some(ps) = t.pins_span {
            members.push((self.line_start(ps), Member::Pins));
        }
        if let Some(ss) = t.spec_span {
            members.push((self.line_start(ss), Member::Spec));
        }
        members.sort_by_key(|(l, _)| *l);

        for (start, m) in members {
            self.flush_leading(start, 1);
            match m {
                Member::Prefix => {
                    let (prefix, span) = t.designator_prefix.as_ref().unwrap();
                    self.push(1, format!("designator_prefix: {}", str_lit(prefix)));
                    self.attach_trailing(self.line_end(*span));
                }
                Member::Pins => {
                    let ps = t.pins_span.unwrap();
                    self.push(1, "pins {");
                    self.attach_trailing(self.line_start(ps));
                    for p in &t.pins {
                        self.flush_leading(self.line_start(p.span), 2);
                        self.push(
                            2,
                            format!("{} {}: pin", p.obligation.keyword(), p.name.name),
                        );
                        self.finish_construct(self.line_start(p.span), self.line_end(p.span), 2);
                    }
                    self.flush_leading(self.line_end(ps), 2);
                    self.push(1, "}");
                    self.attach_trailing(self.line_end(ps));
                }
                Member::Spec => {
                    let ss = t.spec_span.unwrap();
                    self.push(1, "spec {");
                    self.attach_trailing(self.line_start(ss));
                    for s in &t.specs {
                        self.flush_leading(self.line_start(s.span), 2);
                        self.push(2, format!("{}: {}", s.name.name, s.ty.unit.type_name()));
                        self.finish_construct(self.line_start(s.span), self.line_end(s.span), 2);
                    }
                    self.flush_leading(self.line_end(ss), 2);
                    self.push(1, "}");
                    self.attach_trailing(self.line_end(ss));
                }
            }
        }
        self.flush_leading(self.line_end(item.span), 1);
        self.push(0, "}");
    }

    // -- devices -------------------------------------------------------------

    fn device_def(&mut self, vis: &str, item: &Item, d: &DeviceDef) {
        let mut header = format!("{}device {}", vis, d.name.name);
        if !d.generics.is_empty() {
            header.push_str(&generic_params(&d.generics));
        }
        self.push(0, format!("{} {{", header));
        self.attach_trailing(self.line_start(item.decl_span));

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
                    let names: Vec<String> = d.variants.iter().map(|v| v.name.clone()).collect();
                    if names.is_empty() {
                        self.push(1, "variants { }");
                    } else {
                        self.wrapped(1, "variants { ", &names, " }");
                    }
                    if let Some(vs) = d.variants_span {
                        self.finish_construct(self.line_start(vs), self.line_end(vs), 1);
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
        self.attach_trailing(self.line_start(pb.span));
        for pin in &pb.pins {
            self.flush_leading(self.line_start(pin.span), indent + 1);
            // Pin buses wrap aligned under the first pin number, with the
            // role bracket on the last line (RFC-009).
            let prefix = format!("{} {}: ", pin.obligation.keyword(), pin.name.name);
            let numbers: Vec<String> = pin.numbers.iter().map(|n| n.text.clone()).collect();
            let role = match pin.role {
                Some((r, _)) => format!(" [{}]", r.name()),
                None => String::new(),
            };
            self.wrapped(indent + 1, &prefix, &numbers, &role);
            self.finish_construct(
                self.line_start(pin.span),
                self.line_end(pin.span),
                indent + 1,
            );
        }
        self.flush_leading(self.line_end(pb.span), indent + 1);
        self.push(indent, "}");
        self.attach_trailing(self.line_end(pb.span));
    }

    fn spec_block(&mut self, sb: &SpecBlock, indent: usize) {
        let open = match &sb.variant {
            Some(v) => format!("spec[{}] {{", v.name),
            None => "spec {".to_string(),
        };
        self.push(indent, open);
        self.attach_trailing(self.line_start(sb.span));
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
        self.attach_trailing(self.line_end(sb.span));
    }

    // -- impls ---------------------------------------------------------------

    fn impl_def(&mut self, i: &ImplDef) {
        let header = format!("impl {} for {}", i.trait_name.name, i.device_name.name);
        if i.pin_map.is_empty() && i.spec_map.is_empty() {
            self.push(0, format!("{} {{}}", header));
            return;
        }
        self.push(0, format!("{} {{", header));
        self.attach_trailing(self.line_start(i.span));

        // Mapping sub-blocks in source order, comment-aware (like traits).
        let mut blocks: Vec<(u32, bool)> = Vec::new(); // (line, is_pins)
        if let Some(ps) = i.pins_span {
            blocks.push((self.line_start(ps), true));
        }
        if let Some(ss) = i.spec_span {
            blocks.push((self.line_start(ss), false));
        }
        blocks.sort_by_key(|(l, _)| *l);
        for (start, is_pins) in blocks {
            self.flush_leading(start, 1);
            let (kw, span, entries) = if is_pins {
                ("pins {", i.pins_span.unwrap(), &i.pin_map)
            } else {
                ("spec {", i.spec_span.unwrap(), &i.spec_map)
            };
            self.push(1, kw);
            self.attach_trailing(self.line_start(span));
            for e in entries {
                self.flush_leading(self.line_start(e.span), 2);
                self.push(2, format!("{}: {}", e.role.name, e.target.name));
                self.finish_construct(self.line_start(e.span), self.line_end(e.span), 2);
            }
            self.flush_leading(self.line_end(span), 2);
            self.push(1, "}");
            self.attach_trailing(self.line_end(span));
        }
        self.flush_leading(self.line_end(i.span), 1);
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
        // A trailing comment on the fn/design header line survives.
        self.attach_trailing(self.line_start(item.decl_span));
        for (idx, stmt) in stmts.iter().enumerate() {
            // Flush to the FIRST line of the whole statement group — its
            // attributes included — so comments before an attribute stay
            // before it.
            self.flush_leading(self.stmt_first_line(stmt), 1);
            self.stmt(stmt, 1);
            let end = self.line_end(stmt.span());
            // When the NEXT statement starts on this statement's last source
            // line, the line's trailing comment belongs to the last construct
            // on that line — defer it (sweep interiors only).
            let next_shares = stmts
                .get(idx + 1)
                .is_some_and(|n| self.stmt_first_line(n) == end);
            self.finish_construct_ext(self.line_start(stmt.span()), end, 1, !next_shares);
        }
        self.flush_leading(self.line_end(item.span), 1);
    }

    /// The first source line a statement occupies, attributes included.
    fn stmt_first_line(&self, stmt: &Stmt) -> u32 {
        let mut first = self.line_start(stmt.span());
        let mut consider = |span: &Span| {
            let l = self.sm.line_col(self.file, span.start).line;
            if l < first {
                first = l;
            }
        };
        match stmt {
            Stmt::Inst(s) => {
                if let Some((_, sp)) = &s.intent {
                    consider(sp);
                }
                if let Some((_, sp)) = &s.placement_hint {
                    consider(sp);
                }
                for a in &s.attrs {
                    consider(&a.span);
                }
            }
            Stmt::Net(s) => {
                if let Some((_, sp)) = &s.intent {
                    consider(sp);
                }
            }
            Stmt::Nc(s) => {
                if let Some((_, sp)) = &s.intent {
                    consider(sp);
                }
            }
            Stmt::Call(s) => {
                if let Some((_, sp)) = &s.intent {
                    consider(sp);
                }
            }
            Stmt::Layout(_) => {}
        }
        first
    }

    fn stmt(&mut self, stmt: &Stmt, indent: usize) {
        match stmt {
            Stmt::Inst(s) => {
                self.emit_string_attr("intent", &s.intent, indent);
                self.emit_string_attr("placement_hint", &s.placement_hint, indent);
                for attr in &s.attrs {
                    self.flush_leading(self.line_start(attr.span), indent);
                    self.push(indent, attr_text(attr));
                    self.attach_trailing(self.line_end(attr.span));
                }
                self.flush_leading(self.line_start(s.span), indent);
                self.push(
                    indent,
                    format!("inst {}: {}", s.name.name, type_ref_text(&s.ty)),
                );
            }
            Stmt::Net(s) => {
                self.emit_string_attr("intent", &s.intent, indent);
                self.flush_leading(self.line_start(s.span), indent);
                let name = s.name.as_ref().map_or("_".to_string(), |n| n.name.clone());
                let ann = match &s.annotation {
                    Some(NetAnnotation::Voltage(v, _)) => format!(" [{}]", v.text),
                    Some(NetAnnotation::Gnd(_)) => " [gnd]".to_string(),
                    None => String::new(),
                };
                let prefix = format!("net {}{}: ", name, ann);
                let members: Vec<String> = s.members.iter().map(|m| m.to_string()).collect();
                self.wrapped(indent, &prefix, &members, "");
            }
            Stmt::Nc(s) => {
                self.emit_string_attr("intent", &s.intent, indent);
                self.flush_leading(self.line_start(s.span), indent);
                let members: Vec<String> = s.members.iter().map(|m| m.to_string()).collect();
                self.wrapped(indent, "nc: ", &members, "");
            }
            Stmt::Layout(s) => {
                // Comment-aware like pins/spec blocks (RFC-013 uses the same
                // block-formatting rules), with per-constraint wrapping.
                self.push(indent, "layout {");
                self.attach_trailing(self.line_start(s.span));
                for c in &s.constraints {
                    self.flush_leading(self.line_start(c.span()), indent + 1);
                    self.layout_constraint(c, indent + 1);
                    self.finish_construct(
                        self.line_start(c.span()),
                        self.line_end(c.span()),
                        indent + 1,
                    );
                }
                self.flush_leading(self.line_end(s.span), indent + 1);
                self.push(indent, "}");
                self.cursor = self.cursor.max(self.line_end(s.span) + 1);
            }
            Stmt::Call(s) => {
                self.emit_string_attr("intent", &s.intent, indent);
                self.flush_leading(self.line_start(s.span), indent);
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

    /// One layout constraint, wrapping long net lists (RFC-009's 100-column
    /// soft target applies inside `layout {}` too).
    fn layout_constraint(&mut self, c: &LayoutConstraint, indent: usize) {
        match c {
            LayoutConstraint::NetClass { name, nets, .. } => {
                if nets.is_empty() {
                    self.push(indent, format!("net_class {} {{ }}", name.name));
                } else {
                    let names: Vec<String> = nets.iter().map(|n| n.name.clone()).collect();
                    self.wrapped(
                        indent,
                        &format!("net_class {} {{ ", name.name),
                        &names,
                        " }",
                    );
                }
            }
            LayoutConstraint::DiffPair { nets, .. } => {
                let names: Vec<String> = nets.iter().map(|n| n.name.clone()).collect();
                self.wrapped(indent, "diff_pair(", &names, ")");
            }
            LayoutConstraint::LengthMatch {
                nets, tolerance, ..
            } => {
                let names: Vec<String> = nets.iter().map(|n| n.name.clone()).collect();
                let suffix = match tolerance {
                    Some((s, _)) => format!(") [tolerance: {}]", tolerance_text(s)),
                    None => ")".to_string(),
                };
                self.wrapped(indent, "length_match(", &names, &suffix);
            }
        }
    }

    /// Emit a `prefix`-led, comma-separated member list with a closing
    /// `suffix`, wrapping onto continuation lines aligned under the first
    /// member when the single line exceeds `WIDTH`. The suffix rides on the
    /// last line.
    fn wrapped(&mut self, indent: usize, prefix: &str, members: &[String], suffix: &str) {
        let base = indent * INDENT.len();
        let oneline = format!("{}{}{}", prefix, members.join(", "), suffix);
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
                format!("{}{}", m, suffix)
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

    fn part_def(&mut self, vis: &str, item: &Item, p: &PartDef) {
        self.push(
            0,
            format!(
                "{}part {}: {} {{",
                vis,
                p.name.name,
                type_ref_text(&p.device)
            ),
        );
        self.attach_trailing(self.line_start(item.decl_span));
        let mut entries: Vec<(&'static str, &AvlEntry)> = vec![("primary", &p.primary)];
        for alt in &p.alts {
            entries.push(("alt", alt));
        }
        entries.sort_by_key(|(_, e)| self.line_start(e.span));
        for (kw, entry) in entries {
            self.flush_leading(self.line_start(entry.span), 1);
            let fields: Vec<String> = entry
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name.name, str_lit(&f.value)))
                .collect();
            // AVL entries wrap like other long argument lists (RFC-009).
            self.wrapped(1, &format!("{} {{ ", kw), &fields, " }");
            self.finish_construct(self.line_start(entry.span), self.line_end(entry.span), 1);
        }
        self.flush_leading(self.line_end(item.span), 1);
        self.push(0, "}");
    }
}

// ---------------------------------------------------------------------------
// Text builders (pure functions of the AST — no layout state).

/// Canonical tolerance spelling: a value that lexes as a single RFC-001 unit
/// literal is emitted unquoted (`[tolerance: 1ms]`); anything else keeps the
/// quoted-string escape hatch (`[tolerance: "0.15mm"]`). Both spellings parse
/// to the same AST value, so one canonical output is required — and this
/// choice round-trips (unquoted re-lexes to the same pass-through text).
fn tolerance_text(s: &str) -> String {
    let mut sm = SourceMap::new();
    let f = sm.add_file("tolerance", s);
    let mut diags = Diagnostics::new();
    let tokens = crate::lex::lex(f, sm.text(f), &mut diags);
    let is_unit = !diags.has_errors()
        && tokens.len() == 2
        && matches!(tokens[0].kind, crate::lex::TokenKind::Unit(_));
    if is_unit {
        s.to_string()
    } else {
        str_lit(s)
    }
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

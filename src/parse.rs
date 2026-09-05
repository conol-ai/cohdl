//! Recursive-descent parser.
//!
//! Deterministic with bounded lookahead (max 2 tokens), per the Constitution's
//! deterministic-grammar hard constraint. Error recovery is panic-mode: skip
//! to the next top-level keyword (items) or statement keyword / closing brace
//! (bodies), so one mistake doesn't cascade.

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics};
use crate::lex::{Token, TokenKind};
use crate::span::Span;
use crate::units::{UnitType, UnitValue};

pub fn parse(tokens: Vec<Token>, diags: &mut Diagnostics) -> SourceFile {
    Parser {
        tokens,
        pos: 0,
        diags,
    }
    .file()
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    diags: &'a mut Diagnostics,
}

impl<'a> Parser<'a> {
    // -- token plumbing ------------------------------------------------------

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_ahead(&self, n: usize) -> &TokenKind {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx].kind
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn prev_span(&self) -> Span {
        self.tokens[self.pos.saturating_sub(1)].span
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind, ctx: &str) -> bool {
        if self.eat(kind) {
            true
        } else {
            self.error_here(format!(
                "expected {} {}, found {}",
                TokenKind::describe(kind),
                ctx,
                self.peek().describe()
            ));
            false
        }
    }

    fn error_here(&mut self, message: String) {
        self.diags
            .push(Diagnostic::error("E010", self.span(), message));
    }

    fn ident(&mut self, ctx: &str) -> Option<Ident> {
        match self.peek() {
            TokenKind::Ident(_) => {
                let t = self.bump();
                let TokenKind::Ident(name) = t.kind else {
                    unreachable!()
                };
                Some(Ident { name, span: t.span })
            }
            _ => {
                self.error_here(format!(
                    "expected an identifier {}, found {}",
                    ctx,
                    self.peek().describe()
                ));
                None
            }
        }
    }

    /// Contextual identifier check (e.g. `pin`, `gnd`, `primary`).
    fn at_ident(&self, word: &str) -> bool {
        matches!(self.peek(), TokenKind::Ident(n) if n == word)
    }

    /// RFC-016: a possibly-qualified reference — `Name` or
    /// `package::module::Name`. Segments join into ONE Ident whose name
    /// carries the `::`s and whose span covers the whole path (resolution
    /// interprets the text; every downstream consumer keeps treating names
    /// as opaque strings). `::` followed by `<` is turbofish (RFC-007), not
    /// a path separator — fixed two-token lookahead, still deterministic.
    fn path_ident(&mut self, ctx: &str) -> Option<Ident> {
        let mut id = self.ident(ctx)?;
        while self.at(&TokenKind::PathSep) && matches!(self.peek_ahead(1), TokenKind::Ident(_)) {
            self.bump(); // ::
            let seg = self.ident("after `::` in a path")?;
            id.name.push_str("::");
            id.name.push_str(&seg.name);
            id.span = id.span.to(seg.span);
        }
        Some(id)
    }

    // -- file / items --------------------------------------------------------

    fn file(mut self) -> SourceFile {
        let mut items = Vec::new();
        while !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if let Some(item) = self.item() {
                items.push(item);
            }
            if self.pos == before {
                // Ensure progress even on hopeless input.
                self.bump();
            }
        }
        SourceFile { items }
    }

    fn sync_top_level(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Eof
                | TokenKind::Pub
                | TokenKind::Trait
                | TokenKind::Device
                | TokenKind::Impl
                | TokenKind::Fn
                | TokenKind::Part
                | TokenKind::Design
                | TokenKind::Hash => return,
                TokenKind::Ident(n) if n == "use" || n == "footprint" || n == "pad" => return,
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn item(&mut self) -> Option<Item> {
        let start = self.span();
        let (attrs, phys) = self.attrs();
        for pa in &phys {
            self.diags.push(Diagnostic::error(
                "E1009",
                pa.span(),
                format!(
                    "`#[{}]` is only valid on a `net` or `inst` declaration inside a design",
                    pa.name()
                ),
            ));
        }
        // RFC-012: `#[intent("...")]` is opaque metadata valid on any
        // declaration; any other attribute (`#[designator]`) is inst-only.
        let (intent, rest) = self.take_intent(attrs);
        // RFC-017: `#[doc("relative/path")]` — one or MORE per declaration.
        let (docs, rest) = self.take_docs(rest);
        // Where the declaration proper begins — after any attributes.
        let decl_start = self.span();
        let is_pub = self.eat(&TokenKind::Pub);
        // RFC-016 `use path::Name;` — contextual keyword (an item can't
        // otherwise start with a bare identifier).
        if self.at_ident("use") {
            if is_pub {
                // Anchor at the `pub` token itself (decl_start), not `use`.
                self.diags.push(Diagnostic::error(
                    "E010",
                    decl_start,
                    "`pub use` re-exports are not in RFC-016's first pass — remove `pub`"
                        .to_string(),
                ));
            }
            let kind = self.use_decl().map(ItemKind::Use);
            self.reject_attrs(&rest);
            if let Some((_, intent_span)) = &intent {
                // Anchor at the attribute, not whatever token follows the
                // already-consumed statement.
                self.diags.push(Diagnostic::error(
                    "E010",
                    *intent_span,
                    "`#[intent]` is not valid on a `use` import".to_string(),
                ));
            }
            for (_, doc_span) in &docs {
                self.diags.push(Diagnostic::error(
                    "E010",
                    *doc_span,
                    "`#[doc]` is not valid on a `use` import".to_string(),
                ));
            }
            let kind = kind?;
            return Some(Item {
                is_pub: false,
                intent: None,
                docs: Vec::new(),
                decl_span: decl_start,
                span: start.to(self.prev_span()),
                kind,
            });
        }
        // RFC-017 `footprint NAME {}` — contextual keyword, like `use`.
        if self.at_ident("footprint") && self.peek_ahead(1) == &TokenKind::LBrace {
            let span = self.span();
            self.diags.push(Diagnostic::error(
                "E010",
                span,
                "a `footprint` declaration needs a name: `footprint NAME {}`".to_string(),
            ));
            self.bump(); // footprint
            self.bump(); // `{` — skip_braced_body expects the opener consumed
            self.skip_braced_body(span);
            return None;
        }
        if self.at_ident("pad") && matches!(self.peek_ahead(1), TokenKind::Ident(_)) {
            let kind = self.pad_def().map(ItemKind::Pad);
            self.reject_attrs(&rest);
            let kind = kind?;
            return Some(Item {
                is_pub,
                intent,
                docs,
                decl_span: decl_start,
                span: start.to(self.prev_span()),
                kind,
            });
        }
        if self.at_ident("pad") && self.peek_ahead(1) == &TokenKind::LBrace {
            let span = self.span();
            self.diags.push(Diagnostic::error(
                "E010",
                span,
                "a `pad` declaration needs a name: `pad NAME { … }`".to_string(),
            ));
            self.bump(); // pad
            self.bump(); // `{` — skip_braced_body expects the opener consumed
            self.skip_braced_body(span);
            return None;
        }
        if self.at_ident("footprint") && matches!(self.peek_ahead(1), TokenKind::Ident(_)) {
            let kind = self.footprint_def().map(ItemKind::Footprint);
            self.reject_attrs(&rest);
            let kind = kind?;
            return Some(Item {
                is_pub,
                intent,
                docs,
                decl_span: decl_start,
                span: start.to(self.prev_span()),
                kind,
            });
        }
        if !docs.is_empty() && matches!(self.peek(), TokenKind::Impl) {
            for (_, doc_span) in &docs {
                self.diags.push(Diagnostic::error(
                    "E010",
                    *doc_span,
                    "`#[doc]` is not valid on an `impl` — impls are unnamed; attach the document to the trait or device"
                        .to_string(),
                ));
            }
        }
        let kind = match self.peek() {
            TokenKind::Trait => self.trait_def().map(ItemKind::Trait),
            TokenKind::Device => self.device_def().map(ItemKind::Device),
            TokenKind::Impl => self.impl_def().map(ItemKind::Impl),
            TokenKind::Fn => self.fn_def().map(ItemKind::Fn),
            TokenKind::Part => self.part_def().map(ItemKind::Part),
            TokenKind::Design => self.design_def().map(ItemKind::Design),
            other => {
                self.error_here(format!(
                    "expected a top-level declaration (`trait`, `device`, `impl`, `fn`, `part`, `design`, `footprint`, `pad`, or `use`), found {}",
                    other.describe()
                ));
                self.sync_top_level();
                None
            }
        };
        self.reject_attrs(&rest);
        let kind = kind?;
        Some(Item {
            is_pub,
            intent,
            docs,
            decl_span: decl_start,
            span: start.to(self.prev_span()),
            kind,
        })
    }

    /// Split `#[intent("...")]` (RFC-012) out of `attrs`. Intent is opaque
    /// metadata — never threaded into any checking or emission pass — so it can
    /// never affect a verdict, diagnostic, designator, or emitted byte.
    fn take_intent(&mut self, attrs: Vec<Attr>) -> (Option<(String, Span)>, Vec<Attr>) {
        self.take_string_attr("intent", attrs)
    }

    /// RFC-016: `use package::module::Name;` — at least two segments (a
    /// lone `use Name;` imports nothing a bare name doesn't already reach).
    fn use_decl(&mut self) -> Option<UseDecl> {
        let start = self.span();
        self.bump(); // `use`
        let Some(first) = self.ident("as the first path segment of `use`") else {
            self.sync_use();
            return None;
        };
        let mut path = vec![first];
        while self.eat(&TokenKind::PathSep) {
            match self.ident("after `::` in the `use` path") {
                Some(seg) => path.push(seg),
                None => {
                    // Resynchronize past the broken statement so leftover
                    // tokens (e.g. a keyword inside the path) can't misparse
                    // as a phantom declaration.
                    self.sync_use();
                    return None;
                }
            }
        }
        if path.len() < 2 {
            // Anchor at the lone segment, not the token after it.
            self.diags.push(Diagnostic::error(
                "E010",
                path[0].span,
                format!(
                    "`use` needs a qualified path (`use package::module::Name;`) — `{}` has no package segment",
                    path[0].name
                ),
            ));
        }
        // The spec's canonical form carries the semicolon.
        if !self.eat(&TokenKind::Semi) {
            self.error_here(format!(
                "expected `;` to end the `use` import, found {}",
                self.peek().describe()
            ));
        }
        Some(UseDecl {
            path,
            span: start.to(self.prev_span()),
        })
    }

    /// RFC-018: `footprint NAME { pad N: Sym at (x, y) … [courtyard {…}]
    /// [silkscreen_ref {…}] }`. An empty body is RFC-017's stage-one
    /// placeholder and stays legal.
    fn footprint_def(&mut self) -> Option<FootprintDef> {
        let start = self.span();
        self.bump(); // `footprint`
        let name = self.ident("as the footprint name")?;
        let open_span = self.span();
        if !self.expect(&TokenKind::LBrace, "to open the footprint body") {
            self.sync_top_level();
            return None;
        }
        let mut pads = Vec::new();
        let mut mount_holes = Vec::new();
        let mut courtyard: Option<Courtyard> = None;
        let mut window: Option<Box<Courtyard>> = None;
        let mut silkscreen: Option<Box<SilkscreenBlock>> = None;
        let mut silkscreen_ref: Option<(UnitValue, UnitValue, Span)> = None;
        let mut unclosed = false;
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            // A top-level declaration keyword means the body's `}` is
            // missing (nothing top-level is legal in a footprint body) —
            // stop WITHOUT consuming so the declaration survives.
            if self.at_decl_keyword() {
                self.diags.push(Diagnostic::error(
                    "E010",
                    open_span,
                    format!(
                        "unclosed footprint body — missing `}}` before {}",
                        self.peek().describe()
                    ),
                ));
                unclosed = true;
                break;
            }
            if self.at_ident("pad") {
                if let Some(p) = self.pad_place() {
                    pads.push(p);
                } else {
                    self.sync_footprint_body();
                }
            } else if self.at_ident("mount_hole") {
                if let Some(m) = self.mount_hole() {
                    mount_holes.push(m);
                } else {
                    self.sync_footprint_body();
                }
            } else if self.at_ident("courtyard") {
                let c = self.shape_block("courtyard");
                match (&courtyard, c) {
                    (Some(prev), Some(next)) => {
                        self.diags.push(
                            Diagnostic::error(
                                "E806",
                                next.span,
                                "a footprint has at most one `courtyard`".to_string(),
                            )
                            .with_secondary(prev.span, "the first courtyard is here".to_string()),
                        );
                    }
                    (None, Some(next)) => courtyard = Some(next),
                    // courtyard() recovers internally (its body is consumed
                    // or it stopped at a safe boundary) — do not sync again.
                    (_, None) => {}
                }
            } else if self.at_ident("silkscreen") {
                let blk = self.silkscreen_block();
                match (&silkscreen, blk) {
                    (Some(prev), Some(next)) => {
                        self.diags.push(
                            Diagnostic::error(
                                "E812",
                                next.span,
                                "a footprint has at most one `silkscreen` block".to_string(),
                            )
                            .with_secondary(prev.span, "the first one is here".to_string()),
                        );
                    }
                    (None, Some(next)) => silkscreen = Some(Box::new(next)),
                    (_, None) => {}
                }
            } else if self.at_ident("window") {
                let w = self.shape_block("window");
                match (&window, w) {
                    (Some(prev), Some(next)) => {
                        self.diags.push(
                            Diagnostic::error(
                                "E806",
                                next.span,
                                "a footprint has at most one `window`".to_string(),
                            )
                            .with_secondary(prev.span, "the first window is here".to_string()),
                        );
                    }
                    (None, Some(next)) => window = Some(Box::new(next)),
                    (_, None) => {}
                }
            } else if self.at_ident("silkscreen_ref") {
                let start_sr = self.span();
                self.bump();
                if !self.expect(&TokenKind::LBrace, "to open `silkscreen_ref`") {
                    self.sync_footprint_body();
                    continue;
                }
                if !self.eat_ident("at") {
                    self.error_here(format!(
                        "expected `at: (x, y)` in `silkscreen_ref`, found {}",
                        self.peek().describe()
                    ));
                    self.sync_footprint_body();
                    continue;
                }
                self.expect(&TokenKind::Colon, "after `at`");
                let Some((x, y)) = self.length_pair() else {
                    self.sync_footprint_body();
                    continue;
                };
                self.expect(&TokenKind::RBrace, "to close `silkscreen_ref`");
                let span = start_sr.to(self.prev_span());
                if let Some((_, _, prev)) = &silkscreen_ref {
                    self.diags.push(
                        Diagnostic::error(
                            "E806",
                            span,
                            "a footprint has at most one `silkscreen_ref`".to_string(),
                        )
                        .with_secondary(*prev, "the first one is here".to_string()),
                    );
                } else {
                    silkscreen_ref = Some((x, y, span));
                }
            } else {
                self.diags.push(Diagnostic::error(
                    "E806",
                    self.span(),
                    format!(
                        "a footprint body contains `pad N: Symbol at (x, y)` placements, `mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D | size: (w, h)]` holes, an optional `courtyard`, and an optional `silkscreen_ref` — found {}",
                        self.peek().describe()
                    ),
                ));
                self.sync_footprint_body();
            }
            // Recovery stops AT boundary tokens without consuming —
            // guarantee progress so a stray token can never loop forever.
            if self.pos == before {
                self.bump();
            }
        }
        if !unclosed {
            self.expect(&TokenKind::RBrace, "to close the footprint body");
        }
        Some(FootprintDef {
            name,
            pads,
            mount_holes,
            courtyard,
            window,
            silkscreen,
            silkscreen_ref,
            span: start.to(self.prev_span()),
        })
    }

    /// RFC-022: one `mount_hole N: PLATING at (x, y) diameter D` line.
    fn mount_hole(&mut self) -> Option<crate::ast::MountHole> {
        use crate::ast::MountHolePlating;
        let start = self.span();
        self.bump(); // `mount_hole`
        let number = match self.peek() {
            TokenKind::Number(_) | TokenKind::Ident(_) => {
                let t = self.bump();
                let text = match t.kind {
                    TokenKind::Number(text) | TokenKind::Ident(text) => text,
                    _ => unreachable!(),
                };
                PinNumber { text, span: t.span }
            }
            other => {
                self.error_here(format!(
                    "expected the mount-hole number (e.g. `1`), found {}",
                    other.describe()
                ));
                return None;
            }
        };
        self.expect(&TokenKind::Colon, "after the mount-hole number");
        let plating = {
            let v = self.ident("as the plating (`non_plated` or `plated`)")?;
            match MountHolePlating::from_name(&v.name) {
                Some(p) => p,
                None => {
                    self.diags.push(Diagnostic::error(
                        "E810",
                        v.span,
                        format!(
                            "`{}` is not a mount-hole plating — platings are: non_plated, plated",
                            v.name
                        ),
                    ));
                    return None;
                }
            }
        };
        // RFC-023 adds an optional `shape:` and makes the geometry field
        // shape-dependent (`diameter D` for a circle, `size: (w, h)` for a
        // rect/oval). The accepted text's grammar line orders these
        // `[shape:] at (x, y) [geometry]` while its own worked example writes
        // `[shape:] [geometry] at (x, y)`, so both are accepted here — each
        // component is introduced by a distinct keyword, so this stays a
        // single-token decision (no lookahead). `fmt` normalizes to the
        // grammar line's order, which is also RFC-022's existing one.
        let mut shape = None;
        let mut at = None;
        let mut geom = None;
        loop {
            if shape.is_none() && self.at_ident("shape") {
                self.bump();
                self.expect(&TokenKind::Colon, "after `shape`");
                let v = self.ident("as the mount-hole shape")?;
                match PadShape::from_name(&v.name) {
                    Some(PadShape::Annulus) => {
                        self.diags.push(Diagnostic::error(
                            "E810",
                            v.span,
                            "`annulus` is only valid for electrical pads, not `mount_hole`"
                                .to_string(),
                        ));
                        return None;
                    }
                    Some(s) => shape = Some((s, v.span)),
                    None => {
                        self.diags.push(Diagnostic::error(
                            "E810",
                            v.span,
                            format!(
                                "`{}` is not a mount-hole shape — shapes are: rect, circle, oval",
                                v.name
                            ),
                        ));
                        return None;
                    }
                }
            } else if at.is_none() && self.at_ident("at") {
                self.bump();
                at = Some(self.length_pair()?);
            } else if geom.is_none() && self.at_ident("diameter") {
                self.bump();
                geom = Some(crate::ast::MountHoleGeom::Diameter(
                    self.unit_literal("as the mount-hole diameter")?,
                ));
            } else if geom.is_none() && self.at_ident("size") {
                self.bump();
                self.expect(&TokenKind::Colon, "after `size`");
                let (dims, span) = self.length_tuple()?;
                geom = Some(crate::ast::MountHoleGeom::Size(dims, span));
            } else {
                break;
            }
        }
        let Some((x, y)) = at else {
            self.error_here(format!(
                "expected `at (x, y)` in the mount_hole, found {}",
                self.peek().describe()
            ));
            return None;
        };
        // Whichever geometry was written, its agreement with the (explicit or
        // defaulted) shape is checked in `resolve` (E810) — so a mismatch
        // reports the real defect instead of a confusing parse error.
        let Some(geom) = geom else {
            self.error_here(format!(
                "expected `diameter D` (for a circle) or `size: (w, h)` (for a rect/oval) in the mount_hole, found {}",
                self.peek().describe()
            ));
            return None;
        };
        Some(crate::ast::MountHole {
            number,
            plating,
            shape,
            x,
            y,
            geom,
            span: start.to(self.prev_span()),
        })
    }

    /// One `pad N: PadSymbol at (x, y)` placement line.
    fn pad_place(&mut self) -> Option<PadPlace> {
        let start = self.span();
        self.bump(); // `pad`
        let number = match self.peek() {
            TokenKind::Number(_) => {
                let t = self.bump();
                let TokenKind::Number(text) = t.kind else {
                    unreachable!()
                };
                PinNumber { text, span: t.span }
            }
            TokenKind::Ident(_) => {
                let t = self.bump();
                let TokenKind::Ident(text) = t.kind else {
                    unreachable!()
                };
                PinNumber { text, span: t.span }
            }
            other => {
                self.error_here(format!(
                    "expected the pad number (matching a device pin number, e.g. `1` or `A3`), found {}",
                    other.describe()
                ));
                return None;
            }
        };
        self.expect(&TokenKind::Colon, "after the pad number");
        let pad = self.path_ident("as the pad symbol")?;
        if !self.eat_ident("at") {
            self.error_here(format!(
                "expected `at (x, y)` after the pad symbol, found {}",
                self.peek().describe()
            ));
            return None;
        }
        let (x, y) = self.length_pair()?;
        // RFC-025: optional `rotate ANGLE` — any whole degree, validated at
        // declaration check (E811); unparseable values map to the same
        // out-of-range sentinel `place` uses.
        let mut rotate = 0u16;
        if self.at_ident("rotate") {
            self.bump();
            match self.peek() {
                TokenKind::Number(_) => {
                    let t = self.bump();
                    if let TokenKind::Number(n) = t.kind {
                        rotate = n.parse::<u16>().unwrap_or(u16::MAX);
                    }
                }
                _ => {
                    self.error_here(format!(
                        "expected a rotation angle (0, 90, 180, or 270) after `rotate`, found {}",
                        self.peek().describe()
                    ));
                }
            }
        }
        Some(PadPlace {
            number,
            pad,
            x,
            y,
            rotate,
            span: start.to(self.prev_span()),
        })
    }

    /// `courtyard { shape: rect, at: (x, y), size: (…) }`. Recovers from
    /// broken fields internally (sync to the next comma) so a typo never
    /// spills phantom errors past the courtyard; a runaway into a member
    /// keyword or top-level declaration stops WITHOUT consuming, so an
    /// unclosed courtyard cannot steal the footprint's closing brace.
    /// Consume an expected keyword, or report and fail. RFC-031's statement
    /// grammars are keyword-heavy (`line from … to … width …`), so each one
    /// names the keyword it wanted and where it wanted it.
    fn expect_ident(&mut self, word: &str, ctx: &str) -> Option<()> {
        if self.eat_ident(word) {
            return Some(());
        }
        self.error_here(format!(
            "expected `{}` {}, found {}",
            word,
            ctx,
            self.peek().describe()
        ));
        None
    }

    /// A pad number in a reference position (RFC-031 markers) — the same
    /// `1` / `A3` grammar `pad N:` placements accept.
    fn pad_number_ref(&mut self, ctx: &str) -> Option<PinNumber> {
        match self.peek() {
            TokenKind::Number(_) | TokenKind::Ident(_) => {
                let t = self.bump();
                let text = match t.kind {
                    TokenKind::Number(x) | TokenKind::Ident(x) => x,
                    _ => unreachable!(),
                };
                Some(PinNumber { text, span: t.span })
            }
            other => {
                self.error_here(format!(
                    "expected a pad number {} (e.g. `1` or `A3`), found {}",
                    ctx,
                    other.describe()
                ));
                None
            }
        }
    }

    /// `fill FILL` — optional trailing clause on `circle`/`polygon`.
    fn silk_fill(&mut self, default: SilkFill) -> SilkFill {
        if !self.eat_ident("fill") {
            return default;
        }
        match self.ident("as the fill") {
            Some(v) => match SilkFill::from_name(&v.name) {
                Some(f) => f,
                None => {
                    self.diags.push(Diagnostic::error(
                        "E812",
                        v.span,
                        format!("`{}` is not a fill — fills are: none, solid", v.name),
                    ));
                    default
                }
            },
            None => default,
        }
    }

    /// A whole-degree angle for `arc` (RFC-031 allows any 0..=360, unlike the
    /// cardinal set `rotate` is restricted to).
    fn silk_angle(&mut self, what: &str) -> Option<i32> {
        let t = self.bump();
        let TokenKind::Number(text) = &t.kind else {
            self.diags.push(Diagnostic::error(
                "E812",
                t.span,
                format!(
                    "expected {} in whole degrees, found {}",
                    what,
                    t.kind.describe()
                ),
            ));
            return None;
        };
        match text.parse::<i32>() {
            Ok(n) if (0..=360).contains(&n) => Some(n),
            _ => {
                self.diags.push(Diagnostic::error(
                    "E812",
                    t.span,
                    format!(
                        "`{}` is not a whole-degree angle in 0..=360 for {}",
                        text, what
                    ),
                ));
                None
            }
        }
    }

    /// RFC-031 `silkscreen { … }` — the drawable-graphics block.
    fn silkscreen_block(&mut self) -> Option<SilkscreenBlock> {
        let start = self.span();
        self.bump(); // `silkscreen`
        if !self.expect(&TokenKind::LBrace, "to open `silkscreen`") {
            self.sync_footprint_body();
            return None;
        }
        let mut items: Vec<SilkItem> = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            let kw_span = self.span();
            let Some(kw) = self.ident(
                "as a silkscreen statement (`line`, `circle`, `arc`, `polygon`, \
                 `pin_1_marker`, `polarity_marker`)",
            ) else {
                self.sync_in_block();
                self.eat(&TokenKind::Comma);
                if self.pos == before {
                    self.bump();
                }
                continue;
            };
            let item = match kw.name.as_str() {
                "line" => (|| {
                    self.expect_ident("from", "after `line`")?;
                    let from = self.length_pair()?;
                    self.expect_ident("to", "after the start point")?;
                    let to = self.length_pair()?;
                    self.expect_ident("width", "after the end point")?;
                    let width = self.unit_literal("as the stroke width")?;
                    Some(SilkGraphic::Line { from, to, width })
                })(),
                "circle" => (|| {
                    self.expect_ident("at", "after `circle`")?;
                    let at = self.length_pair()?;
                    self.expect_ident("radius", "after the centre")?;
                    let radius = self.unit_literal("as the radius")?;
                    self.expect_ident("width", "after the radius")?;
                    let width = self.unit_literal("as the stroke width")?;
                    let fill = self.silk_fill(SilkFill::None);
                    Some(SilkGraphic::Circle {
                        at,
                        radius,
                        width,
                        fill,
                    })
                })(),
                "arc" => (|| {
                    self.expect_ident("at", "after `arc`")?;
                    let at = self.length_pair()?;
                    self.expect_ident("radius", "after the centre")?;
                    let radius = self.unit_literal("as the radius")?;
                    self.expect_ident("start_angle", "after the radius")?;
                    let start_angle = self.silk_angle("the start angle")?;
                    self.expect_ident("end_angle", "after the start angle")?;
                    let end_angle = self.silk_angle("the end angle")?;
                    self.expect_ident("width", "after the end angle")?;
                    let width = self.unit_literal("as the stroke width")?;
                    Some(SilkGraphic::Arc {
                        at,
                        radius,
                        start_angle,
                        end_angle,
                        width,
                    })
                })(),
                "polygon" => (|| {
                    self.expect(&TokenKind::LBracket, "to open the vertex list");
                    let mut points = Vec::new();
                    while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
                        let p = self.length_pair()?;
                        points.push(p);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBracket, "to close the vertex list");
                    let fill = self.silk_fill(SilkFill::Solid);
                    if points.len() < 3 {
                        self.diags.push(Diagnostic::error(
                            "E812",
                            kw_span.to(self.prev_span()),
                            format!(
                                "a `polygon` needs at least 3 vertices — {} given",
                                points.len()
                            ),
                        ));
                        return None;
                    }
                    Some(SilkGraphic::Polygon { points, fill })
                })(),
                "pin_1_marker" => {
                    let parsed = (|| {
                        self.expect_ident("near", "after `pin_1_marker`")?;
                        self.expect_ident("pad", "after `near`")?;
                        let pad = self.pad_number_ref("the marker refers to")?;
                        self.expect_ident("shape", "after the pad number")?;
                        let v = self.ident("as the marker shape")?;
                        let shape = match v.name.as_str() {
                            "dot" => Pin1Shape::Dot,
                            "triangle" => Pin1Shape::Triangle,
                            other => {
                                self.diags.push(Diagnostic::error(
                                    "E812",
                                    v.span,
                                    format!(
                                        "`{}` is not a pin-1 marker shape — shapes are: dot, triangle",
                                        other
                                    ),
                                ));
                                return None;
                            }
                        };
                        Some((pad, shape))
                    })();
                    if let Some((pad, shape)) = parsed {
                        items.push(SilkItem::Pin1Marker {
                            pad,
                            shape,
                            span: kw_span.to(self.prev_span()),
                        });
                    } else {
                        self.sync_in_block();
                    }
                    if self.pos == before {
                        self.bump();
                    }
                    continue;
                }
                "polarity_marker" => {
                    let parsed = (|| {
                        self.expect_ident("cathode_pin", "after `polarity_marker`")?;
                        let pad = self.pad_number_ref("the cathode terminal")?;
                        self.expect_ident("shape", "after the pad number")?;
                        let v = self.ident("as the marker shape")?;
                        let shape = match v.name.as_str() {
                            "band" => PolarityShape::Band,
                            "arrow" => PolarityShape::Arrow,
                            other => {
                                self.diags.push(Diagnostic::error(
                                    "E812",
                                    v.span,
                                    format!(
                                        "`{}` is not a polarity marker shape — shapes are: band, arrow",
                                        other
                                    ),
                                ));
                                return None;
                            }
                        };
                        Some((pad, shape))
                    })();
                    if let Some((cathode_pad, shape)) = parsed {
                        items.push(SilkItem::PolarityMarker {
                            cathode_pad,
                            shape,
                            span: kw_span.to(self.prev_span()),
                        });
                    } else {
                        self.sync_in_block();
                    }
                    if self.pos == before {
                        self.bump();
                    }
                    continue;
                }
                other => {
                    self.diags.push(Diagnostic::error(
                        "E812",
                        kw.span,
                        format!(
                            "unknown silkscreen statement `{}` (expected `line`, `circle`, \
                             `arc`, `polygon`, `pin_1_marker`, or `polarity_marker`)",
                            other
                        ),
                    ));
                    self.sync_in_block();
                    None
                }
            };
            match item {
                Some(g) => items.push(SilkItem::Graphic(g, kw_span.to(self.prev_span()))),
                None => self.sync_in_block(),
            }
            self.eat(&TokenKind::Comma);
            if self.pos == before {
                self.bump();
            }
        }
        self.expect(&TokenKind::RBrace, "to close `silkscreen`");
        Some(SilkscreenBlock {
            items,
            span: start.to(self.prev_span()),
        })
    }

    /// The shared `{ shape, at, size }` block body — `courtyard` and `window`
    /// differ only in the keyword they report in diagnostics.
    fn shape_block(&mut self, kw: &str) -> Option<Courtyard> {
        let start = self.span();
        self.bump(); // the keyword
        let open_span = self.span();
        if !self.expect(&TokenKind::LBrace, "to open the block") {
            self.sync_footprint_body();
            return None;
        }
        let mut shape: Option<(PadShape, Span)> = None;
        let mut at: Option<(UnitValue, UnitValue)> = None;
        let mut size: Option<(Vec<UnitValue>, Span)> = None;
        let mut unclosed = false;
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if self.at_ident("pad")
                || self.at_ident("courtyard")
                || self.at_ident("window")
                || self.at_ident("silkscreen")
                || self.at_ident("silkscreen_ref")
                || self.at_decl_keyword()
            {
                self.diags.push(Diagnostic::error(
                    "E010",
                    open_span,
                    format!(
                        "unclosed `{}` — missing `}}` before {}",
                        kw,
                        self.peek().describe()
                    ),
                ));
                unclosed = true;
                break;
            }
            let Some(field) = self.ident("as a `shape`/`at`/`size` field") else {
                self.sync_in_block();
                self.eat(&TokenKind::Comma);
                continue;
            };
            self.expect(&TokenKind::Colon, "after the field name");
            match field.name.as_str() {
                "shape" => match self.ident("as the shape") {
                    Some(v) => match PadShape::from_name(&v.name) {
                        Some(PadShape::Annulus) => self.diags.push(Diagnostic::error(
                            "E806",
                            v.span,
                            format!("`annulus` is only valid for electrical pads, not `{}`", kw),
                        )),
                        Some(s) => shape = Some((s, v.span)),
                        None => self.diags.push(Diagnostic::error(
                            "E806",
                            v.span,
                            format!(
                                "`{}` is not a shape — shapes are: rect, circle, oval",
                                v.name
                            ),
                        )),
                    },
                    None => {
                        self.sync_in_block();
                    }
                },
                "at" => match self.length_pair() {
                    Some(pair) => at = Some(pair),
                    None => {
                        self.sync_in_block();
                    }
                },
                "size" => match self.length_tuple() {
                    Some(tuple) => size = Some(tuple),
                    None => {
                        self.sync_in_block();
                    }
                },
                other => {
                    self.diags.push(Diagnostic::error(
                        "E806",
                        field.span,
                        format!(
                            "unknown {} field `{}` (expected `shape`, `at`, or `size`)",
                            kw, other
                        ),
                    ));
                    self.sync_in_block();
                }
            }
            self.eat(&TokenKind::Comma);
            if self.pos == before {
                self.bump();
            }
        }
        if !unclosed {
            self.expect(&TokenKind::RBrace, "to close the block");
        }
        let span = start.to(self.prev_span());
        let (Some(shape), Some(at), Some((size, size_span))) = (shape, at, size) else {
            self.diags.push(Diagnostic::error(
                "E806",
                span,
                format!("`{}` needs `shape`, `at`, and `size`", kw),
            ));
            return None;
        };
        Some(Courtyard {
            shape,
            at,
            size,
            size_span,
            span,
        })
    }

    /// `(x, y)` — exactly two unit literals (unit TYPE checked later).
    fn length_pair(&mut self) -> Option<(UnitValue, UnitValue)> {
        if !self.expect(&TokenKind::LParen, "to open the coordinate pair") {
            // One defect, one diagnostic — a missing `(` already implies the
            // offsets are absent; don't also report each of them.
            return None;
        }
        let x = self.unit_literal("as the x offset")?;
        self.expect(&TokenKind::Comma, "between the coordinates");
        let y = self.unit_literal("as the y offset")?;
        self.expect(&TokenKind::RParen, "to close the coordinate pair");
        Some((x, y))
    }

    /// `(a)` or `(a, b)` — one or two unit literals, with the whole span.
    fn length_tuple(&mut self) -> Option<(Vec<UnitValue>, Span)> {
        let start = self.span();
        if !self.expect(&TokenKind::LParen, "to open the size tuple") {
            return None;
        }
        let mut out = vec![self.unit_literal("as a dimension")?];
        while self.eat(&TokenKind::Comma) {
            out.push(self.unit_literal("as a dimension")?);
        }
        self.expect(&TokenKind::RParen, "to close the size tuple");
        Some((out, start.to(self.prev_span())))
    }

    fn unit_literal(&mut self, ctx: &str) -> Option<UnitValue> {
        match self.peek() {
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                Some(v)
            }
            other => {
                self.error_here(format!(
                    "expected a unit literal {} (e.g. `0.5mm`), found {}",
                    ctx,
                    other.describe()
                ));
                None
            }
        }
    }

    fn eat_ident(&mut self, word: &str) -> bool {
        if self.at_ident(word) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// RFC-018: `pad NAME { shape: …, size: (…), layer: …, plating: …[,
    /// drill: …] }` — a reusable pad definition (closed vocabulary).
    fn pad_def(&mut self) -> Option<PadDef> {
        let start = self.span();
        self.bump(); // `pad`
        let name = self.ident("as the pad name")?;
        let open_span = self.span();
        if !self.expect(&TokenKind::LBrace, "to open the pad body") {
            self.sync_top_level();
            return None;
        }
        let mut unclosed = false;
        let mut def = PadDef {
            name,
            shape: None,
            size: Vec::new(),
            size_span: None,
            layer: None,
            plating: None,
            drill: None,
            chamfer: None,
            corner_radius: None,
            mask_expansion: None,
            paste: None,
            span: start,
        };
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if self.at_decl_keyword() {
                self.diags.push(Diagnostic::error(
                    "E010",
                    open_span,
                    format!(
                        "unclosed pad body — missing `}}` before {}",
                        self.peek().describe()
                    ),
                ));
                unclosed = true;
                break;
            }
            let Some(field) = self.ident(
                "as a pad field (`shape`, `size`, `layer`, `plating`, `drill`, `chamfer`, `corner_radius`, `mask_expansion`, `paste`)",
            )
            else {
                self.sync_in_block();
                self.eat(&TokenKind::Comma);
                continue;
            };
            self.expect(&TokenKind::Colon, "after the pad field name");
            match field.name.as_str() {
                "shape" => {
                    let Some(v) = self.ident("as the shape") else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    };
                    match PadShape::from_name(&v.name) {
                        Some(s) => def.shape = Some((s, v.span)),
                        None => self.diags.push(Diagnostic::error(
                            "E805",
                            v.span,
                            format!(
                                "`{}` is not a pad shape — shapes are: rect, circle, oval, annulus",
                                v.name
                            ),
                        )),
                    }
                }
                "size" => {
                    let Some((vals, span)) = self.length_tuple() else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    };
                    def.size = vals;
                    def.size_span = Some(span);
                }
                "layer" => {
                    let Some(v) = self.ident("as the layer") else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    };
                    match PadLayer::from_name(&v.name) {
                        Some(l) => def.layer = Some((l, v.span)),
                        None => self.diags.push(Diagnostic::error(
                            "E805",
                            v.span,
                            format!(
                                "`{}` is not a pad layer — layers are: top_copper, bottom_copper, through_all",
                                v.name
                            ),
                        )),
                    }
                }
                "plating" => {
                    let Some(v) = self.ident("as the plating") else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    };
                    match PadPlating::from_name(&v.name) {
                        Some(p) => def.plating = Some((p, v.span)),
                        None => self.diags.push(Diagnostic::error(
                            "E805",
                            v.span,
                            format!(
                                "`{}` is not a pad plating — platings are: smd, plated_through_hole",
                                v.name
                            ),
                        )),
                    }
                }
                "drill" => {
                    // `drill: D` (round) or `drill: (w, l)` (slot) — the same
                    // scalar-or-tuple split RFC-023 gave `mount_hole`.
                    let drill = if matches!(self.peek(), TokenKind::LParen) {
                        let Some((vals, span)) = self.length_tuple() else {
                            self.sync_in_block();
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        let [w, l] = vals.as_slice() else {
                            self.diags.push(Diagnostic::error(
                                "E805",
                                span,
                                format!(
                                    "a slot drill is `(width, length)` — {} value{} given",
                                    vals.len(),
                                    if vals.len() == 1 { "" } else { "s" }
                                ),
                            ));
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        crate::ast::PadDrill::Slot(w.clone(), l.clone())
                    } else {
                        let Some(v) = self.unit_literal("as the drill diameter") else {
                            self.sync_in_block();
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        crate::ast::PadDrill::Round(v)
                    };
                    def.drill = Some((drill, field.span));
                }
                "chamfer" => {
                    if !self.expect(&TokenKind::LParen, "to open the chamfer tuple") {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    }
                    let Some(corner) = self.ident("as the chamfer corner") else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    };
                    let parsed_corner = PadCorner::from_name(&corner.name);
                    if parsed_corner.is_none() {
                        self.diags.push(Diagnostic::error(
                            "E805",
                            corner.span,
                            format!(
                                "`{}` is not a pad corner — corners are: top_left, top_right, bottom_left, bottom_right",
                                corner.name
                            ),
                        ));
                    }
                    self.expect(
                        &TokenKind::Comma,
                        "between the chamfer corner and cut length",
                    );
                    let cut = self.unit_literal("as the chamfer cut length");
                    self.expect(&TokenKind::RParen, "to close the chamfer tuple");
                    if let (Some(corner), Some(cut)) = (parsed_corner, cut) {
                        def.chamfer = Some((corner, cut, field.span.to(self.prev_span())));
                    }
                }
                "corner_radius" => {
                    if let Some(v) = self.unit_literal("as the rectangular pad corner radius") {
                        def.corner_radius = Some((v, field.span.to(self.prev_span())));
                    } else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    }
                }
                "mask_expansion" => {
                    if let Some(v) = self.unit_literal("as the solder-mask expansion") {
                        def.mask_expansion = Some((v, field.span.to(self.prev_span())));
                    } else {
                        self.sync_in_block();
                        self.eat(&TokenKind::Comma);
                        continue;
                    }
                }
                "paste" => {
                    if self.eat_ident("none") {
                        def.paste = Some((PadPaste::None, field.span.to(self.prev_span())));
                    } else if self.eat_ident("circle") {
                        let Some((vals, span)) = self.length_tuple() else {
                            self.sync_in_block();
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        let [diameter] = vals.as_slice() else {
                            self.diags.push(Diagnostic::error(
                                "E805",
                                span,
                                format!(
                                    "`circle` paste is `circle(diameter)` — {} value{} given",
                                    vals.len(),
                                    if vals.len() == 1 { "" } else { "s" }
                                ),
                            ));
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        def.paste = Some((PadPaste::Circle(diameter.clone()), field.span.to(span)));
                    } else if self.eat_ident("segmented_annulus") {
                        let Some((vals, span)) = self.length_tuple() else {
                            self.sync_in_block();
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        let [outer, inner, gap] = vals.as_slice() else {
                            self.diags.push(Diagnostic::error(
                                "E805",
                                span,
                                format!(
                                    "`segmented_annulus` is `(outer, inner, gap)` — {} value{} given",
                                    vals.len(),
                                    if vals.len() == 1 { "" } else { "s" }
                                ),
                            ));
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        def.paste = Some((
                            PadPaste::SegmentedAnnulus(Box::new([
                                outer.clone(),
                                inner.clone(),
                                gap.clone(),
                            ])),
                            field.span.to(span),
                        ));
                    } else {
                        let Some((vals, span)) = self.length_tuple() else {
                            self.sync_in_block();
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        let [w, h] = vals.as_slice() else {
                            self.diags.push(Diagnostic::error(
                                "E805",
                                span,
                                format!(
                                    "a centered paste aperture is `paste: (width, height)` — {} value{} given",
                                    vals.len(),
                                    if vals.len() == 1 { "" } else { "s" }
                                ),
                            ));
                            self.eat(&TokenKind::Comma);
                            continue;
                        };
                        def.paste =
                            Some((PadPaste::Rect(w.clone(), h.clone()), field.span.to(span)));
                    }
                }
                other => {
                    self.diags.push(Diagnostic::error(
                        "E805",
                        field.span,
                        format!(
                            "unknown pad field `{}` (expected `shape`, `size`, `layer`, `plating`, `drill`, `chamfer`, `corner_radius`, `mask_expansion`, or `paste`)",
                            other
                        ),
                    ));
                    self.sync_in_block();
                    self.eat(&TokenKind::Comma);
                    continue;
                }
            }
            self.eat(&TokenKind::Comma);
            if self.pos == before {
                self.bump();
            }
        }
        if !unclosed {
            self.expect(&TokenKind::RBrace, "to close the pad body");
        }
        def.span = start.to(self.prev_span());
        Some(def)
    }

    /// Skip a brace-balanced body whose `{` was already consumed. An EOF
    /// before the matching `}` reports an unclosed body anchored at
    /// `opened_at` (never at whatever declaration happens to follow).
    fn skip_braced_body(&mut self, opened_at: Span) {
        let mut depth = 1usize;
        loop {
            match self.peek() {
                TokenKind::Eof => {
                    self.diags.push(Diagnostic::error(
                        "E010",
                        opened_at,
                        "unclosed body — missing `}` before end of file".to_string(),
                    ));
                    return;
                }
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.bump();
                    if depth == 0 {
                        return;
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Panic-mode recovery for a broken `use`: skip to its `;` (consumed) or
    /// the next top-level synchronization point.
    fn sync_use(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Semi => {
                    self.bump();
                    return;
                }
                TokenKind::Eof
                | TokenKind::Pub
                | TokenKind::Trait
                | TokenKind::Device
                | TokenKind::Impl
                | TokenKind::Fn
                | TokenKind::Part
                | TokenKind::Design
                | TokenKind::Hash => return,
                TokenKind::Ident(n) if n == "footprint" || n == "pad" => return,
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// RFC-017: split every `#[doc("...")]` out of `attrs` — multiple are
    /// legitimate (datasheet, app note, errata), each exactly one string.
    fn take_docs(&mut self, attrs: Vec<Attr>) -> (Vec<(String, Span)>, Vec<Attr>) {
        let mut docs = Vec::new();
        let mut rest = Vec::new();
        for a in attrs {
            if a.name.name != "doc" {
                rest.push(a);
                continue;
            }
            if a.args.len() != 1 {
                self.diags.push(Diagnostic::error(
                    "E010",
                    a.span,
                    "`#[doc(…)]` takes exactly one string per attribute — use several `#[doc]`s for several documents"
                        .to_string(),
                ));
                continue;
            }
            // RFC-017: a doc path is PACKAGE-RELATIVE (review R5-9). The
            // compiler never opens the file (existence is a deferred lint),
            // but the relative-path invariant is enforced lexically: reject
            // an absolute path, a parent-directory escape, an empty path, or
            // a URL, so a library never claims a document outside its own
            // package root.
            let path = &a.args[0].0;
            // Canonical package-relative path grammar (review R6-6/R7-5): the
            // ONLY separator is `/`; every component must be non-empty and not
            // `.`/`..`; the FIRST component must not carry a URI scheme or
            // Windows drive (`file:`, `mailto:`, `C:`). Splitting on `/` and
            // validating each component catches leading `./`, `docs//x`,
            // trailing `docs/`, and `./file:/…` (which a substring check
            // missed by normalizing away the `./` first).
            let components: Vec<&str> = path.split('/').collect();
            let bad = if path.trim().is_empty() {
                Some("an empty path")
            } else if path.contains('\\') {
                Some("a `\\` backslash (not a canonical path separator)")
            } else if path.starts_with('/') {
                Some("an absolute path")
            } else if components.iter().any(|c| c.is_empty()) {
                Some("an empty path component (leading, trailing, or doubled `/`)")
            } else if components.iter().any(|c| *c == "." || *c == "..") {
                Some("a `.`/`..` component (not a canonical relative path)")
            } else if components[0].contains(':') {
                Some("a URI scheme or drive root")
            } else {
                None
            };
            if let Some(why) = bad {
                self.diags.push(Diagnostic::error(
                    "E010",
                    a.span,
                    format!(
                        "`#[doc(\"{}\")]` is not a package-relative path ({}) — doc paths resolve under the library's own root (RFC-017)",
                        path, why
                    ),
                ));
                continue;
            }
            docs.push((a.args[0].0.clone(), a.span));
        }
        (docs, rest)
    }

    /// Split a single-string opaque attribute (`#[NAME("...")]`) out of `attrs`,
    /// validating exactly one string argument and at most one occurrence. Used
    /// for RFC-012 `#[intent]` and RFC-013 `#[placement_hint]` — both metadata,
    /// never compiled.
    fn take_string_attr(
        &mut self,
        name: &str,
        attrs: Vec<Attr>,
    ) -> (Option<(String, Span)>, Vec<Attr>) {
        let mut value = None;
        let mut rest = Vec::new();
        for a in attrs {
            if a.name.name != name {
                rest.push(a);
                continue;
            }
            if a.args.len() != 1 {
                self.diags.push(Diagnostic::error(
                    "E010",
                    a.span,
                    format!(
                        "`#[{}(…)]` takes exactly one string, e.g. `#[{}(\"…\")]`",
                        name, name
                    ),
                ));
                continue;
            }
            if value.is_some() {
                self.diags.push(Diagnostic::error(
                    "E010",
                    a.span,
                    format!(
                        "at most one `#[{}(…)]` per declaration — use one string, or a `//` comment for more",
                        name
                    ),
                ));
                continue;
            }
            value = Some((a.args[0].0.clone(), a.span));
        }
        (value, rest)
    }

    fn attrs(&mut self) -> (Vec<Attr>, Vec<PhysAttr>) {
        let mut attrs = Vec::new();
        let mut phys = Vec::new();
        while self.at(&TokenKind::Hash) {
            let start = self.span();
            self.bump(); // #
            if !self.expect(&TokenKind::LBracket, "after `#`") {
                break;
            }
            let Some(name) = self.ident("as the attribute name") else {
                break;
            };
            // RFC-027: the seven physics-constraint attributes carry real,
            // structured argument grammars — parsed here, never as opaque
            // strings. They share only the bracket SYNTAX with generic attrs.
            if matches!(
                name.name.as_str(),
                "ground"
                    | "high_current"
                    | "impedance"
                    | "bypass"
                    | "crystal_oscillator"
                    | "switching_converter"
                    | "bga_fanout"
            ) {
                if let Some(pa) = self.phys_attr(&name, start) {
                    self.expect(&TokenKind::RBracket, "to close the attribute");
                    phys.push(pa);
                }
                continue;
            }
            let mut args = Vec::new();
            if self.eat(&TokenKind::LParen) {
                loop {
                    match self.peek() {
                        TokenKind::Str(_) => {
                            let t = self.bump();
                            let TokenKind::Str(s) = t.kind else {
                                unreachable!()
                            };
                            args.push((s, t.span));
                        }
                        _ => {
                            self.error_here(format!(
                                "expected a string literal in attribute arguments, found {}",
                                self.peek().describe()
                            ));
                            break;
                        }
                    }
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, "to close attribute arguments");
            }
            self.expect(&TokenKind::RBracket, "to close the attribute");
            attrs.push(Attr {
                name,
                args,
                span: start.to(self.prev_span()),
            });
        }
        (attrs, phys)
    }

    /// RFC-027: physics attributes are net/inst-only — reject elsewhere.
    fn reject_phys(&mut self, phys: &[PhysAttr], what: &str) {
        for pa in phys {
            self.diags.push(Diagnostic::error(
                "E1009",
                pa.span(),
                format!("`#[{}]` cannot be attached to {}", pa.name(), what),
            ));
        }
    }

    /// RFC-027: keep the phys attributes matching this statement kind
    /// (`net_target` true = net declaration), rejecting wrong-target ones and
    /// duplicates of the same kind ("at most one of each kind", like intent).
    fn split_phys(&mut self, phys: Vec<PhysAttr>, net_target: bool) -> Vec<PhysAttr> {
        let mut kept: Vec<PhysAttr> = Vec::new();
        for pa in phys {
            if pa.is_net_attr() != net_target {
                self.diags.push(Diagnostic::error(
                    "E1009",
                    pa.span(),
                    format!(
                        "`#[{}]` belongs on {} declaration, not {} one",
                        pa.name(),
                        if pa.is_net_attr() {
                            "a `net`"
                        } else {
                            "an `inst`"
                        },
                        if net_target { "a `net`" } else { "an `inst`" },
                    ),
                ));
                continue;
            }
            if let Some(prev) = kept.iter().find(|k| k.name() == pa.name()) {
                let d = Diagnostic::error(
                    "E1009",
                    pa.span(),
                    format!("duplicate `#[{}]` on one declaration", pa.name()),
                )
                .with_secondary(prev.span(), "first written here".to_string());
                self.diags.push(d);
                continue;
            }
            kept.push(pa);
        }
        kept
    }

    /// RFC-027: one physics-constraint attribute's own argument grammar.
    /// The caller has consumed `#[NAME`; this parses through the closing `)`
    /// (the caller closes the `]`). Unit TYPES are checked here (E110 names
    /// expected vs actual, RFC-001/011); reference EXISTENCE is expansion's
    /// job (E1009), since the referenced instance may be declared later.
    fn phys_attr(&mut self, name: &Ident, start: Span) -> Option<PhysAttr> {
        use crate::units::UnitType;
        let unit_arg = |p: &mut Self, expected: UnitType, what: &str| -> Option<UnitValue> {
            let v = p.unit_literal(what)?;
            if v.unit != expected {
                p.diags.push(Diagnostic::error(
                    "E110",
                    name.span,
                    format!(
                        "`#[{}]` {} is a `{}` value — `{}` is a `{}`",
                        name.name,
                        what,
                        expected.type_name(),
                        v.text,
                        v.unit.type_name()
                    ),
                ));
                return None;
            }
            Some(v)
        };
        match name.name.as_str() {
            "bga_fanout" => {
                // Bare — no argument list at all.
                if self.at(&TokenKind::LParen) {
                    self.diags.push(Diagnostic::error(
                        "E1009",
                        name.span,
                        "`#[bga_fanout]` takes no arguments".to_string(),
                    ));
                    return None;
                }
                Some(PhysAttr::BgaFanout {
                    span: start.to(self.prev_span()),
                })
            }
            "ground" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let v = self.ident("as the ground kind (`primary` or `secondary`)")?;
                let primary = match v.name.as_str() {
                    "primary" => true,
                    "secondary" => false,
                    other => {
                        self.diags.push(Diagnostic::error(
                            "E1009",
                            v.span,
                            format!(
                                "`{}` is not a ground kind — kinds are: primary, secondary",
                                other
                            ),
                        ));
                        return None;
                    }
                };
                let mut region_pour = false;
                if self.eat(&TokenKind::Comma) {
                    let f = self.ident("as the flag (`region_pour`)")?;
                    if f.name != "region_pour" {
                        self.diags.push(Diagnostic::error(
                            "E1009",
                            f.span,
                            format!(
                                "`{}` is not a `#[ground]` flag — the only flag is `region_pour`",
                                f.name
                            ),
                        ));
                        return None;
                    }
                    region_pour = true;
                }
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                Some(PhysAttr::Ground {
                    primary,
                    region_pour,
                    span: start.to(self.prev_span()),
                })
            }
            "high_current" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let current = unit_arg(self, UnitType::Current, "current")?;
                let mut power_pour = false;
                if self.eat(&TokenKind::Comma) {
                    let f = self.ident("as the flag (`power_pour`)")?;
                    if f.name != "power_pour" {
                        self.diags.push(Diagnostic::error(
                            "E1009",
                            f.span,
                            format!("`{}` is not a `#[high_current]` flag — the only flag is `power_pour`", f.name),
                        ));
                        return None;
                    }
                    power_pour = true;
                }
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                Some(PhysAttr::HighCurrent {
                    current,
                    power_pour,
                    span: start.to(self.prev_span()),
                })
            }
            "impedance" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let impedance = unit_arg(self, UnitType::Resistance, "impedance")?;
                self.expect(&TokenKind::Comma, "before `frequency:`");
                let k = self.ident("as the named argument `frequency`")?;
                if k.name != "frequency" {
                    self.diags.push(Diagnostic::error(
                        "E1009",
                        k.span,
                        format!(
                            "`{}` is not an `#[impedance]` argument — expected `frequency:`",
                            k.name
                        ),
                    ));
                    return None;
                }
                self.expect(&TokenKind::Colon, "after `frequency`");
                let frequency = unit_arg(self, UnitType::Frequency, "frequency")?;
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                Some(PhysAttr::Impedance {
                    impedance,
                    frequency,
                    span: start.to(self.prev_span()),
                })
            }
            "bypass" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let inst = self.ident("as the bypassed target")?;
                // RFC-024: an array element is an instance reference like any
                // other, so `NAME[i].PIN` is legal here too. Only a single
                // index — a range would name several targets for one cap.
                let index = if self.at(&TokenKind::LBracket) {
                    match self.index_sel()? {
                        IndexSel::Single(i, sp) => Some((i, sp)),
                        other => {
                            self.diags.push(Diagnostic::error(
                                "E211",
                                other.span(),
                                "`#[bypass]` takes a single element `NAME[i]` — a range or index list names more than one target".to_string(),
                            ));
                            return None;
                        }
                    }
                } else {
                    None
                };
                // RFC-028: `.PIN` is optional — a bare identifier is a
                // `Pin`-typed fn parameter, the same bare-PinRef form already
                // legal in net member lists.
                let pin = if self.eat(&TokenKind::Dot) {
                    Some(self.ident("as the bypassed pin")?)
                } else {
                    None
                };
                self.expect(&TokenKind::Comma, "before the capacitance");
                let capacitance = unit_arg(self, UnitType::Capacitance, "capacitance")?;
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                Some(PhysAttr::Bypass {
                    inst,
                    index,
                    pin,
                    capacitance,
                    span: start.to(self.prev_span()),
                })
            }
            "crystal_oscillator" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let parent = self.ident("as the parent instance")?;
                self.expect(&TokenKind::Comma, "between the arguments");
                let pin1 = self.ident("as the first parent pin")?;
                self.expect(&TokenKind::Comma, "between the arguments");
                let pin2 = self.ident("as the second parent pin")?;
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                Some(PhysAttr::CrystalOscillator {
                    parent,
                    pin1,
                    pin2,
                    span: start.to(self.prev_span()),
                })
            }
            "switching_converter" => {
                self.expect(&TokenKind::LParen, "to open the attribute arguments");
                let mut inductor = None;
                let mut input_capacitor = None;
                let mut output_capacitor = None;
                loop {
                    let k = self.ident(
                        "as a named argument (`inductor`, `input_capacitor`, `output_capacitor`)",
                    )?;
                    self.expect(&TokenKind::Colon, "after the argument name");
                    let v = self.ident("as an instance name")?;
                    match k.name.as_str() {
                        "inductor" => inductor = Some(v),
                        "input_capacitor" => input_capacitor = Some(v),
                        "output_capacitor" => output_capacitor = Some(v),
                        other => {
                            self.diags.push(Diagnostic::error(
                                "E1009",
                                k.span,
                                format!(
                                    "`{}` is not a `#[switching_converter]` argument — arguments are: inductor, input_capacitor, output_capacitor",
                                    other
                                ),
                            ));
                            return None;
                        }
                    }
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, "to close the attribute arguments");
                let Some(inductor) = inductor else {
                    self.diags.push(Diagnostic::error(
                        "E1009",
                        name.span,
                        "`#[switching_converter]` requires the `inductor:` argument".to_string(),
                    ));
                    return None;
                };
                Some(PhysAttr::SwitchingConverter {
                    inductor,
                    input_capacitor,
                    output_capacitor,
                    span: start.to(self.prev_span()),
                })
            }
            _ => unreachable!("caller matched the closed name set"),
        }
    }

    // -- traits --------------------------------------------------------------

    fn trait_def(&mut self) -> Option<TraitDef> {
        self.bump(); // trait
        let name = self.ident("as the trait name")?;
        let mut super_traits = Vec::new();
        if self.eat(&TokenKind::Colon) {
            loop {
                super_traits.push(self.path_ident("as a sub-trait bound")?);
                if !self.eat(&TokenKind::Plus) {
                    break;
                }
            }
        }
        if !self.expect(&TokenKind::LBrace, "to open the trait body") {
            self.sync_top_level();
            return None;
        }
        let mut def = TraitDef {
            name,
            super_traits,
            designator_prefix: None,
            pins: Vec::new(),
            specs: Vec::new(),
            pins_span: None,
            spec_span: None,
        };
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Pins) {
                let block_start = self.span();
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the pins block");
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(pin) = self.trait_pin() {
                        def.pins.push(pin);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the pins block");
                let s = block_start.to(self.prev_span());
                def.pins_span.get_or_insert(s);
            } else if self.at(&TokenKind::Spec) {
                let block_start = self.span();
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the spec block");
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(field) = self.trait_spec_field() {
                        def.specs.push(field);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the spec block");
                let s = block_start.to(self.prev_span());
                def.spec_span.get_or_insert(s);
            } else if self.at_ident("designator_prefix") {
                self.bump();
                self.expect(&TokenKind::Colon, "after `designator_prefix`");
                match self.peek() {
                    TokenKind::Str(_) => {
                        let t = self.bump();
                        let TokenKind::Str(s) = t.kind else {
                            unreachable!()
                        };
                        def.designator_prefix = Some((s, t.span));
                    }
                    _ => self.error_here(format!(
                        "expected a string like \"C\" after `designator_prefix:`, found {}",
                        self.peek().describe()
                    )),
                }
            } else {
                self.error_here(format!(
                    "expected `pins`, `spec`, or `designator_prefix` in the trait body, found {}",
                    self.peek().describe()
                ));
                self.sync_in_block();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the trait body");
        Some(def)
    }

    fn trait_pin(&mut self) -> Option<TraitPin> {
        let start = self.span();
        let obligation = self.obligation();
        let name = self.ident("as the pin role name")?;
        self.expect(&TokenKind::Colon, "after the pin role name");
        if self.at_ident("pin") {
            self.bump();
        } else {
            self.error_here(format!(
                "expected `pin` as the trait pin type (trait pins are abstract roles), found {}",
                self.peek().describe()
            ));
            return None;
        }
        Some(TraitPin {
            obligation,
            name,
            span: start.to(self.prev_span()),
        })
    }

    fn trait_spec_field(&mut self) -> Option<TraitSpecField> {
        let start = self.span();
        let name = self.ident("as the spec field name")?;
        self.expect(&TokenKind::Colon, "after the spec field name");
        let ty = self.unit_type_ref()?;
        Some(TraitSpecField {
            name,
            ty,
            span: start.to(self.prev_span()),
        })
    }

    fn unit_type_ref(&mut self) -> Option<UnitTypeRef> {
        let ident = self.ident("as a unit type")?;
        match UnitType::from_type_name(&ident.name) {
            Some(unit) => Some(UnitTypeRef {
                unit,
                span: ident.span,
            }),
            None => {
                self.diags.push(
                    Diagnostic::error(
                        "E010",
                        ident.span,
                        format!("`{}` is not a unit type", ident.name),
                    )
                    .with_help(
                        "the eleven unit types are: Voltage, Capacitance, Resistance, Current, \
                         Frequency, Time, Inductance, Power, Temperature, Tolerance, Length",
                    ),
                );
                None
            }
        }
    }

    fn obligation(&mut self) -> Obligation {
        if self.eat(&TokenKind::Required) {
            Obligation::Required
        } else if self.eat(&TokenKind::Optional) {
            Obligation::Optional
        } else {
            // Omitted obligation defaults to required (note 10's MLCC example
            // writes `pins { A: 1, B: 2 }` with no keyword).
            Obligation::Required
        }
    }

    // -- devices -------------------------------------------------------------

    fn device_def(&mut self) -> Option<DeviceDef> {
        self.bump(); // device
        let name = self.ident("as the device name")?;
        let generics = if self.at(&TokenKind::Lt) {
            self.generic_params()
        } else {
            Vec::new()
        };
        if self.at(&TokenKind::Colon) {
            // v1-era `device X: impl Trait` — superseded by RFC-003.
            self.diags.push(
                Diagnostic::error(
                    "E010",
                    self.span(),
                    "a `device` declaration never has a trait clause — devices are pins + specs only",
                )
                .with_help(format!(
                    "write a free-standing `impl Trait for {} {{}}` statement instead (RFC-003)",
                    name.name
                )),
            );
            // Skip whatever follows the colon up to the opening brace.
            while !self.at(&TokenKind::LBrace) && !self.at(&TokenKind::Eof) {
                self.bump();
            }
        }
        if !self.expect(&TokenKind::LBrace, "to open the device body") {
            self.sync_top_level();
            return None;
        }
        let mut def = DeviceDef {
            name,
            generics,
            variants: Vec::new(),
            variants_span: None,
            pin_blocks: Vec::new(),
            spec_blocks: Vec::new(),
        };
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.at_ident("variants") {
                let variants_start = self.span();
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the variants block");
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    let Some(v) = self.ident("as a variant name") else {
                        self.sync_in_block_advancing();
                        continue;
                    };
                    // RFC-008 rejects wildcard/default arms outright.
                    if v.name == "_" {
                        self.diags.push(Diagnostic::error(
                            "E010",
                            v.span,
                            "`_` is not a valid variant name — every variant is named explicitly, no wildcard/catch-all arms (RFC-008)",
                        ));
                        self.eat(&TokenKind::Comma);
                        continue;
                    }
                    // RFC-008: the closed set is duplicate-checked at parse.
                    if let Some(prev) = def.variants.iter().find(|x| x.name == v.name) {
                        self.diags.push(
                            Diagnostic::error(
                                "E906",
                                v.span,
                                format!("duplicate variant `{}` in `variants {{ }}`", v.name),
                            )
                            .with_secondary(prev.span, "first declared here"),
                        );
                    } else {
                        def.variants.push(v);
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the variants block");
                def.variants_span = Some(variants_start.to(self.prev_span()));
            } else if self.at(&TokenKind::Pins) {
                let block_start = self.span();
                self.bump();
                let variant = self.block_variant_qualifier();
                self.expect(&TokenKind::LBrace, "to open the pins block");
                let mut pins = Vec::new();
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(pin) = self.device_pin() {
                        pins.push(pin);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the pins block");
                def.pin_blocks.push(PinBlock {
                    variant,
                    pins,
                    span: block_start.to(self.prev_span()),
                });
            } else if self.at(&TokenKind::Spec) {
                let block_start = self.span();
                self.bump();
                let variant = self.block_variant_qualifier();
                self.expect(&TokenKind::LBrace, "to open the spec block");
                let mut fields = Vec::new();
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(field) = self.device_spec_field() {
                        fields.push(field);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the spec block");
                def.spec_blocks.push(SpecBlock {
                    variant,
                    fields,
                    span: block_start.to(self.prev_span()),
                });
            } else {
                self.error_here(format!(
                    "expected `pins`, `spec`, or `variants` in the device body, found {}",
                    self.peek().describe()
                ));
                self.sync_in_block();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the device body");
        Some(def)
    }

    /// The optional `[VARIANT]` qualifier on a `pins`/`spec` block (RFC-008).
    fn block_variant_qualifier(&mut self) -> Option<Ident> {
        if !self.at(&TokenKind::LBracket) {
            return None;
        }
        self.bump();
        let v = self.ident("as the variant qualifier");
        self.expect(&TokenKind::RBracket, "to close the variant qualifier");
        v
    }

    /// One device pin entry: `[required|optional] NAME: 1, 2, 3 [role]`.
    ///
    /// Comma handling needs 2-token lookahead: after a comma, a `Number` (or
    /// an identifier NOT followed by `:`) continues the current pin-number
    /// list; an identifier followed by `:` (or `required`/`optional`) starts
    /// the next entry.
    fn device_pin(&mut self) -> Option<DevicePin> {
        let start = self.span();
        let obligation = self.obligation();
        let name = self.ident("as the pin name")?;
        self.expect(&TokenKind::Colon, "after the pin name");
        let mut numbers = Vec::new();
        loop {
            match self.peek() {
                TokenKind::Number(_) => {
                    let t = self.bump();
                    let TokenKind::Number(text) = t.kind else {
                        unreachable!()
                    };
                    numbers.push(PinNumber { text, span: t.span });
                }
                TokenKind::Ident(n) if is_pad_name(n) => {
                    let t = self.bump();
                    let TokenKind::Ident(text) = t.kind else {
                        unreachable!()
                    };
                    numbers.push(PinNumber { text, span: t.span });
                }
                other => {
                    self.error_here(format!(
                        "expected a physical pin number (e.g. `1` or `A3`), found {}",
                        other.describe()
                    ));
                    return None;
                }
            }
            // Continue the number list only if the comma is followed by
            // another number (not the next pin entry).
            if self.at(&TokenKind::Comma) {
                let next = self.peek_ahead(1).clone();
                let continues = match &next {
                    TokenKind::Number(_) => true,
                    TokenKind::Ident(n) if is_pad_name(n) => {
                        self.peek_ahead(2) != &TokenKind::Colon
                    }
                    _ => false,
                };
                if continues {
                    self.bump(); // comma
                    continue;
                }
            }
            break;
        }
        let mut role = None;
        if self.at(&TokenKind::LBracket) {
            self.bump();
            let role_ident = self.ident("as the pin role")?;
            match PinRole::from_name(&role_ident.name) {
                Some(r) => role = Some((r, role_ident.span)),
                None => {
                    self.diags.push(
                        Diagnostic::error(
                            "E010",
                            role_ident.span,
                            format!("`{}` is not a pin role", role_ident.name),
                        )
                        .with_help(
                            "pin roles are: input, output, bidirectional, passive, power_in, power_out",
                        ),
                    );
                }
            }
            self.expect(&TokenKind::RBracket, "to close the pin role");
        } else {
            // RFC-008: every device pin carries an explicit role — the
            // implicit `passive` default is retired.
            self.diags.push(
                Diagnostic::error(
                    "E901",
                    name.span,
                    format!(
                        "pin `{}` has no role annotation — every device pin needs an explicit role (RFC-008)",
                        name.name
                    ),
                )
                .with_help(
                    "annotate with one of the six roles: [input], [output], [bidirectional], [passive], [power_in], [power_out]",
                ),
            );
        }
        Some(DevicePin {
            obligation,
            name,
            numbers,
            role,
            span: start.to(self.prev_span()),
        })
    }

    fn device_spec_field(&mut self) -> Option<DeviceSpecField> {
        let start = self.span();
        let name = self.ident("as the spec field name")?;
        self.expect(&TokenKind::Colon, "after the spec field name");
        let value = match self.peek() {
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                SpecValue::Lit(v, t.span)
            }
            TokenKind::Ident(_) => {
                let ident = self.ident("")?;
                SpecValue::GenericRef(ident)
            }
            TokenKind::Number(_) => {
                let t = self.bump();
                self.diags.push(
                    Diagnostic::error(
                        "E111",
                        t.span,
                        "a bare number is never valid where a unit-typed value is expected",
                    )
                    .with_help(
                        "write the value with its unit, e.g. `100nF`, `10V`, `1%` (RFC-001: no defaults, no coercion)",
                    ),
                );
                return None;
            }
            other => {
                self.error_here(format!(
                    "expected a unit literal (e.g. `100nF`) or a generic parameter name, found {}",
                    other.describe()
                ));
                return None;
            }
        };
        Some(DeviceSpecField {
            name,
            value,
            span: start.to(self.prev_span()),
        })
    }

    // -- generics ------------------------------------------------------------

    fn generic_params(&mut self) -> Vec<GenericParam> {
        self.bump(); // <
        let mut params = Vec::new();
        while !self.at(&TokenKind::Gt) && !self.at(&TokenKind::Eof) {
            let start = self.span();
            let Some(name) = self.ident("as the generic parameter name") else {
                break;
            };
            if !self.expect(&TokenKind::Colon, "after the generic parameter name") {
                break;
            }
            let Some(first) = self.path_ident("as the generic bound") else {
                break;
            };
            let bound = if let Some(unit) = UnitType::from_type_name(&first.name) {
                GenericBound::Unit(UnitTypeRef {
                    unit,
                    span: first.span,
                })
            } else {
                let mut traits = vec![first];
                while self.eat(&TokenKind::Plus) {
                    match self.path_ident("as a trait bound") {
                        Some(t) => traits.push(t),
                        None => break,
                    }
                }
                GenericBound::Traits(traits)
            };
            let mut default = None;
            if self.eat(&TokenKind::Eq) {
                match self.peek() {
                    TokenKind::Unit(_) => {
                        let t = self.bump();
                        let TokenKind::Unit(v) = t.kind else {
                            unreachable!()
                        };
                        default = Some((v, t.span));
                    }
                    TokenKind::Number(_) => {
                        let t = self.bump();
                        self.diags.push(
                            Diagnostic::error(
                                "E111",
                                t.span,
                                "a bare number is never valid as a generic default — write it with its unit",
                            )
                            .with_help("e.g. `V: Voltage = 10V`, `T: Tolerance = 10%`"),
                        );
                    }
                    other => {
                        let msg = format!(
                            "expected a unit literal as the generic default, found {}",
                            other.describe()
                        );
                        self.error_here(msg);
                    }
                }
            }
            params.push(GenericParam {
                span: start.to(self.prev_span()),
                name,
                bound,
                default,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::Gt, "to close the generic parameter list");
        params
    }

    fn generic_args(&mut self) -> Vec<GenericArg> {
        self.bump(); // <
        let mut args = Vec::new();
        while !self.at(&TokenKind::Gt) && !self.at(&TokenKind::Eof) {
            match self.peek() {
                TokenKind::Unit(_) => {
                    let t = self.bump();
                    let TokenKind::Unit(v) = t.kind else {
                        unreachable!()
                    };
                    args.push(GenericArg::Unit(v, t.span));
                }
                TokenKind::Number(_) => {
                    let t = self.bump();
                    let TokenKind::Number(n) = t.kind else {
                        unreachable!()
                    };
                    args.push(GenericArg::Number(n, t.span));
                }
                TokenKind::Ident(_) => {
                    let ident = self.path_ident("").unwrap();
                    args.push(GenericArg::Name(ident));
                }
                other => {
                    let msg = format!(
                        "expected a generic argument (unit literal or name), found {}",
                        other.describe()
                    );
                    self.error_here(msg);
                    break;
                }
            }
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::Gt, "to close the generic argument list");
        args
    }

    fn type_ref(&mut self) -> Option<TypeRef> {
        let name = self.path_ident("as a type name")?;
        let start = name.span;
        let generic_args = if self.at(&TokenKind::Lt) {
            self.generic_args()
        } else {
            Vec::new()
        };
        // RFC-008 `[VARIANT]` selector: `MLCC<100nF, 16V>[C0603]`.
        let variant = if self.at(&TokenKind::LBracket) {
            self.bump();
            let v = self.ident("as the variant selector");
            self.expect(&TokenKind::RBracket, "to close the variant selector");
            v
        } else {
            None
        };
        Some(TypeRef {
            name,
            generic_args,
            variant,
            span: start.to(self.prev_span()),
        })
    }

    // -- impls ---------------------------------------------------------------

    fn impl_def(&mut self) -> Option<ImplDef> {
        let start = self.span();
        self.bump(); // impl
        let trait_name = self.path_ident("as the trait name")?;
        self.expect(&TokenKind::For, "between the trait and device names");
        let device_name = self.path_ident("as the device name")?;
        if !self.expect(&TokenKind::LBrace, "to open the impl body") {
            self.sync_top_level();
            return None;
        }
        let mut pin_map = Vec::new();
        let mut spec_map = Vec::new();
        let mut pins_span: Option<Span> = None;
        let mut spec_span: Option<Span> = None;
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let block_start = self.span();
            let is_pins = self.at(&TokenKind::Pins);
            let target = if self.at(&TokenKind::Pins) {
                self.bump();
                &mut pin_map
            } else if self.at(&TokenKind::Spec) {
                self.bump();
                &mut spec_map
            } else {
                self.error_here(format!(
                    "an impl body only contains explicit `pins`/`spec` mappings (empty when names match), found {}",
                    self.peek().describe()
                ));
                self.sync_in_block_advancing();
                continue;
            };
            self.expect(&TokenKind::LBrace, "to open the mapping block");
            while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                let Some(role) = self.ident("as the trait's required name") else {
                    self.sync_in_block();
                    break;
                };
                self.expect(&TokenKind::Colon, "in the mapping");
                let Some(map_target) = self.ident("as the device's own name") else {
                    self.sync_in_block();
                    break;
                };
                let span = role.span.to(map_target.span);
                target.push(MapEntry {
                    role,
                    target: map_target,
                    span,
                });
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace, "to close the mapping block");
            let s = block_start.to(self.prev_span());
            if is_pins {
                pins_span.get_or_insert(s);
            } else {
                spec_span.get_or_insert(s);
            }
        }
        self.expect(&TokenKind::RBrace, "to close the impl body");
        Some(ImplDef {
            trait_name,
            device_name,
            pin_map,
            spec_map,
            pins_span,
            spec_span,
            span: start.to(self.prev_span()),
        })
    }

    // -- fns -----------------------------------------------------------------

    fn fn_def(&mut self) -> Option<FnDef> {
        self.bump(); // fn
        let name = self.ident("as the fn name")?;
        let generics = if self.at(&TokenKind::Lt) {
            self.generic_params()
        } else {
            Vec::new()
        };
        self.expect(&TokenKind::LParen, "to open the parameter list");
        let mut params = Vec::new();
        while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
            let start = self.span();
            let Some(pname) = self.ident("as the parameter name") else {
                break;
            };
            if !self.expect(&TokenKind::Colon, "after the parameter name") {
                break;
            }
            let ty = if self.at(&TokenKind::Impl) {
                let impl_start = self.span();
                self.bump();
                let mut traits = Vec::new();
                while let Some(t) = self.path_ident("as a trait bound after `impl`") {
                    traits.push(t);
                    if !self.eat(&TokenKind::Plus) {
                        break;
                    }
                }
                FnParamTy::ImplTrait(traits, impl_start.to(self.prev_span()))
            } else {
                match self.ident("as the parameter type") {
                    Some(t) if t.name == "Pin" => FnParamTy::Pin(t.span),
                    Some(t) => FnParamTy::Generic(t),
                    None => break,
                }
            };
            params.push(FnParam {
                span: start.to(self.prev_span()),
                name: pname,
                ty,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen, "to close the parameter list");
        if !self.expect(&TokenKind::LBrace, "to open the fn body") {
            self.sync_top_level();
            return None;
        }
        let body = self.stmt_block();
        self.expect(&TokenKind::RBrace, "to close the fn body");
        Some(FnDef {
            name,
            generics,
            params,
            body,
        })
    }

    // -- parts ---------------------------------------------------------------

    fn part_def(&mut self) -> Option<PartDef> {
        let start = self.span();
        self.bump(); // part
        let name = self.ident("as the part name")?;
        self.expect(&TokenKind::Colon, "after the part name");
        let device = self.type_ref()?;
        self.expect(&TokenKind::LBrace, "to open the part body");
        let mut primary: Option<AvlEntry> = None;
        let mut alts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let is_primary = if self.at_ident("primary") {
                true
            } else if self.at_ident("alt") {
                false
            } else {
                self.error_here(format!(
                    "expected `primary` or `alt` in the part body, found {}",
                    self.peek().describe()
                ));
                self.sync_in_block_advancing();
                continue;
            };
            let entry_start = self.span();
            self.bump();
            self.expect(&TokenKind::LBrace, "to open the AVL entry");
            let mut fields = Vec::new();
            let mut footprint: Option<Ident> = None;
            while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                let Some(fname) = self.ident("as the AVL field name (e.g. `mpn`)") else {
                    self.sync_in_block();
                    break;
                };
                self.expect(&TokenKind::Colon, "after the AVL field name");
                // RFC-017: `footprint:` takes a SYMBOL reference (resolved
                // via RFC-016) — never a string.
                if fname.name == "footprint" {
                    match self.peek() {
                        TokenKind::Ident(_) => {
                            if let Some(sym) = self.path_ident("as the footprint symbol") {
                                if let Some(prev) = &footprint {
                                    self.diags.push(
                                        Diagnostic::error(
                                            "E802",
                                            sym.span,
                                            "duplicate `footprint` in one AVL entry".to_string(),
                                        )
                                        .with_secondary(prev.span, "first given here".to_string()),
                                    );
                                } else {
                                    footprint = Some(sym);
                                }
                            }
                        }
                        TokenKind::Str(_) => {
                            let t = self.bump();
                            self.diags.push(
                                Diagnostic::error(
                                    "E010",
                                    t.span,
                                    "`footprint:` now references a footprint SYMBOL (RFC-017), not a string"
                                        .to_string(),
                                )
                                .with_help(
                                    "declare `pub footprint SomeName {}` and write `footprint: SomeName` (or a qualified path)",
                                ),
                            );
                        }
                        other => {
                            let msg = format!(
                                "expected a footprint symbol after `footprint:`, found {}",
                                other.describe()
                            );
                            self.error_here(msg);
                        }
                    }
                    self.eat(&TokenKind::Comma);
                    continue;
                }
                match self.peek() {
                    TokenKind::Str(_) => {
                        let t = self.bump();
                        let TokenKind::Str(s) = t.kind else {
                            unreachable!()
                        };
                        let span = fname.span.to(t.span);
                        fields.push(AvlField {
                            name: fname,
                            value: s,
                            span,
                        });
                    }
                    other => {
                        let msg = format!(
                            "expected a string value for AVL field `{}`, found {}",
                            fname.name,
                            other.describe()
                        );
                        self.error_here(msg);
                    }
                }
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace, "to close the AVL entry");
            let entry = AvlEntry {
                fields,
                footprint,
                span: entry_start.to(self.prev_span()),
            };
            if is_primary {
                if primary.is_some() {
                    self.diags.push(Diagnostic::error(
                        "E802",
                        entry.span,
                        format!("part `{}` has more than one `primary` entry", name.name),
                    ));
                } else {
                    primary = Some(entry);
                }
            } else {
                alts.push(entry);
            }
        }
        self.expect(&TokenKind::RBrace, "to close the part body");
        let span = start.to(self.prev_span());
        let Some(primary) = primary else {
            self.diags.push(
                Diagnostic::error(
                    "E802",
                    span,
                    format!("part `{}` has no `primary` entry", name.name),
                )
                .with_help(
                    "every part needs exactly one `primary { mpn: \"…\", footprint: SomeFootprint }` (RFC-017: footprint is a symbol)",
                ),
            );
            return None;
        };
        Some(PartDef {
            name,
            device,
            primary,
            alts,
            span,
        })
    }

    // -- designs & statements -------------------------------------------------

    fn design_def(&mut self) -> Option<DesignDef> {
        self.bump(); // design
        let name = self.ident("as the design name")?;
        if !self.expect(&TokenKind::LBrace, "to open the design body") {
            self.sync_top_level();
            return None;
        }
        let body = self.stmt_block();
        self.expect(&TokenKind::RBrace, "to close the design body");
        Some(DesignDef { name, body })
    }

    fn stmt_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if let Some(stmt) = self.stmt() {
                stmts.push(stmt);
            } else {
                self.sync_stmt();
            }
            if self.pos == before {
                self.bump();
            }
        }
        stmts
    }

    fn sync_stmt(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Eof
                | TokenKind::RBrace
                | TokenKind::Inst
                | TokenKind::Net
                | TokenKind::Nc
                | TokenKind::Hash => return,
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Block-level recovery that is guaranteed to advance.
    ///
    /// `sync_in_block` deliberately stops *at* the `,`/`}` it finds so the
    /// caller can decide what to do with the delimiter. A caller that loops
    /// and re-enters recovery on that same token therefore never terminates
    /// — a hang, not a slow parse, and one that appends a diagnostic per
    /// iteration until memory runs out. Three ordinary malformed shapes
    /// reached it: a bad generic argument in a `part` type (`D<1e+06ohm,
    /// 1%>`), a stray comma in a part body, and a non-string AVL value
    /// (`mfr: 7`). When recovery could not move, consume the token it
    /// stalled on; the loop condition then sees a genuinely new position.
    fn sync_in_block_advancing(&mut self) {
        let before = self.pos;
        self.sync_in_block();
        if self.pos == before {
            self.bump();
        }
    }

    fn sync_in_block(&mut self) {
        // Inside a `{ … }` block: skip to the next comma or closing brace.
        // Paren-aware — a comma inside a tuple like `(x, y)` is part of the
        // broken construct, not a synchronization point.
        let mut depth = 0usize;
        let mut paren = 0usize;
        loop {
            match self.peek() {
                TokenKind::Eof => return,
                TokenKind::Comma if depth == 0 && paren == 0 => return,
                TokenKind::RBrace if depth == 0 => return,
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.bump();
                }
                TokenKind::LParen => {
                    paren += 1;
                    self.bump();
                }
                TokenKind::RParen => {
                    paren = paren.saturating_sub(1);
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// True when the cursor can only be the start of a top-level declaration
    /// — a body loop seeing one of these has run past its own (missing)
    /// closing brace. Used to keep an unclosed pad/footprint body from
    /// swallowing the declarations that follow it.
    fn at_decl_keyword(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Pub
                | TokenKind::Trait
                | TokenKind::Device
                | TokenKind::Impl
                | TokenKind::Fn
                | TokenKind::Part
                | TokenKind::Design
                | TokenKind::Hash
        ) || self.at_ident("use")
    }

    /// Recovery inside a footprint body: skip to the next member keyword
    /// (`pad` / `courtyard` / `silkscreen_ref`), the body's closing brace, or
    /// a top-level declaration start — so one broken member never consumes
    /// the valid members (or declarations) after it. Paren/brace aware.
    fn sync_footprint_body(&mut self) {
        let mut depth = 0usize;
        let mut paren = 0usize;
        loop {
            if depth == 0
                && paren == 0
                && (self.at_ident("pad")
                    || self.at_ident("courtyard")
                    || self.at_ident("silkscreen_ref")
                    || self.at_decl_keyword())
            {
                return;
            }
            match self.peek() {
                TokenKind::Eof => return,
                TokenKind::RBrace if depth == 0 => return,
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.bump();
                }
                TokenKind::LParen => {
                    paren += 1;
                    self.bump();
                }
                TokenKind::RParen => {
                    paren = paren.saturating_sub(1);
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let (attrs, phys) = self.attrs();
        // RFC-013: a `layout { … }` block is a statement that takes no
        // attributes. Detect it before attribute handling so `#[intent]` isn't
        // silently swallowed onto a target that can't carry it.
        if matches!(self.peek(), TokenKind::Ident(n) if n == "layout")
            && self.peek_ahead(1) == &TokenKind::LBrace
        {
            self.reject_attrs(&attrs);
            self.reject_phys(&phys, "a `layout {}` block");
            return self.layout_block();
        }
        // RFC-012: split off `#[intent("...")]` (valid on any statement); the
        // remaining attributes are inst-only (`#[designator]`/`#[placement_hint]`).
        let (intent, attrs) = self.take_intent(attrs);
        match self.peek() {
            TokenKind::Inst => {
                // RFC-013: `#[placement_hint(...)]` is inst-only opaque metadata.
                let (placement_hint, attrs) = self.take_string_attr("placement_hint", attrs);
                // RFC-027: inst-target physics attributes; net-target ones are
                // rejected here, and at most one of each kind is allowed.
                let phys = self.split_phys(phys, false);
                // Attr validation happens HERE, at parse — an inst inside a
                // never-expanded fn must not silently accept garbage
                // (adversarial finding; expansion-time validation only runs
                // for reachable bodies).
                for a in &attrs {
                    if a.name.name != "designator" && a.name.name != "virtual" {
                        self.diags.push(Diagnostic::error(
                            "E010",
                            a.span,
                            format!(
                                "unrecognized attribute `{}` (an `inst` takes `#[designator(\"…\")]`, `#[virtual]`, `#[intent(\"…\")]`, or `#[placement_hint(\"…\")]`)",
                                a.name.name
                            ),
                        ));
                    }
                }
                let attrs: Vec<Attr> = attrs
                    .into_iter()
                    .filter(|a| a.name.name == "designator" || a.name.name == "virtual")
                    .collect();
                let start = self.span();
                self.bump();
                let name = self.ident("as the instance name")?;
                self.expect(&TokenKind::Colon, "after the instance name");
                // RFC-024: `[Device; N]` in TYPE position declares an
                // array-typed instance of fixed length N.
                let (ty, array_len) = if self.at(&TokenKind::LBracket) {
                    let open = self.span();
                    self.bump();
                    let ty = self.type_ref()?;
                    self.expect(
                        &TokenKind::Semi,
                        "between the element type and the array length",
                    );
                    let n = self.index_number("as the array length")?;
                    self.expect(&TokenKind::RBracket, "to close the array type");
                    let span = open.to(self.prev_span());
                    if n < 1 {
                        self.diags.push(Diagnostic::error(
                            "E211",
                            span,
                            format!("array length `{}` must be 1 or more", n),
                        ));
                        return None;
                    }
                    (ty, Some((n, span)))
                } else {
                    (self.type_ref()?, None)
                };
                Some(Stmt::Inst(InstStmt {
                    attrs,
                    intent,
                    placement_hint,
                    phys,
                    name,
                    array_len,
                    span: start.to(self.prev_span()),
                    ty,
                }))
            }
            TokenKind::Net => {
                self.reject_attrs(&attrs);
                // RFC-027: net-target physics attributes.
                let phys = self.split_phys(phys, true);
                let start = self.span();
                self.bump();
                let name_ident = self.ident("as the net name (or `_` for anonymous)")?;
                let name = if name_ident.name == "_" {
                    None
                } else {
                    Some(name_ident)
                };
                let mut annotation = None;
                if self.at(&TokenKind::LBracket) {
                    let ann_start = self.span();
                    self.bump();
                    match self.peek() {
                        TokenKind::Unit(_) => {
                            let t = self.bump();
                            let TokenKind::Unit(v) = t.kind else {
                                unreachable!()
                            };
                            if v.unit == UnitType::Voltage {
                                annotation = Some(NetAnnotation::Voltage(v, t.span));
                            } else {
                                // RFC-001 comparison discipline: the annotation
                                // participates in the D001 Voltage comparison,
                                // so a non-Voltage literal is a unit-type error.
                                self.diags.push(
                                    Diagnostic::error(
                                        "E110",
                                        t.span,
                                        format!(
                                            "net voltage annotation has the wrong unit type: expected `Voltage`, found `{}`",
                                            v.unit.type_name()
                                        ),
                                    )
                                    .with_primary_label(format!(
                                        "`{}` is a `{}`",
                                        v.text,
                                        v.unit.type_name()
                                    ))
                                    .with_help("annotate with a voltage (e.g. `[3.3V]`), or `[gnd]` for ground"),
                                );
                            }
                        }
                        TokenKind::Ident(n) if n == "gnd" => {
                            let t = self.bump();
                            annotation = Some(NetAnnotation::Gnd(t.span));
                        }
                        other => {
                            let msg = format!(
                                "expected a voltage literal (e.g. `3.3V`) or `gnd` as the net annotation, found {}",
                                other.describe()
                            );
                            self.error_here(msg);
                        }
                    }
                    self.expect(&TokenKind::RBracket, "to close the net annotation");
                    let _ = ann_start;
                }
                self.expect(&TokenKind::Colon, "after the net name");
                let members = self.pin_ref_list();
                Some(Stmt::Net(NetStmt {
                    phys,
                    name,
                    annotation,
                    members,
                    intent,
                    span: start.to(self.prev_span()),
                }))
            }
            TokenKind::Nc => {
                self.reject_phys(&phys, "an `nc` statement");
                self.reject_attrs(&attrs);
                let start = self.span();
                self.bump();
                self.expect(&TokenKind::Colon, "after `nc`");
                let members = self.pin_ref_list();
                Some(Stmt::Nc(NcStmt {
                    members,
                    intent,
                    span: start.to(self.prev_span()),
                }))
            }
            TokenKind::Ident(n)
                if n == "pad"
                    && matches!(
                        self.peek_ahead(1),
                        TokenKind::Ident(_) | TokenKind::Number(_)
                    ) =>
            {
                self.reject_attrs(&attrs);
                let span = self.span();
                self.diags.push(Diagnostic::error(
                    "E010",
                    span,
                    "`pad` lines live in `footprint { … }` bodies (placements) or at top level (declarations) — not in a design/fn body"
                        .to_string(),
                ));
                // Consume whichever form it is so the body keeps parsing.
                if matches!(self.peek_ahead(0), TokenKind::Ident(_))
                    && matches!(self.peek_ahead(1), TokenKind::Number(_))
                {
                    let _ = self.pad_place();
                } else {
                    let _ = self.pad_def();
                }
                None
            }
            TokenKind::Ident(n)
                if n == "footprint" && matches!(self.peek_ahead(1), TokenKind::Ident(_)) =>
            {
                self.reject_attrs(&attrs);
                let span = self.span();
                self.diags.push(Diagnostic::error(
                    "E010",
                    span,
                    "`footprint` declarations are top-level — move it out of the design/fn body"
                        .to_string(),
                ));
                // Consume it so the body keeps parsing cleanly.
                let _ = self.footprint_def();
                None
            }
            TokenKind::Ident(n) if n == "use" => {
                self.reject_attrs(&attrs);
                let span = self.span();
                self.diags.push(Diagnostic::error(
                    "E010",
                    span,
                    "`use` imports are file-level — move it above the design/fn body".to_string(),
                ));
                // Consume the statement so it can't misparse as a call.
                let _ = self.use_decl();
                None
            }
            TokenKind::Ident(_) => {
                self.reject_phys(&phys, "a `fn` call");
                self.reject_attrs(&attrs);
                // The callee may be a qualified path; `::<` stays turbofish
                // (path_ident's two-token lookahead never eats `::` + `<`).
                let callee = self.path_ident("").unwrap();
                let start = callee.span;
                let generic_args = if self.at(&TokenKind::PathSep) {
                    self.bump();
                    if self.at(&TokenKind::Lt) {
                        self.generic_args()
                    } else {
                        self.error_here(format!(
                            "expected `<` after `::` in a call, found {}",
                            self.peek().describe()
                        ));
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                self.expect(&TokenKind::LParen, "to open the call arguments")
                    .then_some(())?;
                let mut args = Vec::new();
                while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                    match self.pin_ref() {
                        Some(r) => args.push(r),
                        None => break,
                    }
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, "to close the call arguments");
                Some(Stmt::Call(CallStmt {
                    callee,
                    generic_args,
                    args,
                    intent,
                    span: start.to(self.prev_span()),
                }))
            }
            other => {
                let msg = format!(
                    "expected a statement (`inst`, `net`, `nc`, or a fn call), found {}",
                    other.describe()
                );
                self.reject_phys(&phys, "this statement");
                self.error_here(msg);
                None
            }
        }
    }

    /// Reject any non-`#[intent]` attribute left after `take_intent`.
    /// `#[designator(…)]` is inst-only (RFC-005); no other attribute exists.
    fn reject_attrs(&mut self, attrs: &[Attr]) {
        if let Some(a) = attrs.first() {
            self.diags.push(Diagnostic::error(
                "E010",
                a.span,
                format!(
                    "`#[{}]` is not valid here — declarations take `#[intent(\"…\")]`/`#[doc(\"…\")]`, and `inst` additionally `#[designator]`/`#[placement_hint]`",
                    a.name.name
                ),
            ));
        }
    }

    // -- layout constraints (RFC-013) ----------------------------------------

    fn layout_block(&mut self) -> Option<Stmt> {
        let start = self.span();
        self.bump(); // `layout`
        self.expect(&TokenKind::LBrace, "to open the layout block");
        let mut constraints = Vec::new();
        let mut board_outline: Option<BoardOutline> = None;
        let mut placements = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let before = self.pos;
            if self.at_ident("board_outline") {
                match (&board_outline, self.board_outline()) {
                    (Some(prev), Some(next)) => self.diags.push(
                        Diagnostic::error(
                            "E1006",
                            next.span,
                            "a design has at most one `board_outline`".to_string(),
                        )
                        .with_secondary(prev.span, "the first outline is here".to_string()),
                    ),
                    (None, Some(next)) => board_outline = Some(next),
                    (_, None) => {}
                }
            } else if self.at_ident("place") {
                if let Some(p) = self.placement() {
                    placements.push(p);
                }
            } else if let Some(c) = self.layout_constraint() {
                constraints.push(c);
            }
            if self.pos == before {
                // No progress on a malformed constraint — skip a token so the
                // block can still close.
                self.bump();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the layout block");
        Some(Stmt::Layout(LayoutBlock {
            constraints,
            board_outline,
            placements,
            span: start.to(self.prev_span()),
        }))
    }

    /// `board_outline: "path.dxf"` (RFC-020) — a reference to a DXF file. The
    /// DXF is opened, and its one closed outline entity extracted, at `cohdl
    /// build` (E1006 sub-cases); here we only capture the path string.
    fn board_outline(&mut self) -> Option<BoardOutline> {
        let start = self.span();
        self.bump(); // `board_outline`
        self.expect(&TokenKind::Colon, "after `board_outline`");
        let (path, path_span) = match self.peek() {
            TokenKind::Str(_) => {
                let t = self.bump();
                let TokenKind::Str(s) = t.kind else {
                    unreachable!()
                };
                (s, t.span)
            }
            _ => {
                self.error_here(format!(
                    "expected a DXF file path string after `board_outline:`, found {}",
                    self.peek().describe()
                ));
                return None;
            }
        };
        Some(BoardOutline {
            path,
            path_span,
            span: start.to(self.prev_span()),
        })
    }

    /// `place <inst> at (x, y) [rotate ANGLE]` (RFC-020) — a locked, optionally
    /// rotated component placement. Instance existence, coordinate unit-type,
    /// and the rotation's 0..=359 range are validated at assembly (E1007).
    fn placement(&mut self) -> Option<Placement> {
        let start = self.span();
        self.bump(); // `place`
        let inst = self.ident("as the instance to place")?;
        // RFC-024: `place NAME[i]` — always exactly ONE element; a range has
        // no single sensible meaning here (each element needs its own
        // coordinates).
        let index = if self.at(&TokenKind::LBracket) {
            match self.index_sel()? {
                IndexSel::Single(i, sp) => Some((i, sp)),
                other => {
                    self.diags.push(Diagnostic::error(
                        "E211",
                        other.span(),
                        "`place` takes a single element `NAME[i]` — a range or index list has no single position".to_string(),
                    ));
                    return None;
                }
            }
        } else {
            None
        };
        if !self.at_ident("at") {
            self.error_here("expected `at (x, y)` after the instance name".to_string());
            return None;
        }
        self.bump(); // `at`
        let at = self.length_pair()?;
        // Optional `rotate ANGLE` (E1007) and `side SIDE` (RFC-026, E1008) —
        // independent clauses, accepted in either order per the accepted text;
        // `fmt` canonicalizes to rotate-then-side.
        let mut rotate = 0u16;
        let mut saw_rotate = false;
        let mut side = crate::ast::PlacementSide::Top;
        let mut side_span = None;
        loop {
            if !saw_rotate && self.at_ident("rotate") {
                saw_rotate = true;
                self.bump(); // `rotate`
                match self.peek() {
                    TokenKind::Number(_) => {
                        let t = self.bump();
                        if let TokenKind::Number(n) = t.kind {
                            // Out-of-range / non-integer values are reported
                            // at assembly (E1007); an unparseable value maps to
                            // a sentinel that fails that range check.
                            rotate = n.parse::<u16>().unwrap_or(u16::MAX);
                        }
                    }
                    _ => {
                        self.error_here(format!(
                            "expected a rotation angle in degrees (0..=359) after `rotate`, found {}",
                            self.peek().describe()
                        ));
                    }
                }
            } else if side_span.is_none() && self.at_ident("side") {
                self.bump(); // `side`
                let v = self.ident("as the side (`top` or `bottom`)")?;
                side_span = Some(v.span);
                match crate::ast::PlacementSide::from_name(&v.name) {
                    Some(sd) => side = sd,
                    None => {
                        self.diags.push(Diagnostic::error(
                            "E1008",
                            v.span,
                            format!("`{}` is not a side — sides are: top, bottom", v.name),
                        ));
                        return None;
                    }
                }
            } else {
                break;
            }
        }
        Some(Placement {
            inst,
            index,
            at,
            rotate,
            side,
            side_span,
            span: start.to(self.prev_span()),
        })
    }

    fn layout_constraint(&mut self) -> Option<LayoutConstraint> {
        let start = self.span();
        match self.peek() {
            TokenKind::Ident(n) if n == "net_class" => {
                self.bump();
                let name = self.ident("as the net-class name")?;
                self.expect(&TokenKind::LBrace, "to open the net_class body");
                let mut nets = Vec::new();
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    nets.push(self.ident("as a net name in the net_class")?);
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the net_class body");
                Some(LayoutConstraint::NetClass {
                    name,
                    nets,
                    span: start.to(self.prev_span()),
                })
            }
            TokenKind::Ident(n) if n == "diff_pair" => {
                self.bump();
                let nets = self.layout_net_args()?;
                // RFC-027: optional `[differential_impedance: R,
                // single_ended_impedance: R, frequency: F]` bracket — named
                // fields, any order, each at most once; omitted bracket is
                // RFC-013's original form exactly.
                let mut differential_impedance = None;
                let mut single_ended_impedance = None;
                let mut frequency = None;
                if self.eat(&TokenKind::LBracket) {
                    use crate::units::UnitType;
                    loop {
                        let k = self.ident(
                            "as a diff_pair field (`differential_impedance`, `single_ended_impedance`, `frequency`)",
                        )?;
                        self.expect(&TokenKind::Colon, "after the field name");
                        let (slot, expected): (&mut Option<UnitValue>, UnitType) = match k
                            .name
                            .as_str()
                        {
                            "differential_impedance" => {
                                (&mut differential_impedance, UnitType::Resistance)
                            }
                            "single_ended_impedance" => {
                                (&mut single_ended_impedance, UnitType::Resistance)
                            }
                            "frequency" => (&mut frequency, UnitType::Frequency),
                            other => {
                                self.diags.push(Diagnostic::error(
                                        "E1009",
                                        k.span,
                                        format!(
                                            "`{}` is not a diff_pair field — fields are: differential_impedance, single_ended_impedance, frequency",
                                            other
                                        ),
                                    ));
                                return None;
                            }
                        };
                        if slot.is_some() {
                            self.diags.push(Diagnostic::error(
                                "E1009",
                                k.span,
                                format!("duplicate diff_pair field `{}`", k.name),
                            ));
                            return None;
                        }
                        let v = self.unit_literal("as the field value")?;
                        if v.unit != expected {
                            self.diags.push(Diagnostic::error(
                                "E110",
                                k.span,
                                format!(
                                    "diff_pair `{}` is a `{}` value — `{}` is a `{}`",
                                    k.name,
                                    expected.type_name(),
                                    v.text,
                                    v.unit.type_name()
                                ),
                            ));
                            return None;
                        }
                        *slot = Some(v);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBracket, "to close the diff_pair fields");
                }
                Some(LayoutConstraint::DiffPair {
                    nets,
                    differential_impedance,
                    single_ended_impedance,
                    frequency,
                    span: start.to(self.prev_span()),
                })
            }
            TokenKind::Ident(n) if n == "length_match" => {
                self.bump();
                let nets = self.layout_net_args()?;
                let tolerance = self.layout_tolerance();
                Some(LayoutConstraint::LengthMatch {
                    nets,
                    tolerance,
                    span: start.to(self.prev_span()),
                })
            }
            other => {
                let msg = format!(
                    "expected a layout constraint (`net_class`, `diff_pair`, or `length_match`), found {}",
                    other.describe()
                );
                self.error_here(msg);
                None
            }
        }
    }

    /// The parenthesized net-name list of `diff_pair(...)` / `length_match(...)`.
    fn layout_net_args(&mut self) -> Option<Vec<Ident>> {
        self.expect(&TokenKind::LParen, "to open the net list");
        let mut nets = Vec::new();
        while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
            nets.push(self.ident("as a net name")?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen, "to close the net list");
        Some(nets)
    }

    /// The optional `[tolerance: …]` suffix on `length_match`. Accepts a unit
    /// literal from RFC-001's closed set (`1ms` — pass-through as its source
    /// text) or a quoted string (`"0.15mm"` — the escape hatch for length
    /// units, which RFC-001's ten-type set cannot represent; RFC-013's
    /// unquoted `0.15mm` example needs a note-side amendment before it can
    /// lex). The value is never enforced by CoHDL (RFC-013 Failure modes).
    fn layout_tolerance(&mut self) -> Option<(String, Span)> {
        if !self.at(&TokenKind::LBracket) {
            return None;
        }
        self.bump(); // `[`
        if self.at_ident("tolerance") {
            self.bump();
        } else {
            self.error_here(format!(
                "expected `tolerance` in the length_match bracket, found {}",
                self.peek().describe()
            ));
        }
        self.expect(&TokenKind::Colon, "after `tolerance`");
        let value = match self.peek() {
            TokenKind::Str(_) => {
                let t = self.bump();
                let TokenKind::Str(s) = t.kind else {
                    unreachable!()
                };
                Some((s, t.span))
            }
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                // RFC-013 says `<Time-or-length-unit>`: Time, and — since
                // RFC-018 added the Length unit — mm literals too. The
                // accepted RFC-013 example `[tolerance: 0.15mm]` is finally
                // representable (closing that note-side item).
                if matches!(v.unit, UnitType::Time | UnitType::Length) {
                    Some((v.text.clone(), t.span))
                } else {
                    self.diags.push(
                        Diagnostic::error(
                            "E110",
                            t.span,
                            format!(
                                "a `tolerance` unit literal must be a `Time` or `Length` value, found `{}` (`{}`)",
                                v.unit.type_name(),
                                v.text
                            ),
                        )
                        .with_help(
                            "write a Time or Length literal (e.g. `[tolerance: 1ms]`, `[tolerance: 0.15mm]`) or a string",
                        ),
                    );
                    None
                }
            }
            other => {
                self.error_here(format!(
                    "the `tolerance` value must be a `Time`/`Length` literal or a string (e.g. `[tolerance: 1ms]` or `[tolerance: 0.15mm]`), found {}",
                    other.describe()
                ));
                None
            }
        };
        self.expect(&TokenKind::RBracket, "to close the tolerance bracket");
        value
    }

    /// One non-negative integer index (RFC-024). Indices are plain counting
    /// numbers — an instance name is `{base}{index}`, so nothing else parses.
    fn index_number(&mut self, ctx: &str) -> Option<i64> {
        match self.peek() {
            TokenKind::Number(_) => {
                let t = self.bump();
                let TokenKind::Number(text) = t.kind else {
                    unreachable!()
                };
                match text.parse::<i64>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        self.diags.push(Diagnostic::error(
                            "E211",
                            t.span,
                            format!("`{}` is not a whole-number index", text),
                        ));
                        None
                    }
                }
            }
            other => {
                self.error_here(format!(
                    "expected a whole-number index {} (e.g. `1`), found {}",
                    ctx,
                    other.describe()
                ));
                None
            }
        }
    }

    /// RFC-024 `[…]` after a name: `[S..=E]`, `[S..=E step N]`, or `[i, j, k]`.
    /// Assumes the caller has confirmed the next token is `[`.
    fn index_sel(&mut self) -> Option<IndexSel> {
        let open = self.span();
        self.bump(); // `[`
        let first = self.index_number("in the index")?;
        // `..=` lexes as Dot Dot Eq — the range form; anything else is a list.
        if self.at(&TokenKind::Dot) {
            self.bump();
            self.expect(&TokenKind::Dot, "in the range `..=`");
            self.expect(&TokenKind::Eq, "in the range `..=` (ranges are inclusive)");
            let end = self.index_number("as the range end")?;
            let mut step = 1;
            let mut explicit_step = false;
            if self.at_ident("step") {
                self.bump();
                step = self.index_number("as the stride")?;
                explicit_step = true;
            }
            self.expect(&TokenKind::RBracket, "to close the index bracket");
            let span = open.to(self.prev_span());
            if end < first {
                self.diags.push(Diagnostic::error(
                    "E211",
                    span,
                    format!(
                        "range `{}..={}` is empty — the end must not be below the start",
                        first, end
                    ),
                ));
                return None;
            }
            if step < 1 {
                self.diags.push(Diagnostic::error(
                    "E211",
                    span,
                    format!("stride `{}` must be 1 or more", step),
                ));
                return None;
            }
            Some(IndexSel::Range {
                start: first,
                end,
                step,
                explicit_step,
                span,
            })
        } else {
            let mut items = vec![first];
            let mut had_comma = false;
            while self.eat(&TokenKind::Comma) {
                had_comma = true;
                items.push(self.index_number("in the index list")?);
            }
            self.expect(&TokenKind::RBracket, "to close the index bracket");
            let span = open.to(self.prev_span());
            // `[i]` is the REAL reference form (valid everywhere); only a
            // comma-separated set is the net-member-only list sugar.
            if had_comma {
                Some(IndexSel::List(items, span))
            } else {
                Some(IndexSel::Single(first, span))
            }
        }
    }

    fn pin_ref(&mut self) -> Option<PinRef> {
        let base = self.ident("as a pin reference")?;
        let start = base.span;
        // RFC-024: an index selector binds to the base name. Parsed here for
        // ALL pin-reference positions so the grammar stays uniform; the
        // net-member-list-only scope boundary is enforced by the consumers,
        // which can then say precisely where it is not allowed.
        let index = if self.at(&TokenKind::LBracket) {
            Some(self.index_sel()?)
        } else {
            None
        };
        let pin = if self.eat(&TokenKind::Dot) {
            Some(self.ident("as the pin name after `.`")?)
        } else {
            None
        };
        Some(PinRef {
            base,
            index,
            pin,
            span: start.to(self.prev_span()),
        })
    }

    fn pin_ref_list(&mut self) -> Vec<PinRef> {
        let mut members = Vec::new();
        while let Some(r) = self.pin_ref() {
            members.push(r);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        members
    }
}

/// Non-numeric physical pad names: uppercase alphanumerics starting with a
/// letter — BGA grid positions (`A1`, `C3`) and named pads as they appear in
/// real footprints (`SH`, `EP`).
fn is_pad_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;
    use crate::span::SourceMap;

    fn parse_ok(src: &str) -> SourceFile {
        let mut sm = SourceMap::new();
        let f = sm.add_file("test.cohdl", src);
        let mut diags = Diagnostics::new();
        let tokens = lex(f, src, &mut diags);
        let file = parse(tokens, &mut diags);
        assert!(
            !diags.has_errors(),
            "unexpected parse errors:\n{}",
            diags.render(&sm)
        );
        file
    }

    fn parse_err(src: &str) -> String {
        let mut sm = SourceMap::new();
        let f = sm.add_file("test.cohdl", src);
        let mut diags = Diagnostics::new();
        let tokens = lex(f, src, &mut diags);
        let _ = parse(tokens, &mut diags);
        assert!(diags.has_errors(), "expected parse errors for:\n{}", src);
        diags.render(&sm)
    }

    #[test]
    fn parses_note10_trait_examples() {
        let file = parse_ok(
            r#"
pub trait TwoTerminal {
    pins {
        required A: pin
        required B: pin
    }
}

pub trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec {
        capacitance: Capacitance
        voltage_rating: Voltage
        tolerance: Tolerance
    }
}
"#,
        );
        assert_eq!(file.items.len(), 2);
        let ItemKind::Trait(t) = &file.items[1].kind else {
            panic!()
        };
        assert_eq!(t.super_traits[0].name, "TwoTerminal");
        assert_eq!(t.designator_prefix.as_ref().unwrap().0, "C");
        assert_eq!(t.specs.len(), 3);
    }

    #[test]
    fn parses_note10_device_example() {
        let file = parse_ok(
            r#"
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}
"#,
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.generics.len(), 3);
        assert!(d.generics[1].default.is_some());
        let pins = d.pins_for(None);
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[0].obligation, Obligation::Required);
    }

    #[test]
    fn parses_pin_bus_and_roles() {
        let file = parse_ok(
            r#"
pub device MCU_ESP32S3 {
    pins {
        required VDD: 1 [power_in]
        required GND: 2, 3, 4 [passive]
        optional NC_1: 5 [passive]
        required TX: 6 [output]
    }
}
"#,
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        let pins = d.pins_for(None);
        assert_eq!(pins.len(), 4);
        assert_eq!(pins[1].numbers.len(), 3);
        assert_eq!(pins[2].obligation, Obligation::Optional);
        assert_eq!(pins[3].role.unwrap().0, PinRole::Output);
    }

    #[test]
    fn parses_impls() {
        let file = parse_ok(
            r#"
impl TwoTerminal for MLCC {}
impl TwoTerminal for TantalumCap {
    pins { A: Anode, B: Cathode }
}
"#,
        );
        let ItemKind::Impl(i) = &file.items[1].kind else {
            panic!()
        };
        assert_eq!(i.pin_map.len(), 2);
        assert_eq!(i.pin_map[0].role.name, "A");
        assert_eq!(i.pin_map[0].target.name, "Anode");
    }

    #[test]
    fn parses_note10_fn_and_design() {
        let file = parse_ok(
            r#"
fn decoupling_cap<V: Voltage>(pin: Pin) {
    inst c: MLCC<100nF, V>
    net _: pin, c.A
}

fn power_rail<V: Voltage>(vdd_pin: Pin) {
    inst ferrite: Ferrite_Bead
    net _: vdd_pin, ferrite.IN
    decoupling_cap::<V>(ferrite.OUT)
}

design Board {
    inst mcu: MCU_ESP32S3
    power_rail::<3.3V>(mcu.VDD)
}
"#,
        );
        assert_eq!(file.items.len(), 3);
        let ItemKind::Fn(f) = &file.items[1].kind else {
            panic!()
        };
        assert_eq!(f.body.len(), 3);
        assert!(matches!(&f.body[2], Stmt::Call(c) if c.callee.name == "decoupling_cap"));
        let ItemKind::Design(d) = &file.items[2].kind else {
            panic!()
        };
        assert!(matches!(&d.body[1], Stmt::Call(c) if !c.generic_args.is_empty()));
    }

    #[test]
    fn parses_nets_nc_annotations() {
        let file = parse_ok(
            r#"
design Board {
    inst mcu: MCU_ESP32S3
    net VDD_3V3 [3.3V]: mcu.VDD
    net GND [gnd]: mcu.GND
    net USB_DM: mcu.USB_DM, usb.DM
    nc: mcu.RTC_XTAL_IN, mcu.RTC_XTAL_OUT
}
"#,
        );
        let ItemKind::Design(d) = &file.items[0].kind else {
            panic!()
        };
        assert!(matches!(
            &d.body[1],
            Stmt::Net(n) if matches!(n.annotation, Some(NetAnnotation::Voltage(..)))
        ));
        assert!(matches!(
            &d.body[2],
            Stmt::Net(n) if matches!(n.annotation, Some(NetAnnotation::Gnd(..)))
        ));
        assert!(matches!(&d.body[4], Stmt::Nc(n) if n.members.len() == 2));
    }

    #[test]
    fn parses_designator_attr() {
        let file = parse_ok(
            r#"
design Board {
    #[designator("U7")]
    inst mcu: MCU_ESP32S3
}
"#,
        );
        let ItemKind::Design(d) = &file.items[0].kind else {
            panic!()
        };
        let Stmt::Inst(i) = &d.body[0] else { panic!() };
        assert_eq!(i.attrs[0].name.name, "designator");
        assert_eq!(i.attrs[0].args[0].0, "U7");
    }

    #[test]
    fn parses_part() {
        let file = parse_ok(
            r#"
pub part MLCC_100nF_16V: MLCC<100nF, 16V, 10%> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC", footprint: FP_C_0402 }
    alt { mfr: "Murata", mpn: "GRM155R71C104KA88D" }
}
"#,
        );
        let ItemKind::Part(p) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(p.primary.field("mpn").unwrap().value, "CL05B104KO5NNNC");
        // RFC-017: footprint is a symbol reference, not a string field.
        assert_eq!(p.primary.footprint.as_ref().unwrap().name, "FP_C_0402");
        assert_eq!(p.alts.len(), 1);
    }

    #[test]
    fn parses_impl_trait_param_and_anon_net() {
        let file = parse_ok(
            r#"
fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A
}

fn sugar(target: impl Capacitor + Polarized, pin: Pin) {
    net _: pin, target.A
}
"#,
        );
        let ItemKind::Fn(f) = &file.items[1].kind else {
            panic!()
        };
        assert!(matches!(&f.params[0].ty, FnParamTy::ImplTrait(ts, _) if ts.len() == 2));
    }

    #[test]
    fn rejects_v1_embedded_impl_clause() {
        let rendered = parse_err("device MLCC: impl Capacitor { pins { A: 1 } }");
        assert!(
            rendered.contains("never has a trait clause"),
            "{}",
            rendered
        );
        assert!(rendered.contains("impl Trait for MLCC"), "{}", rendered);
    }

    #[test]
    fn rejects_bare_number_spec() {
        let rendered = parse_err("device X { spec { capacitance: 100 } }");
        assert!(rendered.contains("E111"), "{}", rendered);
        assert!(rendered.contains("bare number"), "{}", rendered);
    }

    #[test]
    fn rejects_part_without_primary() {
        let rendered = parse_err("part P: MLCC<100nF> { alt { mpn: \"X\" } }");
        assert!(rendered.contains("no `primary` entry"), "{}", rendered);
    }

    #[test]
    fn design_with_multiline_net() {
        let file = parse_ok(
            r#"
design B {
    net VDD_3V3: ldo.VOUT,
                 mcu.VDD, mcu.VDDA,
                 c1.A
}
"#,
        );
        let ItemKind::Design(d) = &file.items[0].kind else {
            panic!()
        };
        let Stmt::Net(n) = &d.body[0] else { panic!() };
        assert_eq!(n.members.len(), 4);
    }

    // Error recovery must always advance. `sync_in_block` stops *at* the `,`
    // it finds so the caller can see the delimiter; a loop that re-entered
    // recovery on that same token never terminated — a hang, and one that
    // appended a diagnostic per pass until memory ran out. Each shape below
    // reached it through a different loop (part body, impl body, variants).
    // A bounded diagnostic count is the regression signal: unbounded IS the
    // bug.
    #[test]
    fn malformed_input_never_spins_in_recovery() {
        for src in [
            // a malformed generic argument followed by a comma, in a part type
            "pub part P: D<1e+06ohm, 1%> {\n    primary { mfr: \"Y\", mpn: \"M\", footprint: F }\n}\n",
            // a stray comma in a part body
            "pub part P: D<1Mohm, 1%> {\n    ,\n    primary { mfr: \"Y\", mpn: \"M\", footprint: F }\n}\n",
            // a non-string AVL value
            "pub part P: D<1Mohm, 1%> {\n    primary { mfr: 7, mpn: \"M\", footprint: F }\n}\n",
            // a stray comma in an impl body
            "impl TwoTerminal for D {\n    ,\n}\n",
            // a stray comma in a variants block
            "pub device V {\n    variants { , A }\n    pins[A] { required A: 1 [passive] }\n}\n",
        ] {
            let rendered = parse_err(src);
            let n = rendered.matches("error[").count();
            assert!(
                n < 20,
                "recovery emitted {n} diagnostics (it used to spin) for:\n{src}\n{rendered}"
            );
        }
    }
}

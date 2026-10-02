//! Recursive-descent parser.
//!
//! Deterministic with bounded lookahead (max 2 tokens), per the Constitution's
//! deterministic-grammar hard constraint. Error recovery is panic-mode: skip
//! to the next top-level keyword (items) or statement keyword / closing brace
//! (bodies), so one mistake doesn't cascade.
//!
//! Termination on ANY input is structural, not per call site: every `{ … }`
//! body loop runs on `block_continues` (an iteration that consumed nothing
//! consumes the token it stalled on), and every diagnostic goes through
//! `report`, which stops the file at `MAX_PARSE_ERRORS` (E102).

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics, Severity};
use crate::lex::{Token, TokenKind};
use crate::span::Span;
use crate::units::{UnitType, UnitValue};

/// Maximum recursive syntax/AST path: enclosing `for` nodes plus expression
/// nodes (leaves count as one). Independent of the expansion frame budget.
/// Kept small for parser, downstream visitors, Clone and Drop on ordinary
/// thread stacks; checked BEFORE descending or constructing a parent node.
/// Debug probes on the default test-thread stack overflow with candidate 256
/// on parentheses and candidate 128 on circuit loops; 96 passes both paths.
/// Loop headers use the same budget as their body (a literal costs one).
const MAX_SYNTAX_DEPTH: usize = 96;

/// Syntax errors one file may report before parsing it stops (E102). A
/// backstop, not a style limit: real files with this many independent
/// mistakes are rare, and past it the output is cascade noise to a human
/// and to the repair loop alike. It also bounds the cost of any recovery
/// path that fails to make progress — every such pass reports, so a
/// stalled loop reaches the budget and ends instead of allocating until
/// the OS kills the process (the cohdl 0.8.0 `device X { , }` hang).
const MAX_PARSE_ERRORS: usize = 200;

/// Test builds only: `count_step`'s cap is this many `peek`s per token
/// (plus 10,000). Terminating parses peek fewer than 3 times per token on
/// both fuzz bases, every budget-halted variant included; a stalled loop
/// passes the cap within milliseconds, a few megabytes in.
#[cfg(test)]
const TEST_PEEKS_PER_TOKEN: usize = 100;

/// Test-only switches that turn off one termination defence at a time —
/// the `block_continues` guard or `sync_in_block_advancing`'s bump — so
/// the tests can show each holds on its own, plus a count of the times the
/// guard fired. Thread-local: set them on the thread that parses.
#[cfg(test)]
mod test_hooks {
    use std::cell::Cell;

    thread_local! {
        static GUARD_OFF: Cell<bool> = const { Cell::new(false) };
        static ADVANCING_OFF: Cell<bool> = const { Cell::new(false) };
        static GUARD_FIRED: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn guard_on() -> bool {
        !GUARD_OFF.with(Cell::get)
    }

    pub(super) fn advancing_on() -> bool {
        !ADVANCING_OFF.with(Cell::get)
    }

    pub(super) fn guard_fired() {
        GUARD_FIRED.with(|c| c.set(c.get() + 1));
    }

    /// Enable/disable the two defences on this thread; resets the count.
    pub(super) fn set(guard: bool, advancing: bool) {
        GUARD_OFF.with(|c| c.set(!guard));
        ADVANCING_OFF.with(|c| c.set(!advancing));
        GUARD_FIRED.with(|c| c.set(0));
    }

    /// How often the guard fired on this thread since the last `set`.
    pub(super) fn fired() -> usize {
        GUARD_FIRED.with(Cell::get)
    }
}

pub fn parse(tokens: Vec<Token>, diags: &mut Diagnostics) -> SourceFile {
    parse_with_budget(tokens, diags, MAX_PARSE_ERRORS)
}

/// `parse` with an explicit error budget. Production always passes
/// `MAX_PARSE_ERRORS`; the tests pass every smaller budget so the halt
/// lands at each error point a malformed input reaches (see `report`).
fn parse_with_budget(tokens: Vec<Token>, diags: &mut Diagnostics, max_errors: usize) -> SourceFile {
    let mut local = Diagnostics::new();
    let mut parser = Parser {
        tokens,
        pos: 0,
        diags: &mut local,
        nesting: 0,
        depth_error: None,
        halted: false,
        max_errors,
        #[cfg(test)]
        steps: std::cell::Cell::new(0),
    };
    let mut file = parser.file();
    file.truncated = parser.halted;
    if let Some(span) = parser.depth_error {
        // Resource-limit recovery abandons this file. Do not forward a partial
        // body to consumers or emit cascaded missing-delimiter diagnostics from
        // the bounded unwind. Lexer diagnostics already in `diags` survive.
        diags.push(
            Diagnostic::error(
                "E102",
                span,
                format!("syntax/AST depth limit of {MAX_SYNTAX_DEPTH} exceeded"),
            )
            .with_help(
                "this file's declarations were not read, so no name-resolution or design \
                 checks ran — an \"unknown\" name is not reported until the file parses",
            ),
        );
        SourceFile {
            items: Vec::new(),
            truncated: true,
        }
    } else {
        diags.extend(local);
        file
    }
}

/// Per-loop state for `Parser::block_continues`: where the current
/// iteration of one body loop began.
#[derive(Default)]
struct Progress(Option<usize>);

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    diags: &'a mut Diagnostics,
    nesting: usize,
    depth_error: Option<Span>,
    /// Set once the file has used up its `MAX_PARSE_ERRORS` budget; the
    /// cursor then sits on EOF and further diagnostics are dropped.
    halted: bool,
    /// The error budget `report` enforces — `MAX_PARSE_ERRORS` outside tests.
    max_errors: usize,
    /// Test builds only: `peek` calls so far, capped by `count_step`.
    #[cfg(test)]
    steps: std::cell::Cell<usize>,
}

impl<'a> Parser<'a> {
    /// The one way the parser records a diagnostic. Enforces the per-file
    /// error budget: the error that would exceed `MAX_PARSE_ERRORS` is
    /// replaced by one E102 at its position, and the cursor jumps to EOF —
    /// the same constant-time, forward-only abandonment `depth_exceeded`
    /// uses, so every loop (all of which stop at EOF) unwinds at once.
    ///
    /// INVARIANT this imposes on every production: once any call that can
    /// report has returned, the token the code peeked before it may be
    /// gone — the cursor can now be on EOF. A branch chosen on a peeked
    /// token must re-check the cursor (or propagate `None`) after such a
    /// call, never `unwrap` a parse of that token: `stmt`'s call arm did,
    /// and a file whose 201st error landed on its `reject_attrs` panicked
    /// (exit 101) instead of reporting. The budget-boundary fuzz in the
    /// tests halts every malformed input at each of its error points.
    fn report(&mut self, d: Diagnostic) {
        if self.halted {
            return;
        }
        if d.severity == Severity::Error && self.diags.error_count() >= self.max_errors {
            self.halted = true;
            self.diags.push(
                Diagnostic::error(
                    "E102",
                    d.primary.span,
                    format!(
                        "too many syntax errors — stopped parsing this file after {}",
                        self.max_errors
                    ),
                )
                .with_help(
                    "fix the errors reported above and check again; later ones are often \
                     knock-on effects of the first",
                )
                .with_help(
                    "the rest of this file was not read, so no name-resolution or design \
                     checks ran — an \"unknown\" name is not reported until the file parses",
                ),
            );
            self.pos = self.tokens.len() - 1;
            return;
        }
        self.diags.push(d);
    }

    /// `check::generics::checked_int` (the Int-literal range check), with
    /// its diagnostic routed through `report` so it counts against — and
    /// stops at — the error budget like every other parser diagnostic.
    fn checked_int(&mut self, text: &str, span: Span) -> Option<i64> {
        let mut out = Diagnostics::new();
        let value = crate::check::generics::checked_int(text, span, &mut out);
        for d in out.drain_batch() {
            self.report(d);
        }
        value
    }

    fn depth_exceeded<T>(&mut self, span: Span) -> Option<T> {
        self.depth_error.get_or_insert(span);
        // Constant-time, forward-only recovery; no recursion through the rest
        // of an adversarial file, even if its delimiters are malformed.
        self.pos = self.tokens.len() - 1;
        None
    }

    fn nested<T>(&mut self, parse: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        if self.nesting == MAX_SYNTAX_DEPTH {
            return self.depth_exceeded(self.span());
        }
        self.nesting += 1;
        let result = parse(self);
        self.nesting -= 1;
        result
    }

    // -- token plumbing ------------------------------------------------------

    fn peek(&self) -> &TokenKind {
        #[cfg(test)]
        self.count_step();
        &self.tokens[self.pos].kind
    }

    /// Test builds only: panic once this parse has peeked far more often
    /// than any terminating parse of its input can. The tests parse on
    /// worker threads under a deadline, but a timed-out worker keeps
    /// running — a stall that also escaped the error budget would grow the
    /// test process itself by gigabytes a second. This bounds it on every
    /// platform (macOS does not enforce memory rlimits).
    #[cfg(test)]
    fn count_step(&self) {
        let steps = self.steps.get() + 1;
        self.steps.set(steps);
        let cap = TEST_PEEKS_PER_TOKEN * self.tokens.len() + 10_000;
        assert!(
            steps <= cap,
            "parser stalled: {steps} peeks for {} tokens",
            self.tokens.len()
        );
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
        self.report(Diagnostic::error("E010", self.span(), message));
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

    fn file(&mut self) -> SourceFile {
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
        SourceFile {
            items,
            truncated: false,
        }
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
                TokenKind::Ident(n)
                    if n == "use" || n == "footprint" || n == "pad" || n == "subdesign" =>
                {
                    return
                }
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
            self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
                    "E010",
                    *intent_span,
                    "`#[intent]` is not valid on a `use` import".to_string(),
                ));
            }
            for (_, doc_span) in &docs {
                self.report(Diagnostic::error(
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
            self.report(Diagnostic::error(
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
            self.report(Diagnostic::error(
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
        // RFC-032 `subdesign NAME { … }` — contextual keyword, like `use`.
        if self.at_ident("subdesign") && matches!(self.peek_ahead(1), TokenKind::Ident(_)) {
            let kind = self.subdesign_def().map(ItemKind::Subdesign);
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
        if self.at_ident("subdesign") && self.peek_ahead(1) == &TokenKind::LBrace {
            let span = self.span();
            self.report(Diagnostic::error(
                "E010",
                span,
                "a `subdesign` declaration needs a name: `subdesign NAME { … }`".to_string(),
            ));
            self.bump(); // subdesign
            self.bump(); // `{` — skip_braced_body expects the opener consumed
            self.skip_braced_body(span);
            return None;
        }
        if !docs.is_empty() && matches!(self.peek(), TokenKind::Impl) {
            for (_, doc_span) in &docs {
                self.report(Diagnostic::error(
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
                    "expected a top-level declaration (`trait`, `device`, `impl`, `fn`, `part`, `design`, `subdesign`, `footprint`, `pad`, or `use`), found {}",
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
            self.report(Diagnostic::error(
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            // A top-level declaration keyword means the body's `}` is
            // missing (nothing top-level is legal in a footprint body) —
            // stop WITHOUT consuming so the declaration survives.
            if self.at_decl_keyword() {
                self.report(Diagnostic::error(
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
                        self.report(
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
                        self.report(
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
                        self.report(
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
                    self.report(
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
                self.report(Diagnostic::error(
                    "E806",
                    self.span(),
                    format!(
                        "a footprint body contains `pad N: Symbol at (x, y)` placements, `mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D | size: (w, h)]` holes, an optional `courtyard`, and an optional `silkscreen_ref` — found {}",
                        self.peek().describe()
                    ),
                ));
                self.sync_footprint_body();
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
            // RFC-033: a signed mount-hole number is E102 (`legacy_number`).
            TokenKind::Minus if matches!(self.peek_ahead(1), TokenKind::Number(_)) => {
                let _ = self.legacy_number("as the mount-hole number");
                return None;
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
                    self.report(Diagnostic::error(
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
                        self.report(Diagnostic::error(
                            "E810",
                            v.span,
                            "`annulus` is only valid for electrical pads, not `mount_hole`"
                                .to_string(),
                        ));
                        return None;
                    }
                    Some(s) => shape = Some((s, v.span)),
                    None => {
                        self.report(Diagnostic::error(
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
            // RFC-033: a signed pad number is E102 (`legacy_number`).
            TokenKind::Minus if matches!(self.peek_ahead(1), TokenKind::Number(_)) => {
                let _ = self.legacy_number("as the pad number");
                return None;
            }
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
                    self.report(Diagnostic::error(
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
            self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            let kw_span = self.span();
            let Some(kw) = self.ident(
                "as a silkscreen statement (`line`, `circle`, `arc`, `polygon`, \
                 `pin_1_marker`, `polarity_marker`)",
            ) else {
                self.sync_in_block();
                self.eat(&TokenKind::Comma);
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
                        self.report(Diagnostic::error(
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
                                self.report(Diagnostic::error(
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
                                self.report(Diagnostic::error(
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
                    continue;
                }
                other => {
                    self.report(Diagnostic::error(
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if self.at_ident("pad")
                || self.at_ident("courtyard")
                || self.at_ident("window")
                || self.at_ident("silkscreen")
                || self.at_ident("silkscreen_ref")
                || self.at_decl_keyword()
            {
                self.report(Diagnostic::error(
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
                        Some(PadShape::Annulus) => self.report(Diagnostic::error(
                            "E806",
                            v.span,
                            format!("`annulus` is only valid for electrical pads, not `{}`", kw),
                        )),
                        Some(s) => shape = Some((s, v.span)),
                        None => self.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
        }
        if !unclosed {
            self.expect(&TokenKind::RBrace, "to close the block");
        }
        let span = start.to(self.prev_span());
        let (Some(shape), Some(at), Some((size, size_span))) = (shape, at, size) else {
            self.report(Diagnostic::error(
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
        let (x, y) = self.length_pair_spans()?;
        Some((x.0, y.0))
    }

    /// `length_pair` keeping each literal's own span (RFC-033: placement
    /// coordinates ride as `Expr::Length` nodes).
    fn length_pair_spans(&mut self) -> Option<((UnitValue, Span), (UnitValue, Span))> {
        if !self.expect(&TokenKind::LParen, "to open the coordinate pair") {
            // One defect, one diagnostic — a missing `(` already implies the
            // offsets are absent; don't also report each of them.
            return None;
        }
        let x = self.unit_literal_with_span("as the x offset")?;
        self.expect(&TokenKind::Comma, "between the coordinates");
        let y = self.unit_literal_with_span("as the y offset")?;
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
        // RFC-033: signed literals (`-1.5mm`) assemble here — one source of
        // truth for the sign rule (`signed_unit_literal`, which also handles
        // the plain non-signed case), replacing the blocks previously
        // inlined in this fn and `device_spec_field`.
        let _ = ctx;
        self.signed_unit_literal().map(|(v, _)| v)
    }

    /// `unit_literal` keeping the literal's own span (RFC-033 expression
    /// nodes need it).
    fn unit_literal_with_span(&mut self, ctx: &str) -> Option<(UnitValue, Span)> {
        // RFC-033: `-` lexes as its own token; a `-` byte-adjacent to a unit
        // literal here is still a signed literal (`-1.5mm`, `-40C`).
        if self.at(&TokenKind::Minus) && matches!(self.peek_ahead(1), TokenKind::Unit(_)) {
            let minus_span = self.span();
            let adjacent = {
                let idx = self.pos;
                idx + 1 < self.tokens.len()
                    && self.tokens[idx].span.end == self.tokens[idx + 1].span.start
            };
            if adjacent {
                self.bump(); // -
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                return match v.negate_for_literal() {
                    Ok(v) => Some((v, minus_span.to(t.span))),
                    Err(msg) => {
                        self.report(Diagnostic::error("E105", minus_span.to(t.span), msg));
                        None
                    }
                };
            }
        }
        match self.peek() {
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                Some((v, t.span))
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if self.at_decl_keyword() {
                self.report(Diagnostic::error(
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
                        None => self.report(Diagnostic::error(
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
                        None => self.report(Diagnostic::error(
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
                        None => self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
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
                        self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
            self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(d);
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
                p.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
                        self.report(Diagnostic::error(
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
                        self.report(Diagnostic::error(
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
                        self.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
                        IndexSel::Single(e, sp) => Some((e, sp)),
                        other => {
                            self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
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
                    self.report(Diagnostic::error(
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
        let mut stray = StrayTraitMembers::default();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if self.at(&TokenKind::Pins) {
                let block_start = self.span();
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the pins block");
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
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
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
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
            } else if let Some(kind) = self.stray_member_kind(None) {
                let entry = (self.span(), self.stray_member_name());
                match kind {
                    StrayKind::Pin => {
                        stray.pin_entries.push(entry);
                        match self.trait_pin() {
                            Some(pin) => stray.pins.push(pin),
                            None => self.sync_in_block_advancing(),
                        }
                    }
                    // (`Signed` is device-only; `stray_member_kind(None)`
                    // never returns it.)
                    StrayKind::Spec | StrayKind::Signed => {
                        stray.spec_entries.push(entry);
                        match self.trait_spec_field() {
                            Some(field) => stray.specs.push(field),
                            None => self.sync_in_block_advancing(),
                        }
                    }
                }
                self.eat(&TokenKind::Comma);
            } else {
                self.error_here(format!(
                    "expected `pins`, `spec`, or `designator_prefix` in the trait body, found {}",
                    self.peek().describe()
                ));
                // Advancing: a stray `,` here used to stall this loop
                // forever (cohdl 0.8.0 `trait T { , }`).
                self.sync_in_block_advancing();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the trait body");
        self.finish_stray_trait_members(&mut def, stray);
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
                self.report(
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
            self.report(
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
        let mut stray = StrayDeviceMembers::default();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if self.at_ident("variants") {
                let variants_start = self.span();
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the variants block");
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
                    let Some(v) = self.ident("as a variant name") else {
                        self.sync_in_block_advancing();
                        continue;
                    };
                    // RFC-008 rejects wildcard/default arms outright.
                    if v.name == "_" {
                        self.report(Diagnostic::error(
                            "E010",
                            v.span,
                            "`_` is not a valid variant name — every variant is named explicitly, no wildcard/catch-all arms (RFC-008)",
                        ));
                        self.eat(&TokenKind::Comma);
                        continue;
                    }
                    // RFC-008: the closed set is duplicate-checked at parse.
                    if let Some(prev) = def.variants.iter().find(|x| x.name == v.name) {
                        self.report(
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
                let mut slips = PinSlips::default();
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
                    if let Some(pin) = self.device_pin(&mut slips) {
                        pins.push(pin);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the pins block");
                self.report_pin_slips(&slips);
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
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
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
            } else if let Some(kind) = self.stray_member_kind(Some(&def.generics)) {
                let entry = (self.span(), self.stray_member_name());
                match kind {
                    StrayKind::Pin => {
                        stray.pin_entries.push(entry);
                        match self.device_pin(&mut stray.pin_slips) {
                            Some(pin) => stray.pins.push(pin),
                            None => self.sync_in_block_advancing(),
                        }
                    }
                    StrayKind::Spec => {
                        stray.spec_entries.push(entry);
                        match self.device_spec_field() {
                            Some(field) => stray.specs.push(field),
                            None => self.sync_in_block_advancing(),
                        }
                    }
                    StrayKind::Signed => self.stray_signed_member(&mut stray),
                }
                self.eat(&TokenKind::Comma);
            } else {
                self.error_here(format!(
                    "expected `pins`, `spec`, or `variants` in the device body, found {}",
                    self.peek().describe()
                ));
                // Advancing: a stray `,` here used to stall this loop
                // forever (cohdl 0.8.0 `device X { , }`).
                self.sync_in_block_advancing();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the device body");
        self.finish_stray_device_members(&mut def, stray);
        Some(def)
    }

    /// A member written straight into a trait/device body, where only block
    /// keywords are legal: `[required|optional] NAME: …`. Which block it
    /// belongs in is read off the token after the `:` (2-token lookahead,
    /// like `device_pin`'s list continuation). `generics` is `Some` for a
    /// device body (a generic parameter name there is a spec value).
    fn stray_member_kind(&self, generics: Option<&[GenericParam]>) -> Option<StrayKind> {
        if matches!(self.peek(), TokenKind::Required | TokenKind::Optional) {
            return (matches!(self.peek_ahead(1), TokenKind::Ident(_))
                && self.peek_ahead(2) == &TokenKind::Colon)
                .then_some(StrayKind::Pin);
        }
        if !matches!(self.peek(), TokenKind::Ident(_)) || self.peek_ahead(1) != &TokenKind::Colon {
            return None;
        }
        let value = self.peek_ahead(2);
        match generics {
            // Device: `NAME: 5V` / `NAME: C` is a spec field; `NAME: 1 …`,
            // `NAME: A3 …`, `NAME: [1, 2] …`, `NAME: required …` a pin;
            // `NAME: -…` either, decided by the token after the `-`.
            Some(generics) => match value {
                TokenKind::Unit(_) => Some(StrayKind::Spec),
                TokenKind::Minus => Some(StrayKind::Signed),
                TokenKind::Ident(n) if generics.iter().any(|g| &g.name.name == n) => {
                    Some(StrayKind::Spec)
                }
                TokenKind::Number(_)
                | TokenKind::LBracket
                | TokenKind::Required
                | TokenKind::Optional => Some(StrayKind::Pin),
                TokenKind::Ident(n) if is_pad_name(n) => Some(StrayKind::Pin),
                _ => None,
            },
            // Trait: `NAME: pin` is a pin role, `NAME: Voltage` a spec field.
            None => match value {
                TokenKind::Ident(n) if n == "pin" => Some(StrayKind::Pin),
                TokenKind::Ident(n) if UnitType::from_type_name(n).is_some() => {
                    Some(StrayKind::Spec)
                }
                _ => None,
            },
        }
    }

    /// The member name of the entry `stray_member_kind` just classified.
    fn stray_member_name(&self) -> String {
        let at = usize::from(matches!(
            self.peek(),
            TokenKind::Required | TokenKind::Optional
        ));
        match self.peek_ahead(at) {
            TokenKind::Ident(n) => n.clone(),
            _ => unreachable!("stray_member_kind matched an identifier here"),
        }
    }

    /// A `NAME: -…` member in a device body (`StrayKind::Signed`): with
    /// `NAME :` consumed, `-` + number is a pin entry with a signed pin
    /// number and `-` + unit literal a negative spec value; anything else
    /// is neither, and gets the plain device-body error.
    fn stray_signed_member(&mut self, stray: &mut StrayDeviceMembers) {
        let start = self.span();
        let found = self.peek().describe();
        let t = self.bump();
        let TokenKind::Ident(text) = t.kind else {
            unreachable!("stray_member_kind matched an identifier here")
        };
        let name = Ident {
            name: text,
            span: t.span,
        };
        self.bump(); // `:`
        let entry = (start, name.name.clone());
        let parsed = match self.peek_ahead(1) {
            TokenKind::Number(_) => {
                stray.pin_entries.push(entry);
                self.device_pin_value(
                    start,
                    Obligation::Required,
                    name,
                    true,
                    &mut stray.pin_slips,
                )
                .map(|pin| stray.pins.push(pin))
            }
            TokenKind::Unit(_) => {
                stray.spec_entries.push(entry);
                self.device_spec_value(start, name)
                    .map(|field| stray.specs.push(field))
            }
            _ => {
                self.report(Diagnostic::error(
                    "E010",
                    start,
                    format!(
                        "expected `pins`, `spec`, or `variants` in the device body, found {found}"
                    ),
                ));
                None
            }
        };
        if parsed.is_none() {
            self.sync_in_block_advancing();
        }
    }

    /// One diagnostic per kind of member found outside its block, naming
    /// the first and counting the rest, with the entry rewritten in place
    /// and the help fitted to the blocks the device already has — then
    /// adopt the entries where that help puts them, so the rest of the
    /// pipeline checks the device the author meant instead of cascading
    /// "no pin `X`" errors:
    /// - no unqualified `pins { }`: the entries become that block;
    /// - one exists: they join it (writing a second would be E201);
    /// - the device declares variants: they stay out — which
    ///   `pins[VARIANT]` block each belongs to is the author's call, and an
    ///   unqualified block there is E908.
    ///
    /// `spec` is the same minus the variant case: an unqualified `spec { }`
    /// beside variants is legal (fields shared by every variant).
    fn finish_stray_device_members(&mut self, def: &mut DeviceDef, stray: StrayDeviceMembers) {
        if let Some((span, name)) = stray.pin_entries.first() {
            let n = stray.pin_entries.len();
            let example = stray.pins.first().map_or_else(
                || "required NAME: 1, 2 [passive]".to_string(),
                canonical_pin,
            );
            let more = if n > 1 { " …" } else { "" };
            let these = if n > 1 { "these entries" } else { "this entry" };
            let entries = "one `[required|optional] NAME: N, N, … [ROLE]` entry per line";
            let existing = def.pin_blocks.iter().position(|b| b.variant.is_none());
            let help = if def.variants.is_empty() {
                match existing {
                    Some(_) => format!(
                        "move {these} into the device's existing `pins {{ … }}` block, written `{example}` — {entries}; a second `pins` block would be a duplicate (E201)"
                    ),
                    None => format!(
                        "write `pins {{ {example}{more} }}` — {entries} inside the block"
                    ),
                }
            } else {
                let variants: Vec<&str> = def.variants.iter().map(|v| v.name.as_str()).collect();
                let uncovered = def.variants.iter().find(|v| {
                    !def.pin_blocks
                        .iter()
                        .any(|b| b.variant.as_ref().is_some_and(|q| q.name == v.name))
                });
                match uncovered {
                    Some(v) => format!(
                        "device `{}` declares variants ({}), so its pins go in one qualified block per variant — e.g. `pins[{}] {{ {example}{more} }}`, {entries}",
                        def.name.name,
                        variants.join(", "),
                        v.name
                    ),
                    None => format!(
                        "device `{}` declares variants ({}) and has a `pins[VARIANT] {{ … }}` block for each — move {these} into the block{} {} belong{} to, {entries}",
                        def.name.name,
                        variants.join(", "),
                        if n > 1 { "s" } else { "" },
                        if n > 1 { "they" } else { "it" },
                        if n > 1 { "" } else { "s" },
                    ),
                }
            };
            let mut d = Diagnostic::error(
                "E010",
                *span,
                format!(
                    "pin entry `{name}` is written directly in the device body{} — device pins are declared inside a `pins {{ … }}` block",
                    and_more(n, "in this device")
                ),
            )
            .with_help(help);
            if !stray.pin_slips.obligation_after_name.is_empty() {
                d = d.with_help(
                    "`required`/`optional` comes before the pin name, never after the `:`",
                );
            }
            if !stray.pin_slips.bracketed_numbers.is_empty() {
                d = d.with_help(
                    "pin numbers are a bare comma-separated list — only the role is bracketed",
                );
            }
            self.report(d);
            if def.variants.is_empty() && !stray.pins.is_empty() {
                match existing {
                    Some(i) => def.pin_blocks[i].pins.extend(stray.pins),
                    None => {
                        let span = stray.pins[0].span.to(stray.pins[stray.pins.len() - 1].span);
                        def.pin_blocks.push(PinBlock {
                            variant: None,
                            pins: stray.pins,
                            span,
                        });
                    }
                }
            }
        }
        if let Some((span, name)) = stray.spec_entries.first() {
            let n = stray.spec_entries.len();
            let example = stray.specs.first().map_or_else(
                || format!("{name}: 5V"),
                |f| {
                    let value = match &f.value {
                        SpecValue::Lit(v, _) => v.text.clone(),
                        SpecValue::GenericRef(g) => g.name.clone(),
                    };
                    format!("{}: {}", f.name.name, value)
                },
            );
            let more = if n > 1 { ", …" } else { "" };
            let existing = def.spec_blocks.iter().position(|b| b.variant.is_none());
            let help = match existing {
                Some(_) => format!(
                    "move {} into the device's existing `spec {{ … }}` block, written `{example}`; a second `spec` block would be a duplicate (E201)",
                    if n > 1 { "these entries" } else { "this entry" }
                ),
                None => format!("write `spec {{ {example}{more} }}`"),
            };
            self.report(
                Diagnostic::error(
                    "E010",
                    *span,
                    format!(
                        "spec field `{name}` is written directly in the device body{} — device specs are declared inside a `spec {{ … }}` block",
                        and_more(n, "in this device")
                    ),
                )
                .with_help(help),
            );
            if !stray.specs.is_empty() {
                match existing {
                    Some(i) => def.spec_blocks[i].fields.extend(stray.specs),
                    None => {
                        let span = stray.specs[0]
                            .span
                            .to(stray.specs[stray.specs.len() - 1].span);
                        def.spec_blocks.push(SpecBlock {
                            variant: None,
                            fields: stray.specs,
                            span,
                        });
                    }
                }
            }
        }
    }

    /// The trait-body counterpart of `finish_stray_device_members`.
    fn finish_stray_trait_members(&mut self, def: &mut TraitDef, stray: StrayTraitMembers) {
        if let Some((span, name)) = stray.pin_entries.first() {
            let n = stray.pin_entries.len();
            let example = stray.pins.first().map_or_else(
                || format!("required {name}: pin"),
                |p| format!("{} {}: pin", p.obligation.keyword(), p.name.name),
            );
            let more = if n > 1 { " …" } else { "" };
            self.report(
                Diagnostic::error(
                    "E010",
                    *span,
                    format!(
                        "pin role `{name}` is written directly in the trait body{} — trait pins are declared inside a `pins {{ … }}` block",
                        and_more(n, "in this trait")
                    ),
                )
                .with_help(format!(
                    "write `pins {{ {example}{more} }}` — one `[required|optional] NAME: pin` entry per line inside the block"
                )),
            );
            if def.pins_span.is_none() && !stray.pins.is_empty() {
                def.pins_span = Some(stray.pins[0].span.to(stray.pins[stray.pins.len() - 1].span));
                def.pins = stray.pins;
            }
        }
        if let Some((span, name)) = stray.spec_entries.first() {
            let n = stray.spec_entries.len();
            let example = stray.specs.first().map_or_else(
                || format!("{name}: Voltage"),
                |f| format!("{}: {}", f.name.name, f.ty.unit.type_name()),
            );
            let more = if n > 1 { ", …" } else { "" };
            self.report(
                Diagnostic::error(
                    "E010",
                    *span,
                    format!(
                        "spec field `{name}` is written directly in the trait body{} — trait specs are declared inside a `spec {{ … }}` block",
                        and_more(n, "in this trait")
                    ),
                )
                .with_help(format!("write `spec {{ {example}{more} }}`")),
            );
            if def.spec_span.is_none() && !stray.specs.is_empty() {
                def.spec_span = Some(
                    stray.specs[0]
                        .span
                        .to(stray.specs[stray.specs.len() - 1].span),
                );
                def.specs = stray.specs;
            }
        }
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
    ///
    /// Two slips are recovered rather than cascaded — `NAME: required …`
    /// (the obligation after the colon) and `NAME: [1, 2] [role]` (numbers
    /// bracketed like the role). The entry still parses to its intended
    /// meaning; the slip lands in `slips` and is reported once per block.
    fn device_pin(&mut self, slips: &mut PinSlips) -> Option<DevicePin> {
        let start = self.span();
        let obligation = self.obligation();
        let name = self.ident("as the pin name")?;
        let colon = self.expect(&TokenKind::Colon, "after the pin name");
        self.device_pin_value(start, obligation, name, colon, slips)
    }

    /// The rest of a device pin entry once `[obligation] NAME :` is read
    /// (`colon`: whether the `:` was actually there).
    fn device_pin_value(
        &mut self,
        start: Span,
        mut obligation: Obligation,
        name: Ident,
        colon: bool,
        slips: &mut PinSlips,
    ) -> Option<DevicePin> {
        let mut slipped = false;
        // The slip is the obligation right AFTER the `:`. Not a slip:
        // `required`/`optional` + `NAME :`, which is the NEXT entry (this
        // one has no numbers — `required A:` on a line of its own), or one
        // standing where the `:` is missing (`A required: 1`). Both are
        // reported by the number list below as they are.
        let next_entry = matches!(self.peek_ahead(1), TokenKind::Ident(_))
            && self.peek_ahead(2) == &TokenKind::Colon;
        if colon && matches!(self.peek(), TokenKind::Required | TokenKind::Optional) && !next_entry
        {
            let span = self.span();
            obligation = self.obligation();
            slips
                .obligation_after_name
                .push((span, name.name.clone(), obligation));
            slipped = true;
        }
        // A pad name is uppercase and a role lowercase, so `[` + number or
        // pad name is a bracketed number list, never the role bracket.
        let bracketed = self.at(&TokenKind::LBracket)
            && match self.peek_ahead(1) {
                TokenKind::Number(_) => true,
                TokenKind::Ident(n) => is_pad_name(n),
                _ => false,
            };
        let bracket = bracketed.then(|| self.bump().span); // `[`
        let mut numbers = Vec::new();
        loop {
            match self.peek() {
                // RFC-033: a signed pin number is E102, the pre-RFC code —
                // `legacy_number` owns the exact wording/span.
                TokenKind::Minus if matches!(self.peek_ahead(1), TokenKind::Number(_)) => {
                    let _ = self.legacy_number("as a physical pin number");
                    return None;
                }
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
        if let Some(span) = bracket {
            // Only a list that closes is the slip; an unclosed `[` is one
            // "expected `]`" error, not a guess at what was meant.
            if !self.expect(&TokenKind::RBracket, "to close the bracketed pin numbers") {
                return None;
            }
            slips.bracketed_numbers.push((span, name.name.clone()));
            slipped = true;
        }
        let mut role = None;
        if self.at(&TokenKind::LBracket) {
            self.bump();
            let role_ident = self.ident("as the pin role")?;
            match PinRole::from_name(&role_ident.name) {
                Some(r) => role = Some((r, role_ident.span)),
                None => {
                    self.report(
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
            self.report(
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
        let pin = DevicePin {
            obligation,
            name,
            numbers,
            role,
            span: start.to(self.prev_span()),
        };
        if slipped && slips.example.is_none() {
            slips.example = Some(canonical_pin(&pin));
        }
        Some(pin)
    }

    /// Report the slips `device_pin` recovered in one `pins { }` block — one
    /// diagnostic per kind, naming the first entry and counting the rest, so
    /// a 50-pin device written in the wrong shape yields two lines, not 100.
    fn report_pin_slips(&mut self, slips: &PinSlips) {
        let example = slips
            .example
            .as_ref()
            .map(|e| format!("write the entry as `{e}`"));
        if let Some((span, pin, obligation)) = slips.obligation_after_name.first() {
            let kw = obligation.keyword();
            let mut d = Diagnostic::error(
                "E010",
                *span,
                format!(
                    "`{kw}` is written after the pin name `{pin}`{} — the obligation comes first: `{kw} {pin}: …`",
                    and_more(slips.obligation_after_name.len(), "in this block")
                ),
            );
            if let Some(e) = &example {
                d = d.with_help(e.clone());
            }
            self.report(d);
        }
        if let Some((span, pin)) = slips.bracketed_numbers.first() {
            let mut d = Diagnostic::error(
                "E010",
                *span,
                format!(
                    "the pin numbers of `{pin}` are bracketed{} — they are a bare comma-separated list; only the role takes brackets: `{pin}: 1, 2 [passive]`",
                    and_more(slips.bracketed_numbers.len(), "in this block")
                ),
            );
            if let Some(e) = &example {
                d = d.with_help(e.clone());
            }
            self.report(d);
        }
    }

    fn device_spec_field(&mut self) -> Option<DeviceSpecField> {
        let start = self.span();
        let name = self.ident("as the spec field name")?;
        self.expect(&TokenKind::Colon, "after the spec field name");
        self.device_spec_value(start, name)
    }

    /// The value of a device spec field once `NAME :` is read.
    fn device_spec_value(&mut self, start: Span, name: Ident) -> Option<DeviceSpecField> {
        // RFC-033: `-5V`-style signed literals resolve through the SAME
        // `signed_unit_literal` every other legacy unit position uses (E105
        // for unsigned types) — one source of truth, no duplicated block.
        if self.at(&TokenKind::Minus) && matches!(self.peek_ahead(1), TokenKind::Unit(_)) {
            if let Some((v, span)) = self.signed_unit_literal() {
                return Some(DeviceSpecField {
                    name,
                    value: SpecValue::Lit(v, span),
                    span: start.to(self.prev_span()),
                });
            }
            return None;
        }
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
                self.report(
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
            // RFC-033: `const N: Int [= LITERAL]` — the `const` keyword
            // PRECEDES the name. Contextual: anything else is the legacy
            // `NAME: Bound` shape, so `const`-named instances keep working.
            let is_const_int = self.at_ident("const")
                && matches!(self.peek_ahead(1), TokenKind::Ident(_))
                && self.peek_ahead(2) == &TokenKind::Colon;
            if is_const_int {
                self.bump(); // const
                let Some(name) = self.ident("as the const parameter name") else {
                    break;
                };
                self.expect(&TokenKind::Colon, "after the const parameter name");
                let ty_span = self.span();
                let Some(ty_id) = self.ident("as the const parameter type (`Int`)") else {
                    break;
                };
                if ty_id.name != "Int" {
                    self.report(Diagnostic::error(
                        "E406",
                        ty_id.span,
                        format!("`const` parameters are `Int` — `{}` is not", ty_id.name),
                    ));
                    break;
                }
                let mut default = None;
                if self.eat(&TokenKind::Eq) {
                    match self.peek() {
                        TokenKind::Number(_) => {
                            let t = self.bump();
                            let TokenKind::Number(n) = t.kind else {
                                unreachable!()
                            };
                            default = self
                                .checked_int(&n, t.span)
                                .map(|v| GenericDefault::Int(v, t.span));
                        }
                        TokenKind::Minus if matches!(self.peek_ahead(1), TokenKind::Number(_)) => {
                            let minus = self.bump();
                            let t = self.bump();
                            let TokenKind::Number(n) = t.kind else {
                                unreachable!()
                            };
                            let span = minus.span.to(t.span);
                            default = self
                                .checked_int(&format!("-{n}"), span)
                                .map(|v| GenericDefault::Int(v, span));
                        }
                        other => {
                            // An Int default must be an integer literal — a
                            // unit literal (`2mm`) is a kind error (E406).
                            self.report(Diagnostic::error(
                                "E406",
                                self.span(),
                                format!(
                                    "an Int default must be an integer literal — `{}` is {}",
                                    other.describe(),
                                    if matches!(other, TokenKind::Unit(_)) {
                                        "a unit value"
                                    } else {
                                        "not an integer"
                                    }
                                ),
                            ));
                        }
                    }
                }
                params.push(GenericParam {
                    span: start.to(self.prev_span()),
                    name,
                    bound: GenericBound::Int(ty_span),
                    default,
                });
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                continue;
            }
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
                // RFC-033: signed unit defaults (`-5V` → E105) resolve through
                // the shared `signed_unit_literal` (single sign-rule source).
                if self.at(&TokenKind::Minus) && matches!(self.peek_ahead(1), TokenKind::Unit(_)) {
                    if let Some((v, span)) = self.signed_unit_literal() {
                        default = Some(GenericDefault::Unit(v, span));
                    }
                } else {
                    match self.peek() {
                        TokenKind::Unit(_) => {
                            let t = self.bump();
                            let TokenKind::Unit(v) = t.kind else {
                                unreachable!()
                            };
                            default = Some(GenericDefault::Unit(v, t.span));
                        }
                        TokenKind::Number(_) => {
                            let t = self.bump();
                            self.report(
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
            // RFC-033: an argument starting with a Length literal, `(`
            // or `+`, or a number/name/unit FOLLOWED by an arithmetic
            // operator, is a full expression (`bank::<1 + 1, 2mm * 2>`).
            // A bare number stays `GenericArg::Number` (E113's precise
            // report at type check); a NON-Length unit literal stays
            // `GenericArg::Unit` (`MLCC<100nF, 16V, 10%>`); a bare ident
            // stays a Name; a name followed by `.len` is an expression.
            // Re-review: a leading `-` byte-adjacent to a TEMPERATURE unit
            // literal immediately followed by `,` or `>` is the legacy
            // signed bare literal (`Td<-40C>`) via `signed_unit_literal`.
            // Length keeps the expression path so `-1mm + 2mm` arithmetic
            // is not truncated; other negative units keep their original
            // path (expression grammar / E105).
            let signed_temperature_bare = self.at(&TokenKind::Minus)
                && matches!(self.peek_ahead(1), TokenKind::Unit(v) if v.unit == UnitType::Temperature)
                && {
                    let idx = self.pos;
                    idx + 1 < self.tokens.len()
                        && self.tokens[idx].span.end == self.tokens[idx + 1].span.start
                }
                && matches!(
                    self.tokens.get(self.pos + 2).map(|t| &t.kind),
                    Some(TokenKind::Comma) | Some(TokenKind::Gt)
                );
            let op_ahead = matches!(
                self.peek_ahead(1),
                TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Percent
            );
            let starts_expr = match self.peek() {
                TokenKind::LParen | TokenKind::Plus => true,
                TokenKind::Minus => !signed_temperature_bare,
                TokenKind::Number(_) => op_ahead,
                TokenKind::Unit(_) => {
                    op_ahead
                        || matches!(self.peek(), TokenKind::Unit(v) if v.unit == UnitType::Length)
                }
                TokenKind::Ident(_) => op_ahead || matches!(self.peek_ahead(1), TokenKind::Dot),
                _ => false,
            };
            if signed_temperature_bare {
                match self.signed_unit_literal() {
                    Some((v, span)) => args.push(GenericArg::Unit(v, span)),
                    // E105 already reported (unsigned type made negative)
                    None => break,
                }
            } else if starts_expr {
                match self.expr() {
                    Some(e) => args.push(GenericArg::Expr(e)),
                    None => break,
                }
            } else {
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
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
            let mut progress = Progress::default();
            while self.block_continues(&mut progress) {
                // A broken entry skips to its own `,`/`}` and the block
                // continues — `break` left the rest of the block to the
                // impl-body loop, one "only contains" error per token.
                let Some(role) = self.ident("as the trait's required name") else {
                    self.sync_in_block();
                    self.eat(&TokenKind::Comma);
                    continue;
                };
                self.expect(&TokenKind::Colon, "in the mapping");
                let Some(map_target) = self.ident("as the device's own name") else {
                    self.sync_in_block();
                    self.eat(&TokenKind::Comma);
                    continue;
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
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
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
            let mut progress = Progress::default();
            while self.block_continues(&mut progress) {
                // A broken field skips to its own `,`/`}` and the entry
                // continues — `break` left the rest of the entry to the
                // part-body loop, one "expected `primary`" error per field.
                let Some(fname) = self.ident("as the AVL field name (e.g. `mpn`)") else {
                    self.sync_in_block();
                    self.eat(&TokenKind::Comma);
                    continue;
                };
                self.expect(&TokenKind::Colon, "after the AVL field name");
                // RFC-017: `footprint:` takes a SYMBOL reference (resolved
                // via RFC-016) — never a string.
                if fname.name == "footprint" {
                    match self.peek() {
                        TokenKind::Ident(_) => {
                            if let Some(sym) = self.path_ident("as the footprint symbol") {
                                if let Some(prev) = &footprint {
                                    self.report(
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
                            self.report(
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
                            self.sync_in_block();
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
                        self.sync_in_block();
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
                    self.report(Diagnostic::error(
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
            self.report(
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

    /// RFC-032 `subdesign NAME<G…> { ports { … } … }` — the declaration.
    fn subdesign_def(&mut self) -> Option<SubdesignDef> {
        self.bump(); // subdesign
        let name = self.ident("as the subdesign name")?;
        let generics = if self.at(&TokenKind::Lt) {
            self.generic_params()
        } else {
            Vec::new()
        };
        if !self.expect(&TokenKind::LBrace, "to open the subdesign body") {
            self.sync_top_level();
            return None;
        }
        let mut ports: Vec<SubdesignPort> = Vec::new();
        let mut ports_span: Option<Span> = None;
        let mut body = Vec::new();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if self.at_ident("ports") && self.peek_ahead(1) == &TokenKind::LBrace {
                let block_start = self.span();
                self.bump(); // ports
                self.bump(); // {
                if ports_span.is_some() {
                    self.report(Diagnostic::error(
                        "E010",
                        block_start,
                        "a subdesign has exactly one `ports { … }` block".to_string(),
                    ));
                }
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
                    let entry_start = self.span();
                    let obligation = self.obligation();
                    let Some(pname) = self.ident("as the port name") else {
                        self.sync_in_block_advancing();
                        continue;
                    };
                    self.expect(&TokenKind::Colon, "after the port name");
                    match self.ident("as the port type") {
                        // The one port type: `Pin`. Ports reuse RFC-002's pin
                        // semantics; there is nothing else a port could be.
                        Some(t) if t.name == "Pin" => {}
                        Some(t) => {
                            self.report(Diagnostic::error(
                                "E1303",
                                t.span,
                                format!(
                                    "`{}` is not a port type — every subdesign port is `Pin`-typed",
                                    t.name
                                ),
                            ));
                        }
                        None => {
                            self.sync_in_block_advancing();
                            continue;
                        }
                    }
                    ports.push(SubdesignPort {
                        obligation,
                        name: pname,
                        span: entry_start.to(self.prev_span()),
                    });
                    if !self.eat(&TokenKind::Comma) && !self.at(&TokenKind::RBrace) {
                        // Newline-separated entries are fine; anything else
                        // resynchronizes at the next comma/brace.
                        if !matches!(
                            self.peek(),
                            TokenKind::Ident(_) | TokenKind::Required | TokenKind::Optional
                        ) {
                            self.sync_in_block_advancing();
                        }
                    }
                }
                self.expect(&TokenKind::RBrace, "to close the ports block");
                ports_span.get_or_insert(block_start.to(self.prev_span()));
                continue;
            }
            if let Some(stmt) = self.stmt() {
                body.push(stmt);
            } else {
                self.sync_stmt();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the subdesign body");
        Some(SubdesignDef {
            name,
            generics,
            ports,
            ports_span,
            body,
        })
    }

    /// RFC-032 use site (statement position). The caller verified the shape
    /// `subdesign IDENT :`.
    fn subdesign_use_stmt(&mut self, intent: Option<(String, Span)>) -> Option<Stmt> {
        let start = self.span();
        self.bump(); // subdesign
        let name = self.ident("as the use-site name")?;
        self.expect(&TokenKind::Colon, "after the use-site name");
        // RFC-024 array form, exactly as `inst` spells it.
        let (ty, array_len) = if self.at(&TokenKind::LBracket) {
            let open = self.span();
            self.bump();
            let ty = self.type_ref()?;
            self.expect(
                &TokenKind::Semi,
                "between the subdesign type and the array length",
            );
            let len_expr = self.expr()?;
            self.expect(&TokenKind::RBracket, "to close the array type");
            let span = open.to(self.prev_span());
            if let Expr::Int(n, nspan) = &len_expr {
                if *n < 1 {
                    self.report(Diagnostic::error(
                        "E211",
                        *nspan,
                        format!("array length `{}` must be 1 or more", n),
                    ));
                    return None;
                }
            }
            // RFC-033: the length rides as an expression node.
            (ty, Some((len_expr, span)))
        } else {
            (self.type_ref()?, None)
        };
        let mut conns = Vec::new();
        if self.at(&TokenKind::LBrace) {
            let block_start = self.span();
            self.bump();
            let mut progress = Progress::default();
            while self.block_continues(&mut progress) {
                let entry_start = self.span();
                let Some(port) = self.ident("as the port name") else {
                    self.sync_in_block_advancing();
                    continue;
                };
                self.expect(&TokenKind::Colon, "after the port name");
                let Some(value) = self.pin_ref() else {
                    self.sync_in_block_advancing();
                    continue;
                };
                conns.push(PortConn {
                    port,
                    value,
                    span: entry_start.to(self.prev_span()),
                });
                // Newline-separated entries (the canonical form) carry no
                // comma; only a genuinely malformed continuation resyncs.
                if !self.eat(&TokenKind::Comma)
                    && !self.at(&TokenKind::RBrace)
                    && !matches!(self.peek(), TokenKind::Ident(_))
                {
                    self.sync_in_block_advancing();
                }
            }
            self.expect(&TokenKind::RBrace, "to close the port-connection block");
            if array_len.is_some() && !conns.is_empty() {
                self.report(Diagnostic::error(
                    "E1303",
                    block_start.to(self.prev_span()),
                    format!(
                        "an array-typed use site connects through `net` statements (`{}[i].PORT`) — a port block would bind every element to the same pins",
                        name.name
                    ),
                ));
                conns.clear();
            }
        }
        Some(Stmt::SubdesignUse(SubdesignUseStmt {
            intent,
            name,
            array_len,
            ty,
            conns,
            span: start.to(self.prev_span()),
        }))
    }

    fn stmt_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if let Some(stmt) = self.stmt() {
                stmts.push(stmt);
            } else {
                self.sync_stmt();
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

    /// The loop condition of EVERY `{ … }` body loop in this parser: true
    /// while the cursor is still inside the block (not at its `}` or EOF).
    ///
    /// It is also the parser's termination guarantee. Recovery stops *at*
    /// the delimiter it finds (`sync_in_block` at a `,`) so the caller can
    /// decide what to do with it, and a body loop that re-enters the same
    /// recovery on that same token never ends — not a slow parse but a hang
    /// that appends a diagnostic per pass until the OS kills the process.
    /// cohdl 0.8.0 shipped five such shapes (`device X { , }`, `trait T {
    /// , }`, a comma after `designator_prefix: "C"` or between device
    /// blocks, pin entries written outside `pins { }`) after the part,
    /// impl, and variants loops had been fixed one call site at a time
    /// (`malformed_input_never_spins_in_recovery`). Valid input consumes at
    /// least one token per iteration, so an iteration that ended where it
    /// began has already reported an error: consume the token it stalled
    /// on before testing the condition again. Being the condition, this
    /// runs after every iteration — `continue` included — so a new body
    /// loop gets the guarantee by construction. (A unit test fails if a
    /// body loop is written with a bare `!self.at(RBrace)` condition.)
    fn block_continues(&mut self, progress: &mut Progress) -> bool {
        let inside = |p: &Self| !p.at(&TokenKind::RBrace) && !p.at(&TokenKind::Eof);
        if !inside(self) {
            return false;
        }
        let stalled = progress.0 == Some(self.pos);
        #[cfg(test)]
        let stalled = stalled && test_hooks::guard_on();
        if stalled {
            #[cfg(test)]
            test_hooks::guard_fired();
            self.bump();
            if !inside(self) {
                return false;
            }
        }
        progress.0 = Some(self.pos);
        true
    }

    /// Block-level recovery that always moves past a stray `,`.
    ///
    /// `sync_in_block` deliberately stops *at* the `,`/`}` it finds so the
    /// caller can decide what to do with the delimiter. When it could not
    /// move at all, the cursor is on a `,` the caller has no use for:
    /// consume it so the next pass starts on a genuinely new token. A `}`
    /// is never consumed — every caller is a body loop that stops there,
    /// and eating it ran recovery on into the enclosing block (`ports {
    /// required }` swallowed the rest of the subdesign). `block_continues`
    /// is the loop-level backstop; this keeps the recovery itself tight.
    fn sync_in_block_advancing(&mut self) {
        let before = self.pos;
        self.sync_in_block();
        let stuck = self.pos == before && !self.at(&TokenKind::RBrace);
        #[cfg(test)]
        let stuck = stuck && test_hooks::advancing_on();
        if stuck {
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
            || (self.at_ident("subdesign") && matches!(self.peek_ahead(1), TokenKind::Ident(_)))
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
        // RFC-033 `const NAME: Int|Length = EXPR` — `const` is contextual: the
        // three-token shape (Ident "const", Ident, Colon) never collides with
        // a `const`-named instance being declared.
        if matches!(self.peek(), TokenKind::Ident(n) if n == "const")
            && matches!(self.peek_ahead(1), TokenKind::Ident(_))
            && self.peek_ahead(2) == &TokenKind::Colon
        {
            self.reject_attrs(&attrs);
            self.reject_phys(&phys, "a `const`");
            return self.const_stmt().map(Stmt::Const);
        }
        // RFC-033 `for LABEL: binder in EXPR..EXPR { … }` — `for` was already
        // a keyword (impl for).
        if self.at(&TokenKind::For) {
            self.reject_attrs(&attrs);
            self.reject_phys(&phys, "a `for` loop");
            return self.for_stmt().map(Stmt::For);
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
                    if a.name.name != "designator" {
                        self.report(Diagnostic::error(
                            "E010",
                            a.span,
                            format!(
                                "unrecognized attribute `{}` (an `inst` takes `#[designator(\"…\")]`, `#[intent(\"…\")]`, or `#[placement_hint(\"…\")]`)",
                                a.name.name
                            ),
                        ));
                    }
                }
                let attrs: Vec<Attr> = attrs
                    .into_iter()
                    .filter(|a| a.name.name == "designator")
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
                    // RFC-033: the array length is a full expression.
                    let len_expr = self.expr()?;
                    self.expect(&TokenKind::RBracket, "to close the array type");
                    let span = open.to(self.prev_span());
                    // The literal fast-path keeps the pre-RFC E211 checks
                    // byte-identical; a computed length is judged at
                    // expansion (Task 7) — never here.
                    if let Expr::Int(n, nspan) = &len_expr {
                        if *n < 1 {
                            self.report(Diagnostic::error(
                                "E211",
                                *nspan,
                                format!("array length `{}` must be 1 or more", n),
                            ));
                            return None;
                        }
                    }
                    (ty, Some((len_expr, span)))
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
                                self.report(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
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
                self.report(Diagnostic::error(
                    "E010",
                    span,
                    "`use` imports are file-level — move it above the design/fn body".to_string(),
                ));
                // Consume the statement so it can't misparse as a call.
                let _ = self.use_decl();
                None
            }
            // RFC-032 use site: `subdesign local: Name<…> { PORT: t, … }` or
            // the array form `subdesign local: [Name; N]`.
            TokenKind::Ident(n)
                if n == "subdesign"
                    && matches!(self.peek_ahead(1), TokenKind::Ident(_))
                    && self.peek_ahead(2) == &TokenKind::Colon =>
            {
                self.reject_phys(&phys, "a `subdesign` use site");
                self.reject_attrs(&attrs);
                self.subdesign_use_stmt(intent)
            }
            TokenKind::Ident(n)
                if n == "subdesign" && matches!(self.peek_ahead(1), TokenKind::Ident(_)) =>
            {
                self.reject_attrs(&attrs);
                let span = self.span();
                self.report(Diagnostic::error(
                    "E010",
                    span,
                    "`subdesign` declarations are top-level — inside a body, a use site reads `subdesign name: Type { … }`"
                        .to_string(),
                ));
                // Consume the misplaced declaration so the body keeps parsing.
                let _ = self.subdesign_def();
                None
            }
            TokenKind::Ident(_) => {
                self.reject_phys(&phys, "a `fn` call");
                self.reject_attrs(&attrs);
                // The callee may be a qualified path; `::<` stays turbofish
                // (path_ident's two-token lookahead never eats `::` + `<`).
                // `?`, not `unwrap`: either reject above can be the error
                // that exhausts the budget, which moves the cursor to EOF
                // (see `report`).
                let callee = self.path_ident("")?;
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
            self.report(Diagnostic::error(
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
        let mut consts = Vec::new();
        let mut loops = Vec::new();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            // RFC-033: layout consts and labelled placement loops.
            if matches!(self.peek(), TokenKind::Ident(n) if n == "const")
                && matches!(self.peek_ahead(1), TokenKind::Ident(_))
                && self.peek_ahead(2) == &TokenKind::Colon
            {
                if let Some(c) = self.const_stmt() {
                    consts.push(c);
                }
            } else if self.at(&TokenKind::For) {
                if let Some(l) = self.layout_for() {
                    loops.push(l);
                }
            } else if self.at_ident("board_outline") {
                match (&board_outline, self.board_outline()) {
                    (Some(prev), Some(next)) => self.report(
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
        }
        self.expect(&TokenKind::RBrace, "to close the layout block");
        Some(Stmt::Layout(LayoutBlock {
            constraints,
            board_outline,
            placements,
            consts,
            loops,
            span: start.to(self.prev_span()),
        }))
    }

    /// RFC-033 `for LABEL: binder in EXPR..EXPR { place/const/for }` — a
    /// layout loop admits only `const`, `place` and nested `for` (E1406 for
    /// anything else; static validation lands with Task 9, but the shape is
    /// rejected right here so the grammar stays closed).
    fn layout_for(&mut self) -> Option<LayoutFor> {
        self.nested(Self::layout_for_inner)
    }

    fn layout_for_inner(&mut self) -> Option<LayoutFor> {
        let (label, binder, start_e, end_e, start) = self.for_header()?;
        self.expect(&TokenKind::LBrace, "to open the layout loop body");
        let mut consts = Vec::new();
        let mut placements = Vec::new();
        let mut loops = Vec::new();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if matches!(self.peek(), TokenKind::Ident(n) if n == "const")
                && matches!(self.peek_ahead(1), TokenKind::Ident(_))
                && self.peek_ahead(2) == &TokenKind::Colon
            {
                if let Some(c) = self.const_stmt() {
                    consts.push(c);
                }
            } else if self.at(&TokenKind::For) {
                if let Some(l) = self.layout_for() {
                    loops.push(l);
                }
            } else if self.at_ident("place") {
                if let Some(p) = self.placement() {
                    placements.push(p);
                }
            } else {
                self.report(Diagnostic::error(
                    "E1406",
                    self.span(),
                    format!(
                        "`{}` is not admitted inside a layout loop — only `const`, `place` and nested `for`",
                        self.peek().describe()
                    ),
                ));
                self.bump();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the layout loop body");
        Some(LayoutFor {
            label,
            binder,
            start: start_e,
            end: end_e,
            consts,
            placements,
            loops,
            span: start.to(self.prev_span()),
        })
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

    /// `place <path> at (x, y) [rotate ANGLE]` (RFC-020/032) — a locked,
    /// optionally rotated placement. The target is a dotted path (RFC-032
    /// reach-in); existence, coordinate unit-type, and the rotation's 0..=359
    /// range are validated at assembly (E1007/E1305).
    fn placement(&mut self) -> Option<Placement> {
        let start = self.span();
        self.bump(); // `place`
        let mut path = Vec::new();
        loop {
            let name = self.ident(if path.is_empty() {
                "as the instance to place"
            } else {
                "as the next segment of the placement path"
            })?;
            // RFC-024: `NAME[i]` — always exactly ONE element; a range has no
            // single sensible meaning here (each element needs its own
            // coordinates).
            let index = if self.at(&TokenKind::LBracket) {
                match self.index_sel()? {
                    IndexSel::Single(e, sp) => Some((e, sp)),
                    other => {
                        self.report(Diagnostic::error(
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
            path.push(PlacementSeg { name, index });
            if !self.eat(&TokenKind::Dot) {
                break;
            }
        }
        if !self.at_ident("at") {
            self.error_here("expected `at (x, y)` after the instance name".to_string());
            return None;
        }
        self.bump(); // `at`
        self.expect(&TokenKind::LParen, "to open the coordinate pair");
        // RFC-033: coordinates are full expressions (`10mm + n * PITCH`).
        let x = self.expr()?;
        self.expect(&TokenKind::Comma, "between the coordinates");
        let y = self.expr()?;
        self.expect(&TokenKind::RParen, "to close the coordinate pair");
        let at = (x, y);
        // Optional `rotate ANGLE` (E1007) and `side SIDE` (RFC-026, E1008) —
        // independent clauses, accepted in either order per the accepted text;
        // `fmt` canonicalizes to rotate-then-side.
        let mut rotate = None;
        let mut saw_rotate = false;
        let mut side = crate::ast::PlacementSide::Top;
        let mut side_span = None;
        loop {
            if !saw_rotate && self.at_ident("rotate") {
                saw_rotate = true;
                self.bump(); // `rotate`
                             // RFC-033: the angle is a full expression (`90 * n`).
                match self.peek() {
                    TokenKind::Number(_)
                    | TokenKind::Unit(_)
                    | TokenKind::LParen
                    | TokenKind::Ident(_)
                    | TokenKind::Minus
                    | TokenKind::Plus => {
                        rotate = Some(self.expr()?);
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
                        self.report(Diagnostic::error(
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
            path,
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
                let mut progress = Progress::default();
                while self.block_continues(&mut progress) {
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
                                self.report(Diagnostic::error(
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
                            self.report(Diagnostic::error(
                                "E1009",
                                k.span,
                                format!("duplicate diff_pair field `{}`", k.name),
                            ));
                            return None;
                        }
                        let v = self.unit_literal("as the field value")?;
                        if v.unit != expected {
                            self.report(Diagnostic::error(
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
                    self.report(
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

    // -- expressions (RFC-033) ----------------------------------------------

    fn const_stmt(&mut self) -> Option<ConstStmt> {
        let start = self.span();
        self.bump(); // const
        let name = self.ident("as the constant name")?;
        self.expect(&TokenKind::Colon, "after the constant name");
        let ty_id = self.ident("as the constant type (`Int` or `Length`)")?;
        let ty = match ty_id.name.as_str() {
            "Int" => ConstTy::Int,
            "Length" => ConstTy::Length,
            other => {
                self.report(Diagnostic::error(
                    "E1401",
                    ty_id.span,
                    format!(
                        "`{}` is not a constant type — a `const` is `Int` or `Length`",
                        other
                    ),
                ));
                return None;
            }
        };
        self.expect(&TokenKind::Eq, "before the constant's value");
        let value = self.expr()?;
        Some(ConstStmt {
            name,
            ty,
            value,
            span: start.to(self.prev_span()),
        })
    }

    fn for_stmt(&mut self) -> Option<ForStmt> {
        self.nested(Self::for_stmt_inner)
    }

    fn for_stmt_inner(&mut self) -> Option<ForStmt> {
        let (label, binder, start_e, end_e, start) = self.for_header()?;
        self.expect(&TokenKind::LBrace, "to open the loop body");
        let mut body = Vec::new();
        let mut progress = Progress::default();
        while self.block_continues(&mut progress) {
            if let Some(s) = self.stmt() {
                body.push(s);
            }
        }
        self.expect(&TokenKind::RBrace, "to close the loop body");
        Some(ForStmt {
            label,
            binder,
            start: start_e,
            end: end_e,
            body,
            span: start.to(self.prev_span()),
        })
    }

    /// `for LABEL: IDENT in expr .. expr` — shared by body and layout loops.
    fn for_header(&mut self) -> Option<(Ident, Ident, Expr, Expr, Span)> {
        let start = self.span();
        self.bump(); // for
                     // Every loop is labelled (RFC-033 §6): `for LINKS: n in …`. The label
                     // is how each generated object's provenance is spelled, so its
                     // absence gets the maximally specific hint.
        let label = match self.peek() {
            TokenKind::Ident(_) => {
                let id = self
                    .ident("as the loop label (every loop is labelled: `for links: n in 0..N`)")?;
                if !self.expect(&TokenKind::Colon, "after the loop label") {
                    self.report(
                        Diagnostic::error(
                            "E010",
                            id.span,
                            format!(
                                "every loop is labelled — write `for {}: BINDER in 0..N {{ … }}`",
                                id.name
                            ),
                        )
                        .with_help("the label names this loop's generated objects (provenance), e.g. `for links: n in 0..N`".to_string()),
                    );
                    return None;
                }
                id
            }
            other => {
                let m = format!(
                    "expected a loop label (every loop is labelled: `for links: n in 0..N`), found {}",
                    other.describe()
                );
                self.error_here(m);
                return None;
            }
        };
        let binder = self.ident("as the loop variable")?;
        match self.peek() {
            TokenKind::Ident(n) if n == "in" => {
                self.bump();
            }
            other => {
                let m = format!(
                    "expected `in` after the loop variable, found {}",
                    other.describe()
                );
                self.error_here(m);
                return None;
            }
        }
        let lo = self.expr()?;
        if !self.expect(&TokenKind::DotDot, "as the half-open range delimiter `..` (loops are exclusive at the end; `..=` is only for net fan-out)") {
            return None;
        }
        let hi = self.expr()?;
        Some((label, binder, lo, hi, start))
    }

    fn expr(&mut self) -> Option<Expr> {
        self.expr_add(MAX_SYNTAX_DEPTH - self.nesting)
            .map(|(expr, _)| expr)
    }

    // Private expression routines carry exact subtree depth beside the node.
    // Each operator updates it in O(1); no rescanning or public AST metadata.
    // Reserving a slot before parsing each child bounds both descent and the
    // left-deep trees produced by these otherwise iterative precedence loops.
    fn expr_add(&mut self, budget: usize) -> Option<(Expr, usize)> {
        let (mut lhs, mut depth) = self.expr_mul(budget)?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            let operator = self.span();
            if depth == budget {
                return self.depth_exceeded(operator);
            }
            self.bump();
            let (rhs, rhs_depth) = self.expr_mul(budget - 1)?;
            depth = depth.max(rhs_depth) + 1;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Some((lhs, depth))
    }

    fn expr_mul(&mut self, budget: usize) -> Option<(Expr, usize)> {
        let (mut lhs, mut depth) = self.expr_unary(budget)?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Rem,
                _ => break,
            };
            let operator = self.span();
            if depth == budget {
                return self.depth_exceeded(operator);
            }
            self.bump();
            let (rhs, rhs_depth) = self.expr_unary(budget - 1)?;
            depth = depth.max(rhs_depth) + 1;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Some((lhs, depth))
    }

    fn expr_unary(&mut self, budget: usize) -> Option<(Expr, usize)> {
        if budget == 0 {
            return self.depth_exceeded(self.span());
        }
        let start = self.span();
        match self.peek() {
            TokenKind::Minus => {
                let minus = self.bump();
                // Signed-literal assembly: byte-adjacent number/unit.
                let adjacent = self.span().start == minus.span.end;
                match self.peek() {
                    TokenKind::Number(_) if adjacent => {
                        let t = self.bump();
                        let TokenKind::Number(text) = t.kind else {
                            unreachable!()
                        };
                        let span = minus.span.to(t.span);
                        return self
                            .checked_int(&format!("-{text}"), span)
                            .map(|n| (Expr::Int(n, span), 1));
                    }
                    TokenKind::Unit(_) if adjacent => {
                        let t = self.bump();
                        let TokenKind::Unit(v) = t.kind else {
                            unreachable!()
                        };
                        let span = minus.span.to(t.span);
                        // The E105 wording is exactly `negate_for_literal`'s
                        // (one source of truth for the sign rule).
                        match v.negate_for_literal() {
                            Ok(neg) => return Some((Expr::Length(neg, span), 1)),
                            Err(msg) => {
                                self.report(Diagnostic::error("E105", span, msg));
                                return None;
                            }
                        }
                    }
                    _ => {}
                }
                let (rhs, depth) = self.expr_unary(budget - 1)?;
                let span = start.to(rhs.span());
                Some((
                    Expr::Unary {
                        op: UnaryOp::Neg,
                        rhs: Box::new(rhs),
                        span,
                    },
                    depth + 1,
                ))
            }
            TokenKind::Plus => {
                self.bump();
                let (rhs, depth) = self.expr_unary(budget - 1)?;
                let span = start.to(rhs.span());
                Some((
                    Expr::Unary {
                        op: UnaryOp::Plus,
                        rhs: Box::new(rhs),
                        span,
                    },
                    depth + 1,
                ))
            }
            _ => self.expr_primary(budget),
        }
    }

    fn expr_primary(&mut self, budget: usize) -> Option<(Expr, usize)> {
        match self.peek() {
            TokenKind::Number(_) => {
                let t = self.bump();
                let TokenKind::Number(text) = t.kind else {
                    unreachable!()
                };
                self.checked_int(&text, t.span)
                    .map(|n| (Expr::Int(n, t.span), 1))
            }
            TokenKind::Unit(_) => {
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                if v.unit == UnitType::Length {
                    Some((Expr::Length(v, t.span), 1))
                } else {
                    // A non-Length unit in an expression position. `place`
                    // coordinates historically report E1007 AT CHECK (the
                    // fixture contract — `place r1 at (0mm, 3V)`), so the node
                    // parses and the check-side unit validation stays the
                    // single owner of that diagnostic. Everywhere else the
                    // expression kinds are wrong (E1401 at evaluation, Task 5).
                    Some((Expr::Length(v, t.span), 1))
                }
            }
            TokenKind::LParen => {
                let open = self.span();
                self.bump();
                let Some((inner, depth)) = self.expr_add(budget - 1) else {
                    // A rejected numeric literal has already consumed its token.
                    // Close its parentheses so a generic use does not acquire
                    // unrelated delimiter errors during recovery.
                    self.eat(&TokenKind::RParen);
                    return None;
                };
                self.expect(&TokenKind::RParen, "to close the parenthesized expression");
                Some((
                    Expr::Paren(Box::new(inner), open.to(self.prev_span())),
                    depth + 1,
                ))
            }
            TokenKind::Ident(_) => {
                let id = self.ident("in an expression")?;
                if self.at(&TokenKind::Dot)
                    && matches!(self.peek_ahead(1), TokenKind::Ident(n) if n == "len")
                {
                    self.bump();
                    let t = self.bump();
                    return Some((Expr::Len(id.clone(), id.span.to(t.span)), 1));
                }
                Some((Expr::Name(id), 1))
            }
            other => {
                let msg = format!(
                    "expected an Int or Length expression, found {}",
                    other.describe()
                );
                self.error_here(msg);
                None
            }
        }
    }

    /// RFC-033: legacy number positions (pin numbers, pad numbers,
    /// mount-hole numbers) keep E102 — a bare number may not carry a sign.
    fn legacy_number(&mut self, ctx: &str) -> Option<i64> {
        if self.at(&TokenKind::Minus) && matches!(self.peek_ahead(1), TokenKind::Number(_)) {
            let minus_span = self.span();
            let adjacent = {
                let idx = self.pos;
                idx + 1 < self.tokens.len()
                    && self.tokens[idx].span.end == self.tokens[idx + 1].span.start
            };
            if adjacent {
                self.bump(); // -
                let t = self.bump(); // the number itself
                self.report(Diagnostic::error(
                    "E102",
                    minus_span.to(t.span),
                    "a bare number cannot be negative — only `Temperature` and `Length` literals may carry a leading `-` (e.g. `-40C`, `-0.5mm`)".to_string(),
                ));
                let _ = ctx;
                return None;
            }
        }
        self.index_number(ctx)
    }

    /// RFC-033 signed unit literal for LEGACY positions (spec values, generic
    /// defaults, coordinates…): `-` byte-adjacent to a unit literal negates
    /// Temperature/Length, E105 otherwise. One source of truth — the pre-RFC
    /// inlined blocks in `unit_literal`/`device_spec_field` collapsed here.
    fn signed_unit_literal(&mut self) -> Option<(UnitValue, Span)> {
        if self.at(&TokenKind::Minus) && matches!(self.peek_ahead(1), TokenKind::Unit(_)) {
            let minus_span = self.span();
            let adjacent = {
                let idx = self.pos;
                idx + 1 < self.tokens.len()
                    && self.tokens[idx].span.end == self.tokens[idx + 1].span.start
            };
            if adjacent {
                self.bump(); // -
                let t = self.bump();
                let TokenKind::Unit(v) = t.kind else {
                    unreachable!()
                };
                return match v.negate_for_literal() {
                    Ok(v) => Some((v, minus_span.to(t.span))),
                    Err(msg) => {
                        self.report(Diagnostic::error("E105", minus_span.to(t.span), msg));
                        None
                    }
                };
            }
        }
        self.unit_literal_with_span("")
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
                        self.report(Diagnostic::error(
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
    /// RFC-033: every position is a full expression.
    fn index_sel(&mut self) -> Option<IndexSel> {
        let open = self.span();
        self.bump(); // `[`
        let first = self.expr()?;
        // `..=` lexes as DotDot Eq — the range form; anything else is a list.
        if self.at(&TokenKind::DotDot) {
            self.bump();
            self.expect(&TokenKind::Eq, "in the range `..=` (ranges are inclusive)");
            let end = self.expr()?;
            let mut step = None;
            if self.at_ident("step") {
                self.bump();
                step = Some(self.expr()?);
            }
            self.expect(&TokenKind::RBracket, "to close the index bracket");
            let span = open.to(self.prev_span());
            // Literal fast-path: the pre-RFC E211 checks stay byte-identical;
            // computed bounds are judged by Task 7's evaluator.
            if let (Some(f), Some(e)) = (first.as_int_literal(), end.as_int_literal()) {
                if e < f {
                    self.report(Diagnostic::error(
                        "E211",
                        span,
                        format!(
                            "range `{}..={}` is empty — the end must not be below the start",
                            f, e
                        ),
                    ));
                    return None;
                }
                if let Some(Expr::Int(s, _)) = &step {
                    if *s < 1 {
                        self.report(Diagnostic::error(
                            "E211",
                            span,
                            format!("stride `{}` must be 1 or more", s),
                        ));
                        return None;
                    }
                }
            }
            Some(IndexSel::Range {
                start: first,
                end,
                step,
                span,
            })
        } else {
            let mut items = vec![first];
            while self.eat(&TokenKind::Comma) {
                items.push(self.expr()?);
            }
            self.expect(&TokenKind::RBracket, "to close the index bracket");
            let span = open.to(self.prev_span());
            // `[i]` is the REAL reference form (valid everywhere); only a
            // comma-separated set is the net-member-only list sugar.
            if items.len() > 1 {
                Some(IndexSel::List(items, span))
            } else {
                Some(IndexSel::Single(items.pop().expect("one item"), span))
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

/// Slips `device_pin` recovers, collected per `pins { }` block (or per
/// device body, for entries written outside one) so each kind is reported
/// once with a count rather than once per line.
#[derive(Default)]
struct PinSlips {
    /// `NAME: required …` — the obligation after the colon.
    obligation_after_name: Vec<(Span, String, Obligation)>,
    /// `NAME: [1, 2] [role]` — the pin numbers bracketed.
    bracketed_numbers: Vec<(Span, String)>,
    /// The canonical spelling of the first slipped entry, for the help line.
    example: Option<String>,
}

/// Which block a member written straight into a trait/device body belongs
/// in (`IOVDD: required [1] [power_in]` at body level is a `pins` entry).
#[derive(Clone, Copy)]
enum StrayKind {
    Pin,
    Spec,
    /// Device body only: `NAME: -…` — a signed pin number (`A: -1 …`, the
    /// legacy E102) or a negative spec value (`v: -5V`). The token after
    /// the `-` decides, read once `NAME :` is consumed (2-token lookahead).
    Signed,
}

/// `pins`/`spec` entries found directly in a device body: where each began
/// (all of them, for the count) and the ones that parsed (for recovery).
#[derive(Default)]
struct StrayDeviceMembers {
    pin_entries: Vec<(Span, String)>,
    pins: Vec<DevicePin>,
    pin_slips: PinSlips,
    spec_entries: Vec<(Span, String)>,
    specs: Vec<DeviceSpecField>,
}

/// The trait-body counterpart of `StrayDeviceMembers`.
#[derive(Default)]
struct StrayTraitMembers {
    pin_entries: Vec<(Span, String)>,
    pins: Vec<TraitPin>,
    spec_entries: Vec<(Span, String)>,
    specs: Vec<TraitSpecField>,
}

/// ` (and N more entries PLACE)` when a slip repeats, else nothing.
fn and_more(n: usize, place: &str) -> String {
    match n {
        0 | 1 => String::new(),
        2 => format!(" (and 1 more entry {place})"),
        _ => format!(" (and {} more entries {place})", n - 1),
    }
}

/// A device pin entry in canonical form: `required IOVDD: 1, 10 [power_in]`.
fn canonical_pin(pin: &DevicePin) -> String {
    let numbers: Vec<&str> = pin.numbers.iter().map(|n| n.text.as_str()).collect();
    let role = pin.role.map_or("ROLE", |(r, _)| r.name());
    format!(
        "{} {}: {} [{}]",
        pin.obligation.keyword(),
        pin.name.name,
        numbers.join(", "),
        role
    )
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

    // -- termination guarantee (the cohdl 0.8.0 device/trait-body hang) ------
    //
    // `device X { , }` made 0.8.0 allocate ~4 GB/s with no output until the
    // OS killed it: the device- and trait-body loops re-entered
    // `sync_in_block` on the `,` it stops at. Every test here parses on a
    // worker thread with a deadline, so a regression FAILS instead of
    // hanging the suite; the per-file error budget bounds its memory.

    use std::sync::mpsc;
    use std::time::Duration;

    /// Lex + parse `src` on a worker thread; panic if it has not finished
    /// within `limit`. Returns the AST, the error count, and the rendering.
    fn parse_within(src: &str, limit: Duration) -> (SourceFile, usize, String) {
        let owned = src.to_string();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut sm = SourceMap::new();
            let f = sm.add_file("test.cohdl", owned.as_str());
            let mut diags = Diagnostics::new();
            let tokens = lex(f, &owned, &mut diags);
            let file = parse(tokens, &mut diags);
            let _ = tx.send((file, diags.error_count(), diags.render(&sm)));
        });
        rx.recv_timeout(limit)
            .unwrap_or_else(|e| panic!("parse did not finish ({e}) within {limit:?} for:\n{src}"))
    }

    const LIMIT: Duration = Duration::from_secs(10);

    #[test]
    fn stray_comma_in_a_device_or_trait_body_is_one_error() {
        for (src, found, blocks) in [
            ("device X { , }\n", "in the device body, found `,`", 0),
            ("trait T { , }\n", "in the trait body, found `,`", 0),
            // a trailing comma after a trait field, and between device blocks
            (
                "pub trait T {\n    designator_prefix: \"C\",\n    pins { required A: pin }\n}\n",
                "in the trait body, found `,`",
                1,
            ),
            (
                "pub device D {\n    pins { A: 1 [passive] },\n    spec { v: 5V }\n}\n",
                "in the device body, found `,`",
                2,
            ),
        ] {
            let (file, errors, rendered) = parse_within(src, LIMIT);
            assert_eq!(errors, 1, "{src}\n{rendered}");
            assert!(rendered.contains(found), "{rendered}");
            // ...and the blocks around the comma still parse.
            let parsed = match &file.items[0].kind {
                ItemKind::Device(d) => d.pin_blocks.len() + d.spec_blocks.len(),
                ItemKind::Trait(t) => t.pins.len() + t.specs.len(),
                _ => panic!("{src}"),
            };
            assert_eq!(parsed, blocks, "{src}\n{rendered}");
        }
    }

    // The shape a model wrote for a whole board (pd-meter, 51 RP2040 pins):
    // pin entries straight in the device body, obligation after the colon,
    // numbers bracketed like the role. One diagnostic per device, the
    // entry rewritten in canonical form, and the pins kept so later passes
    // check the device the author meant.
    #[test]
    fn pin_entries_outside_pins_get_one_targeted_diagnostic() {
        let src = "device X { A: required [1, 2] [passive] }\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(
            rendered.contains("pin entry `A` is written directly in the device body — device pins are declared inside a `pins { … }` block"),
            "{rendered}"
        );
        assert!(
            rendered.contains("write `pins { required A: 1, 2 [passive] }`"),
            "{rendered}"
        );
        assert!(rendered.contains("comes before the pin name"), "{rendered}");
        assert!(
            rendered.contains("only the role is bracketed"),
            "{rendered}"
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.pin_blocks.len(), 1);
        let pin = &d.pin_blocks[0].pins[0];
        assert_eq!(pin.name.name, "A");
        assert_eq!(pin.obligation, Obligation::Required);
        let numbers: Vec<&str> = pin.numbers.iter().map(|n| n.text.as_str()).collect();
        assert_eq!(numbers, ["1", "2"]);
        assert_eq!(pin.role.map(|(r, _)| r), Some(PinRole::Passive));
    }

    #[test]
    fn stray_members_are_counted_per_body_and_classified_by_value() {
        // Canonically spelled entries, just not in their blocks: no slip
        // hints, one diagnostic per kind, the count, and both adopted.
        let src = "pub device MCU<V: Voltage> {\n    required VDD: 1 [power_in]\n    GND: 2, 3 [power_in]\n    vmax: V\n    vmin: 1V\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 2, "{rendered}");
        assert!(rendered.contains("pin entry `VDD` is written directly in the device body (and 1 more entry in this device)"), "{rendered}");
        assert!(
            rendered.contains("write `pins { required VDD: 1 [power_in] … }`"),
            "{rendered}"
        );
        assert!(rendered.contains("spec field `vmax` is written directly in the device body (and 1 more entry in this device)"), "{rendered}");
        assert!(
            rendered.contains("write `spec { vmax: V, … }`"),
            "{rendered}"
        );
        assert!(
            !rendered.contains("comes before the pin name"),
            "{rendered}"
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.pin_blocks[0].pins.len(), 2);
        assert_eq!(d.spec_blocks[0].fields.len(), 2);

        let src =
            "pub trait T {\n    required A: pin\n    B: pin\n    capacitance: Capacitance\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 2, "{rendered}");
        assert!(rendered.contains("pin role `A` is written directly in the trait body (and 1 more entry in this trait)"), "{rendered}");
        assert!(
            rendered.contains("write `pins { required A: pin … }`"),
            "{rendered}"
        );
        assert!(
            rendered.contains("spec field `capacitance` is written directly in the trait body"),
            "{rendered}"
        );
        let ItemKind::Trait(t) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!((t.pins.len(), t.specs.len()), (2, 1));
    }

    #[test]
    fn misordered_pin_entries_inside_pins_are_recovered_once_per_block() {
        let src = "pub device X {\n    pins {\n        A: required [1, 2] [passive]\n        B: optional [3] [passive]\n    }\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 2, "{rendered}");
        assert!(
            rendered.contains("`required` is written after the pin name `A` (and 1 more entry in this block) — the obligation comes first: `required A: …`"),
            "{rendered}"
        );
        assert!(
            rendered
                .contains("the pin numbers of `A` are bracketed (and 1 more entry in this block)"),
            "{rendered}"
        );
        assert!(
            rendered.contains("write the entry as `required A: 1, 2 [passive]`"),
            "{rendered}"
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        let pins = &d.pin_blocks[0].pins;
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[1].obligation, Obligation::Optional);
        assert_eq!(pins[1].numbers[0].text, "3");
    }

    // `sync_in_block_advancing` used to consume the `}` it stalled on, so
    // a broken port entry ran recovery on through the subdesign's own
    // statements and into the next declaration.
    #[test]
    fn advancing_recovery_never_eats_the_closing_brace() {
        let src = "pub subdesign S {\n    ports { required }\n    inst r: R\n}\n\npub device R {\n    pins { A: 1 [passive] }\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert_eq!(file.items.len(), 2, "{rendered}");
        let ItemKind::Subdesign(s) = &file.items[0].kind else {
            panic!()
        };
        assert!(matches!(s.body.as_slice(), [Stmt::Inst(_)]), "{rendered}");
    }

    #[test]
    fn a_file_stops_parsing_after_its_error_budget() {
        let src = format!(
            "device X {{ {}}}\n\npub device Y {{\n    pins {{ A: 1 [passive] }}\n}}\n",
            ", ".repeat(MAX_PARSE_ERRORS + 50)
        );
        let (file, errors, rendered) = parse_within(&src, LIMIT);
        assert_eq!(errors, MAX_PARSE_ERRORS + 1, "{rendered}");
        assert_eq!(
            rendered
                .matches(
                    "error[E102]: too many syntax errors — stopped parsing this file after 200"
                )
                .count(),
            1
        );
        // Abandoned, not resumed: nothing after the budget is parsed.
        assert_eq!(file.items.len(), 1);
    }

    /// Every `{ … }` body loop runs on `block_continues` — the loop-level
    /// termination guarantee. A bare `!self.at(RBrace)` condition is how
    /// both 0.8.0 hang loops were written.
    #[test]
    fn every_body_loop_uses_the_progress_guard() {
        let bare = concat!("while !self.at(&TokenKind::", "RBrace)");
        let hits: Vec<usize> = include_str!("parse.rs")
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains(bare))
            .map(|(i, _)| i + 1)
            .collect();
        assert!(
            hits.is_empty(),
            "write body loops as `while self.block_continues(&mut progress)` — bare `{bare}` at src/parse.rs lines {hits:?}"
        );
    }

    /// `parse_within` on a worker whose two termination defences are set
    /// as given (see `test_hooks`); also returns how often the
    /// `block_continues` guard fired.
    fn parse_within_hooked(
        src: &str,
        guard: bool,
        advancing: bool,
    ) -> (SourceFile, usize, String, usize) {
        let owned = src.to_string();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            test_hooks::set(guard, advancing);
            let mut sm = SourceMap::new();
            let f = sm.add_file("test.cohdl", owned.as_str());
            let mut diags = Diagnostics::new();
            let tokens = lex(f, &owned, &mut diags);
            let file = parse(tokens, &mut diags);
            let _ = tx.send((
                file,
                diags.error_count(),
                diags.render(&sm),
                test_hooks::fired(),
            ));
        });
        rx.recv_timeout(LIMIT).unwrap_or_else(|e| {
            panic!("parse panicked or did not finish ({e}) within {LIMIT:?} for:\n{src}")
        })
    }

    /// Each termination defence holds on its own — the device/trait body
    /// recovery that advances past a stray `,`, and the loop-level
    /// progress guard — and with both off, the error budget still ends the
    /// parse. (Defence in depth hides a broken half: before these switches,
    /// disabling either one alone failed no test.)
    #[test]
    fn each_termination_defence_holds_on_its_own() {
        for src in [
            "device X { , }\n",
            "trait T { , }\n",
            "pub trait T {\n    designator_prefix: \"C\",\n    pins { required A: pin }\n}\n",
            "pub device D {\n    pins { A: 1 [passive] },\n    spec { v: 5V }\n}\n",
        ] {
            for (guard, advancing) in [(false, true), (true, false)] {
                let (_, errors, rendered, _) = parse_within_hooked(src, guard, advancing);
                assert_eq!(
                    errors, 1,
                    "guard {guard}, advancing {advancing}:\n{src}\n{rendered}"
                );
            }
            let (file, errors, rendered, _) = parse_within_hooked(src, false, false);
            assert_eq!(errors, MAX_PARSE_ERRORS + 1, "{src}\n{rendered}");
            assert!(file.truncated, "{src}");
        }
    }

    /// The progress guard alone moves these loops past an unknown member:
    /// `layout { }` and `for` bodies report it without consuming it (the
    /// per-loop bumps they had were folded into the guard).
    #[test]
    fn the_progress_guard_moves_layout_and_for_bodies_past_an_unknown_member() {
        for src in [
            "design D {\n    layout {\n        diff_pair(A, B)\n        x\n    }\n}\n",
            "design D {\n    for l: n in 0..2 {\n        1\n        net _: a.B\n    }\n}\n",
        ] {
            let (file, errors, rendered, fired) = parse_within_hooked(src, true, true);
            assert_eq!(errors, 1, "{src}\n{rendered}");
            assert!(fired > 0, "the guard did not fire:\n{src}\n{rendered}");
            assert!(!file.truncated, "{src}");
            // Without the guard the same loop stalls into the error budget.
            let (file, errors, rendered, _) = parse_within_hooked(src, false, true);
            assert_eq!(errors, MAX_PARSE_ERRORS + 1, "{src}\n{rendered}");
            assert!(file.truncated, "{src}");
        }
    }

    /// The test-build peek cap ends a stall that escapes every other bound
    /// (both defences off, a budget too large to matter), so a regression
    /// fails its test within milliseconds instead of growing the test
    /// process on a timed-out worker. Without the cap this input still
    /// ends — at the 10,000-error budget — and the test fails.
    #[test]
    fn the_test_peek_cap_ends_a_stall_that_escapes_the_budget() {
        let outcome = std::thread::spawn(|| {
            test_hooks::set(false, false);
            let src = "device X { , }\n";
            let mut sm = SourceMap::new();
            let f = sm.add_file("test.cohdl", src);
            let mut diags = Diagnostics::new();
            let tokens = lex(f, src, &mut diags);
            let _ = parse_with_budget(tokens, &mut diags, 10_000);
        })
        .join();
        let payload = outcome.expect_err("an unbounded stall must hit the peek cap");
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .unwrap_or_default();
        assert!(message.contains("parser stalled"), "{message}");
    }

    /// A file whose 201st error lands between a production's branch choice
    /// and its use of the token it chose on: the budget moves the cursor to
    /// EOF in between. `stmt`'s call arm `unwrap`ped the callee there and
    /// the compiler panicked (exit 101) where 0.8.0 reported 203 errors.
    #[test]
    fn an_error_at_the_budget_boundary_never_panics() {
        let src = format!(
            "{}design D {{\n    #[placement_hint(\"x\")] foo(a.B)\n}}\n",
            "pub 1\n".repeat(MAX_PARSE_ERRORS)
        );
        let (file, errors, rendered) = parse_within(&src, LIMIT);
        assert_eq!(errors, MAX_PARSE_ERRORS + 1, "{rendered}");
        assert!(
            rendered.contains("too many syntax errors — stopped parsing this file after 200"),
            "{rendered}"
        );
        assert!(file.truncated);
    }

    /// `required A:` with no numbers, then the next entry: the next
    /// entry's obligation is not this one's misplaced obligation (and its
    /// name not a pad number) — one accurate error, no slip rewrite. Nor
    /// is an obligation where the `:` is missing, or a `[` that never
    /// closes: a slip is reported only where the entry's shape is certain.
    #[test]
    fn an_entry_without_numbers_is_not_an_obligation_slip() {
        for (src, want, found) in [
            (
                "pub device D {\n    pins { A required: 1 [passive] }\n}\n",
                2,
                "expected `:` after the pin name, found `required`",
            ),
            (
                "pub device D {\n    pins { A: [ 21 [power_in] }\n}\n",
                1,
                "expected `]` to close the bracketed pin numbers, found `[`",
            ),
            (
                "pub device D {\n    pins {\n        required A:\n        required B: 2 [passive]\n    }\n}\n",
                1,
                "expected a physical pin number (e.g. `1` or `A3`), found `required`",
            ),
            (
                "pub device D {\n    pins {\n        required A: 1 [passive]\n        required B:\n        optional C: 3 [passive]\n        required E: 4 [passive]\n    }\n}\n",
                1,
                "expected a physical pin number (e.g. `1` or `A3`), found `optional`",
            ),
            // Body level: the same error, plus the entry's placement.
            (
                "pub device D {\n    A:\n    required B: 2 [passive]\n}\n",
                2,
                "expected a physical pin number (e.g. `1` or `A3`), found `required`",
            ),
        ] {
            let (_, errors, rendered) = parse_within(src, LIMIT);
            assert_eq!(errors, want, "{src}\n{rendered}");
            assert!(rendered.contains(found), "{rendered}");
            for slip in [
                "is written after the pin name",
                "comes before the pin name",
                "are bracketed",
                "write the entry as",
                "has no role annotation",
            ] {
                assert!(!rendered.contains(slip), "{slip}:\n{rendered}");
            }
            if src.contains("    A:\n") {
                assert!(
                    rendered.contains("pin entry `A` is written directly in the device body"),
                    "{rendered}"
                );
            }
        }
        // The real slip, with a pad-name number, is still recognised.
        let src = "pub device D {\n    pins { A: required B3 [passive] }\n}\n";
        let (_, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(
            rendered.contains("write the entry as `required A: B3 [passive]`"),
            "{rendered}"
        );
    }

    /// The stray-entry help (and where the entries are adopted) follows
    /// the blocks the device already has, so following the help literally
    /// never produces E201/E908.
    #[test]
    fn stray_entry_help_and_adoption_follow_the_devices_own_blocks() {
        let device = |file: &SourceFile| match &file.items[0].kind {
            ItemKind::Device(d) => d.clone(),
            _ => panic!(),
        };
        // A variant device: pins live in `pins[VARIANT]` blocks. Nothing is
        // adopted (an unqualified block there is E908); the help names one.
        let src = "pub device V {\n    variants { X, Y }\n    A: required [1] [passive]\n    B: required [2] [passive]\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(rendered.contains("device `V` declares variants (X, Y), so its pins go in one qualified block per variant — e.g. `pins[X] { required A: 1 [passive] … }`"), "{rendered}");
        assert!(!rendered.contains("write `pins {"), "{rendered}");
        assert!(device(&file).pin_blocks.is_empty());
        let checked = crate::pipeline::check_files_in(
            "p",
            &[("src/main.cohdl".to_string(), src.to_string())],
            None,
        )
        .unwrap();
        assert!(
            !checked.diags.render(&checked.sm).contains("E908"),
            "{}",
            checked.diags.render(&checked.sm)
        );
        // The example names a variant that has no block yet...
        let src = "pub device V {\n    variants { X, Y }\n    pins[X] { required A: 1 [passive] }\n    A: 2 [passive]\n}\n";
        let (_, _, rendered) = parse_within(src, LIMIT);
        assert!(
            rendered.contains("e.g. `pins[Y] { required A: 2 [passive] }`"),
            "{rendered}"
        );
        // ...and with every variant covered, says to move the entry.
        let src = "pub device V {\n    variants { X }\n    pins[X] { required A: 1 [passive] }\n    B: 2 [passive]\n}\n";
        let (_, _, rendered) = parse_within(src, LIMIT);
        assert!(rendered.contains("has a `pins[VARIANT] { … }` block for each — move this entry into the block it belongs to"), "{rendered}");
        // A real `pins { }` already there: the strays join it.
        let src = "pub device D {\n    IOVDD: required [1, 10] [power_in]\n    pins { required GND: 2 [power_in] }\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(rendered.contains("move this entry into the device's existing `pins { … }` block, written `required IOVDD: 1, 10 [power_in]`"), "{rendered}");
        assert!(!rendered.contains("write `pins {"), "{rendered}");
        let d = device(&file);
        assert_eq!(d.pin_blocks.len(), 1);
        let names: Vec<&str> = d.pin_blocks[0]
            .pins
            .iter()
            .map(|p| p.name.name.as_str())
            .collect();
        assert_eq!(names, ["GND", "IOVDD"]);
        // Same for `spec { }`.
        let src = "pub device D {\n    spec { v: 5V }\n    i: 1A\n    pins { A: 1 [passive] }\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(
            rendered.contains(
                "move this entry into the device's existing `spec { … }` block, written `i: 1A`"
            ),
            "{rendered}"
        );
        let d = device(&file);
        assert_eq!(d.spec_blocks.len(), 1);
        assert_eq!(d.spec_blocks[0].fields.len(), 2);
    }

    /// `NAME: -…` in a device body is a pin entry when a number follows
    /// the `-` (a signed pin number, the legacy E102) and a spec field
    /// when a unit literal does — not a spec field for every `-`.
    #[test]
    fn a_signed_stray_member_is_classified_by_what_follows_the_minus() {
        let src = "pub device D {\n    A: -1 [passive]\n}\n";
        let (_, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 2, "{rendered}");
        assert!(
            rendered.contains("pin entry `A` is written directly in the device body"),
            "{rendered}"
        );
        assert!(
            rendered.contains("error[E102]: a bare number cannot be negative"),
            "{rendered}"
        );
        assert!(!rendered.contains("spec field"), "{rendered}");

        let src = "pub device D {\n    t: -40C\n}\n";
        let (file, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(
            rendered.contains("spec field `t` is written directly in the device body"),
            "{rendered}"
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.spec_blocks[0].fields[0].name.name, "t");

        // Neither: the plain device-body error, nothing invented.
        let src = "pub device D {\n    A: - x\n}\n";
        let (_, errors, rendered) = parse_within(src, LIMIT);
        assert_eq!(errors, 1, "{rendered}");
        assert!(
            rendered
                .contains("expected `pins`, `spec`, or `variants` in the device body, found `A`"),
            "{rendered}"
        );
    }

    /// A broken AVL field or impl mapping skips to its own `,` and the
    /// block goes on — one error, and the entries after it still parse.
    #[test]
    fn a_broken_avl_field_or_impl_mapping_resyncs_at_the_next_entry() {
        for (entry, found) in [
            (
                "mfr: 7",
                "expected a string value for AVL field `mfr`, found number `7`",
            ),
            (
                "9: \"x\"",
                "expected an identifier as the AVL field name (e.g. `mpn`), found number `9`",
            ),
        ] {
            let src = format!(
                "pub part P: D {{\n    primary {{ {entry}, mpn: \"M\", footprint: F }}\n}}\n"
            );
            let (file, errors, rendered) = parse_within(&src, LIMIT);
            assert_eq!(errors, 1, "{rendered}");
            assert!(rendered.contains(found), "{rendered}");
            let ItemKind::Part(p) = &file.items[0].kind else {
                panic!("{rendered}")
            };
            assert!(
                p.primary
                    .fields
                    .iter()
                    .any(|f| f.name.name == "mpn" && f.value == "M"),
                "{:?}",
                p.primary.fields
            );
            assert_eq!(
                p.primary.footprint.as_ref().map(|f| f.name.as_str()),
                Some("F")
            );
        }
        for (src, found) in [
            (
                "impl T for D {\n    pins { A: 7, B: C }\n}\n",
                "expected an identifier as the device's own name, found number `7`",
            ),
            (
                "impl T for D {\n    pins { 7: A, B: C }\n}\n",
                "expected an identifier as the trait's required name, found number `7`",
            ),
        ] {
            let (file, errors, rendered) = parse_within(src, LIMIT);
            assert_eq!(errors, 1, "{src}\n{rendered}");
            assert!(rendered.contains(found), "{rendered}");
            let ItemKind::Impl(i) = &file.items[0].kind else {
                panic!("{rendered}")
            };
            assert!(
                i.pin_map
                    .iter()
                    .any(|m| m.role.name == "B" && m.target.name == "C"),
                "{rendered}"
            );
        }
    }

    /// Every parser diagnostic goes through `report`: the error budget —
    /// and with it the bound on any stalled recovery — holds only for the
    /// diagnostics it sees.
    #[test]
    fn every_parser_diagnostic_goes_through_report() {
        let src = include_str!("parse.rs");
        let prod = &src[..src.find("#[cfg(test)]\nmod tests").unwrap()];
        let start = prod.find("    fn report(&mut self").unwrap();
        let end = start + prod[start..].find("\n    }\n").unwrap();
        let hits: Vec<usize> = prod
            .match_indices("self.diags")
            .map(|(at, _)| at)
            .filter(|at| !(start..end).contains(at))
            .map(|at| prod[..at].matches('\n').count() + 1)
            .collect();
        assert!(
            hits.is_empty(),
            "record parser diagnostics with `self.report(…)`, which enforces the error budget — `self.diags` used directly at src/parse.rs lines {hits:?}"
        );
    }

    /// A file stopped at its error budget lacks every declaration after
    /// the stop point. Resolving against it reported declarations that
    /// exist as unknown in other files — and a `design` past the stop as a
    /// selection error, exit 2, an "invocation" failure to a caller — so
    /// the pipeline reports the syntax errors alone.
    #[test]
    fn a_truncated_file_reports_only_its_syntax_errors() {
        let commas = ", ".repeat(MAX_PARSE_ERRORS + 60);
        let devices = |x_body: &str| {
            format!(
                "pub device R {{\n    pins {{ required A: 1 [passive], required B: 2 [passive] }}\n}}\n\npub device X {{ {x_body} }}\n\npub device Y {{\n    pins {{ required A: 1 [passive], required B: 2 [passive] }}\n}}\n"
            )
        };
        let top = "design Top {\n    inst r: R\n    inst y: Y\n    net N1: r.A, y.A\n    net N2: r.B, y.B\n}\n";
        let project = |x_body: &str| {
            vec![
                ("src/devices.cohdl".to_string(), devices(x_body)),
                ("src/top.cohdl".to_string(), top.to_string()),
            ]
        };
        let one_file = vec![(
            "src/main.cohdl".to_string(),
            format!("{}\n{top}", devices(&commas)),
        )];
        for (files, design) in [
            (project(&commas), None),
            (project(&commas), Some("Top")),
            (one_file, Some("Top")),
        ] {
            let checked = crate::pipeline::check_files_in("p", &files, design).unwrap();
            let rendered = checked.diags.render(&checked.sm);
            assert_eq!(
                checked.diags.error_count(),
                MAX_PARSE_ERRORS + 1,
                "{rendered}"
            );
            assert!(!rendered.contains("E202"), "{rendered}");
            assert!(
                rendered.contains("no name-resolution or design checks ran"),
                "{rendered}"
            );
            assert_eq!(checked.selection_error, None);
        }
        // Control: the same project with a well-formed `X` checks clean.
        let checked =
            crate::pipeline::check_files_in("p", &project("pins { A: 1 [passive] }"), None)
                .unwrap();
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
    }

    /// The small valid trait/device/impl/part file the insertion fuzz
    /// mutates first.
    const BASE: &str = "pub trait T: TwoTerminal {\n    designator_prefix: \"U\"\n    pins {\n        required A: pin\n        optional B: pin\n    }\n    spec {\n        v: Voltage\n    }\n}\n\npub device D<V: Voltage = 5V> {\n    variants { X, Y }\n    pins[X] {\n        required A: 1, 2 [passive]\n        optional B: A3 [power_in]\n    }\n    pins[Y] { required A: 1 [passive], optional B: 2 [power_in] }\n    spec { v: V }\n}\n\nimpl T for D {\n    pins { A: A, B: B }\n}\n\npub part P: D<5V>[X] {\n    primary { mfr: \"M\", mpn: \"N\", footprint: F }\n    alt { mfr: \"M2\", mpn: \"N2\", footprint: lib::F }\n}\n";

    /// The second fuzz base: the statement- and geometry-level bodies —
    /// pad, footprint (pads, a mount hole, silkscreen, courtyard), a
    /// subdesign with ports and a layout, a `fn` with an attributed `inst`,
    /// and a design with an attributed array `inst`, a `for` loop, a call,
    /// a subdesign use site, `nc`, and a layout with a loop and net rules.
    const BASE2: &str = r#"pub pad P {
    shape: rect
    size: (0.6mm, 0.8mm)
    layer: top_copper
    plating: smd
}

pub footprint F {
    pad 1: P at (-1mm, 0mm)
    pad 2: P at (1mm, 0mm) rotate 90
    mount_hole 1: non_plated at (0mm, 2mm) diameter 2.2mm
    silkscreen {
        line from (-2mm, -1mm) to (2mm, -1mm) width 0.15mm
        pin_1_marker near pad 1 shape dot
    }
    courtyard { shape: rect, at: (0mm, 0mm), size: (3mm, 2mm) }
    silkscreen_ref { at: (0mm, -2mm) }
}

pub subdesign S<C: Capacitance> {
    ports {
        required VIN: Pin
        optional VOUT: Pin
    }
    inst c: Cap<C>
    net _: VIN, c.A
    layout { place c at (1mm, 2mm) rotate 90 }
}

pub fn link(a: Pin, b: Pin) {
    #[bypass(a, 100nF)]
    inst c: Cap<100nF>
    net _: a, c.A, b
}

design Top {
    const N: Int = 3
    inst host: Host
    #[placement_hint("left")]
    inst leds: [Led; N]
    net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
    for chain: n in 0..(leds.len - 1) {
        net _: leds[n].DOUT, leds[n + 1].DIN
    }
    link(host.DATA, leds[0].DIN)
    subdesign reg: S<1uF> { VIN: host.V5 }
    nc: leds[2].DOUT
    layout {
        for grid: n in 0..leds.len {
            place leds[n] at (10mm + n * 4mm, 10mm) side bottom
        }
        net_class Power { VCC }
        diff_pair(VCC, VCC)
    }
}
"#;

    const INSERTS: &[&str] = &[
        ",",
        ":",
        ";",
        "{",
        "}",
        "(",
        ")",
        "[",
        "]",
        "<",
        ">",
        "=",
        "+",
        "-",
        "*",
        "/",
        "%",
        ".",
        "..",
        "::",
        "#",
        "\"s\"",
        "1",
        "5V",
        "-5V",
        "A1",
        "x",
        "_",
        "pins",
        "spec",
        "variants",
        "required",
        "optional",
        "pin",
        "designator_prefix",
        "pub",
        "trait",
        "device",
        "impl",
        "for",
        "fn",
        "part",
        "design",
        "inst",
        "net",
        "nc",
        "use",
        "footprint",
        "pad",
        "subdesign",
        "primary",
        "alt",
        "layout",
        "const",
        "Voltage",
        "passive",
        "\u{3a9}",
    ];

    /// Fuzz-ish: insert each of `INSERTS` before every token of the valid
    /// file `base`. Every variant must finish quickly with at most `bound`
    /// errors. And since the error budget can halt a parse at ANY of its
    /// errors — leaving the cursor on EOF mid-production (see `report`) —
    /// each variant with `n` errors is parsed again under every budget
    /// below `n`, so the halt lands at each error point the variant
    /// reaches; each must end cleanly with exactly that many errors plus
    /// the E102 (it once panicked at `stmt`'s call arm instead). One worker
    /// runs them all; the test fails, naming the input, if a parse panics
    /// or overruns the deadline. Returns (variants, worst error count,
    /// times the `block_continues` guard fired).
    fn insertion_fuzz(base: &'static str, bound: usize) -> (usize, usize, usize) {
        parse_ok(base);
        let mut sm = SourceMap::new();
        let f = sm.add_file("base.cohdl", base);
        let mut scratch = Diagnostics::new();
        let at: Vec<usize> = lex(f, base, &mut scratch)
            .iter()
            .map(|t| t.span.start as usize)
            .collect();

        enum Msg {
            Start(String),
            Errors(usize),
            Done(usize),
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            test_hooks::set(true, true);
            for &i in &at {
                for ins in INSERTS {
                    let src = format!("{} {ins} {}", &base[..i], &base[i..]);
                    tx.send(Msg::Start(src.clone())).unwrap();
                    // Parser errors only (the budget does not count the
                    // lexer's), and whether the parse was cut short.
                    let parse_errors = |budget: usize| {
                        let mut sm = SourceMap::new();
                        let f = sm.add_file("fuzz.cohdl", src.as_str());
                        let tokens = lex(f, &src, &mut Diagnostics::new());
                        let mut diags = Diagnostics::new();
                        let file = parse_with_budget(tokens, &mut diags, budget);
                        (diags.error_count(), file.truncated)
                    };
                    let (n, truncated) = parse_errors(MAX_PARSE_ERRORS);
                    assert!(!truncated, "the full budget halted:\n{src}");
                    for budget in 0..n {
                        let (m, truncated) = parse_errors(budget);
                        assert!(
                            truncated && m == budget + 1,
                            "budget {budget}: {m} errors, truncated: {truncated}, for:\n{src}"
                        );
                    }
                    tx.send(Msg::Errors(n)).unwrap();
                }
            }
            tx.send(Msg::Done(test_hooks::fired())).unwrap();
        });
        let (mut current, mut cases, mut worst) = (String::new(), 0usize, 0usize);
        loop {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Msg::Start(src)) => current = src,
                Ok(Msg::Errors(n)) => {
                    cases += 1;
                    worst = worst.max(n);
                    assert!(
                        n <= bound,
                        "{n} errors (bound {bound}) for one inserted token:\n{current}"
                    );
                }
                Ok(Msg::Done(fired)) => return (cases, worst, fired),
                Err(e) => panic!("parse panicked or did not finish ({e}) within 5s on:\n{current}"),
            }
        }
    }

    #[test]
    fn single_token_insertions_terminate_with_bounded_diagnostics() {
        // The point is "bounded", against the unbounded growth (or the
        // 200-error budget) of a stall. Worst today is 11: an inserted `}`
        // or keyword ends a declaration early and the remainder of its
        // body is reported token group by token group.
        let (cases, worst, fired) = insertion_fuzz(BASE, 20);
        assert!(cases > 5_000, "only {cases} variants ran");
        eprintln!(
            "{cases} single-token insertions, worst case {worst} errors, guard fired {fired}x"
        );
    }

    #[test]
    fn single_token_insertions_into_statement_bodies_terminate() {
        // Worst today is 20: an inserted `}` ends the design early and the
        // rest of its statements are reported at the top level.
        let (cases, worst, fired) = insertion_fuzz(BASE2, 30);
        assert!(cases > 20_000, "only {cases} variants ran");
        // The `block_continues` guard is load-bearing here: unknown members
        // of `layout { }` and `for` bodies are reported without consuming.
        assert!(fired > 0, "the progress guard never fired");
        eprintln!(
            "{cases} single-token insertions, worst case {worst} errors, guard fired {fired}x"
        );
    }
}

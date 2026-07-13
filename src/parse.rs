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
use crate::units::UnitType;

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
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn item(&mut self) -> Option<Item> {
        let start = self.span();
        let attrs = self.attrs();
        let is_pub = self.eat(&TokenKind::Pub);
        let kind = match self.peek() {
            TokenKind::Trait => self.trait_def().map(ItemKind::Trait),
            TokenKind::Device => self.device_def().map(ItemKind::Device),
            TokenKind::Impl => self.impl_def().map(ItemKind::Impl),
            TokenKind::Fn => self.fn_def().map(ItemKind::Fn),
            TokenKind::Part => self.part_def().map(ItemKind::Part),
            TokenKind::Design => self.design_def().map(ItemKind::Design),
            other => {
                self.error_here(format!(
                    "expected a top-level declaration (`trait`, `device`, `impl`, `fn`, `part`, or `design`), found {}",
                    other.describe()
                ));
                self.sync_top_level();
                None
            }
        };
        if !attrs.is_empty() {
            // Attributes are only meaningful on `inst` statements for MVP.
            self.diags.push(Diagnostic::error(
                "E010",
                attrs[0].span,
                "attributes are only supported on `inst` statements",
            ));
        }
        let kind = kind?;
        Some(Item {
            is_pub,
            span: start.to(self.prev_span()),
            kind,
        })
    }

    fn attrs(&mut self) -> Vec<Attr> {
        let mut attrs = Vec::new();
        while self.at(&TokenKind::Hash) {
            let start = self.span();
            self.bump(); // #
            if !self.expect(&TokenKind::LBracket, "after `#`") {
                break;
            }
            let Some(name) = self.ident("as the attribute name") else {
                break;
            };
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
        attrs
    }

    // -- traits --------------------------------------------------------------

    fn trait_def(&mut self) -> Option<TraitDef> {
        self.bump(); // trait
        let name = self.ident("as the trait name")?;
        let mut super_traits = Vec::new();
        if self.eat(&TokenKind::Colon) {
            loop {
                super_traits.push(self.ident("as a sub-trait bound")?);
                if !self.eat(&TokenKind::Plus) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::LBrace, "to open the trait body");
        let mut def = TraitDef {
            name,
            super_traits,
            designator_prefix: None,
            pins: Vec::new(),
            specs: Vec::new(),
        };
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Pins) {
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
            } else if self.at(&TokenKind::Spec) {
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
                        "the ten unit types are: Voltage, Capacitance, Resistance, Current, \
                         Frequency, Time, Inductance, Power, Temperature, Tolerance",
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
        self.expect(&TokenKind::LBrace, "to open the device body");
        let mut def = DeviceDef {
            name,
            generics,
            pins: Vec::new(),
            specs: Vec::new(),
        };
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Pins) {
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the pins block");
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(pin) = self.device_pin() {
                        def.pins.push(pin);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the pins block");
            } else if self.at(&TokenKind::Spec) {
                self.bump();
                self.expect(&TokenKind::LBrace, "to open the spec block");
                while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                    if let Some(field) = self.device_spec_field() {
                        def.specs.push(field);
                    } else {
                        self.sync_in_block();
                    }
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "to close the spec block");
            } else {
                self.error_here(format!(
                    "expected `pins` or `spec` in the device body, found {}",
                    self.peek().describe()
                ));
                self.sync_in_block();
            }
        }
        self.expect(&TokenKind::RBrace, "to close the device body");
        Some(def)
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
                TokenKind::Ident(n) if is_bga_pin_number(n) => {
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
                    TokenKind::Ident(n) if is_bga_pin_number(n) => {
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
            let Some(first) = self.ident("as the generic bound") else {
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
                    match self.ident("as a trait bound") {
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
                    let ident = self.ident("").unwrap();
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
        let name = self.ident("as a type name")?;
        let start = name.span;
        let generic_args = if self.at(&TokenKind::Lt) {
            self.generic_args()
        } else {
            Vec::new()
        };
        Some(TypeRef {
            name,
            generic_args,
            span: start.to(self.prev_span()),
        })
    }

    // -- impls ---------------------------------------------------------------

    fn impl_def(&mut self) -> Option<ImplDef> {
        let start = self.span();
        self.bump(); // impl
        let trait_name = self.ident("as the trait name")?;
        self.expect(&TokenKind::For, "between the trait and device names");
        let device_name = self.ident("as the device name")?;
        self.expect(&TokenKind::LBrace, "to open the impl body");
        let mut pin_map = Vec::new();
        let mut spec_map = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
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
                self.sync_in_block();
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
        }
        self.expect(&TokenKind::RBrace, "to close the impl body");
        Some(ImplDef {
            trait_name,
            device_name,
            pin_map,
            spec_map,
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
                while let Some(t) = self.ident("as a trait bound after `impl`") {
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
        self.expect(&TokenKind::LBrace, "to open the fn body");
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
                self.sync_in_block();
                continue;
            };
            let entry_start = self.span();
            self.bump();
            self.expect(&TokenKind::LBrace, "to open the AVL entry");
            let mut fields = Vec::new();
            while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
                let Some(fname) = self.ident("as the AVL field name (e.g. `mpn`)") else {
                    self.sync_in_block();
                    break;
                };
                self.expect(&TokenKind::Colon, "after the AVL field name");
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
                    "every part needs exactly one `primary { mpn: \"…\", footprint: \"…\" }`",
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
        self.expect(&TokenKind::LBrace, "to open the design body");
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

    fn sync_in_block(&mut self) {
        // Inside a `{ … }` block: skip to the next comma or closing brace.
        let mut depth = 0usize;
        loop {
            match self.peek() {
                TokenKind::Eof => return,
                TokenKind::Comma if depth == 0 => return,
                TokenKind::RBrace if depth == 0 => return,
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let attrs = self.attrs();
        match self.peek() {
            TokenKind::Inst => {
                let start = self.span();
                self.bump();
                let name = self.ident("as the instance name")?;
                self.expect(&TokenKind::Colon, "after the instance name");
                let ty = self.type_ref()?;
                Some(Stmt::Inst(InstStmt {
                    attrs,
                    name,
                    span: start.to(self.prev_span()),
                    ty,
                }))
            }
            TokenKind::Net => {
                self.reject_attrs(&attrs);
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
                    name,
                    annotation,
                    members,
                    span: start.to(self.prev_span()),
                }))
            }
            TokenKind::Nc => {
                self.reject_attrs(&attrs);
                let start = self.span();
                self.bump();
                self.expect(&TokenKind::Colon, "after `nc`");
                let members = self.pin_ref_list();
                Some(Stmt::Nc(NcStmt {
                    members,
                    span: start.to(self.prev_span()),
                }))
            }
            TokenKind::Ident(_) => {
                self.reject_attrs(&attrs);
                let callee = self.ident("").unwrap();
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
                    span: start.to(self.prev_span()),
                }))
            }
            other => {
                let msg = format!(
                    "expected a statement (`inst`, `net`, `nc`, or a fn call), found {}",
                    other.describe()
                );
                self.error_here(msg);
                None
            }
        }
    }

    fn reject_attrs(&mut self, attrs: &[Attr]) {
        if let Some(a) = attrs.first() {
            self.diags.push(Diagnostic::error(
                "E010",
                a.span,
                "attributes are only supported on `inst` statements",
            ));
        }
    }

    fn pin_ref(&mut self) -> Option<PinRef> {
        let base = self.ident("as a pin reference")?;
        let start = base.span;
        let pin = if self.eat(&TokenKind::Dot) {
            Some(self.ident("as the pin name after `.`")?)
        } else {
            None
        };
        Some(PinRef {
            base,
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

/// BGA-style pin numbers: letters followed by at least one digit (`A1`, `C3`).
fn is_bga_pin_number(s: &str) -> bool {
    let mut chars = s.chars();
    let mut saw_alpha = false;
    for c in chars.by_ref() {
        if c.is_ascii_uppercase() {
            saw_alpha = true;
        } else if c.is_ascii_digit() {
            return saw_alpha && chars.all(|c| c.is_ascii_alphanumeric());
        } else {
            return false;
        }
    }
    false
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
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}
"#,
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.generics.len(), 3);
        assert!(d.generics[1].default.is_some());
        assert_eq!(d.pins.len(), 2);
        assert_eq!(d.pins[0].obligation, Obligation::Required);
    }

    #[test]
    fn parses_pin_bus_and_roles() {
        let file = parse_ok(
            r#"
pub device MCU_ESP32S3 {
    pins {
        required VDD: 1 [power_in]
        required GND: 2, 3, 4
        optional NC_1: 5
        required TX: 6 [output]
    }
}
"#,
        );
        let ItemKind::Device(d) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(d.pins.len(), 4);
        assert_eq!(d.pins[1].numbers.len(), 3);
        assert_eq!(d.pins[2].obligation, Obligation::Optional);
        assert_eq!(d.pins[3].role.unwrap().0, PinRole::Output);
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
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC", footprint: "Capacitor_SMD:C_0402_1005Metric" }
    alt { mfr: "Murata", mpn: "GRM155R71C104KA88D" }
}
"#,
        );
        let ItemKind::Part(p) = &file.items[0].kind else {
            panic!()
        };
        assert_eq!(p.primary.field("mpn").unwrap().value, "CL05B104KO5NNNC");
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
}

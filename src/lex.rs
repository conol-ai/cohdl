//! Hand-written lexer.
//!
//! Deterministic, single-pass, no backtracking (Constitution hard constraint:
//! deterministic grammar, no context-sensitive tricks). Unit literals are
//! lexed as single tokens — a number immediately followed by its suffix, no
//! space — and validated against the RFC-001 (unit × prefix) table right here,
//! so a bad prefix or a Unicode `Ω`/`°C` gets a targeted diagnostic instead of
//! a generic parse error (RFC-001 "Failure modes").

use crate::diag::{Diagnostic, Diagnostics};
use crate::span::{FileId, Span};
use crate::units::{self, UnitLexError, UnitValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords.
    Pub,
    Trait,
    Device,
    Impl,
    For,
    Fn,
    Design,
    Inst,
    Net,
    Nc,
    Part,
    Pins,
    Spec,
    Required,
    Optional,

    /// Identifier (also `_`).
    Ident(String),
    /// Bare integer/decimal number (no unit suffix), e.g. a pin number.
    Number(String),
    /// A unit literal, validated against the RFC-001 table.
    Unit(UnitValue),
    /// A double-quoted string literal (part MPNs, designator overrides…).
    Str(String),

    // Punctuation.
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Lt,
    Gt,
    Colon,
    PathSep, // ::
    Comma,
    Dot,
    Eq,
    Plus,
    Hash,
    Percent,
    /// `;` — only the RFC-016 `use` import ends with one.
    Semi,

    Eof,
}

impl TokenKind {
    /// How the token reads in "expected X, found Y" messages.
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Ident(name) => format!("`{}`", name),
            TokenKind::Number(n) => format!("number `{}`", n),
            TokenKind::Unit(v) => format!("unit literal `{}`", v.text),
            TokenKind::Str(s) => format!("string \"{}\"", s),
            TokenKind::Eof => "end of file".to_string(),
            other => format!("`{}`", other.token_text()),
        }
    }

    fn token_text(&self) -> &'static str {
        match self {
            TokenKind::Pub => "pub",
            TokenKind::Trait => "trait",
            TokenKind::Device => "device",
            TokenKind::Impl => "impl",
            TokenKind::For => "for",
            TokenKind::Fn => "fn",
            TokenKind::Design => "design",
            TokenKind::Inst => "inst",
            TokenKind::Net => "net",
            TokenKind::Nc => "nc",
            TokenKind::Part => "part",
            TokenKind::Pins => "pins",
            TokenKind::Spec => "spec",
            TokenKind::Required => "required",
            TokenKind::Optional => "optional",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::Colon => ":",
            TokenKind::PathSep => "::",
            TokenKind::Semi => ";",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Eq => "=",
            TokenKind::Plus => "+",
            TokenKind::Hash => "#",
            TokenKind::Percent => "%",
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// A reserved keyword — a word the lexer tokenizes as something other than an
/// `Ident`, so it can never appear as an identifier (or a qualified-path
/// segment). Used by module-path validation (RFC-016 segments must be
/// spellable identifiers — review R5-3).
pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "pub"
            | "trait"
            | "device"
            | "impl"
            | "for"
            | "fn"
            | "design"
            | "inst"
            | "net"
            | "nc"
            | "part"
            | "pins"
            | "spec"
            | "required"
            | "optional"
    )
}

/// True when `s` can appear as a CoHDL identifier / qualified-path segment:
/// a non-empty non-keyword word of `[A-Za-z_][A-Za-z0-9_]*`.
pub fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !is_keyword(s)
}

pub fn lex(file: FileId, text: &str, diags: &mut Diagnostics) -> Vec<Token> {
    Lexer {
        file,
        text,
        bytes: text.as_bytes(),
        pos: 0,
        tokens: Vec::new(),
        diags,
    }
    .run()
}

struct Lexer<'a> {
    file: FileId,
    text: &'a str,
    bytes: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
    diags: &'a mut Diagnostics,
}

impl<'a> Lexer<'a> {
    fn run(mut self) -> Vec<Token> {
        while self.pos < self.bytes.len() {
            let start = self.pos;
            let b = self.bytes[self.pos];
            match b {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.pos += 1;
                }
                b'/' if self.peek(1) == Some(b'/') => {
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                b'{' => self.punct(TokenKind::LBrace),
                b'}' => self.punct(TokenKind::RBrace),
                b'(' => self.punct(TokenKind::LParen),
                b')' => self.punct(TokenKind::RParen),
                b'[' => self.punct(TokenKind::LBracket),
                b']' => self.punct(TokenKind::RBracket),
                b'<' => self.punct(TokenKind::Lt),
                b'>' => self.punct(TokenKind::Gt),
                b',' => self.punct(TokenKind::Comma),
                b'.' => self.punct(TokenKind::Dot),
                b'=' => self.punct(TokenKind::Eq),
                b'+' => self.punct(TokenKind::Plus),
                b'#' => self.punct(TokenKind::Hash),
                b'%' => self.punct(TokenKind::Percent),
                b';' => self.punct(TokenKind::Semi),
                b':' => {
                    if self.peek(1) == Some(b':') {
                        self.pos += 2;
                        self.push(TokenKind::PathSep, start);
                    } else {
                        self.punct(TokenKind::Colon);
                    }
                }
                b'"' => self.string(start),
                b'-' => {
                    // A `-` is only meaningful directly before a numeric
                    // literal (Temperature is the sole signed unit type).
                    if self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
                        self.pos += 1;
                        self.number(start, true);
                    } else {
                        self.error_char(start, "`-` is only valid as the sign of a `Temperature` or `Length` literal (e.g. `-40C`, `-1.5mm`)");
                    }
                }
                b'0'..=b'9' => self.number(start, false),
                _ if b == b'_' || b.is_ascii_alphabetic() => self.ident(start),
                _ => {
                    // Multi-byte / unknown character. Special-case the two
                    // Unicode traps the RFCs call out.
                    let ch = self.text[self.pos..].chars().next().unwrap();
                    match ch {
                        '\u{03A9}' | '\u{2126}' => {
                            // Greek omega / ohm sign, standalone (no preceding
                            // number — the `<num>Ω` case is caught in number()
                            // as E101). RFC-011 gives the standalone case its
                            // own E1xx code, E107, so its message can be maximally
                            // specific instead of falling through to E001.
                            let span = self.consume_char_span(start);
                            self.diags.push(Diagnostic::error(
                                "E107",
                                span,
                                "use `ohm`, not `Ω` — CoHDL resistance literals are ASCII-only (e.g. `10kohm`)",
                            ));
                        }
                        '\u{00B0}' => {
                            self.error_char(
                                start,
                                "use `C`, not `°C` — CoHDL temperature literals are ASCII-only (e.g. `85C`, `-40C`)",
                            );
                        }
                        _ => {
                            self.error_char(start, format!("unexpected character `{}`", ch));
                        }
                    }
                }
            }
        }
        let span = Span::new(self.file, self.pos as u32, self.pos as u32);
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span,
        });
        self.tokens
    }

    fn peek(&self, ahead: usize) -> Option<u8> {
        self.bytes.get(self.pos + ahead).copied()
    }

    fn punct(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.pos += 1;
        self.push(kind, start);
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(self.file, start as u32, self.pos as u32),
        });
    }

    fn error_char(&mut self, start: usize, message: impl Into<String>) {
        let span = self.consume_char_span(start);
        self.diags.push(Diagnostic::error("E001", span, message));
    }

    /// Consume one char (so lexing can continue) and return its span.
    fn consume_char_span(&mut self, start: usize) -> Span {
        let ch_len = self.text[self.pos..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        self.pos += ch_len;
        Span::new(self.file, start as u32, self.pos as u32)
    }

    fn string(&mut self, start: usize) {
        self.pos += 1; // opening quote
        let content_start = self.pos;
        while self.pos < self.bytes.len()
            && self.bytes[self.pos] != b'"'
            && self.bytes[self.pos] != b'\n'
        {
            self.pos += 1;
        }
        if self.peek(0) == Some(b'"') {
            let content = self.text[content_start..self.pos].to_string();
            self.pos += 1; // closing quote
            self.push(TokenKind::Str(content), start);
        } else {
            let span = Span::new(self.file, start as u32, self.pos as u32);
            self.diags.push(Diagnostic::error(
                "E002",
                span,
                "unterminated string literal",
            ));
        }
    }

    fn ident(&mut self, start: usize) {
        while self
            .peek(0)
            .is_some_and(|b| b == b'_' || b.is_ascii_alphanumeric())
        {
            self.pos += 1;
        }
        let text = &self.text[start..self.pos];
        let kind = match text {
            "pub" => TokenKind::Pub,
            "trait" => TokenKind::Trait,
            "device" => TokenKind::Device,
            "impl" => TokenKind::Impl,
            "for" => TokenKind::For,
            "fn" => TokenKind::Fn,
            "design" => TokenKind::Design,
            "inst" => TokenKind::Inst,
            "net" => TokenKind::Net,
            "nc" => TokenKind::Nc,
            "part" => TokenKind::Part,
            "pins" => TokenKind::Pins,
            "spec" => TokenKind::Spec,
            "required" => TokenKind::Required,
            "optional" => TokenKind::Optional,
            _ => TokenKind::Ident(text.to_string()),
        };
        self.push(kind, start);
    }

    /// Lex a number, and — if it is immediately followed by letters or `%` —
    /// a unit literal. `100nF` is one token; `100 nF` is not (and the parser
    /// will reject the stray identifier).
    fn number(&mut self, start: usize, negative: bool) {
        let digits_start = self.pos;
        while self.peek(0).is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek(0) == Some(b'.') && self.peek(1).is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
            while self.peek(0).is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let mantissa = self.text[digits_start..self.pos].to_string();

        // Check for a Unicode unit-symbol trap directly after the digits
        // (`10kΩ`, `85°C`) before the ASCII suffix path.
        let rest = &self.text[self.pos..];
        let suffix_end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '%'))
            .unwrap_or(rest.len());
        let ascii_suffix = &rest[..suffix_end];
        let after_suffix = rest[suffix_end..].chars().next();

        // A Unicode unit glyph directly after the (possibly SI-prefixed)
        // digits — `10Ω`, `10kΩ`, `85°C` — is ONE targeted E101, never a
        // cascade of unknown-suffix + stray-character errors (RFC-011).
        if matches!(after_suffix, Some('\u{03A9}') | Some('\u{2126}')) {
            self.pos += ascii_suffix.len();
            self.pos += after_suffix.unwrap().len_utf8();
            let span = Span::new(self.file, start as u32, self.pos as u32);
            self.diags.push(
                Diagnostic::error(
                    "E101",
                    span,
                    "use `ohm`, not `Ω` — CoHDL resistance literals are ASCII-only",
                )
                .with_help(format!("write `{}{}ohm`", mantissa, ascii_suffix)),
            );
            return;
        }
        if matches!(after_suffix, Some('\u{00B0}')) {
            // `85°C` / `1m°C` — consume the suffix, the degree sign, and a
            // following C if present.
            self.pos += ascii_suffix.len();
            self.pos += '\u{00B0}'.len_utf8();
            if self.peek(0) == Some(b'C') {
                self.pos += 1;
            }
            let span = Span::new(self.file, start as u32, self.pos as u32);
            self.diags.push(
                Diagnostic::error(
                    "E101",
                    span,
                    "use `C`, not `°C` — CoHDL temperature literals are ASCII-only",
                )
                .with_help(format!("write `{}C`", mantissa)),
            );
            return;
        }

        if ascii_suffix.is_empty() {
            if negative {
                let span = Span::new(self.file, start as u32, self.pos as u32);
                self.diags.push(Diagnostic::error(
                    "E102",
                    span,
                    "a bare number cannot be negative — only `Temperature` and `Length` literals may carry a leading `-` (e.g. `-40C`, `-0.5mm`)",
                ));
                return;
            }
            self.push(TokenKind::Number(mantissa), start);
            return;
        }

        // `%` can only terminate the suffix; letters+digits then '%' (e.g.
        // "10n%") would have been split above only if '%' were first — handle
        // the plain cases: all-letters, or single '%'.
        self.pos += ascii_suffix.len();
        let span = Span::new(self.file, start as u32, self.pos as u32);
        let full_text = &self.text[start..self.pos];

        let parsed = units::parse_suffix(ascii_suffix).and_then(|(prefix, unit)| {
            units::make_value(negative, &mantissa, prefix, unit, full_text)
        });
        match parsed {
            Ok(value) => self.push(TokenKind::Unit(value), start),
            Err(err) => self.diags.push(unit_error_diag(err, span, full_text)),
        }
    }
}

fn unit_error_diag(err: UnitLexError, span: Span, literal: &str) -> Diagnostic {
    match err {
        UnitLexError::UnknownSuffix { suffix } => Diagnostic::error(
            "E103",
            span,
            format!(
                "`{}` is not a unit: `{}` is not one of CoHDL's unit symbols",
                literal, suffix
            ),
        )
        .with_help(
            "the eleven unit symbols are: V, F, ohm, A, Hz, s, H, W, C, %, mm \
             (optionally SI-prefixed, e.g. `100nF`, `10kohm`)",
        ),
        UnitLexError::PrefixNotAllowed { unit, prefix } => Diagnostic::error(
            "E104",
            span,
            format!(
                "SI prefix `{}` is not valid for `{}` in `{}`",
                prefix.letter(),
                unit.type_name(),
                literal
            ),
        )
        .with_help(units::prefix_table_help(unit)),
        UnitLexError::NegativeNotAllowed { unit } => Diagnostic::error(
            "E105",
            span,
            format!(
                "`{}` cannot be negative — only `Temperature` and `Length` literals may carry a leading `-`",
                unit.type_name()
            ),
        ),
        UnitLexError::TooPrecise => Diagnostic::error(
            "E106",
            span,
            format!("`{}` has more decimal places than CoHDL values can represent exactly", literal),
        ),
        UnitLexError::Overflow => Diagnostic::error(
            "E106",
            span,
            format!("`{}` is out of range for an exactly-representable value", literal),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::SourceMap;
    use crate::units::UnitType;

    fn lex_ok(src: &str) -> Vec<TokenKind> {
        let mut sm = SourceMap::new();
        let f = sm.add_file("test.cohdl", src);
        let mut diags = Diagnostics::new();
        let tokens = lex(f, src, &mut diags);
        assert!(
            !diags.has_errors(),
            "unexpected lex errors:\n{}",
            diags.render(&sm)
        );
        tokens.into_iter().map(|t| t.kind).collect()
    }

    fn lex_err(src: &str) -> String {
        let mut sm = SourceMap::new();
        let f = sm.add_file("test.cohdl", src);
        let mut diags = Diagnostics::new();
        let _ = lex(f, src, &mut diags);
        assert!(diags.has_errors(), "expected lex errors for {:?}", src);
        diags.render(&sm)
    }

    #[test]
    fn keywords_and_idents() {
        let toks = lex_ok("pub trait Capacitor: TwoTerminal { }");
        assert_eq!(toks[0], TokenKind::Pub);
        assert_eq!(toks[1], TokenKind::Trait);
        assert_eq!(toks[2], TokenKind::Ident("Capacitor".into()));
        assert_eq!(toks[3], TokenKind::Colon);
    }

    #[test]
    fn unit_literals_one_token() {
        let toks = lex_ok("100nF 3.3V 10kohm -40C 0.5% 16MHz");
        let units: Vec<UnitType> = toks
            .iter()
            .filter_map(|t| match t {
                TokenKind::Unit(v) => Some(v.unit),
                _ => None,
            })
            .collect();
        assert_eq!(
            units,
            vec![
                UnitType::Capacitance,
                UnitType::Voltage,
                UnitType::Resistance,
                UnitType::Temperature,
                UnitType::Tolerance,
                UnitType::Frequency
            ]
        );
    }

    #[test]
    fn turbofish_and_generics() {
        let toks = lex_ok("decoupling_cap::<V>(ferrite.OUT)");
        assert!(toks.contains(&TokenKind::PathSep));
        assert!(toks.contains(&TokenKind::Lt));
    }

    #[test]
    fn bare_numbers_stay_bare() {
        let toks = lex_ok("pins { A: 1, B: 2 }");
        assert!(toks.contains(&TokenKind::Number("1".into())));
    }

    #[test]
    fn unicode_omega_targeted_error() {
        let rendered = lex_err("10kΩ");
        assert!(rendered.contains("use `ohm`, not `Ω`"), "{}", rendered);
        // RFC-011: the SI-prefixed form is ONE targeted E101 with the full
        // rewrite (`10kohm`), not an unknown-suffix + stray-character cascade.
        assert!(rendered.contains("E101"), "{}", rendered);
        assert!(rendered.contains("write `10kohm`"), "{}", rendered);
        assert_eq!(
            rendered.matches("error[").count(),
            1,
            "expected exactly one diagnostic:\n{}",
            rendered
        );
    }

    #[test]
    fn unicode_degree_targeted_error() {
        let rendered = lex_err("85°C");
        assert!(rendered.contains("use `C`, not `°C`"), "{}", rendered);
    }

    #[test]
    fn bad_prefix_targeted_error() {
        let rendered = lex_err("1mC");
        assert!(
            rendered.contains("SI prefix `m` is not valid for `Temperature`"),
            "{}",
            rendered
        );
        let rendered = lex_err("10mF");
        assert!(
            rendered.contains("SI prefix `m` is not valid for `Capacitance`"),
            "{}",
            rendered
        );
    }

    #[test]
    fn negative_voltage_rejected() {
        let rendered = lex_err("-5V");
        assert!(rendered.contains("cannot be negative"), "{}", rendered);
    }

    #[test]
    fn comments_skipped() {
        let toks = lex_ok("inst c: MLCC // trailing comment\nnet _: c.A");
        assert!(toks.contains(&TokenKind::Inst));
        assert!(toks.contains(&TokenKind::Net));
    }
}

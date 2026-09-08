// CoHDL grammar for tree-sitter — the Zed extension's highlighting layer.
//
// Deliberately an EDITOR grammar, not a second compiler front end (the same
// stance as the VS Code TextMate grammar, RFC-019/DR-025): declaration
// headers, the statements worth naming, and the token vocabulary are parsed
// precisely; everything else is a sequence of typed tokens inside recursive
// bracket groups, so a construct this grammar does not know can never
// produce an ERROR node — it degrades to plain tokens. The compiler's
// hand-written parser (src/parse.rs) stays the one authority on what CoHDL
// IS. Any RFC that adds a top-level keyword must update this file AND the
// queries in ../languages/cohdl/ in the same change.

/**
 * @file CoHDL grammar for tree-sitter
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'cohdl',

  extras: $ => [/\s+/, $.comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._node),

    // Every construct is either a precisely-parsed form or a loose token;
    // groups recurse, so bodies at any depth get the same treatment.
    _node: $ => choice(
      $.declaration,
      $.use_declaration,
      $.impl_declaration,
      $.inst_statement,
      $.net_statement,
      $.pad_placement,
      $.attribute,
      $.block,
      $.parens,
      $.brackets,
      $.string,
      $.unit_literal,
      $.number,
      $.wildcard,
      $.identifier,
      $._keyword_fallback,
      ':', ',', '.', '::', '..=', '<', '>', '=', ';',
    ),

    // A structured-rule keyword in a position its rule cannot parse — e.g.
    // `footprint:` as a part-block KEY, exactly the contextual-keyword
    // discipline the compiler's parser has. The structured rules carry
    // higher precedence, so this fallback only wins where they cannot.
    _keyword_fallback: _ => choice(
      'device', 'trait', 'part', 'fn', 'design', 'subdesign',
      'footprint', 'pad', 'use', 'impl', 'for', 'inst', 'net', 'pub',
    ),

    comment: _ => token(seq('//', /[^\n]*/)),

    string: _ => token(seq('"', repeat(choice(/[^"\\\n]/, /\\./)), '"')),

    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,

    // RFC-001/018 unit literals: number + SI prefix + unit suffix, one
    // token — the same single-class shape as the TextMate grammar.
    unit_literal: _ => token(prec(2,
      /-?[0-9]+(\.[0-9]+)?(p|n|u|m|k|M|G)?(ohm|Hz|mm|V|F|A|s|H|W|C|%)/,
    )),

    number: _ => token(prec(1, /-?[0-9]+(\.[0-9]+)?/)),

    wildcard: _ => '_',

    // ---- precisely-parsed forms ----------------------------------------

    // `pub? KIND Name` — one rule for all eight named declaration kinds
    // (the header only; the body is an ordinary block that follows).
    declaration: $ => prec(1, seq(
      optional('pub'),
      field('kind', choice(
        'device', 'trait', 'part', 'fn',
        'design', 'subdesign', 'footprint', 'pad',
      )),
      field('name', $.identifier),
    )),

    use_declaration: $ => prec(1, seq('use', field('path', $.path))),

    impl_declaration: $ => prec(1, seq(
      'impl',
      field('trait', $.path),
      'for',
      field('device', $.path),
    )),

    inst_statement: $ => prec(1, seq('inst', field('name', $.identifier))),

    net_statement: $ => prec(1, seq(
      'net',
      field('name', choice($.identifier, $.wildcard)),
    )),

    // `pad N: …` inside a footprint body — the declaration rule above wins
    // when `pad` is followed by a name instead of a number.
    pad_placement: $ => prec(1, seq('pad', field('number', $.number))),

    path: $ => prec.right(seq(
      $.identifier,
      repeat(seq('::', $.identifier)),
    )),

    // ---- groups ---------------------------------------------------------

    attribute: $ => seq('#[', repeat($._node), ']'),
    block: $ => seq('{', repeat($._node), '}'),
    parens: $ => seq('(', repeat($._node), ')'),
    brackets: $ => seq('[', repeat($._node), ']'),
  },
});

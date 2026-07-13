//! CoHDL v2 compiler library.
//!
//! Pipeline (the verdict ladder): parses ⊂ resolves ⊂ type-checks ⊂ connects
//! ⊂ passes residual DRC ⊂ emits netlist.

pub mod ast;
pub mod check;
pub mod diag;
pub mod ir;
pub mod lex;
pub mod parse;
pub mod resolve;
pub mod span;
pub mod units;

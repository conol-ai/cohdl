//! CoHDL v2 compiler library.
//!
//! Pipeline (the verdict ladder): parses ⊂ resolves ⊂ type-checks ⊂ connects
//! ⊂ passes residual DRC ⊂ emits netlist.

pub mod ast;
pub mod check;
pub mod deps;
pub mod diag;
pub mod drc;
pub mod dxf;
pub mod emit;
pub mod fmt;
pub mod hash;
pub mod ir;
pub mod lex;
pub mod lock;
pub mod lsp;
pub mod parse;
pub mod pipeline;
pub mod project;
pub mod resolve;
pub mod span;
pub mod units;

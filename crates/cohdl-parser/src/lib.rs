use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CohdlParser;

pub mod lower;

pub use cohdl_syntax::ast::*;
pub use lower::{parse_source_file, ParseError};

#[cfg(test)]
mod tests;

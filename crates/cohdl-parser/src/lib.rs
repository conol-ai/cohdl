use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CohdlParser;

#[cfg(test)]
mod tests;

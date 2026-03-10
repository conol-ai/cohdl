//! `cohdl-syntax` — Typed AST definitions for the cohdl hardware-description
//! language.
//!
//! This crate provides the data structures that represent every syntactic
//! construct in a `.hdl` source file.  The root node is [`ast::SourceFile`].

pub mod ast;

// Re-export all public AST types at crate root for convenience.
pub use ast::*;

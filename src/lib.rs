//! CoHDL v2 compiler library.
//!
//! Pipeline (the verdict ladder): parses ⊂ resolves ⊂ type-checks ⊂ connects
//! ⊂ passes residual DRC ⊂ emits netlist.

pub mod diag;
pub mod span;
pub mod units;

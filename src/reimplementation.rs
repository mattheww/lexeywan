//! Reimplementation of rustc's lexical analysis.

pub mod cleaning;
pub mod doc_lowering;
pub mod fine_tokens;
pub mod tokenisation;

mod pegs;

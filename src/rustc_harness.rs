//! Run rustc's lexer and extract the results.
//!
//! The submodules provide different ways to run the relevant parts of rustc, and support code for
//! integrating with rustc's internals.

// Everything that uses rustc_private should be inside this module.

// The code in this module compiles with
// rustc 1.93.0-nightly (b33119ffd 2025-12-04)

pub mod lex_via_decl_macros;
pub mod lex_via_rustc;
pub mod rustc_tokens;

mod error_accumulator;
mod rustc_tokenstreams;

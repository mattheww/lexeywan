//! Run rustc's lexer and extract the results.
//!
//! The submodules provide different ways to run the relevant parts of rustc, and support code for
//! integrating with rustc's internals.

// Everything that uses rustc_private should be inside this module.

// The code in this module compiles with
// rustc 1.92.0-nightly (caccb4d03 2025-09-24)

pub mod decl_via_rustc;
pub mod lex_via_rustc;
pub mod rustc_tokens;
pub mod rustc_tokenstreams;

mod error_accumulator;

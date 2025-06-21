//! Run rustc's lexer and extract the results.
//!
//! The submodules provide different ways to run the relevant parts of rustc, and support code for
//! integrating with rustc's internals.

// Everything that uses rustc_private should be inside this module.

pub mod decl_via_rustc;
pub mod lex_via_rustc;

mod error_accumulator;

#![feature(rustc_private)]

mod char_sequences;
mod cleaning;
mod combination;
mod command_line;
mod comparison;
mod direct_lexing;
mod doc_lowering;
mod fine_tokens;
mod lex_via_peg;
mod lex_via_rustc;
mod proptesting;
mod regular_tokens;
mod simple_reports;
mod testcases;
mod tokens_common;
mod tree_construction;
mod tree_flattening;
mod trees;
mod utils;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Edition {
    /// Rust 2015 and Rust 2018
    E2015,
    /// Rust 2021
    E2021,
    /// Rust 2024
    E2024,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Lowering {
    /// Omit the "Convert doc-comments to attributes" pass
    NoLowering,
    /// Include the "Convert doc-comments to attributes" pass
    LowerDocComments,
}

fn main() -> impl std::process::Termination {
    command_line::run_cli()
}

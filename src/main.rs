#![feature(rustc_private)]

mod char_sequences;
mod cleaning;
mod combination;
mod command_line;
mod comparison;
mod lex_via_rustc;
mod lexlucid;
mod proptesting;
mod regular_tokens;
mod simple_reports;
mod testcases;
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

fn main() -> impl std::process::Termination {
    command_line::run_cli()
}

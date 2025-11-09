#![feature(rustc_private)]

mod alignment;
mod combination;
mod datatypes;
mod framework;
mod reimplementation;
mod rustc_harness;
mod tokens_common;

#[derive(Copy, Clone, PartialEq, Eq, std::fmt::Debug)]
enum Edition {
    /// Rust 2015 and Rust 2018
    E2015,
    /// Rust 2021
    E2021,
    /// Rust 2024
    E2024,
}

const ALL_EDITIONS: &[Edition] = [Edition::E2015, Edition::E2021, Edition::E2024].as_slice();
const LATEST_EDITION: Edition = Edition::E2024;

#[derive(Copy, Clone, PartialEq, Eq, std::fmt::Debug)]
enum Lowering {
    /// Omit the "Convert doc-comments to attributes" pass
    NoLowering,
    /// Include the "Convert doc-comments to attributes" pass
    LowerDocComments,
}

#[derive(Copy, Clone, PartialEq, Eq, std::fmt::Debug)]
enum CleaningMode {
    /// Strip neither shebang nor frontmatter
    NoCleaning,
    /// Strip the shebang but not frontmatter
    CleanShebang,
    /// Strip both shebang and frontmatter
    CleanShebangAndFrontmatter,
}

fn main() -> impl std::process::Termination {
    framework::command_line::run_cli()
}

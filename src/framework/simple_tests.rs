//! Implementation of the test-like cli subcommands.
//!
//! These subcommands are:
//!  `test`
//!  `identcheck`

use std::io::Write as _;

use crate::alignment::decl_lexing::{stringified_via_declarative_macros, stringified_via_peg};
use crate::alignment::direct_lexing::{regularised_from_peg, regularised_from_rustc};
use crate::{ALL_EDITIONS, CleaningMode, Edition, LATEST_EDITION, Lowering};

use super::command_line::SubcommandStatus;
use super::comparison::{Comparison, compare};

/// Implements the `test` (default) CLI command.
pub fn run_test_subcommand(inputs: &[&str]) -> SubcommandStatus {
    use {CleaningMode::*, Lowering::*};
    let mut any_failed = false;
    let start = |label| {
        print!("{label:48} ...");
        std::io::stdout().flush().expect("failed to flush stdout");
    };
    let mut finish = |comparison| match comparison {
        Comparison::Agree => {
            println!(" ok");
        }
        Comparison::Differ => {
            println!(" failed");
            any_failed = true;
        }
        Comparison::ModelErrors => {
            println!(" model error");
            any_failed = true;
        }
    };

    let comparison_runs = [
        (NoLowering, NoCleaning),
        (NoLowering, CleanShebang),
        (NoLowering, CleanShebangAndFrontmatter),
        (LowerDocComments, CleanShebang),
    ]
    .as_slice();
    for edition in ALL_EDITIONS.iter().copied() {
        for (lowering, cleaning) in comparison_runs.iter().copied() {
            start(format!("{edition:?} / {lowering:?} / {cleaning:?}"));
            finish(compare_directly(inputs, edition, cleaning, lowering));
        }
        start(format!("{edition:?} / via declarative macros"));
        finish(compare_via_decl(inputs, edition));
    }
    if any_failed {
        println!("*** failed ***");
        SubcommandStatus::ChecksFailed
    } else {
        SubcommandStatus::Normal
    }
}

fn compare_directly(
    inputs: &[&str],
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> Comparison {
    for input in inputs {
        let rustc = regularised_from_rustc(input, edition, cleaning, lowering);
        let lex_via_peg = regularised_from_peg(input, edition, cleaning, lowering);
        let comparison = compare(&rustc, &lex_via_peg);
        if !matches!(comparison, Comparison::Agree) {
            return comparison;
        }
    }
    Comparison::Agree
}

fn compare_via_decl(inputs: &[&str], edition: Edition) -> Comparison {
    for input in inputs {
        let rustc = stringified_via_declarative_macros(input, edition);
        let lex_via_peg = stringified_via_peg(input, edition);
        let comparison = compare(&rustc, &lex_via_peg);
        if !matches!(comparison, Comparison::Agree) {
            return comparison;
        }
    }
    Comparison::Agree
}

/// Implements the `identcheck` CLI command.
pub fn run_identcheck_subcommand() -> SubcommandStatus {
    // This will report errors if there's a unicode version mismatch.
    // At present I think CleanShebang is the fastest mode
    let edition = LATEST_EDITION;
    let cleaning = CleaningMode::CleanShebang;
    let lowering = Lowering::NoLowering;
    println!("Checking all characters as XID_Start and XID_Continue");
    let mut passes = 0;
    let mut failures = 0;
    let mut model_errors = 0;
    for c in char::MIN..=char::MAX {
        let input = format!("{c} a{c}");
        let rustc = regularised_from_rustc(&input, edition, cleaning, lowering);
        let lex_via_peg = regularised_from_peg(&input, edition, cleaning, lowering);
        match compare(&rustc, &lex_via_peg) {
            Comparison::Agree => passes += 1,
            Comparison::Differ => failures += 1,
            Comparison::ModelErrors => model_errors += 1,
        }
    }
    println!("\n{passes} passed, {failures} failed");
    if model_errors != 0 {
        println!("*** {model_errors} model errors ***");
    }
    if failures == 0 && model_errors == 0 {
        SubcommandStatus::Normal
    } else {
        SubcommandStatus::ChecksFailed
    }
}

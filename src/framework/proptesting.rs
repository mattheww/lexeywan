//! Uses `proptest` to compare the two lexer implementations.

use proptest::{
    strategy::{BoxedStrategy, Strategy},
    test_runner::{Config, TestCaseError, TestError, TestRunner},
};

use crate::alignment::comparison::{Comparison, compare};
use crate::alignment::direct_lexing::{regularised_from_peg, regularised_from_rustc};
use crate::datatypes::char_sequences::escape_for_display;
use crate::{CleaningMode, Edition, Lowering};

use super::command_line::SubcommandStatus;

pub use self::strategies::DEFAULT_STRATEGY;
use self::strategies::SIMPLE_STRATEGIES;

mod strategies;

/// Implements the `proptest` cli subcommand.
pub fn run_proptests(
    strategy_name: &str,
    count: u32,
    verbosity: Verbosity,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> SubcommandStatus {
    println!("Running property tests with strategy {strategy_name} for {count} iterations");
    let mut runner = TestRunner::new(Config {
        cases: count,
        verbose: verbosity.into(),
        failure_persistence: None,
        ..Config::default()
    });
    let strategy = &named_strategy(strategy_name).expect("unknown strategy");
    let result = runner.run(strategy, |input| {
        match check_lexing(&input, edition, cleaning, lowering) {
            ComparisonStatus::Pass => Ok(()),
            ComparisonStatus::Fail(msg) => Err(TestCaseError::Fail(msg.into())),
            ComparisonStatus::Unsupported(msg) => Err(TestCaseError::Reject(msg.into())),
        }
    });
    match result {
        Ok(_) => {
            println!("No discrepancies found");
            SubcommandStatus::Normal
        }
        Err(TestError::Fail(reason, value)) => {
            println!(
                "Found minimal failing case: {}: {}",
                escape_for_display(&value),
                reason
            );
            SubcommandStatus::ChecksFailed
        }
        Err(TestError::Abort(reason)) => {
            println!("Proptest aborted: {reason}");
            SubcommandStatus::ChecksFailed
        }
    }
}

/// Checks whether the lex_via_peg and rustc models agree for the specified input.
///
/// This is the "test" function given to proptest.
///
/// Returns Unsupported for input that may trigger known problems.
fn check_lexing(
    input: &str,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> ComparisonStatus {
    // See the history of this function for how to use `Unsupported`

    let rustc = regularised_from_rustc(input, edition, cleaning, lowering);
    let lex_via_peg = regularised_from_peg(input, edition, cleaning, lowering);
    match compare(&rustc, &lex_via_peg) {
        Comparison::Agree => ComparisonStatus::Pass,
        Comparison::Differ => ComparisonStatus::Fail(format!("disagreement for «{input}»")),
        Comparison::ModelErrors => ComparisonStatus::Fail("model error".into()),
    }
}

enum ComparisonStatus {
    Pass,
    Fail(String),
    #[allow(unused)]
    Unsupported(String),
}

/// Returns a list of the names of the available strategies.
pub fn strategy_names() -> Vec<&'static str> {
    let mut names = vec!["mix"];
    names.extend(SIMPLE_STRATEGIES.iter().map(|(name, _)| name).copied());
    names
}

fn named_strategy(name: &str) -> Option<BoxedStrategy<String>> {
    let strategy = SIMPLE_STRATEGIES
        .iter()
        .find(|(strategy_name, _)| *strategy_name == name)
        .map(|(_, strategy)| strategy.boxed());
    if strategy.is_some() {
        return strategy;
    }
    if name == "mix" {
        return Some(strategies::mix());
    }
    None
}

pub enum Verbosity {
    Quiet,
    PrintFailures,
    PrintAll,
}

impl From<Verbosity> for u32 {
    fn from(verbosity: Verbosity) -> Self {
        match verbosity {
            Verbosity::Quiet => 0,
            Verbosity::PrintFailures => 1,
            Verbosity::PrintAll => 2,
        }
    }
}

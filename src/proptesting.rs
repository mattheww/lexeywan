//! Uses `proptest` to compare the two lexer implementations.

use proptest::{
    strategy::{BoxedStrategy, Strategy},
    test_runner::{Config, TestCaseError, TestError, TestRunner},
};
use regex::Regex;

use crate::Edition;
use crate::{
    comparison::{compare, regularised_from_lexlucid, regularised_from_rustc, Comparison},
    utils::escape_for_display,
};

pub use self::strategies::DEFAULT_STRATEGY;
use self::strategies::SIMPLE_STRATEGIES;

mod strategies;

macro_rules! make_regex_with_default_flags {
    ($re:literal $(,)?) => {{
        static RE: ::std::sync::OnceLock<regex::Regex> = ::std::sync::OnceLock::new();
        RE.get_or_init(|| Regex::new($re).unwrap())
    }};
}

/// Implements the `proptest` cli subcommand.
pub fn run_proptests(strategy_name: &str, count: u32, verbosity: Verbosity, edition: Edition) {
    println!("Running property tests with strategy {strategy_name} for {count} iterations");
    let mut runner = TestRunner::new(Config {
        cases: count,
        verbose: verbosity.into(),
        failure_persistence: None,
        ..Config::default()
    });
    let strategy = &named_strategy(strategy_name).expect("unknown strategy");
    let result = runner.run(strategy, |input| match check_lexing(&input, edition) {
        ComparisonStatus::Pass => Ok(()),
        ComparisonStatus::Fail(msg) => Err(TestCaseError::Fail(msg.into())),
        ComparisonStatus::Unsupported(msg) => Err(TestCaseError::Reject(msg.into())),
    });
    match result {
        Ok(_) => println!("No discrepancies found"),
        Err(TestError::Fail(reason, value)) => {
            println!(
                "Found minimal failing case: {}: {}",
                escape_for_display(&value),
                reason
            );
        }
        Err(TestError::Abort(reason)) => {
            println!("Proptest aborted: {}", reason);
        }
    }
}

/// Checks whether the lexlucid and rustc models agree for the specified input.
///
/// This is the "test" function given to proptest.
///
/// Returns Unsupported for input that may trigger known problems.
fn check_lexing(input: &str, edition: Edition) -> ComparisonStatus {
    if edition == Edition::E2015 {
        // In Rust 2015 and 2018, emoji in unknown prefixes aren't reported as an error.
        // I think this is a rustc bug, so exclude such cases from testing.
        let re =
            make_regex_with_default_flags!(r#"[\p{EMOJI}--!-~][\p{XID_Continue}\p{EMOJI}]*[#'"]"#);
        if re.is_match(input) {
            return ComparisonStatus::Unsupported("emoji-in-unknown-prefix".into());
        }
    }

    let rustc = regularised_from_rustc(input, edition);
    let lexlucid = regularised_from_lexlucid(input, edition);
    match compare(&rustc, &lexlucid) {
        Comparison::Agree => ComparisonStatus::Pass,
        Comparison::Differ => ComparisonStatus::Fail("rustc and lexlucid disagree".into()),
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
    let mut names = vec!["any-char", "mix"];
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
    if name == "any-char" {
        return Some(strategies::any_char());
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

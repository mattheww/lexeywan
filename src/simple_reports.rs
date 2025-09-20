//! Implementation of the 'reporting' cli subcommands.
//!
//! These subcommands are:
//!  `compare`
//!  `decl-compare`
//!  `inspect`
//!  `coarse`
//!  `identcheck`

use std::fmt::Debug;

use crate::cleaning;
use crate::combination;
use crate::comparison::{compare, Comparison, Verdict};
use crate::decl_lexing::{stringified_via_declarative_macros, stringified_via_peg};
use crate::direct_lexing::{regularised_from_peg, regularised_from_rustc};
use crate::doc_lowering::lower_doc_comments;
use crate::fine_tokens::FineToken;
use crate::lex_via_peg;
use crate::rustc_harness::lex_via_rustc;
use crate::tokens_common::Origin;
use crate::tree_construction;
use crate::tree_flattening::flatten;
use crate::trees::Forest;
use crate::utils::escape_for_display;
use crate::{CleaningMode, Edition, Lowering};

/// Implements the `compare` (default) CLI command.
pub fn run_compare_subcommand(
    inputs: &[&str],
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
    details_mode: DetailsMode,
    show_failures_only: bool,
) {
    let mut passes = 0;
    let mut failures = 0;
    let mut model_errors = 0;
    for input in inputs {
        match show_comparison(
            input,
            edition,
            cleaning,
            lowering,
            details_mode,
            show_failures_only,
        ) {
            Comparison::Agree => passes += 1,
            Comparison::Differ => failures += 1,
            Comparison::ModelErrors => model_errors += 1,
        }
    }
    println!("\n{passes} passed, {failures} failed");
    if model_errors != 0 {
        println!("*** {model_errors} model errors ***");
    }
}

/// Implements the `decl-compare` CLI command.
pub fn run_decl_compare_subcommand(
    inputs: &[&str],
    edition: Edition,
    details_mode: DetailsMode,
    show_failures_only: bool,
) {
    let mut passes = 0;
    let mut failures = 0;
    let mut model_errors = 0;
    for input in inputs {
        match show_decl_compare(input, edition, details_mode, show_failures_only) {
            Comparison::Agree => passes += 1,
            Comparison::Differ => failures += 1,
            Comparison::ModelErrors => model_errors += 1,
        }
    }
    println!("\n{passes} passed, {failures} failed");
    if model_errors != 0 {
        println!("*** {model_errors} model errors ***");
    }
}

/// Implements the `inspect` CLI command.
pub fn run_inspect_subcommand(
    inputs: &[&str],
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) {
    for input in inputs {
        show_inspect(input, edition, cleaning, lowering);
        println!();
    }
}

/// Implements the `coarse` CLI command.
pub fn run_coarse_subcommand(
    inputs: &[&str],
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) {
    for input in inputs {
        show_coarse(input, edition, cleaning, lowering);
        println!();
    }
}

/// Implements the `identcheck` CLI command.
pub fn run_identcheck_subcommand(edition: Edition) {
    // At present I think CleanShebang is the fastest mode
    show_identcheck(edition, CleaningMode::CleanShebang, Lowering::NoLowering);
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DetailsMode {
    Never,
    Failures,
    Always,
}

fn format_pretoken(pretoken: &lex_via_peg::Pretoken) -> String {
    format!("{:?}, {:?}", pretoken.data, pretoken.extent)
}
fn format_token(token: &FineToken) -> String {
    match &token.origin {
        Origin::Natural { extent } => format!("{:?}, {:?}", token.data, extent),
        Origin::Synthetic { lowered_from, .. } => {
            format!("{:?}, lowered from {:?}", token.data, lowered_from)
        }
    }
}

/// Returns a symbol indicating how a single model responded to the input.
fn single_model_symbol<T: Eq>(reg: &Verdict<T>) -> char {
    match reg {
        Verdict::Accepts(_) => '✓',
        Verdict::Rejects(_) => '✗',
        Verdict::ModelError(_) => '💣',
    }
}

/// Compares 'regularised' tokens from rustc and lex_via_peg.
///
/// Shows whether the tokenisations match.
/// May also show detail, depending on `details_mode`.
///
/// Returns the result of the comparison.
fn show_comparison(
    input: &str,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
    details_mode: DetailsMode,
    show_failures_only: bool,
) -> Comparison {
    let rustc = regularised_from_rustc(input, edition, cleaning, lowering);
    let lex_via_peg = regularised_from_peg(input, edition, lowering);
    report_verdict(input, details_mode, show_failures_only, rustc, lex_via_peg)
}

/// Compares stringified forms from rustc declarative macros and the reimplementation.
///
/// Shows whether the stringified forms match.
/// May also show detail, depending on `details_mode`.
///
/// Returns the result of the comparison.
fn show_decl_compare(
    input: &str,
    edition: Edition,
    details_mode: DetailsMode,
    show_failures_only: bool,
) -> Comparison {
    let rustc = stringified_via_declarative_macros(input, edition);
    let lex_via_peg = stringified_via_peg(input, edition);
    report_verdict(input, details_mode, show_failures_only, rustc, lex_via_peg)
}

/// Lexes with both rustc and lex_via_peg, and prints the results.
fn show_inspect(input: &str, edition: Edition, cleaning: CleaningMode, lowering: Lowering) {
    println!("Lexing «{}»", escape_for_display(input));
    match lex_via_rustc::analyse(input, edition, cleaning, lowering) {
        lex_via_rustc::Analysis::Accepts(tokens) => {
            println!("rustc: accepted");
            for item in flatten(&tokens) {
                println!("  {item:?}");
            }
        }
        lex_via_rustc::Analysis::Rejects(tokens, messages) => {
            println!("rustc: rejected");
            for s in messages {
                println!("  error: {s}");
            }
            if !tokens.is_empty() {
                println!("  -- tokens reported --");
                for item in flatten(&tokens) {
                    println!("  {item:?}");
                }
            }
        }
        lex_via_rustc::Analysis::CompilerError => {
            println!("rustc: internal compiler error");
        }
        lex_via_rustc::Analysis::HarnessError(message) => {
            println!("rustc: internal error in harness: {message}");
        }
    }
    let cleaned = cleaning::clean(&input.into(), edition);
    match lex_via_peg::analyse(&cleaned, edition) {
        lex_via_peg::Analysis::Accepts(pretokens, mut tokens) => {
            if lowering == Lowering::LowerDocComments {
                tokens = lower_doc_comments(tokens, edition);
            }
            match tree_construction::construct_forest(tokens.clone()) {
                Ok(_) => {
                    println!("lex_via_peg: accepted");
                }
                Err(message) => {
                    println!("lex_via_peg: rejected in step 3 (tree construction)");
                    println!("  error: {message}");
                }
            }
            println!("  -- pretokens --");
            for pretoken in pretokens {
                println!("  {}", format_pretoken(&pretoken));
            }
            println!("  -- tokens --");
            for token in tokens.iter() {
                println!("  {}", format_token(token));
            }
        }
        lex_via_peg::Analysis::Rejects(lex_via_peg::Reason::Pretokenisation(
            messages,
            pretokens,
            _,
        )) => {
            println!("lex_via_peg: rejected in step 1 (pretokenisation)");
            for message in messages {
                println!("  error: {message}");
            }
            println!("  -- previous pretokens --");
            for pretoken in pretokens {
                println!("  {}", format_pretoken(&pretoken));
            }
        }
        lex_via_peg::Analysis::Rejects(lex_via_peg::Reason::Reprocessing(
            message,
            rejected,
            pretokens,
            mut tokens,
        )) => {
            if lowering == Lowering::LowerDocComments {
                tokens = lower_doc_comments(tokens, edition);
            }
            println!("lex_via_peg: rejected in step 2 (reprocessing)");
            println!("  error: {message}");
            println!("  -- rejected pretoken: --");
            println!("  {}", format_pretoken(&rejected));
            println!("  -- previous pretokens --");
            for pretoken in pretokens {
                println!("  {}", format_pretoken(&pretoken));
            }
            println!("  -- previous tokens --");
            for token in tokens {
                println!("  {}", format_token(&token));
            }
        }
        lex_via_peg::Analysis::ModelError(reason) => {
            println!("lex_via_peg: reported a bug in its model");
            for s in reason.into_description() {
                println!("  error: {s}");
            }
        }
    }
}

fn show_coarse(input: &str, edition: Edition, cleaning: CleaningMode, lowering: Lowering) {
    println!("Lexing «{}»", escape_for_display(input));
    let cleaned = cleaning::clean(&input.into(), edition);
    match lex_via_peg::analyse(&cleaned, edition) {
        lex_via_peg::Analysis::Accepts(_, mut tokens) => {
            if lowering == Lowering::LowerDocComments {
                tokens = lower_doc_comments(tokens, edition);
            }
            println!("lex_via_peg: accepted");
            println!("  -- fine-grained --");
            for token in tokens.iter() {
                println!("  {}", format_token(token));
            }
            match tree_construction::construct_forest(tokens) {
                Ok(forest) => {
                    let combined = combination::coarsen(forest);
                    println!("  -- coarse --");
                    for item in flatten(&combined) {
                        println!("  {item:?}");
                    }
                }
                Err(message) => {
                    println!("lex_via_peg: rejected during tree construction: {message}");
                }
            }
        }
        lex_via_peg::Analysis::Rejects(reason) => {
            println!("lex_via_peg: rejected");
            for message in reason.into_description() {
                println!("  {message}");
            }
        }
        lex_via_peg::Analysis::ModelError(reason) => {
            println!("lex_via_peg: reported a bug in its model:");
            for s in reason.into_description() {
                println!("  error: {s}");
            }
        }
    }
}

/// Common implementation for reports which compare two models of the lexer.
fn report_verdict<TOKEN: Eq + Debug>(
    input: &str,
    details_mode: DetailsMode,
    show_failures_only: bool,
    rustc: Verdict<Forest<TOKEN>>,
    lex_via_peg: Verdict<Forest<TOKEN>>,
) -> Comparison {
    let comparison = compare(&rustc, &lex_via_peg);

    let passes = matches!(comparison, Comparison::Agree);
    if passes && show_failures_only {
        return comparison;
    }
    let show_detail = (details_mode == DetailsMode::Always)
        || ((details_mode == DetailsMode::Failures) && !passes);

    println!(
        "{} R:{} L:{} «{}»",
        match comparison {
            Comparison::Agree => '✔',
            Comparison::Differ => '‼',
            Comparison::ModelErrors => '💣',
        },
        single_model_symbol(&rustc),
        single_model_symbol(&lex_via_peg),
        escape_for_display(input)
    );

    if show_detail {
        match rustc {
            Verdict::Accepts(tokens) => {
                println!("  rustc: accepted");
                for item in flatten(&tokens) {
                    println!("    {item:?}");
                }
            }
            Verdict::Rejects(messages) => {
                println!("  rustc: rejected");
                for msg in messages {
                    println!("    {msg}");
                }
            }
            Verdict::ModelError(messages) => {
                println!("  rustc: reported model error");
                for msg in messages {
                    println!("    {msg}");
                }
            }
        };
        match lex_via_peg {
            Verdict::Accepts(tokens) => {
                println!("  lex_via_peg: accepted");
                for item in flatten(&tokens) {
                    println!("    {item:?}");
                }
            }
            Verdict::Rejects(messages) => {
                println!("  lex_via_peg: rejected");
                for msg in messages {
                    println!("    {msg}");
                }
            }
            Verdict::ModelError(messages) => {
                println!("  lex_via_peg: reported a bug in its model");
                for msg in messages {
                    println!("    {msg}");
                }
            }
        }
    }
    comparison
}

fn show_identcheck(edition: Edition, cleaning: CleaningMode, lowering: Lowering) {
    // This will report errors if there's a unicode version mismatch.
    println!("Checking all characters as XID_Start and XID_Continue");
    let mut passes = 0;
    let mut failures = 0;
    let mut model_errors = 0;
    for c in char::MIN..=char::MAX {
        for input in [format!("{c}"), format!("a{c}")] {
            match show_comparison(
                &input,
                edition,
                cleaning,
                lowering,
                DetailsMode::Never,
                true,
            ) {
                Comparison::Agree => passes += 1,
                Comparison::Differ => failures += 1,
                Comparison::ModelErrors => model_errors += 1,
            }
        }
    }
    println!("\n{passes} passed, {failures} failed");
    if model_errors != 0 {
        println!("*** {model_errors} model errors ***");
    }
}

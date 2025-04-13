//! Implementation of the 'reporting' cli subcommands.
//!
//! These subcommands are:
//!  `compare`
//!  `inspect`
//!  `course`

use crate::cleaning;
use crate::combination;
use crate::comparison::{
    compare, regularised_from_lexlucid, regularised_from_rustc, Comparison, Regularisation,
};
use crate::fine_tokens::FineToken;
use crate::lex_via_rustc;
use crate::lexlucid;
use crate::utils::escape_for_display;
use crate::Edition;

/// Implements the `compare` (default) CLI command.
pub fn run_compare_subcommand(
    inputs: &[&str],
    edition: Edition,
    details_mode: DetailsMode,
    show_failures_only: bool,
) {
    let mut passes = 0;
    let mut failures = 0;
    let mut model_errors = 0;
    for input in inputs {
        match show_comparison(input, edition, details_mode, show_failures_only) {
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
pub fn run_inspect_subcommand(inputs: &[&str], edition: Edition) {
    for input in inputs {
        show_detail(input, edition);
        println!();
    }
}

/// Implements the `coarse` CLI command.
pub fn run_coarse_subcommand(inputs: &[&str], edition: Edition) {
    for input in inputs {
        show_coarse(input, edition);
        println!();
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DetailsMode {
    Never,
    Failures,
    Always,
}

fn format_pretoken(pretoken: &lexlucid::Pretoken) -> String {
    format!("{:?}, {:?}", pretoken.data, pretoken.extent)
}
fn format_token(token: &FineToken) -> String {
    format!("{:?}, {:?}", token.data, token.extent)
}
fn format_coarse_token(ctoken: &combination::CoarseToken) -> String {
    format!("{:?}, {:?}", ctoken.data, ctoken.extent)
}

/// Returns a symbol indicating how a single model responded to the input.
fn single_model_symbol(reg: &Regularisation) -> char {
    match reg {
        Regularisation::Accepts(_) => '✓',
        Regularisation::Rejects(_) => '✗',
        Regularisation::ModelError(_) => '💣',
    }
}

/// Compares 'regularised' tokens from rustc and lexlucid.
///
/// Shows whether the tokenisations match.
/// May also show detail, depending on `details_mode`.
///
/// Returns the result of the comparison.
fn show_comparison(
    input: &str,
    edition: Edition,
    details_mode: DetailsMode,
    show_failures_only: bool,
) -> Comparison {
    let rustc = regularised_from_rustc(input, edition);
    let lexlucid = regularised_from_lexlucid(input, edition);
    let comparison = compare(&rustc, &lexlucid);

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
        single_model_symbol(&lexlucid),
        escape_for_display(input)
    );

    if show_detail {
        match rustc {
            Regularisation::Accepts(tokens) => {
                println!("  rustc: accepted");
                for token in tokens {
                    println!("    {:?}", token);
                }
            }
            Regularisation::Rejects(messages) => {
                println!("  rustc: rejected");
                for msg in messages {
                    println!("    {msg}");
                }
            }
            Regularisation::ModelError(messages) => {
                println!("  rustc: reported model error");
                for msg in messages {
                    println!("    {msg}");
                }
            }
        };
        match lexlucid {
            Regularisation::Accepts(tokens) => {
                println!("  lexlucid: accepted");
                for token in tokens {
                    println!("    {:?}", token);
                }
            }
            Regularisation::Rejects(messages) => {
                println!("  lexlucid: rejected");
                for msg in messages {
                    println!("    {msg}");
                }
            }
            Regularisation::ModelError(messages) => {
                println!("  lexlucid: reported a bug in its model");
                for msg in messages {
                    println!("    {msg}");
                }
            }
        }
    }
    comparison
}

/// Lexes with both rustc and lexlucid, and prints the results.
fn show_detail(input: &str, edition: Edition) {
    println!("Lexing «{}»", escape_for_display(input));
    match lex_via_rustc::analyse(input, edition) {
        lex_via_rustc::Analysis::Accepts(tokens) => {
            println!("rustc: accepted");
            for token in tokens {
                println!("  {}", token.summary);
            }
        }
        lex_via_rustc::Analysis::Rejects(tokens, messages) => {
            println!("rustc: rejected");
            for s in messages {
                println!("  error: {}", s);
            }
            if !tokens.is_empty() {
                println!("  -- tokens reported --");
                for token in tokens {
                    println!("  {}", token.summary);
                }
            }
        }
        lex_via_rustc::Analysis::CompilerError => {
            println!("rustc: internal compiler error");
        }
    }
    let cleaned = cleaning::clean(input);
    match lexlucid::analyse(&cleaned, edition) {
        lexlucid::Analysis::Accepts(pretokens, tokens) => {
            println!("lexlucid: accepted");
            println!("  -- pretokens --");
            for pretoken in pretokens {
                println!("  {}", format_pretoken(&pretoken));
            }
            println!("  -- tokens --");
            for token in tokens {
                println!("  {}", format_token(&token));
            }
        }
        lexlucid::Analysis::Rejects(lexlucid::Reason::Pretokenisation(messages, pretokens, _)) => {
            println!("lexlucid: rejected in step 1 (pretokenisation)");
            for message in messages {
                println!("  error: {message}");
            }
            println!("  -- previous pretokens --");
            for pretoken in pretokens {
                println!("  {}", format_pretoken(&pretoken));
            }
        }
        lexlucid::Analysis::Rejects(lexlucid::Reason::Reprocessing(
            message,
            rejected,
            pretokens,
            tokens,
        )) => {
            println!("lexlucid: rejected in step 2 (reprocessing)");
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
        lexlucid::Analysis::ModelError(reason) => {
            println!("lexlucid: reported a bug in its model");
            for s in reason.into_description() {
                println!("  error: {}", s);
            }
        }
    }
}

fn show_coarse(input: &str, edition: Edition) {
    println!("Lexing «{}»", escape_for_display(input));
    let cleaned = cleaning::clean(input);
    match lexlucid::analyse(&cleaned, edition) {
        lexlucid::Analysis::Accepts(_, tokens) => {
            println!("lexlucid: accepted");
            println!("  -- fine-grained --");
            for token in tokens.iter() {
                println!("  {}", format_token(token));
            }
            let combined = combination::coarsen(tokens);
            println!("  -- coarse --");
            for ctoken in combined {
                println!("  {} {:?}", format_coarse_token(&ctoken), &ctoken.spacing);
            }
        }
        lexlucid::Analysis::Rejects(reason) => {
            println!("lexlucid: rejected");
            for message in reason.into_description() {
                println!("  {message}");
            }
        }
        lexlucid::Analysis::ModelError(reason) => {
            println!("lexlucid: reported a bug in its model:");
            for s in reason.into_description() {
                println!("  error: {}", s);
            }
        }
    }
}

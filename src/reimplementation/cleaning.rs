//! Transformations we make to input text before tokenisation.
//!
//! See "Processing that happens before tokenising" for the behaviour we're imitating.
//!
//! Note the reimplementation doesn't model the "Decoding" step (it isn't interesting, and it's more
//! convenient to have the testcases supplied as `str`).

use crate::char_sequences::Charseq;
use crate::{CleaningMode, Edition};

use super::fine_tokens::{FineToken, FineTokenData};
use super::tokenisation::first_nonwhitespace_token;

use self::frontmatter::{FrontmatterOutcome, find_frontmatter};

mod frontmatter;

/// Apply the transformations we make to input text before tokenisation.
///
/// Honours the requested cleaning mode.
pub fn clean(input: &Charseq, edition: Edition, cleaning: CleaningMode) -> CleaningOutcome {
    use CleaningMode::*;
    use CleaningOutcome::*;
    let cleaned = input.chars();
    let cleaned = remove_bom(cleaned);
    let mut cleaned = replace_crlf(cleaned);
    if matches!(cleaning, CleanShebang | CleanShebangAndFrontmatter) {
        cleaned = clean_shebang(cleaned, edition);
    }
    if matches!(cleaning, CleanShebangAndFrontmatter) {
        match find_frontmatter(cleaned.chars()) {
            FrontmatterOutcome::NotFound => {}
            FrontmatterOutcome::Found(range) => {
                cleaned.remove_range(range);
            }
            FrontmatterOutcome::Reserved => return Rejects("malformed frontmatter".into()),
            FrontmatterOutcome::ModelError(message) => {
                return ModelError(format!("frontmatter processing failed: {message}"));
            }
        }
    }
    Accepts(cleaned)
}

pub enum CleaningOutcome {
    /// Cleaning succeeded.
    Accepts(Charseq),

    /// Cleaning rejected the input.
    ///
    /// The string is an explanation.
    Rejects(String),

    /// The input demonstrated a problem in the reimplementation.
    ///
    /// The string is an error message.
    ModelError(String),
}

/// Apply the transformations we make to input text before tokenisation inside a declarative macro.
///
/// This always behaves like cleaning mode NoCleaning.
#[allow(clippy::let_and_return)]
pub fn clean_for_macro_input(input: &Charseq, _edition: Edition) -> Charseq {
    let cleaned = input.chars();
    let cleaned = replace_crlf(cleaned);
    cleaned
}

/// Performs "Byte order mark removal"
fn remove_bom(input: &[char]) -> &[char] {
    if input.starts_with(&['\u{feff}']) {
        &input[1..]
    } else {
        input
    }
}

/// Performs "CRLF normalisation"
fn replace_crlf(input: &[char]) -> Charseq {
    let mut rewritten = Vec::with_capacity(input.len());
    let mut it = input.iter().copied().peekable();
    while let Some(c) = it.next() {
        if c != '\r' || it.peek() != Some(&'\n') {
            rewritten.push(c);
        }
    }
    Charseq::new(rewritten)
}

/// Performs "Shebang removal"
fn clean_shebang(mut input: Charseq, edition: Edition) -> Charseq {
    if !input.chars().starts_with(&['#', '!']) {
        return input;
    };
    if let Some(FineToken {
        data: FineTokenData::Punctuation { mark: '[' },
        ..
    }) = first_nonwhitespace_token(&input[2..], edition)
    {
        return input;
    }
    let first_nl = input.iter().position(|c| c == '\n');
    match first_nl {
        Some(idx) => input.remove_range(..idx),
        None => input.remove_range(..),
    };
    input
}

//! Transformations we make to input text before tokenisation.
//!
//! See "Processing that happens before tokenising" for the behaviour we're
//! imitating.

use crate::char_sequences::Charseq;
use crate::fine_tokens::{FineToken, FineTokenData};
use crate::lex_via_peg::first_nonwhitespace_token;
use crate::Edition;

/// Apply the transformations we make to input text before tokenisation.
#[allow(clippy::let_and_return)]
pub fn clean(input: &Charseq, edition: Edition) -> Charseq {
    let cleaned = input.chars();
    let cleaned = remove_bom(cleaned);
    let cleaned = replace_crlf(cleaned);
    let cleaned = clean_shebang(cleaned, edition);
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
    let first_nl = input.iter().position(|c| *c == '\n');
    match first_nl {
        Some(idx) => input.remove_range(..idx),
        None => input.remove_range(..),
    };
    input
}

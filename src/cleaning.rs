//! Transformations we make to input text before tokenisation.
//!
//! See <https://doc.rust-lang.org/nightly/reference/input-format.html> for the behavour we're
//! imitating.

use crate::char_sequences::Charseq;
use crate::fine_tokens::{FineToken, FineTokenData};
use crate::lex_via_peg::first_nonwhitespace_token;

/// Apply the transformations we make to input text before tokenisation.
#[allow(clippy::let_and_return)]
pub fn clean(input: &Charseq) -> Charseq {
    let cleaned = input.chars();
    let cleaned = remove_bom(cleaned);
    let cleaned = replace_crlf(cleaned);
    let cleaned = clean_shebang(cleaned);
    cleaned
}

/// Skips the first character if it's a byte order mark.
fn remove_bom(input: &[char]) -> &[char] {
    if input.starts_with(&['\u{feff}']) {
        &input[1..]
    } else {
        input
    }
}

/// Replaces each sequence of CRLF in the input with a single LF.
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

/// Removes the first line of the input if it appears to be a shebang.
///
/// We don't modify the input if it might start with a Rust attribute. That isn't trivial to
/// check, because there can be whitespace and (non-doc) comments after the `!`.
/// rustc deals with this by running its lexer for long enough to answer this question and throwing
/// away the result, so we do the same.
fn clean_shebang(mut input: Charseq) -> Charseq {
    if !input.chars().starts_with(&['#', '!']) {
        return input;
    };
    if let Some(FineToken {
        data: FineTokenData::Punctuation { mark: '[' },
        ..
    }) = first_nonwhitespace_token(&input[2..])
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

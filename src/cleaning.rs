//! Transformations we make to input text before tokenisation.
//!
//! See <https://doc.rust-lang.org/nightly/reference/input-format.html> for the behavour we're
//! imitating.

use regex::{Regex, RegexBuilder};

use crate::char_sequences::Charseq;

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

fn mkre(s: &str) -> Regex {
    RegexBuilder::new(s)
        .ignore_whitespace(true)
        .build()
        .unwrap()
}

macro_rules! make_regex {
    ($re:literal $(,)?) => {{
        static RE: ::std::sync::OnceLock<regex::Regex> = ::std::sync::OnceLock::new();
        RE.get_or_init(|| mkre($re))
    }};
}

/// Approximation to rustc's shebang-cleaning.
///
/// We're not supposed to remove the first line if it looks like the start of a Rust attribute.
///
/// The implementation below for that exception only accepts whitespace after the `#!`, so
/// it goes wrong if there's a comment there.
/// rustc deals with this by running its lexer for long enough to answer this question and throwing
/// away the result. I suppose we could do something similar.
fn clean_shebang(input: Charseq) -> Charseq {
    let mut input = input.to_string();

    #[rustfmt::skip]
    let attributelike_re = make_regex!(r##"\A
        \# !
        [ \p{Pattern_White_Space} ] *
        \[
    "##);
    if !attributelike_re.is_match(&input) {
        #[rustfmt::skip]
        let shebang_re = make_regex!(r##"\A
            \# !
            .*?
            ( \n | \z )
        "##);
        if let Some(m) = shebang_re.find(&input) {
            input.replace_range(..m.end(), "");
        }
    }
    input.into()
}

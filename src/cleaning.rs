//! Transformations we make to input text before tokenisation.
//!
//! See <https://doc.rust-lang.org/nightly/reference/input-format.html> for the behavour we're
//! imitating.

use regex::{Regex, RegexBuilder};

/// Apply the transformations we make to input text before tokenisation.
pub fn clean(input: &str) -> String {
    let mut rest = input;

    // Remove BOM
    if rest.starts_with('\u{feff}') {
        rest = &rest[3..];
    }

    // CRLF -> LF
    let mut cleaned = rest.replace("\r\n", "\n");

    // Remove shebang
    clean_shebang(&mut cleaned);

    cleaned
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
fn clean_shebang(input: &mut String) {
    #[rustfmt::skip]
    let attributelike_re = make_regex!(r##"\A
        \# !
        [ \p{Pattern_White_Space} ] *
        \[
    "##);
    if !attributelike_re.is_match(input) {
        #[rustfmt::skip]
        let shebang_re = make_regex!(r##"\A
            \# !
            .*?
            ( \n | \z )
        "##);
        if let Some(m) = shebang_re.find(input) {
            input.replace_range(..m.end(), "");
        }
    }
}

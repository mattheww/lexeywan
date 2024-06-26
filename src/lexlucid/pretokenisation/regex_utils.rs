use std::iter::once;

use regex::{Captures, Regex, RegexBuilder};

/// Makes a `Regex` with the options used by the pretokeniser.
pub fn pretokeniser_regex(s: &str) -> Regex {
    RegexBuilder::new(s)
        .ignore_whitespace(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
}

/// Matches a regular expression against a string, requiring a constraint to be satisfied.
///
/// Finds the shortest maximal match (see writeup) of `re` in the haystack which satisfies the
/// constraint function.
///
/// `re` must be anchored at both ends (ie, begin with `\A` and end with `\z`).
/// The constraint function is given the captures from a successful match of `re`. It must return
/// true iff the constraint is satisfied.
pub fn constrained_captures<'hs>(
    re: &Regex,
    constraint: fn(&Captures) -> bool,
    haystack: &'hs str,
) -> Option<Captures<'hs>> {
    let prefixes = haystack
        .char_indices()
        .map(|(idx, _)| &haystack[..idx])
        .chain(once(haystack));
    let mut longest_found = None;
    for candidate in prefixes {
        match re.captures(candidate) {
            Some(captures) if constraint(&captures) => {
                longest_found = Some(captures);
            }
            _ if longest_found.is_some() => break,
            _ => {}
        }
    }
    longest_found
}

#[cfg(test)]
mod tests;

//! Implementations of the block-comment and raw-string-literal rules in imperative code.
//!
//! These aren't used. They could be used to cross check the constrained-pattern-based rules.

use regex::Regex;

use super::regex_utils::match_chars;
use super::{
    make_regex, PretokenData,
    RuleOutcome::{self, *},
};

/// Explicit rule for block comments.
#[allow(unused)]
pub fn match_block_comment(input: &[char]) -> RuleOutcome {
    if !input.starts_with(&['/', '*']) {
        return Failure;
    }
    let mut token_length = 2;
    let mut depth = 1_i32;
    let mut after_slash = false;
    let mut after_star = false;
    for c in &input[2..] {
        token_length += 1;
        match c {
            '*' if after_slash => {
                depth += 1;
                after_slash = false;
            }
            '/' if after_star => {
                depth -= 1;
                if depth == 0 {
                    return Success(
                        token_length,
                        PretokenData::BlockComment {
                            comment_content: (&input[2..token_length - 2]).into(),
                        },
                    );
                }
                after_star = false;
            }
            _ => {
                after_slash = *c == '/';
                after_star = *c == '*';
            }
        }
    }
    Failure
}

/// Explicit rule for double-quoted literals with prefix 'r' or 'br'
#[allow(unused)]
pub fn match_raw_string_literal_for_edition_2015(input: &[char]) -> RuleOutcome {
    #[rustfmt::skip]
    let raw_prefix_re = make_regex!(r##"\A
        ( r | br )
    "##);
    match_raw_string_literal(input, raw_prefix_re)
}

/// Explicit rule for double-quoted literals with prefix 'r', 'br', or 'cr'
#[allow(unused)]
pub fn match_raw_string_literal_for_edition_2021(input: &[char]) -> RuleOutcome {
    #[rustfmt::skip]
    let raw_prefix_re = make_regex!(r##"\A
        ( r | br | cr )
    "##);
    match_raw_string_literal(input, raw_prefix_re)
}

/// Explicit rule for double-quoted literals using "raw" quoting rules
fn match_raw_string_literal(input: &[char], raw_prefix_re: &Regex) -> RuleOutcome {
    let Some(prefix_length) = match_chars(raw_prefix_re, input) else {
        return Failure;
    };

    let mut hashes_in_prefix = 0;
    'counted: {
        for c in &input[prefix_length..] {
            if *c == '"' {
                break 'counted;
            }
            if *c != '#' {
                return Failure;
            }
            hashes_in_prefix += 1;
            if hashes_in_prefix > 255 {
                return Failure;
                // return ForceError("raw string with too many hashes".into());
            }
        }
        return Failure;
    };
    let content_start = prefix_length + hashes_in_prefix + 1;

    let mut suffix_start = content_start;
    'terminated: {
        enum TerminationState {
            NotTerminating,
            HashesRequired(usize),
        }
        use TerminationState::*;
        let mut termination_state = NotTerminating;
        for c in &input[content_start..] {
            suffix_start += 1;
            if *c == '"' {
                if hashes_in_prefix == 0 {
                    break 'terminated;
                }
                termination_state = HashesRequired(hashes_in_prefix);
            } else if let HashesRequired(required) = termination_state {
                if *c == '#' {
                    if required == 1 {
                        break 'terminated;
                    }
                    termination_state = HashesRequired(required - 1);
                } else {
                    termination_state = NotTerminating;
                }
            }
        }
        return Failure;
        // return ForceError("unterminated raw string".into());
    };
    let content_end = suffix_start - hashes_in_prefix - 1;

    #[rustfmt::skip]
    let suffix_re = make_regex!(r##"\A
        # <identifier>
        [ \p{XID_Start} _ ]
        \p{XID_Continue} *
    "##);
    let suffix_length = match_chars(suffix_re, &input[suffix_start..]).unwrap_or(0);
    let token_length = suffix_start + suffix_length;

    Success(
        token_length,
        PretokenData::RawDoubleQuoteLiteral {
            prefix: input[..prefix_length].into(),
            literal_content: input[content_start..content_end].into(),
            suffix: input[suffix_start..token_length].into(),
        },
    )
}

//! Reimplementation of rustc's lexical analysis.

use crate::Edition;
use crate::char_sequences::Charseq;
use crate::fine_tokens::FineToken;
use crate::utils::escape_for_display;

mod processing;
mod token_matching;

pub use token_matching::MatchData;
use token_matching::TokensMatchData;

const MAX_INPUT_LENGTH: usize = 0x100_0000;

/// Runs lexical analysis on the specified input.
///
/// If the input is accepted, returns a list of fine-grained tokens.
///
/// If the input is rejected, returns an error message and whatever lists of matches or tokens are
/// available.
///
/// (Strictly to follow the writeup we needn't bother with the processing step if we didn't match
///  the complete input, but it's helpful when troubleshooting to be able to see the additional
///  information.)
///
/// May instead report a problem with lex_via_peg's model or implementation.
///
/// Panics if the input is longer than 2^24 characters (this is a sanity check, not part of the model).
pub fn analyse(input: &Charseq, edition: Edition) -> Analysis {
    assert_eq!(
        unicode_normalization::UNICODE_VERSION,
        (16, 0, 0),
        "Unicode version for unicode-normalization"
    );

    if input.len() > MAX_INPUT_LENGTH {
        panic!("input too long");
    }

    let TokensMatchData {
        token_kind_matches,
        consumed_entire_input: matched_entire_input,
    } = match token_matching::match_tokens(edition, input.chars()) {
        Ok(tokens_match_data) => tokens_match_data,
        Err(message) => {
            return Analysis::ModelError(Reason::Matching(message, Vec::new(), Vec::new()));
        }
    };

    // Note that if there's a processing error we only report the token-kind matches up to the match
    // that failed processing.
    let mut tokens = Vec::new();
    let mut reported_matches = Vec::new();
    for match_data in token_kind_matches {
        match processing::process(&match_data) {
            Ok(token) => {
                reported_matches.push(match_data);
                tokens.push(token);
            }
            Err(processing::Error::Rejected(error_message)) => {
                return Analysis::Rejects(Reason::Processing(
                    error_message,
                    match_data,
                    reported_matches,
                    tokens,
                ));
            }
            Err(processing::Error::ModelError(error_message)) => {
                return Analysis::ModelError(Reason::Processing(
                    error_message,
                    match_data,
                    reported_matches,
                    tokens,
                ));
            }
        }
    }

    if matched_entire_input {
        Analysis::Accepts(reported_matches, tokens)
    } else {
        Analysis::Rejects(Reason::Matching(
            "The tokens nonterminal did not match the complete input".to_owned(),
            reported_matches,
            tokens,
        ))
    }
}

/// Result of running lexical analysis on a string.
pub enum Analysis {
    /// Lexical analysis accepted the input.
    Accepts(Vec<MatchData>, Vec<FineToken>),

    /// Lexical analysis rejected the input.
    Rejects(Reason),

    /// The input demonstrated a problem in lex_via_peg's model or implementation.
    ModelError(Reason),
}

/// Explanation of why and where input was rejected.
pub enum Reason {
    /// Rejected when trying to match the edition's token nonterminal.
    ///
    /// The string describes the reason for rejection (or a model error).
    ///
    /// The lists of matches and tokens represent what was lexed successfully before the token
    /// nonterminal ceased to match.
    Matching(String, Vec<MatchData>, Vec<FineToken>),

    /// Rejected when processing a match of a token-kind nonterminal.
    ///
    /// The string describes the reason for rejection (or a model error).
    ///
    /// The single MatchData describes the match which was rejected (or which was being processed
    /// when we encountered a problem with the model).
    ///
    /// The lists of matches and tokens represent what was lexed successfully first.
    Processing(String, MatchData, Vec<MatchData>, Vec<FineToken>),
}

impl Reason {
    /// Describes a rejection or problem as a list of strings (one per line).
    pub fn into_description(self) -> Vec<String> {
        let mut description = Vec::new();
        match self {
            Reason::Matching(message, matches, _) => {
                if matches.is_empty() {
                    description.push("matching failed at the start of the input".into());
                } else {
                    let s: String = matches.iter().flat_map(|p| p.consumed.chars()).collect();
                    description.push(format!(
                        "matching failed after «{}»",
                        escape_for_display(&s)
                    ));
                }
                description.push(message);
            }
            Reason::Processing(message, rejected_match, _, _) => {
                description.push(format!("processing rejected match of {rejected_match:?}"));
                description.push(message);
                description.extend(
                    rejected_match
                        .describe_submatches()
                        .map(|s| format!("  {s}")),
                );
            }
        };
        description
    }
}

/// Runs lexical analysis, expecting to find a single token.
///
/// If the complete input is accepted as a single token, returns that (fine-grained) token.
///
/// Otherwise returns None.
pub fn lex_as_single_token(input: &[char], edition: Edition) -> Option<FineToken> {
    let Ok(TokensMatchData {
        token_kind_matches,
        consumed_entire_input: true,
    }) = token_matching::match_tokens(edition, input)
    else {
        return None;
    };
    let [match_data] = &token_kind_matches[..] else {
        return None;
    };
    processing::process(match_data).ok()
}

/// Returns the first non-whitespace token in the input.
///
/// Returns None if there are no tokens in the input, or if it reaches a point where lexical
/// analysis would reject the input.
///
/// For this purpose, comment tokens with style `NonDoc` count as whitespace.
///
/// Panics if the input is longer than 2^24 characters (this is a sanity check, not part of the model).
pub fn first_nonwhitespace_token(input: &[char], edition: Edition) -> Option<FineToken> {
    if input.len() > MAX_INPUT_LENGTH {
        panic!("input too long");
    }

    use crate::fine_tokens::{CommentStyle, FineTokenData::*};

    let token_kind_matches = match token_matching::match_tokens(edition, input) {
        Ok(TokensMatchData {
            token_kind_matches, ..
        }) => token_kind_matches,
        Err(_) => return None,
    };

    for match_data in token_kind_matches {
        let Ok(token) = processing::process(&match_data) else {
            return None;
        };
        match token.data {
            Whitespace => {}
            LineComment {
                style: CommentStyle::NonDoc,
                ..
            } => {}
            BlockComment {
                style: CommentStyle::NonDoc,
                ..
            } => {}
            _ => return Some(token),
        }
    }
    None
}

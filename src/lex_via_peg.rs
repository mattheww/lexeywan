//! Reimplementation of rustc's lexical analysis.

use crate::char_sequences::Charseq;
use crate::fine_tokens::FineToken;
use crate::utils::escape_for_display;
use crate::Edition;

mod processing;
mod token_matching;

pub use token_matching::MatchData;
use token_matching::Outcome;

const MAX_INPUT_LENGTH: usize = 0x100_0000;

/// Runs lexical analysis on the specified input.
///
/// If the input is accepted, returns a list of fine-grained tokens.
///
/// If the input is rejected, returns an error message and whatever lists of matches or tokens are
/// available.
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

    let mut matches = Vec::new();
    let mut tokens = Vec::new();
    for outcome in tokenise(input.chars(), edition) {
        use Outcome::*;
        let match_data = match outcome {
            Matched(match_data) => match_data,
            NoMatch => {
                return Analysis::Rejects(Reason::Matching(
                    "The edition nonterminal did not match".to_owned(),
                    matches,
                    tokens,
                ))
            }
            ModelError(message) => {
                return Analysis::ModelError(Reason::Matching(message, matches, tokens))
            }
        };
        match processing::process(&match_data) {
            Ok(token) => {
                matches.push(match_data);
                tokens.push(token)
            }
            Err(processing::Error::Rejected(error_message)) => {
                return Analysis::Rejects(Reason::Processing(
                    error_message,
                    match_data,
                    matches,
                    tokens,
                ));
            }
            Err(processing::Error::ModelError(error_message)) => {
                return Analysis::ModelError(Reason::Processing(
                    error_message,
                    match_data,
                    matches,
                    tokens,
                ));
            }
        }
    }

    Analysis::Accepts(matches, tokens)
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
    /// Rejected when trying to match the edition nonterminal.
    ///
    /// The string describes the reason for rejection (or a model error).
    ///
    /// The lists of matches and tokens represent what was lexed successfully first.
    Matching(String, Vec<MatchData>, Vec<FineToken>),

    /// Rejected when processing a match of the edition nonterminal.
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
                    let s: String = matches.iter().flat_map(|p| p.extent.chars()).collect();
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
/// If the complete input is accepted as a single token, retuns that (fine-grained) token.
///
/// Otherwise returns None.
pub fn lex_as_single_token(input: &[char], edition: Edition) -> Option<FineToken> {
    let mut iter = tokenise(input, edition);
    let Some(Outcome::Matched(match_data)) = iter.next() else {
        return None;
    };
    let None = iter.next() else {
        return None;
    };
    processing::process(&match_data).ok()
}

/// Repeatedly matches the appropriate edition nonterminal against the specified input.
///
/// Returns an iterator which yields [`Outcome`]s.
///
/// The outcome usually provides a [`MatchData`] or indicates that the input is unacceptable to the
/// lexer.
///
/// It may instead report a problem with lex_via_peg's model or implementation.
fn tokenise(input: &[char], edition: Edition) -> impl Iterator<Item = Outcome> + use<'_> {
    Tokeniser {
        edition,
        input,
        index: 0,
    }
}

struct Tokeniser<'a> {
    edition: Edition,
    input: &'a [char],
    index: usize,
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Outcome;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = &self.input[self.index..];
        if rest.is_empty() {
            return None;
        }
        let outcome = token_matching::match_once(self.edition, rest);
        if let Outcome::Matched(match_data) = &outcome {
            self.index += match_data.extent.len();
        }
        Some(outcome)
    }
}

/// Returns the first non-whitespace token in the input.
///
/// Returns None if there are no tokens in the input, or if lexical analysis wouldn't accept the
/// input.
///
/// For this purpose, comment tokens with style `NonDoc` count as whitespace.
pub fn first_nonwhitespace_token(input: &[char], edition: Edition) -> Option<FineToken> {
    use crate::fine_tokens::{CommentStyle, FineTokenData::*};
    for outcome in tokenise(input, edition) {
        let Outcome::Matched(match_data) = outcome else {
            return None;
        };
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

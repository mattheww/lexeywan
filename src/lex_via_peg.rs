//! Reimplementation of rustc's lexical analysis.

use crate::fine_tokens::FineToken;
use crate::utils::escape_for_display;
use crate::Edition;

mod pretokenisation;
mod reprocessing;

pub use pretokenisation::Pretoken;

const MAX_INPUT_LENGTH: usize = 0x100_0000;

/// Runs lexical analysis on the specified input.
///
/// If the input is accepted, returns lists of both pretokens and fine-grained tokens.
///
/// If the input is rejected, returns an error message and whatever lists of tokens are available.
///
/// May instead report a problem with lex_via_peg's model or implementation.
///
/// Panics if the input is longer than 2^24 bytes (this is a sanity check, not part of the model).
pub fn analyse(input: &str, edition: Edition) -> Analysis {
    assert_eq!(
        unicode_normalization::UNICODE_VERSION,
        (16, 0, 0),
        "Unicode version for unicode-normalization"
    );

    if input.len() > MAX_INPUT_LENGTH {
        panic!("input too long");
    }

    let mut pretokens = Vec::new();
    let mut tokens = Vec::new();
    for outcome in pretokenisation::pretokenise(input.into(), edition) {
        use pretokenisation::Outcome::*;
        let pretoken = match outcome {
            Found(pretoken) => pretoken,
            Rejected(error_message) => {
                return Analysis::Rejects(Reason::Pretokenisation(
                    vec![error_message],
                    pretokens,
                    tokens,
                ))
            }
            ModelError(messages) => {
                return Analysis::ModelError(Reason::Pretokenisation(messages, pretokens, tokens))
            }
        };
        match reprocessing::reprocess(&pretoken) {
            Ok(token) => {
                pretokens.push(pretoken);
                tokens.push(token)
            }
            Err(reprocessing::Error::Rejected(error_message)) => {
                return Analysis::Rejects(Reason::Reprocessing(
                    error_message,
                    pretoken,
                    pretokens,
                    tokens,
                ));
            }
            Err(reprocessing::Error::ModelError(error_message)) => {
                return Analysis::ModelError(Reason::Reprocessing(
                    error_message,
                    pretoken,
                    pretokens,
                    tokens,
                ));
            }
        }
    }

    Analysis::Accepts(pretokens, tokens)
}

/// Result of running lexical analysis on a string.
pub enum Analysis {
    /// Lexical analysis accepted the input.
    Accepts(Vec<Pretoken>, Vec<FineToken>),

    /// Lexical analysis rejected the input.
    Rejects(Reason),

    /// The input demonstrated a problem in lex_via_peg's model or implementation.
    ModelError(Reason),
}

/// Explanation of why and where input was rejected.
pub enum Reason {
    /// Rejected during step 1 (pretokenisation).
    ///
    /// The strings describe the reason for rejection (or a model error), one string per line.
    ///
    /// The token lists represent what was lexed successfully first.
    #[allow(unused)]
    Pretokenisation(Vec<String>, Vec<Pretoken>, Vec<FineToken>),

    /// Rejected during step 2 (reprocessing).
    ///
    /// The string describes the reason for rejection (or a model error).
    ///
    /// The first pretoken is the one which reprocessing rejected (or was handling when it
    /// encountered a problem with the model).
    ///
    /// The token lists represent what was lexed successfully first.
    Reprocessing(String, Pretoken, Vec<Pretoken>, Vec<FineToken>),
}

impl Reason {
    /// Describes a rejection or problem as a list of strings (one per line).
    pub fn into_description(self) -> Vec<String> {
        let mut description = Vec::new();
        match self {
            Reason::Pretokenisation(messages, pretokens, _) => {
                description.extend(messages);
                if pretokens.is_empty() {
                    description.push("pretokenisation failed at the start of the input".into());
                } else {
                    let s: String = pretokens.iter().flat_map(|p| p.extent.chars()).collect();
                    description.push(format!(
                        "pretokenisation failed after «{}»",
                        escape_for_display(&s)
                    ));
                }
            }
            Reason::Reprocessing(message, rejected, _, _) => {
                description.push(message);
                description.push(format!("reprocessing rejected {:?}", rejected))
            }
        };
        description
    }
}

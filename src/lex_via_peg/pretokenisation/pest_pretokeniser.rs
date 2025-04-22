//! Pest-specific code for the PEG-based pretokeniser.
//!
//! See pretokenise.pest in this directory for the grammar this is using.
//!
//! This module is responsible for extracting pretoken attributes from the parser's output.

use pest::{iterators::Pair, Parser};

use crate::Edition;
use crate::{char_sequences::Charseq, lex_via_peg::pretokenisation::PretokenData};

use super::{NumericBase, Pretoken};

/// Attempts to match a single pretoken at the start of the input.
pub fn lex_one_pretoken(edition: Edition, input: &[char]) -> LexOutcome {
    use LexOutcome::*;
    let s: String = input.iter().collect();
    let pretoken_rule = pretoken_rule_for_edition(edition);
    let Ok(mut pretoken_pairs) = PretokenParser::parse(pretoken_rule, &s) else {
        return Failed;
    };
    let Some(pretoken) = pretoken_pairs.next() else {
        return ModelError("Pest reported empty response".to_owned());
    };
    let None = pretoken_pairs.next() else {
        return ModelError("Pest reported multiple pretokens".to_owned());
    };
    let mut subs = pretoken.into_inner();
    let Some(pair) = subs.next() else {
        return ModelError("Pest reported empty pretoken".to_owned());
    };
    let None = subs.next() else {
        return ModelError("Pest reported multiple sub-matches for the pretoken rule".to_owned());
    };
    let extent = pair.as_str().into();
    let rule = pair.as_rule();
    match interpret_pest_pair(pair) {
        Ok(pretoken_data) => Lexed(Pretoken {
            data: pretoken_data,
            extent,
        }),
        Err(msg) => ModelError(format!("{rule:?}: {msg}")),
    }
}

/// Returns the PRETOKEN rule to use for the specified Rust edition.
fn pretoken_rule_for_edition(edition: Edition) -> Rule {
    match edition {
        Edition::E2015 => Rule::PRETOKEN_2015,
        Edition::E2021 => Rule::PRETOKEN_2021,
        Edition::E2024 => Rule::PRETOKEN_2024,
    }
}

/// Result of a single attempt to match a pretoken.
pub enum LexOutcome {
    /// The edition's PRETOKEN nonterminal matched a prefix of the input
    Lexed(Pretoken),

    /// The edition's PRETOKEN nonterminal didn't match at the start of the input
    Failed,

    /// The input demonstrated a problem in lex_via_peg's model or implementation.
    ///
    /// The string is a description of the problem.
    ModelError(String),
}

#[derive(pest_derive::Parser)]
#[grammar = "lex_via_peg/pretokenisation/pretokenise.pest"]
pub struct PretokenParser;

/// Converts the output from a successful Pest parse to data for a pretoken.
///
/// `pair` should be the result of parsing against one of the edition PRETOKEN rules.
///
/// An Err return value indicates a problem in the model (for example, that the rule for assigning
/// properties isn't well-defined) or the implementation (for example, that the assumptions made in
/// this function don't match the current grammar).
fn interpret_pest_pair(pair: Pair<Rule>) -> Result<PretokenData, &'static str> {
    match pair.as_rule() {
        Rule::Whitespace => Ok(PretokenData::Whitespace),
        Rule::Line_comment => {
            let mut content = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::LINE_COMMENT_CONTENT => content = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::LineComment {
                comment_content: extracted(content, "no content")?,
            })
        }
        Rule::Block_comment => {
            let mut content = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::BLOCK_COMMENT_CONTENT => content = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::BlockComment {
                comment_content: extracted(content, "no content")?,
            })
        }
        Rule::Unterminated_block_comment => Ok(PretokenData::Reserved),
        Rule::Single_quoted_literal => {
            let mut prefix = None;
            let mut literal_content = None;
            let mut suffix = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::SQ_PREFIX => {
                        prefix = Some(sub.as_str());
                    }
                    Rule::SQ_CONTENT => {
                        literal_content = Some(sub.as_str());
                    }
                    Rule::SUFFIX => suffix = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::SingleQuotedLiteral {
                prefix: extracted(prefix, "missing prefix")?,
                literal_content: extracted(literal_content, "missing content")?,
                suffix: suffix.map(Into::into),
            })
        }
        Rule::Double_quoted_literal_2015 | Rule::Double_quoted_literal_2021 => {
            let mut prefix = None;
            let mut literal_content = None;
            let mut suffix = None;
            for sub in pair.into_inner().flatten() {
                match sub.as_rule() {
                    Rule::DQ_PREFIX_2015 | Rule::DQ_PREFIX_2021 => {
                        prefix = Some(sub.as_str());
                    }
                    Rule::DQ_CONTENT => {
                        literal_content = Some(sub.as_str());
                    }
                    Rule::SUFFIX => suffix = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::DoubleQuotedLiteral {
                prefix: extracted(prefix, "missing prefix")?,
                literal_content: extracted(literal_content, "missing content")?,
                suffix: suffix.map(Into::into),
            })
        }
        Rule::Raw_double_quoted_literal_2015 | Rule::Raw_double_quoted_literal_2021 => {
            let mut prefix = None;
            let mut literal_content = None;
            let mut suffix = None;
            for sub in pair.into_inner().flatten() {
                match sub.as_rule() {
                    Rule::RAW_DQ_PREFIX_2015 | Rule::RAW_DQ_PREFIX_2021 => {
                        prefix = Some(sub.as_str());
                    }
                    Rule::RAW_DQ_CONTENT => {
                        literal_content = Some(sub.as_str());
                    }
                    Rule::SUFFIX => suffix = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::RawDoubleQuotedLiteral {
                prefix: extracted(prefix, "missing prefix")?,
                literal_content: extracted(literal_content, "missing content")?,
                suffix: suffix.map(Into::into),
            })
        }
        Rule::Unterminated_literal_2015 | Rule::Reserved_literal_2021 => Ok(PretokenData::Reserved),
        Rule::Reserved_guard_2024 => Ok(PretokenData::Reserved),
        Rule::Float_literal_1 | Rule::Float_literal_2 => {
            let mut body = None;
            let mut suffix = None;
            for sub in pair.into_inner().flatten() {
                match sub.as_rule() {
                    Rule::FLOAT_BODY_WITH_EXPONENT
                    | Rule::FLOAT_BODY_WITHOUT_EXPONENT
                    | Rule::FLOAT_BODY_WITH_FINAL_DOT => {
                        body = Some(sub.as_str());
                    }
                    Rule::SUFFIX => suffix = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::FloatLiteral {
                body: extracted(body, "missing body")?,
                suffix: suffix.map(Into::into),
            })
        }
        Rule::Reserved_float_empty_exponent | Rule::Reserved_float_based => {
            Ok(PretokenData::Reserved)
        }
        Rule::Integer_literal => {
            let mut base = None;
            let mut digits = None;
            let mut suffix = None;
            for sub in pair.into_inner().flatten() {
                match sub.as_rule() {
                    Rule::INTEGER_BINARY_LITERAL => {
                        base = Some(NumericBase::Binary);
                    }
                    Rule::INTEGER_OCTAL_LITERAL => {
                        base = Some(NumericBase::Octal);
                    }
                    Rule::INTEGER_HEXADECIMAL_LITERAL => {
                        base = Some(NumericBase::Hexadecimal);
                    }
                    Rule::INTEGER_DECIMAL_LITERAL => {
                        base = Some(NumericBase::Decimal);
                    }
                    Rule::LOW_BASE_PRETOKEN_DIGITS
                    | Rule::HEXADECIMAL_DIGITS
                    | Rule::DECIMAL_PART => {
                        digits = Some(sub.as_str());
                    }
                    Rule::SUFFIX => suffix = Some(sub.as_str()),
                    _ => {}
                }
            }
            Ok(PretokenData::IntegerLiteral {
                base: base.ok_or("missing base")?,
                digits: extracted(digits, "missing digits")?,
                suffix: suffix.map(Into::into),
            })
        }
        Rule::Raw_lifetime_or_label_2021 => {
            let mut name = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::IDENT => {
                        name = Some(sub.as_str());
                    }
                    _ => {}
                }
            }
            Ok(PretokenData::RawLifetimeOrLabel {
                name: extracted(name, "missing name")?,
            })
        }
        Rule::Reserved_lifetime_or_label_prefix_2021 => Ok(PretokenData::Reserved),
        Rule::Lifetime_or_label => {
            let mut name = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::IDENT => {
                        name = Some(sub.as_str());
                    }
                    _ => {}
                }
            }
            Ok(PretokenData::LifetimeOrLabel {
                name: extracted(name, "missing name")?,
            })
        }
        Rule::Raw_identifier => {
            let mut identifier = None;
            for sub in pair.into_inner() {
                match sub.as_rule() {
                    Rule::IDENT => {
                        identifier = Some(sub.as_str());
                    }
                    _ => {}
                }
            }
            Ok(PretokenData::RawIdentifier {
                identifier: extracted(identifier, "missing identifier")?,
            })
        }
        Rule::Reserved_prefix_2015 | Rule::Reserved_prefix_2021 => Ok(PretokenData::Reserved),
        Rule::Identifier => Ok(PretokenData::Identifier {
            identifier: pair.as_str().into(),
        }),
        Rule::Punctuation => {
            let Some(c) = pair.as_str().chars().next() else {
                return Err("no character");
            };
            Ok(PretokenData::Punctuation { mark: c })
        }
        _ => Err("unhandled pretoken rule"),
    }
}

fn extracted(matched: Option<&str>, error_msg: &'static str) -> Result<Charseq, &'static str> {
    matched.ok_or(error_msg).map(Into::into)
}

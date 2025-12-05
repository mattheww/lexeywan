//! Implementation of the writeup's "Escape processing" page.

use crate::datatypes::char_sequences::Charseq;
use crate::reimplementation::pegs::{self, MatchData, Outcome, WrittenUp, attempt_pest_match};

/// Error from the `escape_processing` module.
///
/// In this module all errors indicate some form of model error (as opposed to rejection).
pub enum Error {
    /// A function in this module was asked to provide information which isn't defined in the writeup.
    ///
    /// This is likely to indicate a bug in the processing model or code.
    ///
    /// The string describes the problem.
    Undefined(&'static str),

    /// A function in this module saw parsed output from the escape-processing grammar of a form it
    /// didn't expect.
    ///
    /// This is likely to indicate a bug in the escape-processing grammar or in this module.
    ///
    /// The string describes the problem.
    BadParse(String),

    /// Some other situation which shouldn't happen happened.
    ///
    /// This indicates a bug in this module.
    ///
    /// The string describes the problem.
    Internal(String),
}

/// Classify a LITERAL_COMPONENT match as described under "Classifying escapes" in the writeup.
///
/// Performs enough interpretation of the escape body or consumed character to report any
/// appropriate BadParse errors.
fn classify_escape(m: &EscapingMatch) -> Result<LiteralComponent, Error> {
    let impossible = |s: &str| Err(Error::BadParse(format!("impossible {s}: {m:?}")));
    if m.matched_nonterminal != Nonterminal::LITERAL_COMPONENT {
        return Err(Error::Internal(format!(
            "classify_escape called on {:?}",
            m.matched_nonterminal
        )));
    }

    enum Classification {
        NonEscape,
        SimpleEscape,
        UnicodeEscape,
        HexadecimalEscape,
        StringContinuationEscape,
    }

    let classification = match (
        m.participated(Nonterminal::ESCAPE_BODY),
        m.participated(Nonterminal::SIMPLE_ESCAPE_BODY),
        m.participated(Nonterminal::UNICODE_ESCAPE_BODY),
        m.participated(Nonterminal::HEXADECIMAL_ESCAPE_BODY),
        m.participated(Nonterminal::STRING_CONTINUATION_ESCAPE_BODY),
    ) {
        (false, false, false, false, false) => Classification::NonEscape,
        (true, true, false, false, false) => Classification::SimpleEscape,
        (true, false, true, false, false) => Classification::UnicodeEscape,
        (true, false, false, true, false) => Classification::HexadecimalEscape,
        (true, false, false, false, true) => Classification::StringContinuationEscape,
        // "It follows from the definitions of LITERAL_COMPONENT AND ESCAPE_BODY that each match of
        //  LITERAL_COMPONENT is exactly one of the above forms."
        _ => return impossible("match"),
    };

    match classification {
        Classification::NonEscape => match m.consumed.chars() {
            [c] => Ok(LiteralComponent::NonEscape {
                represented_character: *c,
            }),
            _ => impossible("non-escape"),
        },
        Classification::SimpleEscape => {
            match interpret_simple_escape_body(m.consumed(Nonterminal::SIMPLE_ESCAPE_BODY)?) {
                Ok(represented_character) => Ok(LiteralComponent::SimpleEscape {
                    represented_character,
                }),
                Err(_) => impossible("simple escape"),
            }
        }
        Classification::UnicodeEscape => {
            match interpret_unicode_escape_digits(
                &m.consumed_by_all_participating_matches(Nonterminal::HEXADECIMAL_DIGIT),
            ) {
                Ok(numeric_value) => Ok(LiteralComponent::UnicodeEscape { numeric_value }),
                Err(reason) => impossible(reason),
            }
        }
        Classification::HexadecimalEscape => {
            match interpret_hexadecimal_escape_digits(
                &m.consumed_by_all_participating_matches(Nonterminal::HEXADECIMAL_DIGIT),
            ) {
                Ok(represented_byte) => {
                    Ok(LiteralComponent::HexadecimalEscape { represented_byte })
                }
                Err(reason) => impossible(reason),
            }
        }
        Classification::StringContinuationEscape => Ok(LiteralComponent::StringContinuationEscape),
    }
}

/// Classification of a LITERAL_COMPONENT match, in each case with the most fundamental attribute
/// defined under "Classifying escapes" in the writeup.
#[derive(std::fmt::Debug)]
pub enum LiteralComponent {
    NonEscape { represented_character: char },
    SimpleEscape { represented_character: char },
    UnicodeEscape { numeric_value: u32 },
    HexadecimalEscape { represented_byte: u8 },
    StringContinuationEscape,
}

impl LiteralComponent {
    /// Returns the match's _represented character_ as defined under "Classifying escapes" in the writeup.
    ///
    /// Returns None if the match has no represented character.
    pub fn represented_character(&self) -> Result<Option<char>, Error> {
        match *self {
            LiteralComponent::NonEscape {
                represented_character,
            } => Ok(Some(represented_character)),
            LiteralComponent::SimpleEscape {
                represented_character,
            } => Ok(Some(represented_character)),
            LiteralComponent::UnicodeEscape { numeric_value } => Ok(char::from_u32(numeric_value)),
            LiteralComponent::HexadecimalEscape { represented_byte } => {
                if represented_byte < 128 {
                    Ok(Some(represented_byte.into()))
                } else {
                    Ok(None)
                }
            }
            LiteralComponent::StringContinuationEscape => Err(Error::Undefined(
                "represented character of a string continuation escape",
            )),
        }
    }

    /// Returns the match's _represented byte_ as defined under "Classifying escapes" in the writeup.
    ///
    /// Returns None if the match has no represented byte.
    pub fn represented_byte(&self) -> Result<Option<u8>, Error> {
        match *self {
            LiteralComponent::NonEscape {
                represented_character,
            } => {
                let scalar_value = represented_character as u32;
                if scalar_value < 128 {
                    Ok(Some(scalar_value.try_into().unwrap()))
                } else {
                    Ok(None)
                }
            }
            LiteralComponent::SimpleEscape {
                represented_character,
            } => {
                let scalar_value = represented_character as u32;
                if scalar_value < 128 {
                    Ok(Some(scalar_value.try_into().unwrap()))
                } else {
                    Err(Error::Internal(
                        "simple escape has non-ascii represented character".to_owned(),
                    ))
                }
            }
            LiteralComponent::UnicodeEscape { .. } => {
                Err(Error::Undefined("represented byte of a unicode escape"))
            }
            LiteralComponent::HexadecimalEscape { represented_byte } => Ok(Some(represented_byte)),
            LiteralComponent::StringContinuationEscape => Err(Error::Undefined(
                "represented byte of a string continuation escape",
            )),
        }
    }
}

/// Processes the SIMPLE_ESCAPE_BODY from a _simple escape_, returning the represented character.
///
/// An error return indicates that the grammar accepted something we didn't expect.
fn interpret_simple_escape_body(body: &Charseq) -> Result<char, ()> {
    match body.chars() {
        ['0'] => Ok('\u{0000}'),
        ['t'] => Ok('\u{0009}'),
        ['n'] => Ok('\u{000a}'),
        ['r'] => Ok('\u{000d}'),
        ['"'] => Ok('\u{0022}'),
        ['\''] => Ok('\u{0027}'),
        ['\\'] => Ok('\u{005c}'),
        _ => Err(()),
    }
}

/// Processes the HEXADECIMAL_DIGITs from a _Unicode escape_, returning the numeric value.
///
/// An error return indicates that the grammar accepted something we didn't expect.
///
/// Doesn't check that the returned numeric value is a Unicode scalar value.
fn interpret_unicode_escape_digits(digits: &Charseq) -> Result<u32, &'static str> {
    if digits.is_empty() {
        return Err("unicode escape: empty digits");
    }
    if !&digits.iter().all(|c| c.is_ascii_hexdigit()) {
        return Err("unicode escape: bad digit");
    }
    if digits.len() > 6 {
        return Err("unicode escape: too many digits");
    }
    u32::from_str_radix(&digits.to_string(), 16)
        .map_err(|_| "unicode escape: rejected by from_str_radix")
}

/// Processes the HEXADECIMAL_DIGITs from a _hexadecimal escape_, returning the represented byte.
///
/// An error return indicates that the grammar accepted something we didn't expect.
pub fn interpret_hexadecimal_escape_digits(digits: &Charseq) -> Result<u8, &'static str> {
    if digits.len() != 2 {
        return Err("hexadecimal escape: wrong number of digits");
    }
    if !&digits.iter().all(|c| c.is_ascii_hexdigit()) {
        return Err("hexadecimal escape: bad digit");
    }
    u8::from_str_radix(&digits.to_string(), 16)
        .map_err(|_| "hexadecimal escape: rejected by from_str_radix")
}

/// Attempts to find a single-escape interpretation of a sequence of characters.
///
/// See "Single-escape interpretation" in the writeup.
///
/// When there is an interpretation, we're promising that it isn't a string continuation escape.
pub fn try_single_escape_interpretation(
    charseq: &Charseq,
) -> Result<MaybeInterpretation<LiteralComponent>, Error> {
    use MaybeInterpretation::*;
    let s: String = charseq.iter().collect();
    let literal_component_pair =
        match attempt_escape_processing_match(Nonterminal::LITERAL_COMPONENT, &s)? {
            // "If a match attempt of LITERAL_COMPONENT against a character sequence succeeds and
            // consumes the entire sequence"
            Outcome::Success {
                consumed_entire_input: true,
                pair,
            } => pair,
            Outcome::Success {
                consumed_entire_input: false,
                ..
            } => {
                return Ok(HasNoInterpretation(
                    "LITERAL_COMPONENT did not consume the entire input",
                ));
            }
            Outcome::Failure => {
                return Ok(HasNoInterpretation(
                    "LITERAL_COMPONENT match attempt failed",
                ));
            }
        };
    let m = EscapingMatch::new(literal_component_pair);
    let component = classify_escape(&m)?;
    match component {
        // "and the match is not a string continuation escape"
        LiteralComponent::StringContinuationEscape => {
            Ok(HasNoInterpretation("string continuation escape"))
        }
        _ => Ok(HasInterpretation(component)),
    }
}

/// Attempts to find an escape interpretation of a sequence of characters.
///
/// See "Escape interpretation" in the writeup.
///
/// When there is an interpretation, we're promising that no component is a string continuation
/// escape.
pub fn try_escape_interpretation(
    charseq: &Charseq,
) -> Result<MaybeInterpretation<Vec<LiteralComponent>>, Error> {
    use MaybeInterpretation::*;
    let s: String = charseq.iter().collect();
    let literal_components_pair =
        match attempt_escape_processing_match(Nonterminal::LITERAL_COMPONENTS, &s)? {
            // "If a match attempt of LITERAL_COMPONENTS against a character sequence succeeds and
            // consumes the entire sequence"
            Outcome::Success {
                consumed_entire_input: true,
                pair,
            } => pair,
            Outcome::Success {
                consumed_entire_input: false,
                ..
            } => {
                return Ok(HasNoInterpretation(
                    "LITERAL_COMPONENTS did not consume the entire input",
                ));
            }
            // This can't really fail, because LITERAL_COMPONENTS's expression is a zero-or-more
            // repetitions operator.
            Outcome::Failure => {
                return Ok(HasNoInterpretation(
                    "LITERAL_COMPONENTS match attempt failed",
                ));
            }
        };
    // "sequence of participating matches of LITERAL_COMPONENT in the resulting match"
    let mut components = Vec::new();
    for literal_component_pair in literal_components_pair.into_inner() {
        if literal_component_pair.as_rule() != Nonterminal::LITERAL_COMPONENT {
            return Err(Error::BadParse(format!(
                "matched {:?} under LITERAL_COMPONENTS",
                literal_component_pair.as_rule(),
            )));
        }
        let m = EscapingMatch::new(literal_component_pair);
        let component = classify_escape(&m)?;
        match component {
            // "omitting any string continuation escapes"
            LiteralComponent::StringContinuationEscape => {}
            _ => components.push(component),
        }
    }
    Ok(HasInterpretation(components))
}

/// Return value for [try_single_escape_interpretation] and [try_escape_interpretation].
pub enum MaybeInterpretation<T> {
    /// The character sequence has an interpretation.
    HasInterpretation(T),
    /// The character sequence doesn't have an interpretation.
    /// The string explains why not.
    HasNoInterpretation(&'static str),
}

/// Attempt to match the specified nonterminal from the escape-processing grammar against the
/// specified string.
fn attempt_escape_processing_match<'a>(
    nonterminal: Nonterminal,
    against: &'a str,
) -> Result<pegs::Outcome<'a, Nonterminal>, Error> {
    attempt_pest_match::<Nonterminal, EscapeProcessingParser>(nonterminal, against)
        .map_err(Error::Internal)
}

#[derive(pest_derive::Parser)]
#[grammar = "reimplementation/tokenisation/processing/escape_processing.pest"]
struct EscapeProcessingParser;

/// Enumeration of the nonterminals used in the escape-processing grammar.
///
/// Some members are nonterminals in the Pest grammar but documented as terminals in the writeup;
/// see [is_documented_as_terminal] below.
pub type Nonterminal = Rule;

/// Information from a successful match attempt from the escape-processing grammar
type EscapingMatch = MatchData<Nonterminal>;

impl WrittenUp for Nonterminal {
    fn is_documented_as_terminal(&self) -> bool {
        *self == Nonterminal::TAB
            || *self == Nonterminal::CR
            || *self == Nonterminal::LF
            || *self == Nonterminal::DOUBLEQUOTE
            || *self == Nonterminal::BACKSLASH
    }
}

impl EscapingMatch {
    /// Returns the characters consumed by the specified nonterminal.
    ///
    /// Reports BadParse if that nonterminal did not participate in the match, or participated in
    /// the match more than once.
    fn consumed(&self, nonterminal: Nonterminal) -> Result<&Charseq, Error> {
        match self.consumed_by_only_participating_match(nonterminal) {
            Ok(Some(charseq)) => Ok(charseq),
            Ok(None) => Err(Error::BadParse(format!(
                "{nonterminal:?} did not participate in the match"
            ))),
            Err(_) => Err(Error::BadParse(format!(
                "{nonterminal:?} participated more than once"
            ))),
        }
    }
}

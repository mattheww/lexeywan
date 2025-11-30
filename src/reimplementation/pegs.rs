//! Support for working with Pest parsing expression grammars.

use pest::iterators::Pair;
use pest::{Parser, RuleType};

use crate::datatypes::char_sequences::Charseq;

/// Attempt to match the specified nonterminal against the specified string.
///
/// A returned error indicates that Pest didn't behave the way we expect.
pub fn attempt_pest_match<'a, NONTERMINAL: RuleType, PARSER: Parser<NONTERMINAL>>(
    nonterminal: NONTERMINAL,
    against: &'a str,
) -> Result<Outcome<'a, NONTERMINAL>, String> {
    use Multiplicity::*;
    let Ok(top_level_pairs) = PARSER::parse(nonterminal, against) else {
        return Ok(Outcome::Failure);
    };
    // Pest's top-level Pairs is 'above' the match for the nonterminal you asked for,
    // with no useful information. It contains a single Pair which is the match for the nonterminal
    // you asked for.
    let requested_pair = extract_only_item(top_level_pairs).map_err(|m| match m {
        NoItems => "Pest reported empty response".to_owned(),
        Multiple => "Pest reported multiple top-level matches".to_owned(),
    })?;
    if requested_pair.as_rule() != nonterminal {
        return Err(format!(
            "Pest's match wasn't for the expected {nonterminal:?}"
        ));
    }
    let consumed_entire_input = requested_pair.as_span().end() == against.len();
    Ok(Outcome::Success {
        pair: requested_pair,
        consumed_entire_input,
    })
}

/// Information from the outcome of a Pest match attempt.
///
/// If we want to know exactly what was consumed, we can find out from 'pair'.
pub enum Outcome<'a, NONTERMINAL: RuleType> {
    Success {
        /// Pest Pair representing the match of the specified nonterminal.
        pair: Pair<'a, NONTERMINAL>,
        /// Whether the match's consumed characters were the whole of 'against'.
        consumed_entire_input: bool,
    },
    Failure,
}

/// Returns the only item from an iterator, or reports an error if it didn't have exactly one item.
pub fn extract_only_item<T>(mut stream: impl Iterator<Item = T>) -> Result<T, Multiplicity> {
    let Some(item) = stream.next() else {
        return Err(Multiplicity::NoItems);
    };
    let None = stream.next() else {
        return Err(Multiplicity::Multiple);
    };
    Ok(item)
}

pub enum Multiplicity {
    NoItems,
    Multiple,
}

/// Information from a successful match attempt of a PEG nonterminal.
pub struct MatchData<NONTERMINAL: RuleType> {
    /// The nonterminal whose match is being described.
    pub matched_nonterminal: NONTERMINAL,
    /// The input characters which were consumed by the match.
    pub consumed: Charseq,
    /// The subsidiary nonterminals which participated in the match and the characters they consumed.
    /// See "elaboration" in the writeup for the order.
    /// (Strictly, this is the elaboration of the match of the nonterminal's expression, not the
    ///  elaboration of the match of the nonterminal itself.)
    /// Omits nonterminals which are documented as terminals.
    elaboration: Vec<(NONTERMINAL, Charseq)>,
}

impl<NONTERMINAL: RuleType> std::fmt::Debug for MatchData<NONTERMINAL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} consuming {:?}",
            self.matched_nonterminal, self.consumed
        )
    }
}

impl<NONTERMINAL: RuleType + WrittenUp> MatchData<NONTERMINAL> {
    /// Make a MatchData instance from the raw data provided by Pest.
    pub fn new(pair: Pair<NONTERMINAL>) -> Self {
        Self {
            consumed: pair.as_str().into(),
            matched_nonterminal: pair.as_rule(),
            elaboration: pair
                .into_inner()
                .flatten()
                .filter(|sub| !(sub.as_rule().is_documented_as_terminal()))
                .map(|sub| (sub.as_rule(), sub.as_str().into()))
                .collect(),
        }
    }

    /// Says whether the specified subsidiary nonterminal participated in this match.
    pub fn participated(&self, nonterminal: NONTERMINAL) -> bool {
        self.elaboration.iter().any(|&(nt, _)| nt == nonterminal)
    }

    /// Returns the characters consumed by the only participating match of the specified subsidiary
    /// nonterminal, or None if that nonterminal did not participate in this match.
    ///
    /// Reports an error if that nonterminal participated in this match more than once.
    pub fn consumed_by_only_participating_match(
        &self,
        nonterminal: NONTERMINAL,
    ) -> Result<Option<&Charseq>, ()> {
        let mut found = None;
        for (candidate, consumed) in self.elaboration.iter() {
            if *candidate == nonterminal {
                match found {
                    Some(_) => {
                        return Err(());
                    }
                    None => {
                        found = Some(consumed);
                    }
                }
            }
        }
        Ok(found)
    }

    /// Returns the characters consumed by the first participating match of the specified subsidiary
    /// nonterminal in this match, or None if that nonterminal did not participate in this match.
    pub fn consumed_by_first_participating_match(
        &self,
        nonterminal: NONTERMINAL,
    ) -> Option<&Charseq> {
        for (candidate, consumed) in self.elaboration.iter() {
            if *candidate == nonterminal {
                return Some(consumed);
            }
        }
        None
    }

    /// Returns the characters consumed by the participating matches of the specified subsidiary
    /// nonterminal (see "Sequences of matches" in the writeup.
    pub fn consumed_by_all_participating_matches(&self, nonterminal: NONTERMINAL) -> Charseq {
        Charseq::new(
            self.elaboration
                .iter()
                .filter(|(candidate, _)| *candidate == nonterminal)
                .flat_map(|(_, consumed)| consumed.iter())
                .collect(),
        )
    }

    /// Describes the subsidiary nonterminals making up this match, with their consumed extents.
    ///
    /// Omits nonterminals which are documented as terminals.
    pub fn describe_submatches(&self) -> impl Iterator<Item = String> + use<'_, NONTERMINAL> {
        self.elaboration
            .iter()
            .map(|(rule, consumed)| format!("{rule:?} {consumed:?}"))
    }
}

pub trait WrittenUp {
    /// Reports whether a nonterminal is documented as a terminal in the writeup.
    fn is_documented_as_terminal(&self) -> bool {
        true
    }
}

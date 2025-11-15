//! Support for working with Pest parsing expression grammars.

use pest::RuleType;
use pest::iterators::Pair;

use crate::datatypes::char_sequences::Charseq;

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

    /// Returns the characters consumed by the specified subsidiary nonterminal, or None if that
    /// nonterminal did not participate in this match.
    ///
    /// Reports an error if that nonterminal participated in this match more than once.
    pub fn get_checked(&self, nonterminal: NONTERMINAL) -> Result<Option<&Charseq>, ()> {
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
    pub fn get_first(&self, nonterminal: NONTERMINAL) -> Option<&Charseq> {
        for (candidate, consumed) in self.elaboration.iter() {
            if *candidate == nonterminal {
                return Some(consumed);
            }
        }
        None
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

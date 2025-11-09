//! Runs all phases of lexing, through to producing coarse token forests.
//!
//! This module works with `RegularToken`s. These track more than simply which sequence of
//! characters was matched, but they don't track everything we might be interested in. See
//! `regular_tokens` for defails.

use crate::combination;
use crate::comparison::Verdict;
use crate::regular_tokens::{RegularToken, regularise_from_coarse, regularise_from_rustc};
use crate::reimplementation::cleaning::{self, CleaningOutcome};
use crate::reimplementation::doc_lowering::lower_doc_comments;
use crate::reimplementation::tokenisation;
use crate::rustc_harness::lex_via_rustc;
use crate::tree_construction;
use crate::trees::Forest;
use crate::{CleaningMode, Edition, Lowering};

/// Runs rustc's lexical analysis and returns the regularised result.
pub fn regularised_from_rustc(
    input: &str,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> Verdict<Forest<RegularToken>> {
    use lex_via_rustc::Analysis::*;
    match lex_via_rustc::analyse(input, edition, cleaning, lowering) {
        Accepts(tokens) => Verdict::Accepts(regularise_from_rustc(tokens)),
        Rejects(_, messages) => Verdict::Rejects(messages),
        CompilerError => Verdict::ModelError(vec!["rustc compiler error".into()]),
        HarnessError(message) => Verdict::ModelError(vec![message]),
    }
}

/// Runs lex_via_peg's lexical analysis and returns the regularised result.
pub fn regularised_from_peg(
    input: &str,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> Verdict<Forest<RegularToken>> {
    use tokenisation::Analysis::*;
    let cleaned = match cleaning::clean(&input.into(), edition, cleaning) {
        CleaningOutcome::Accepts(charseq) => charseq,
        CleaningOutcome::Rejects(reason) => return Verdict::Rejects(vec![reason]),
        CleaningOutcome::ModelError(message) => return Verdict::ModelError(vec![message]),
    };
    match tokenisation::analyse(&cleaned, edition) {
        Accepts(_, mut fine_tokens) => {
            if lowering == Lowering::LowerDocComments {
                fine_tokens = lower_doc_comments(fine_tokens, edition);
            }
            match tree_construction::construct_forest(fine_tokens) {
                Ok(forest) => {
                    Verdict::Accepts(regularise_from_coarse(combination::coarsen(forest)))
                }
                Err(message) => Verdict::Rejects(vec![message]),
            }
        }
        Rejects(reason) => Verdict::Rejects(reason.into_description()),
        ModelError(reason) => Verdict::ModelError(reason.into_description()),
    }
}

//! Run all phases of lexing, through to producing coarse token forests.
//!
//! This module works with `RegularToken`s. These track more than simply which sequence of
//! characters was matched, but they don't track everything we might be interested in. See
//! `regular_tokens` for defails.

use crate::cleaning;
use crate::combination;
use crate::comparison::Verdict;
use crate::doc_lowering::lower_doc_comments;
use crate::lex_via_peg;
use crate::regular_tokens::{regularise_from_coarse, regularise_from_rustc, RegularToken};
use crate::rustc_harness::lex_via_rustc;
use crate::tree_construction;
use crate::trees::Forest;
use crate::{Edition, Lowering};

/// Run rustc's lexical analysis and return the regularised result.
pub fn regularised_from_rustc(
    input: &str,
    edition: Edition,
    lowering: Lowering,
) -> Verdict<Forest<RegularToken>> {
    use lex_via_rustc::Analysis::*;
    match lex_via_rustc::analyse(input, edition, lowering) {
        Accepts(tokens) => Verdict::Accepts(regularise_from_rustc(tokens)),
        Rejects(_, messages) => Verdict::Rejects(messages),
        CompilerError => Verdict::ModelError(vec!["rustc compiler error".into()]),
    }
}

/// Run lex_via_peg's lexical analysis and return the regularised result.
pub fn regularised_from_peg(
    input: &str,
    edition: Edition,
    lowering: Lowering,
) -> Verdict<Forest<RegularToken>> {
    use lex_via_peg::Analysis::*;
    let cleaned = cleaning::clean(&input.into(), edition);
    match lex_via_peg::analyse(&cleaned, edition) {
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

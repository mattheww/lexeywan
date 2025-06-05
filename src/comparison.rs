//! High-level support for comparing the rustc and peg analyses.

use crate::cleaning;
use crate::combination;
use crate::doc_lowering::lower_doc_comments;
use crate::lex_via_peg;
use crate::lex_via_rustc;
use crate::regular_tokens::{regularise_from_coarse, regularise_from_rustc, RegularToken};
use crate::tree_construction;
use crate::trees::Forest;
use crate::{Edition, Lowering};

/// The "regularised" result of running a lexer.
pub enum Regularisation {
    /// The lexer accepted the input.
    ///
    /// Contains the lexer's output, in "regularised" form (suitable for comparing implementations).
    Accepts(Forest<RegularToken>),

    /// The lexer rejected the input.
    ///
    /// The strings describe why the input was rejected.
    Rejects(Vec<String>),

    /// The lexer reported a problem in its model or implementation.
    ModelError(Vec<String>),
}

/// Run rustc's lexical analysis and return the regularised result.
pub fn regularised_from_rustc(input: &str, edition: Edition, lowering: Lowering) -> Regularisation {
    use lex_via_rustc::Analysis::*;
    match lex_via_rustc::analyse(input, edition, lowering) {
        Accepts(tokens) => Regularisation::Accepts(regularise_from_rustc(tokens)),
        Rejects(_, messages) => Regularisation::Rejects(messages),
        CompilerError => Regularisation::ModelError(vec!["rustc compiler error".into()]),
    }
}

/// Run lex_via_peg's lexical analysis and return the regularised result.
pub fn regularised_from_peg(input: &str, edition: Edition, lowering: Lowering) -> Regularisation {
    use lex_via_peg::Analysis::*;
    let cleaned = cleaning::clean(&input.into(), edition);
    match lex_via_peg::analyse(&cleaned, edition) {
        Accepts(_, mut fine_tokens) => {
            if lowering == Lowering::LowerDocComments {
                fine_tokens = lower_doc_comments(fine_tokens);
            }
            match tree_construction::construct_forest(fine_tokens) {
                Ok(forest) => {
                    Regularisation::Accepts(regularise_from_coarse(combination::coarsen(forest)))
                }
                Err(message) => Regularisation::Rejects(vec![message]),
            }
        }
        Rejects(reason) => Regularisation::Rejects(reason.into_description()),
        ModelError(reason) => Regularisation::ModelError(reason.into_description()),
    }
}

/// The result of comparing the output of two lexers.
pub enum Comparison {
    /// The two regularisations were equivalent.
    ///
    /// This means either they produced equal sequences of `RegularToken`s, or they both rejected
    /// the input (there's no attempt to check for equivalent reasons for rejection).
    ///
    /// The `RegularToken`s track more than simply which sequence of characters was matched, but
    /// they don't track everything we might be interested in. See `regular_tokens` for defails.
    Agree,

    /// The two regularisations disagreed.
    ///
    /// This means either they produced different sequences of `RegularToken`s, or one accepted the
    /// input but the other rejected it.
    Differ,

    /// One of the lexers reported a problem in its model or implementation.
    ModelErrors,
}

/// Compare the output of two lexers.
pub fn compare(r1: &Regularisation, r2: &Regularisation) -> Comparison {
    use Comparison::*;
    use Regularisation::*;
    match (r1, r2) {
        (Accepts(tokens1), Accepts(tokens2)) if tokens1 == tokens2 => Agree,
        (Accepts(_), Accepts(_)) => Differ,
        (Rejects(_), Rejects(_)) => Agree,
        (Accepts(_), Rejects(_)) => Differ,
        (Rejects(_), Accepts(_)) => Differ,
        _ => ModelErrors,
    }
}

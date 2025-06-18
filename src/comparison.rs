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

/// The result of running a lexer.
pub enum Verdict<T: Eq> {
    /// The lexer accepted the input.
    ///
    /// Contains the lexer's output, in a form suitable for comparing implementations.
    Accepts(T),

    /// The lexer rejected the input.
    ///
    /// The strings describe why the input was rejected.
    Rejects(Vec<String>),

    /// The lexer reported a problem in its model or implementation.
    ModelError(Vec<String>),
}

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

/// The result of comparing the output of two lexers.
pub enum Comparison {
    /// The two lexers produced equivalent verdicts.
    ///
    /// This means either they produced equal data (in practice, identical forests of tokens), or
    /// they both rejected the input (there's no attempt to check for equivalent reasons for
    /// rejection).
    Agree,

    /// The two lexers disagreed.
    ///
    /// This means either they produced different data, or one accepted the input but the other
    /// rejected it.
    Differ,

    /// One of the lexers reported a problem in its model or implementation.
    ModelErrors,
}

/// Compare the output of two lexers.
pub fn compare<T: Eq>(r1: &Verdict<T>, r2: &Verdict<T>) -> Comparison {
    use Comparison::*;
    use Verdict::*;
    match (r1, r2) {
        (Accepts(tokens1), Accepts(tokens2)) if tokens1 == tokens2 => Agree,
        (Accepts(_), Accepts(_)) => Differ,
        (Rejects(_), Rejects(_)) => Agree,
        (Accepts(_), Rejects(_)) => Differ,
        (Rejects(_), Accepts(_)) => Differ,
        _ => ModelErrors,
    }
}

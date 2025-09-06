//! Runs the form of lexing used by declarative macros.
//!
//! This module works with the stringified representation of coarse tokens.

use crate::char_sequences::Charseq;
use crate::combination::{self, CoarseToken};
use crate::comparison::Verdict;
use crate::doc_lowering::lower_doc_comments;
use crate::rustc_harness::decl_via_rustc;
use crate::trees::Forest;
use crate::{cleaning, lex_via_peg, tree_construction, Edition};

/// Runs rustc's lexical analysis by embedding the tokens in a declarative macro invocation.
///
/// We use the `tt` fragment matcher to recover the tokens, and stringify!() to describe them.
pub fn stringified_via_declarative_macros(
    input: &str,
    edition: Edition,
) -> Verdict<Forest<Charseq>> {
    use decl_via_rustc::Analysis::*;
    match decl_via_rustc::analyse(input, edition) {
        Accepts(forest) => Verdict::Accepts(forest.map(|token| token.stringified.into())),
        Rejects(messages) => Verdict::Rejects(messages),
        FrameworkFailed(message) => {
            Verdict::ModelError(vec!["macro-based framework failed:".into(), message])
        }
        CompilerError => Verdict::ModelError(vec![
            "rustc compiler error, or panic in the macro-based framework".into(),
        ]),
    }
}

/// Runs the reimplementation's lexical analysis, as if for a declarative macro invocation.
///
/// Unconditionally lowers doc-comments.
///
/// Doesn't run the parts of cleaning that apply to files as a whole (byte order mark removal and
/// shebang removal).
///
/// Models stringify!().
pub fn stringified_via_peg(input: &str, edition: Edition) -> Verdict<Forest<Charseq>> {
    use lex_via_peg::Analysis::*;
    let cleaned = cleaning::clean_for_macro_input(&input.into(), edition);
    match lex_via_peg::analyse(&cleaned, edition) {
        Accepts(_, fine_tokens) => {
            let fine_tokens = lower_doc_comments(fine_tokens, edition);
            match tree_construction::construct_forest(fine_tokens) {
                Ok(forest) => {
                    Verdict::Accepts(combination::coarsen(forest).map(|token| stringify(&token)))
                }
                Err(message) => Verdict::Rejects(vec![message]),
            }
        }
        Rejects(reason) => Verdict::Rejects(reason.into_description()),
        ModelError(reason) => Verdict::ModelError(reason.into_description()),
    }
}

/// Returns the character sequence that `stringify!()` would produce for a token.
///
/// The returned sequence matches the value of the literal expression which would be produced by
/// `stringify!()`.
fn stringify(token: &CoarseToken) -> Charseq {
    use crate::combination::CoarseTokenData::*;
    match &token.origin {
        crate::tokens_common::Origin::Natural { extent } => match &token.data {
            Identifier {
                represented_identifier,
            } => represented_identifier.clone(),
            RawIdentifier {
                represented_identifier,
            } => ['r', '#']
                .iter()
                .chain(represented_identifier.chars())
                .copied()
                .collect(),
            _ => extent.clone(),
        },
        crate::tokens_common::Origin::Synthetic { stringified, .. } => stringified.clone(),
    }
}

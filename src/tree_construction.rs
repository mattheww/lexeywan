//! Converts a sequence of ["fine-grained"][FineToken] tokens into trees.

use crate::fine_tokens::{FineToken, FineTokenData};
use crate::trees::{Forest, GroupKind, Tree};

/// Converts a sequence of `FineToken`s into a TokenForest.
pub fn construct_forest(
    tokens: impl IntoIterator<Item = FineToken>,
) -> Result<Forest<FineToken>, String> {
    construct_forest_inner(&mut tokens.into_iter(), None)
}

fn construct_forest_inner(
    tokens: &mut impl Iterator<Item = FineToken>,
    in_group: Option<GroupKind>,
) -> Result<Forest<FineToken>, String> {
    let mut constructed = Forest::<FineToken>::new();
    while let Some(token) = tokens.next() {
        let tree = match token.data {
            FineTokenData::Punctuation { mark } => {
                if let Some(group_kind) = GroupKind::for_open_char(mark) {
                    Tree::<FineToken>::Group(
                        group_kind,
                        construct_forest_inner(tokens, Some(group_kind))?,
                    )
                } else if let Some(group_kind) = GroupKind::for_close_char(mark) {
                    if Some(group_kind) == in_group {
                        return Ok(constructed);
                    } else {
                        return Err(format!("unexpected close delimiter: {mark}"));
                    }
                } else {
                    Tree::<FineToken>::Token(token)
                }
            }
            _ => Tree::<FineToken>::Token(token),
        };
        constructed.push(tree);
    }
    match in_group {
        Some(group_kind) => Err(format!(
            "missing close delimiter: {}",
            group_kind.close_char()
        )),
        None => Ok(constructed),
    }
}

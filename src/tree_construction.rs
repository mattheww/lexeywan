//! Converts a sequence of ["fine-grained"][FineToken] tokens into trees.

use crate::trees::{Forest, GroupKind, Tree};

pub trait Token {
    /// If this token might represent a delimiter, returns the delimiter character.
    ///
    /// For tokens which don't represent delimiters, it doesn't matter whether this returns None or
    /// a non-delimiter character.
    fn as_delimiter(&self) -> Option<char>;
}

/// Converts a sequence of tokens into a TokenForest.
pub fn construct_forest<TOKEN: Token>(
    tokens: impl IntoIterator<Item = TOKEN>,
) -> Result<Forest<TOKEN>, String> {
    construct_forest_inner(&mut tokens.into_iter(), None)
}

fn construct_forest_inner<TOKEN: Token>(
    tokens: &mut impl Iterator<Item = TOKEN>,
    in_group: Option<GroupKind>,
) -> Result<Forest<TOKEN>, String> {
    let mut constructed = Forest::<TOKEN>::new();
    while let Some(token) = tokens.next() {
        let tree = match token.as_delimiter() {
            Some(mark) => {
                if let Some(group_kind) = GroupKind::for_open_char(mark) {
                    Tree::<TOKEN>::Group(
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
                    Tree::<TOKEN>::Token(token)
                }
            }
            None => Tree::<TOKEN>::Token(token),
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

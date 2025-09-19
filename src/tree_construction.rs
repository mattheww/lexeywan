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
    match construct_forest_inner(&mut tokens.into_iter())? {
        (forest, None) => Ok(forest),
        (_, Some(closing_group_kind)) => Err(format!(
            "unexpected close delimiter: {}",
            closing_group_kind.close_char()
        )),
    }
}

/// Converts tokens until end-of-stream or an unexpected closing delimiter.
///
/// Returns the converted forest and the GroupKind of the closing delimiter.
fn construct_forest_inner<TOKEN: Token>(
    tokens: &mut impl Iterator<Item = TOKEN>,
) -> Result<(Forest<TOKEN>, Option<GroupKind>), String> {
    let mut constructed = Forest::<TOKEN>::new();
    while let Some(token) = tokens.next() {
        let tree = match token.as_delimiter() {
            Some(mark) => {
                if let Some(opening_group_kind) = GroupKind::for_open_char(mark) {
                    match construct_forest_inner(tokens)? {
                        (forest, Some(closing_group_kind))
                            if closing_group_kind == opening_group_kind =>
                        {
                            Tree::<TOKEN>::Group(opening_group_kind, forest)
                        }
                        (_, Some(closing_group_kind)) => {
                            return Err(format!(
                                "unexpected close delimiter: {}",
                                closing_group_kind.close_char()
                            ));
                        }
                        (_, None) => {
                            return Err(format!(
                                "missing close delimiter: {}",
                                opening_group_kind.close_char()
                            ));
                        }
                    }
                } else if let Some(closing_group_kind) = GroupKind::for_close_char(mark) {
                    return Ok((constructed, Some(closing_group_kind)));
                } else {
                    Tree::<TOKEN>::Token(token)
                }
            }
            None => Tree::<TOKEN>::Token(token),
        };
        constructed.push(tree);
    }
    Ok((constructed, None))
}

//! Convert a forest of tokens into a linear sequence.
//!
//! This isn't needed anywhere in our model of Rust; it's used for reporting.

use crate::trees::{Forest, GroupKind, Tree};

/// Convert a forest of tokens into a linear sequence.
///
/// The result contains references to the original tokens, together with `OpenDelimiter` and
/// `CloseDelimiter` items indicating the forest's group structure.
pub fn flatten<T>(forest: &Forest<T>) -> Vec<FlatItem<'_, T>> {
    Flattener::process(forest)
}

struct Flattener<'a, T> {
    output: Vec<FlatItem<'a, T>>,
}

impl<'a, T> Flattener<'a, T> {
    fn process(forest: &'a Forest<T>) -> Vec<FlatItem<'a, T>> {
        let mut flattener = Self { output: Vec::new() };
        flattener.add_tokens_from_forest(forest);
        flattener.output
    }

    fn add_tokens_from_forest(&mut self, forest: &'a Forest<T>) {
        for token_tree in forest.contents.iter() {
            self.add_tokens_from_tree(token_tree);
        }
    }

    fn add_tokens_from_tree(&mut self, tree: &'a Tree<T>) {
        match tree {
            Tree::Token(token) => self.output.push(FlatItem::Token(token)),
            Tree::Group(group_kind, forest) => {
                self.output.push(FlatItem::OpenDelimiter(*group_kind));
                self.add_tokens_from_forest(forest);
                self.output.push(FlatItem::CloseDelimiter(*group_kind));
            }
        }
    }
}

/// An item in the sequence returned by [`flatten()`].
///
/// The normal way to use one of these items is to report its `Debug` representation.
pub enum FlatItem<'a, T> {
    Token(&'a T),
    OpenDelimiter(GroupKind),
    CloseDelimiter(GroupKind),
}

impl<'a, T> std::fmt::Debug for FlatItem<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlatItem::Token(token) => token.fmt(f),
            FlatItem::OpenDelimiter(group_kind) => write!(f, "{}", group_kind.open_char()),
            FlatItem::CloseDelimiter(group_kind) => write!(f, "{}", group_kind.close_char()),
        }
    }
}

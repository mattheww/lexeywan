//! Representation of trees and forests of tokens.

use std::iter::Peekable;

/// A token or delimited group of tokens.
#[derive(PartialEq, Eq)]
pub enum Tree<T> {
    Token(T),
    Group(GroupKind, Forest<T>),
}

/// A sequence of tokens and delimited groups of tokens.
#[derive(PartialEq, Eq)]
pub struct Forest<T> {
    pub contents: Vec<Tree<T>>,
}

impl<T> Forest<T> {
    /// Returns a new empty forest.
    pub fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }

    /// Adds a single token or delimited group of tokens to the sequence.
    pub fn push(&mut self, value: Tree<T>) {
        self.contents.push(value)
    }

    /// Says whether the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

impl<T> IntoIterator for Forest<T> {
    type Item = Tree<T>;

    type IntoIter = <Vec<Tree<T>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.contents.into_iter()
    }
}

impl<T> FromIterator<Tree<T>> for Forest<T> {
    fn from_iter<I: IntoIterator<Item = Tree<T>>>(iter: I) -> Self {
        Self {
            contents: iter.into_iter().collect(),
        }
    }
}

/// A kind of delimited group of tokens.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum GroupKind {
    /// `()` delimiters
    Parenthesised,
    /// `{}` delimiters
    Braced,
    /// `[]` delimiters
    Bracketed,
}

impl GroupKind {
    /// The delimiter character used to start this kind of group.
    pub fn open_char(&self) -> char {
        match self {
            GroupKind::Parenthesised => '(',
            GroupKind::Braced => '{',
            GroupKind::Bracketed => '[',
        }
    }

    /// The delimiter character used to end this kind of group.
    pub fn close_char(&self) -> char {
        match self {
            GroupKind::Parenthesised => ')',
            GroupKind::Braced => '}',
            GroupKind::Bracketed => ']',
        }
    }

    /// Returns the kind of group which uses the specified character as its opening delimiter, or
    /// `None` if the character cannot start a group.
    pub fn for_open_char(c: char) -> Option<GroupKind> {
        match c {
            '(' => Some(GroupKind::Parenthesised),
            '{' => Some(GroupKind::Braced),
            '[' => Some(GroupKind::Bracketed),
            _ => None,
        }
    }

    /// Returns the kind of group which uses the specified character as its closing delimiter, or
    /// `None` if the character cannot start a group.
    pub fn for_close_char(c: char) -> Option<GroupKind> {
        match c {
            ')' => Some(GroupKind::Parenthesised),
            '}' => Some(GroupKind::Braced),
            ']' => Some(GroupKind::Bracketed),
            _ => None,
        }
    }
}

impl<T> Forest<T> {
    /// Converts a forest into a new forest with the same structure, mapping individual tokens.
    pub fn map<F, U>(self, f: F) -> Forest<U>
    where
        F: Fn(T) -> U + Copy,
    {
        self.into_iter()
            .map(move |tree| match tree {
                Tree::Token(token) => Tree::Token(f(token)),
                Tree::Group(group_kind, inner) => Tree::Group(group_kind, inner.map(f)),
            })
            .collect()
    }
}

impl<T> Forest<T> {
    /// Converts a forest into a new forest with a similar structure, possibly omitting and/or
    /// combining tokens.
    ///
    /// The callback `f` is given an input token and a peekable stream of the remaining trees from
    /// the same group. It can call `next()` on the stream to consume future tokens or groups. It
    /// can return `None` to skip the input token.
    pub fn combining_map<F, U>(self, f: F) -> Forest<U>
    where
        F: Fn(T, &mut Peekable<<Self as IntoIterator>::IntoIter>) -> Option<U> + Copy,
    {
        let mut transformed = Forest::new();
        let mut stream = self.into_iter().peekable();
        while let Some(tree) = stream.next() {
            match tree {
                Tree::Token(token) => {
                    if let Some(token) = f(token, &mut stream) {
                        transformed.push(Tree::Token(token));
                    }
                }
                Tree::Group(group_kind, forest) => {
                    transformed.push(Tree::Group(group_kind, forest.combining_map(f)))
                }
            }
        }
        transformed
    }
}

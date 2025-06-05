//! Defines "fine-grained" tokens and the associated data types.
//!
//! This representation uses explicit whitespace tokens.

use crate::char_sequences::Charseq;

/// A "Fine-grained" token.
///
/// This is the form of token used in lex_via_peg's output.
///
/// It's fine-grained in the sense that each punctuation token contains only a single character. A
/// [`LifetimeOrLabel`][`FineTokenData::LifetimeOrLabel`] token contains both the leading `'` and
/// the identifier.

#[derive(Clone, std::fmt::Debug)]
pub struct FineToken {
    /// The token's kind and attributes.
    pub data: FineTokenData,

    /// The input characters which make up the token.
    pub extent: Charseq,
}

/// A fine-grained token's kind and attributes.
#[derive(Clone, std::fmt::Debug)]
pub enum FineTokenData {
    Whitespace,
    LineComment {
        style: CommentStyle,
        body: Charseq,
    },
    BlockComment {
        style: CommentStyle,
        body: Charseq,
    },
    Punctuation {
        mark: char,
    },
    Identifier {
        represented_identifier: Charseq,
    },
    RawIdentifier {
        represented_identifier: Charseq,
    },
    LifetimeOrLabel {
        name: Charseq,
    },
    RawLifetimeOrLabel {
        name: Charseq,
    },
    CharacterLiteral {
        represented_character: char,
        suffix: Charseq,
    },
    ByteLiteral {
        represented_byte: u8,
        suffix: Charseq,
    },
    StringLiteral {
        represented_string: Charseq,
        suffix: Charseq,
    },
    RawStringLiteral {
        represented_string: Charseq,
        suffix: Charseq,
    },
    ByteStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    RawByteStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    CStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    RawCStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    IntegerLiteral {
        base: NumericBase,
        digits: Charseq,
        suffix: Charseq,
    },
    FloatLiteral {
        body: Charseq,
        suffix: Charseq,
    },
}

/// Whether a comment is a doc-comment, and if so which sort of doc-comment.
#[derive(PartialEq, Eq, Copy, Clone, std::fmt::Debug)]
#[allow(clippy::enum_variant_names)]
pub enum CommentStyle {
    NonDoc,
    InnerDoc,
    OuterDoc,
}

/// Base (radix) of a numeric literal.
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum NumericBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl FineTokenData {
    /// Says whether this token counts as whitespace.
    ///
    /// Comments count as whitespace, except for doc-comments.
    pub fn is_whitespace(&self) -> bool {
        match self {
            FineTokenData::Whitespace => true,
            FineTokenData::LineComment {
                style: CommentStyle::NonDoc,
                ..
            } => true,
            FineTokenData::LineComment { .. } => false,
            FineTokenData::BlockComment {
                style: CommentStyle::NonDoc,
                ..
            } => true,
            FineTokenData::BlockComment { .. } => false,
            _ => false,
        }
    }
}

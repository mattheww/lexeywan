//! Converts ["fine-grained"][FineToken] tokens into ["coarse"][CoarseToken] tokens.
//!
//! These combine some adjacent punctuation marks into single tokens, in the same way as by-example
//! macros with the `tt` fragment specifier.
//!
//! This representation doesn't have explicit whitespace tokens. It has explicit [`Spacing`]
//! information instead.

use crate::char_sequences::{concat_charseqs, Charseq};
use crate::fine_tokens::{self, CommentStyle, FineToken, FineTokenData};

/// A "Coarse-grained" token.
///
/// This is close to the [`FineToken`] returned by Lexclucid step 2, but:
/// - There are no tokens for whitespace
/// - Tokens for comments always represent doc-comments
/// - Punctuation can have multiple characters
pub struct CoarseToken {
    /// The token's kind and attributes.
    pub data: CoarseTokenData,

    /// The input characters which make up the token.
    pub extent: Charseq,

    /// This token's relationship to the following token.
    pub spacing: Spacing,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Spacing {
    /// This token is followed by whitespace, a (non-doc) comment, or end-of-input.
    Alone,
    /// There is no space between this token and the next.
    Joint,
}

/// A coarse-grained token's kind and attributes.
#[derive(Clone, std::fmt::Debug)]
pub enum CoarseTokenData {
    LineComment {
        style: DocCommentStyle,
        body: Charseq,
    },
    BlockComment {
        style: DocCommentStyle,
        body: Charseq,
    },
    Punctuation {
        marks: Charseq,
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
    ByteLiteral {
        represented_byte: u8,
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
    CharacterLiteral {
        represented_character: char,
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
    CStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    RawCStringLiteral {
        represented_bytes: Vec<u8>,
        suffix: Charseq,
    },
    IntegerLiteral {
        #[allow(unused)]
        base: NumericBase,
        #[allow(unused)]
        digits: Charseq,
        suffix: Charseq,
    },
    FloatLiteral {
        #[allow(unused)]
        body: Charseq,
        suffix: Charseq,
    },
}

/// Whether a doc-comment is an inner or outer doc-comment.
///
/// Note that non-doc-comments have disappeared in this representation (they're treated as
/// whitespace).
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum DocCommentStyle {
    Inner,
    Outer,
}

/// Base (radix) of a numeric literal.
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum NumericBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

/// Converts a sequence of `FineToken`s into a sequence of `CoarseToken`s.
pub fn coarsen(tokens: impl IntoIterator<Item = FineToken>) -> Vec<CoarseToken> {
    combine(process_whitespace(tokens))
}

/// Calculates spacing information for fine-grained tokens, dropping tokens representing whitespace.
fn process_whitespace(tokens: impl IntoIterator<Item = FineToken>) -> Vec<(FineToken, Spacing)> {
    let mut processed = Vec::new();
    let mut stream = tokens.into_iter().peekable();
    while let Some(token) = stream.next() {
        if !token.data.is_whitespace() {
            let spacing = match stream.peek() {
                Some(peeked) => {
                    if peeked.data.is_whitespace() {
                        Spacing::Alone
                    } else {
                        Spacing::Joint
                    }
                }
                None => Spacing::Alone,
            };
            processed.push((token, spacing));
        }
    }
    processed
}

/// "Glue"s `FineToken`s with spacing information into `CoarseToken`s.
fn combine(stream: Vec<(FineToken, Spacing)>) -> Vec<CoarseToken> {
    let mut result = Vec::new();
    let mut stream = stream.into_iter().peekable();
    while let Some((token1, spacing)) = stream.next() {
        if spacing == Spacing::Joint {
            if let Some((token2, spacing2)) = stream.peek() {
                if let Some(double_token) = merge_two(&token1.data, &token2.data) {
                    let mut combined_token = CoarseToken {
                        data: double_token,
                        extent: concat_charseqs(&token1.extent, &token2.extent),
                        spacing: *spacing2,
                    };
                    // skip the second token
                    stream.next();
                    if combined_token.spacing == Spacing::Joint {
                        if let Some((token3, spacing3)) = stream.peek() {
                            if let Some(triple_token) =
                                merge_three(&combined_token.data, &token3.data)
                            {
                                combined_token = CoarseToken {
                                    data: triple_token,
                                    extent: concat_charseqs(&combined_token.extent, &token3.extent),
                                    spacing: *spacing3,
                                };
                                // skip the third token
                                stream.next();
                            }
                        }
                    }
                    result.push(combined_token);
                    continue;
                }
            }
        }
        result.push(CoarseToken {
            data: token1.data.try_into().unwrap(),
            extent: token1.extent,
            spacing,
        });
    }
    result
}

/// Merges two fine-grained tokens if they're mergeable.
///
/// Returns the merged token as a coarse token, or None if they don't merge.
fn merge_two(first: &FineTokenData, second: &FineTokenData) -> Option<CoarseTokenData> {
    match (&first, &second) {
        (
            FineTokenData::Punctuation { mark: mark1 },
            FineTokenData::Punctuation { mark: mark2 },
        ) => {
            if PAIRS.contains(&(*mark1, *mark2)) {
                Some(CoarseTokenData::Punctuation {
                    marks: [*mark1, *mark2].as_slice().into(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Merges a coarse token with a fine-grained token, if they're mergeable.
///
/// Returns the merged token as a coarse token, or None if they don't merge.
fn merge_three(first: &CoarseTokenData, second: &FineTokenData) -> Option<CoarseTokenData> {
    match (&first, &second) {
        (
            CoarseTokenData::Punctuation { marks: marks1 },
            FineTokenData::Punctuation { mark: mark2 },
        ) => {
            if marks1.len() == 2 && TRIPLES.contains(&(marks1[0], marks1[1], *mark2)) {
                Some(CoarseTokenData::Punctuation {
                    marks: [marks1[0], marks1[1], *mark2].as_slice().into(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

const PAIRS: [(char, char); 21] = [
    ('<', '='),
    ('=', '='),
    ('!', '='),
    ('>', '='),
    ('&', '&'),
    ('|', '|'),
    ('.', '.'),
    (':', ':'),
    ('-', '>'),
    ('<', '-'),
    ('=', '>'),
    ('<', '<'),
    ('>', '>'),
    ('+', '='),
    ('-', '='),
    ('*', '='),
    ('/', '='),
    ('%', '='),
    ('^', '='),
    ('&', '='),
    ('|', '='),
];

const TRIPLES: [(char, char, char); 4] = [
    ('.', '.', '.'),
    ('.', '.', '='),
    ('<', '<', '='),
    ('>', '>', '='),
];

impl TryFrom<FineTokenData> for CoarseTokenData {
    type Error = ();

    /// Converts the kind and attributes of a fine-grained token to those for a coarse token.
    ///
    /// This will succeed for all tokens which survive `process_whitespace()`.
    fn try_from(data: FineTokenData) -> Result<Self, Self::Error> {
        match data {
            FineTokenData::Whitespace => Err(()),
            FineTokenData::LineComment {
                style: CommentStyle::InnerDoc,
                body,
            } => Ok(CoarseTokenData::LineComment {
                style: DocCommentStyle::Inner,
                body,
            }),
            FineTokenData::LineComment {
                style: CommentStyle::OuterDoc,
                body,
            } => Ok(CoarseTokenData::LineComment {
                style: DocCommentStyle::Outer,
                body,
            }),
            FineTokenData::LineComment {
                style: CommentStyle::NonDoc,
                ..
            } => Err(()),
            FineTokenData::BlockComment {
                style: CommentStyle::InnerDoc,
                body,
            } => Ok(CoarseTokenData::BlockComment {
                style: DocCommentStyle::Inner,
                body,
            }),
            FineTokenData::BlockComment {
                style: CommentStyle::OuterDoc,
                body,
            } => Ok(CoarseTokenData::BlockComment {
                style: DocCommentStyle::Outer,
                body,
            }),
            FineTokenData::BlockComment {
                style: CommentStyle::NonDoc,
                ..
            } => Err(()),
            FineTokenData::Punctuation { mark } => {
                Ok(CoarseTokenData::Punctuation { marks: mark.into() })
            }
            FineTokenData::Identifier {
                represented_identifier,
            } => Ok(CoarseTokenData::Identifier {
                represented_identifier,
            }),
            FineTokenData::RawIdentifier {
                represented_identifier,
            } => Ok(CoarseTokenData::RawIdentifier {
                represented_identifier,
            }),
            FineTokenData::LifetimeOrLabel { name } => {
                Ok(CoarseTokenData::LifetimeOrLabel { name })
            }
            FineTokenData::RawLifetimeOrLabel { name } => {
                Ok(CoarseTokenData::RawLifetimeOrLabel { name })
            }
            FineTokenData::ByteLiteral {
                represented_byte,
                suffix,
            } => Ok(CoarseTokenData::ByteLiteral {
                represented_byte,
                suffix,
            }),
            FineTokenData::CharacterLiteral {
                represented_character,
                suffix,
            } => Ok(CoarseTokenData::CharacterLiteral {
                represented_character,
                suffix,
            }),
            FineTokenData::StringLiteral {
                represented_string,
                suffix,
            } => Ok(CoarseTokenData::StringLiteral {
                represented_string,
                suffix,
            }),
            FineTokenData::ByteStringLiteral {
                represented_bytes,
                suffix,
            } => Ok(CoarseTokenData::ByteStringLiteral {
                represented_bytes,
                suffix,
            }),
            FineTokenData::CStringLiteral {
                represented_bytes,
                suffix,
            } => Ok(CoarseTokenData::CStringLiteral {
                represented_bytes,
                suffix,
            }),
            FineTokenData::RawStringLiteral {
                represented_string,
                suffix,
            } => Ok(CoarseTokenData::RawStringLiteral {
                represented_string,
                suffix,
            }),
            FineTokenData::RawByteStringLiteral {
                represented_bytes,
                suffix,
            } => Ok(CoarseTokenData::RawByteStringLiteral {
                represented_bytes,
                suffix,
            }),
            FineTokenData::RawCStringLiteral {
                represented_bytes,
                suffix,
            } => Ok(CoarseTokenData::RawCStringLiteral {
                represented_bytes,
                suffix,
            }),
            FineTokenData::IntegerLiteral {
                base,
                digits,
                suffix,
            } => Ok(CoarseTokenData::IntegerLiteral {
                base: base.into(),
                digits,
                suffix,
            }),
            FineTokenData::FloatLiteral { body, suffix } => {
                Ok(CoarseTokenData::FloatLiteral { body, suffix })
            }
        }
    }
}

impl From<fine_tokens::NumericBase> for NumericBase {
    fn from(base: fine_tokens::NumericBase) -> Self {
        match base {
            fine_tokens::NumericBase::Binary => NumericBase::Binary,
            fine_tokens::NumericBase::Octal => NumericBase::Octal,
            fine_tokens::NumericBase::Decimal => NumericBase::Decimal,
            fine_tokens::NumericBase::Hexadecimal => NumericBase::Hexadecimal,
        }
    }
}

//! Converts ["fine-grained"][FineToken] tokens into ["coarse"][CoarseToken] tokens.
//!
//! These combine some adjacent punctuation marks into single tokens, in the same way as by-example
//! macros with the `tt` fragment specifier.
//!
//! This representation doesn't have whitespace tokens: we've used all the information we need
//! from them to perform combination.

use crate::char_sequences::{Charseq, concat_charseqs};
use crate::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::tokens_common::{NumericBase, Origin};
use crate::trees::{Forest, Tree};

/// A "Coarse-grained" token.
///
/// This is close to [`FineToken`], but:
/// - There are no tokens for whitespace
/// - Tokens for comments always represent doc-comments
/// - Punctuation can have multiple characters
/// - Punctuation never represents a delimiter
pub struct CoarseToken {
    /// The token's kind and attributes.
    pub data: CoarseTokenData,

    /// Where this token came from.
    pub origin: Origin,
}

impl std::fmt::Debug for CoarseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.origin {
            Origin::Natural { extent } => write!(f, "{:?}, {:?}", self.data, extent),
            Origin::Synthetic { lowered_from, .. } => {
                write!(f, "{:?}, lowered from {:?}", self.data, lowered_from)
            }
        }
    }
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
    Ident {
        represented_ident: Charseq,
    },
    RawIdent {
        represented_ident: Charseq,
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

/// Converts a fine-grained token forest into a coarse-grained one.
pub fn coarsen(forest: Forest<FineToken>) -> Forest<CoarseToken> {
    map_combine(map_process_whitespace(forest))
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum Spacing {
    /// This token is followed by whitespace, a (non-doc) comment, or end-of-input.
    Alone,
    /// There is no space between this token and the next.
    Joint,
}

/// Calculates spacing information for fine-grained tokens, dropping tokens representing whitespace.
///
/// The Spacing returned describes the spacing after the token it's paired with.
///
/// We don't try to track spacing around delimiters:
/// - we don't provide a way to represent spacing after a delimiter
/// - we don't track spacing before a delimiter (we always say 'Alone')
fn map_process_whitespace(forest: Forest<FineToken>) -> Forest<(FineToken, Spacing)> {
    forest.combining_map(|token, tokens| {
        (!token.data.is_whitespace()).then(|| {
            let spacing = match tokens.peek() {
                Some(Tree::Token(token)) if !token.data.is_whitespace() => Spacing::Joint,
                _ => Spacing::Alone,
            };
            (token, spacing)
        })
    })
}

/// "Glue"s `FineToken`s with spacing information into `CoarseToken`s.
fn map_combine(forest: Forest<(FineToken, Spacing)>) -> Forest<CoarseToken> {
    forest.combining_map(|(token1, spacing), tokens| {
        if spacing == Spacing::Joint {
            if let Some(Tree::Token((token2, spacing2))) = tokens.peek() {
                if let Some(double_token) = merge_two(&token1.data, &token2.data) {
                    let mut combined_token = CoarseToken {
                        data: double_token,
                        origin: combine_origins(&token1.origin, &token2.origin),
                    };
                    let may_combine_further = *spacing2 == Spacing::Joint;
                    // skip the second token
                    tokens.next();
                    if may_combine_further {
                        if let Some(Tree::Token((token3, _))) = tokens.peek() {
                            if let Some(triple_token) =
                                merge_three(&combined_token.data, &token3.data)
                            {
                                combined_token = CoarseToken {
                                    data: triple_token,
                                    origin: combine_origins(&combined_token.origin, &token3.origin),
                                };
                                // skip the third token
                                tokens.next();
                            }
                        }
                    }
                    return Some(combined_token);
                }
            }
        }
        Some(CoarseToken {
            data: token1.data.try_into().unwrap(),
            origin: token1.origin,
        })
    })
}

/// Merge the origin information for two tokens.
///
/// The cases where one of the tokens is synthetic don't actually happen, so we just make up
/// something plausible.
fn combine_origins(o1: &Origin, o2: &Origin) -> Origin {
    match (o1, o2) {
        (Origin::Natural { extent: e1 }, Origin::Natural { extent: e2 }) => Origin::Natural {
            extent: concat_charseqs(e1, e2),
        },
        (
            Origin::Natural { .. },
            Origin::Synthetic {
                lowered_from: lf2, ..
            },
        ) => Origin::Synthetic {
            lowered_from: lf2.clone(),
            stringified: "".into(),
        },
        (
            Origin::Synthetic {
                lowered_from: lf1, ..
            },
            Origin::Natural { .. },
        ) => Origin::Synthetic {
            lowered_from: lf1.clone(),
            stringified: "".into(),
        },
        (
            Origin::Synthetic {
                lowered_from: lf1, ..
            },
            Origin::Synthetic { .. },
        ) => Origin::Synthetic {
            lowered_from: lf1.clone(),
            stringified: "".into(),
        },
    }
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
            FineTokenData::Ident { represented_ident } => {
                Ok(CoarseTokenData::Ident { represented_ident })
            }
            FineTokenData::RawIdent { represented_ident } => {
                Ok(CoarseTokenData::RawIdent { represented_ident })
            }
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
                base,
                digits,
                suffix,
            }),
            FineTokenData::FloatLiteral { body, suffix } => {
                Ok(CoarseTokenData::FloatLiteral { body, suffix })
            }
        }
    }
}

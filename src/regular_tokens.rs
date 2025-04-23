//! Converts rustc and lex_via_peg tokenisations to a common form for comparison.
//!
//! These ['regularised' tokens][`RegularToken`] use coarse-grained punctuation.
//!
//! At present each regularised token tracks:
//!  - the span of source characters matched
//!  - the token's "kind" (see [`RegularTokenData`])
//!  - information about the [spacing][Spacing] to the following-token
//!  - the suffix for literal tokens
//!  - the 'kinds' of literal tokens (but not suffixed string-like ones)
//!  - how string-family literals would be "unescaped"
//!  - the (normalised) representation of identifiers
//!  - the 'name' of a lifetime/label
//!  - the contents of doc-comment tokens

use std::iter::once;

use crate::{
    char_sequences::Charseq,
    combination::{self, CoarseToken, CoarseTokenData},
    lex_via_rustc::{
        RustcCommentKind, RustcDocCommentStyle, RustcIdentIsRaw, RustcLiteralData,
        RustcStringStyle, RustcToken, RustcTokenData, RustcTokenSpacing,
    },
};

#[derive(PartialEq, Eq)]
pub struct RegularToken {
    pub extent: Charseq,
    pub spacing: Spacing,
    pub data: RegularTokenData,
}

impl std::fmt::Debug for RegularToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "extent: {:?} {:?}", self.extent, self.data)?;
        if matches!(self.spacing, Spacing::Alone) {
            write!(f, " |")?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub enum Spacing {
    /// This token is followed by whitespace, a (non-doc) comment, or end-of-input.
    Alone,
    /// There is no space between this token and the next.
    Joint,
}

/// A regularised token's kind and attributes.
///
/// We use Charseq rather than String here for the sake of its Debug representation.
#[derive(PartialEq, Eq, Debug)]
pub enum RegularTokenData {
    DocComment {
        comment_kind: CommentKind,
        style: DocCommentStyle,
        body: Charseq,
    },
    Punctuation,
    Identifier {
        represented_identifier: Charseq,
        style: IdentifierStyle,
    },
    LifetimeOrLabel {
        /// This includes the leading '
        symbol: Charseq,
        style: IdentifierStyle,
    },
    ByteLiteral {
        represented_byte: u8,
    },
    ByteStringLiteral {
        represented_bytes: Vec<u8>,
        style: StringStyle,
    },
    CharacterLiteral {
        represented_character: char,
    },
    StringLiteral {
        represented_string: Charseq,
        style: StringStyle,
    },
    CstringLiteral {
        represented_bytes: Vec<u8>,
        style: StringStyle,
    },
    IntegerLiteral {
        suffix: Charseq,
    },
    FloatLiteral {
        suffix: Charseq,
    },
    /// A string-like literal with nonempty suffix.
    ///
    /// We have to treat these separately because rustc isn't willing to unescape them. So we do
    /// without tracking their kind.
    LiteralWithForbiddenSuffix {
        suffix: Charseq,
    },
    Other,
}

/// Line or block comment
#[derive(PartialEq, Eq, Copy, Clone, std::fmt::Debug)]
pub enum CommentKind {
    Line,
    Block,
}

/// Whether a doc-comment is an inner or outer doc-comment.
///
/// Note that non-doc-comments have disappeared in this representation (they're treated as
/// whitespace).
#[derive(PartialEq, Eq, Copy, Clone, std::fmt::Debug)]
pub enum DocCommentStyle {
    Inner,
    Outer,
}

/// Whether an identifier was written in raw form.
#[derive(PartialEq, Eq, Copy, Clone, std::fmt::Debug)]
pub enum IdentifierStyle {
    NonRaw,
    Raw,
}

/// Whether a stringlike literal was written in raw form.
#[derive(PartialEq, Eq, Copy, Clone, std::fmt::Debug)]
pub enum StringStyle {
    NonRaw,
    Raw,
}

#[allow(unused)]
/// Converts a sequence of `RustcToken`s into a sequence of `RegularToken`s.
///
/// May panic if any of the tokens represent an error condition (this won't happen if the tokens
/// came from a lex_via_rustc::analyse() call which reported success).
pub fn regularise_from_rustc(tokens: impl IntoIterator<Item = RustcToken>) -> Vec<RegularToken> {
    tokens
        .into_iter()
        .map(|token| RegularToken {
            extent: token.extent.into(),
            spacing: token.spacing.into(),
            data: match token.data {
                RustcTokenData::DocComment {
                    comment_kind,
                    style,
                    body,
                } => RegularTokenData::DocComment {
                    comment_kind: comment_kind.into(),
                    style: (style).into(),
                    body: body.into(),
                },
                RustcTokenData::Punctuation => RegularTokenData::Punctuation,
                RustcTokenData::Ident { style, identifier } => RegularTokenData::Identifier {
                    represented_identifier: identifier.into(),
                    style: style.into(),
                },
                RustcTokenData::Lifetime {
                    style,
                    symbol: name,
                } => RegularTokenData::LifetimeOrLabel {
                    symbol: name.into(),
                    style: style.into(),
                },
                RustcTokenData::Lit { literal_data } => regularise_rustc_literal(literal_data)
                    .expect("rustc token represented an error"),
                RustcTokenData::Other => RegularTokenData::Other,
            },
        })
        .collect()
}

fn regularise_rustc_literal(literal_data: RustcLiteralData) -> Result<RegularTokenData, ()> {
    match literal_data {
        RustcLiteralData::Byte(byte) => Ok(RegularTokenData::ByteLiteral {
            represented_byte: byte,
        }),
        RustcLiteralData::Character(c) => Ok(RegularTokenData::CharacterLiteral {
            represented_character: c,
        }),
        RustcLiteralData::String(s, style) => Ok(RegularTokenData::StringLiteral {
            represented_string: s.into(),
            style: style.into(),
        }),
        RustcLiteralData::ByteString(bytes, style) => Ok(RegularTokenData::ByteStringLiteral {
            represented_bytes: bytes,
            style: style.into(),
        }),
        RustcLiteralData::CString(bytes, style) => Ok(RegularTokenData::CstringLiteral {
            represented_bytes: bytes,
            style: style.into(),
        }),
        RustcLiteralData::Integer(suffix) => Ok(RegularTokenData::IntegerLiteral {
            suffix: suffix.into(),
        }),
        RustcLiteralData::Float(suffix) => Ok(RegularTokenData::FloatLiteral {
            suffix: suffix.into(),
        }),
        RustcLiteralData::ForbiddenSuffix(suffix) => {
            Ok(RegularTokenData::LiteralWithForbiddenSuffix {
                suffix: suffix.into(),
            })
        }
        RustcLiteralData::Error => Err(()),
    }
}

impl From<RustcTokenSpacing> for Spacing {
    fn from(spacing: RustcTokenSpacing) -> Self {
        match spacing {
            RustcTokenSpacing::Alone => Spacing::Alone,
            RustcTokenSpacing::Joint => Spacing::Joint,
        }
    }
}

impl From<RustcCommentKind> for CommentKind {
    fn from(kind: RustcCommentKind) -> Self {
        match kind {
            RustcCommentKind::Line => CommentKind::Line,
            RustcCommentKind::Block => CommentKind::Block,
        }
    }
}

impl From<RustcDocCommentStyle> for DocCommentStyle {
    fn from(style: RustcDocCommentStyle) -> Self {
        match style {
            RustcDocCommentStyle::Inner => DocCommentStyle::Inner,
            RustcDocCommentStyle::Outer => DocCommentStyle::Outer,
        }
    }
}

impl From<RustcIdentIsRaw> for IdentifierStyle {
    fn from(style: RustcIdentIsRaw) -> Self {
        match style {
            RustcIdentIsRaw::No => Self::NonRaw,
            RustcIdentIsRaw::Yes => Self::Raw,
        }
    }
}

impl From<RustcStringStyle> for StringStyle {
    fn from(style: RustcStringStyle) -> Self {
        match style {
            RustcStringStyle::NonRaw => Self::NonRaw,
            RustcStringStyle::Raw => Self::Raw,
        }
    }
}

/// Converts a sequence of `CoarseToken`s into a sequence of `RegularToken`s.
pub fn regularise_from_coarse(tokens: impl IntoIterator<Item = CoarseToken>) -> Vec<RegularToken> {
    tokens
        .into_iter()
        .map(|ctoken| RegularToken {
            extent: ctoken.extent.clone(),
            spacing: ctoken.spacing.into(),
            data: from_coarse_token(ctoken),
        })
        .collect()
}

fn from_coarse_token(token: CoarseToken) -> RegularTokenData {
    match forbidden_literal_suffix(&token) {
        Some(suffix) if !suffix.is_empty() => {
            return RegularTokenData::LiteralWithForbiddenSuffix {
                suffix: suffix.clone(),
            };
        }
        _ => (),
    }
    match token.data {
        CoarseTokenData::LineComment { style, body } => RegularTokenData::DocComment {
            comment_kind: CommentKind::Line,
            style: style.into(),
            body,
        },
        CoarseTokenData::BlockComment { style, body } => RegularTokenData::DocComment {
            comment_kind: CommentKind::Block,
            style: style.into(),
            body,
        },
        CoarseTokenData::Punctuation { .. } => RegularTokenData::Punctuation,
        CoarseTokenData::Identifier {
            represented_identifier,
        } => RegularTokenData::Identifier {
            represented_identifier,
            style: IdentifierStyle::NonRaw,
        },
        CoarseTokenData::RawIdentifier {
            represented_identifier,
        } => RegularTokenData::Identifier {
            represented_identifier,
            style: IdentifierStyle::Raw,
        },
        CoarseTokenData::LifetimeOrLabel { name } => RegularTokenData::LifetimeOrLabel {
            symbol: once('\'').chain(name.iter().copied()).collect(),
            style: IdentifierStyle::NonRaw,
        },
        CoarseTokenData::RawLifetimeOrLabel { name } => RegularTokenData::LifetimeOrLabel {
            symbol: once('\'').chain(name.iter().copied()).collect(),
            style: IdentifierStyle::Raw,
        },
        CoarseTokenData::CharacterLiteral {
            represented_character,
            ..
        } => RegularTokenData::CharacterLiteral {
            represented_character,
        },
        CoarseTokenData::ByteLiteral {
            represented_byte, ..
        } => RegularTokenData::ByteLiteral { represented_byte },
        CoarseTokenData::StringLiteral {
            represented_string, ..
        } => RegularTokenData::StringLiteral {
            represented_string,
            style: StringStyle::NonRaw,
        },
        CoarseTokenData::ByteStringLiteral {
            represented_bytes, ..
        } => RegularTokenData::ByteStringLiteral {
            represented_bytes,
            style: StringStyle::NonRaw,
        },
        CoarseTokenData::CStringLiteral {
            mut represented_bytes,
            ..
        } => {
            represented_bytes.push(0);
            RegularTokenData::CstringLiteral {
                represented_bytes,
                style: StringStyle::NonRaw,
            }
        }
        CoarseTokenData::RawStringLiteral {
            represented_string, ..
        } => RegularTokenData::StringLiteral {
            represented_string,
            style: StringStyle::Raw,
        },
        CoarseTokenData::RawByteStringLiteral {
            represented_bytes, ..
        } => RegularTokenData::ByteStringLiteral {
            represented_bytes,
            style: StringStyle::Raw,
        },
        CoarseTokenData::RawCStringLiteral {
            mut represented_bytes,
            ..
        } => {
            represented_bytes.push(0);
            RegularTokenData::CstringLiteral {
                represented_bytes,
                style: StringStyle::Raw,
            }
        }
        CoarseTokenData::IntegerLiteral { suffix, .. } => {
            RegularTokenData::IntegerLiteral { suffix }
        }
        CoarseTokenData::FloatLiteral { suffix, .. } => RegularTokenData::FloatLiteral { suffix },
    }
}

/// Checks for suffixes on tokens of kinds which shouldn't have suffixes.
///
/// Returns None if the token isn't a string-family literal, or an empty string if is such a literal
/// but has no suffix.
fn forbidden_literal_suffix(token: &CoarseToken) -> Option<&Charseq> {
    match &token.data {
        CoarseTokenData::CharacterLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::ByteLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::StringLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::ByteStringLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::CStringLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::RawStringLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::RawByteStringLiteral { suffix, .. } => Some(suffix),
        CoarseTokenData::RawCStringLiteral { suffix, .. } => Some(suffix),
        _ => None,
    }
}

impl From<combination::Spacing> for Spacing {
    fn from(spacing: combination::Spacing) -> Self {
        match spacing {
            combination::Spacing::Alone => Spacing::Alone,
            combination::Spacing::Joint => Spacing::Joint,
        }
    }
}

impl From<combination::DocCommentStyle> for DocCommentStyle {
    fn from(style: combination::DocCommentStyle) -> Self {
        match style {
            combination::DocCommentStyle::Inner => DocCommentStyle::Inner,
            combination::DocCommentStyle::Outer => DocCommentStyle::Outer,
        }
    }
}

//! Convert doc-comments to attributes.

use std::iter;

use crate::Edition;
use crate::datatypes::char_sequences::Charseq;
use crate::tokens_common::Origin;

use super::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use super::tokenisation::lex_as_single_token;

const MAX_HASH_COUNT: usize = 255;

/// Convert doc-comments to attributes.
///
/// Each comment token in the input with style other than NonDoc is replaced by a sequence of
/// synthetic tokens, which together represent an attribute.
///
/// The sequence does't include any synthetic whitespace tokens (and so I think it doesn't provide
/// enough information to reproduce the Spacing that a proc macro would see).
///
/// The comment body is represented using a synthetic raw string literal token. The stringified form
/// recorded in that token uses the minimal number of hashes which would be required in the input to
/// create a raw string literal representing the comment body, or 255 if there is no input form of
/// raw string literal which represents the comment body (this matches the way rustc calculates the
/// hash_count field for this token).
pub fn lower_doc_comments(
    tokens: impl IntoIterator<Item = FineToken>,
    edition: Edition,
) -> Vec<FineToken> {
    let mut processed = Vec::new();
    for token in tokens {
        let lowered_from = match &token.origin {
            Origin::Natural { extent } => extent,
            Origin::Synthetic { lowered_from, .. } => lowered_from,
        };
        match token.data {
            FineTokenData::LineComment { style, body }
            | FineTokenData::BlockComment { style, body }
                if style == CommentStyle::InnerDoc || style == CommentStyle::OuterDoc =>
            {
                processed.extend(lowered(body, style, lowered_from, edition))
            }
            _ => processed.push(token),
        }
    }
    processed
}

fn lowered(
    comment_body: Charseq,
    style: CommentStyle,
    lowered_from: &Charseq,
    edition: Edition,
) -> Vec<FineToken> {
    let whitespace = || FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
            stringified: " ".into(),
        },
        data: FineTokenData::Whitespace,
    };
    let punct = |c: char| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
            stringified: c.into(),
        },
        data: FineTokenData::Punctuation { mark: c },
    };
    let ident = |name: &str| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
            // The name is always ascii so we don't have to worry about normalisation
            stringified: name.into(),
        },
        data: FineTokenData::Ident {
            represented_ident: name.into(),
        },
    };
    let rawstring = |represented_string, stringified| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
            stringified,
        },
        data: FineTokenData::RawStringLiteral {
            represented_string,
            suffix: Charseq::default(),
        },
    };

    let stringified = stringified_as_raw_literal(&comment_body, edition);

    let mut tokens = Vec::new();
    tokens.push(punct('#'));
    tokens.push(whitespace());
    if style != CommentStyle::OuterDoc {
        tokens.push(punct('!'));
    }
    tokens.push(punct('['));
    tokens.push(ident("doc"));
    tokens.push(punct('='));
    tokens.push(whitespace());
    tokens.push(rawstring(comment_body, stringified));
    tokens.push(punct(']'));
    tokens
}

fn stringified_as_raw_literal(represented_string: &Charseq, edition: Edition) -> Charseq {
    fn quote(represented_string: &Charseq, hash_count: usize) -> Charseq {
        iter::once('r')
            .chain(iter::repeat_n('#', hash_count))
            .chain(iter::once('"'))
            .chain(represented_string.iter())
            .chain(iter::once('"'))
            .chain(iter::repeat_n('#', hash_count))
            .collect()
    }
    for hash_count in 0..MAX_HASH_COUNT {
        let candidate = quote(represented_string, hash_count);
        if lex_as_single_token(candidate.chars(), edition).is_some() {
            return candidate;
        }
    }
    quote(represented_string, MAX_HASH_COUNT)
}

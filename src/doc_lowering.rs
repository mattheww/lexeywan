//! Convert doc-comments to attributes.

use crate::char_sequences::Charseq;
use crate::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::tokens_common::Origin;
use crate::Edition;

/// Convert doc-comments to attributes.
///
/// Each comment token in the input with style other than NonDoc is replaced by a sequence of
/// synthetic tokens, which together represent an attribute.
///
/// The sequence does't include any synthetic whitespace tokens (and so I think it doesn't provide
/// enough information to reproduce the Spacing that a proc macro would see).
pub fn lower_doc_comments(
    tokens: impl IntoIterator<Item = FineToken>,
    _edition: Edition,
) -> Vec<FineToken> {
    let mut processed = Vec::new();
    for token in tokens {
        let lowered_from = match &token.origin {
            Origin::Natural { extent } => extent,
            Origin::Synthetic { lowered_from } => lowered_from,
        };
        match token.data {
            FineTokenData::LineComment { style, body }
            | FineTokenData::BlockComment { style, body }
                if style != CommentStyle::NonDoc =>
            {
                processed.extend(lowered(body, style, lowered_from))
            }
            _ => processed.push(token),
        }
    }
    processed
}

fn lowered(comment_body: Charseq, style: CommentStyle, lowered_from: &Charseq) -> Vec<FineToken> {
    let punct = |c| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
        },
        data: FineTokenData::Punctuation { mark: c },
    };
    let ident = |name: &str| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
        },
        data: FineTokenData::Identifier {
            represented_identifier: name.into(),
        },
    };
    let rawstring = |represented_string| FineToken {
        origin: Origin::Synthetic {
            lowered_from: lowered_from.clone(),
        },
        data: FineTokenData::RawStringLiteral {
            represented_string,
            suffix: Charseq::default(),
        },
    };

    let mut tokens = Vec::new();
    tokens.push(punct('#'));
    if style == CommentStyle::InnerDoc {
        tokens.push(punct('!'));
    }
    tokens.push(punct('['));
    tokens.push(ident("doc"));
    tokens.push(punct('='));
    tokens.push(rawstring(comment_body));
    tokens.push(punct(']'));
    tokens
}

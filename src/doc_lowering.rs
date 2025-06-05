//! Convert doc-comments to attributes.

use crate::{
    char_sequences::Charseq,
    fine_tokens::{CommentStyle, FineToken, FineTokenData},
};

/// Convert doc-comments to attributes.
///
/// Each comment token in the input with style other than NonDoc is replaced by a sequence of
/// synthetic tokens, which together represent an attribute.
///
/// The sequence does't include any synthetic whitespace tokens (and so I think it doesn't provide
/// enough information to reproduce the Spacing that a proc macro would see).
///
/// The extent of each synthetic token is the extent of the entire doc-comment it's derived from.
/// That means the returned sequence may not have the property that concatenating the extents
/// reproduces the original input.
pub fn lower_doc_comments(tokens: impl IntoIterator<Item = FineToken>) -> Vec<FineToken> {
    let mut processed = Vec::new();
    for token in tokens {
        match token.data {
            FineTokenData::LineComment { style, body }
            | FineTokenData::BlockComment { style, body }
                if style != CommentStyle::NonDoc =>
            {
                processed.extend(lowered(body, style, &token.extent))
            }
            _ => processed.push(token),
        }
    }
    processed
}

fn lowered(comment_body: Charseq, style: CommentStyle, full_extent: &Charseq) -> Vec<FineToken> {
    let punct = |c| FineToken {
        extent: full_extent.clone(),
        data: FineTokenData::Punctuation { mark: c },
    };
    let ident = |name: &str| FineToken {
        extent: full_extent.clone(),
        data: FineTokenData::Identifier {
            represented_identifier: name.into(),
        },
    };
    let rawstring = |represented_string| FineToken {
        extent: full_extent.clone(),
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

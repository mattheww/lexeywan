//! Work with rustc TokenStream and TokenTree.

extern crate rustc_ast;

use rustc_ast::{
    token::{Delimiter, Token, TokenKind},
    tokenstream::{DelimSpacing, DelimSpan, Spacing, TokenStream, TokenTree},
};

/// Turns a sequence of ruct_ast `Token`s into a `TokenStream`.
///
/// All the tokens and delimiters in the result have spacing `Alone`.
///
/// Reports an error if the sequence doesn't have well-balanced delimiters.
///
/// In practice this is used with sequences that are known to be well-balanced, so we don't bother
/// with detail in error reports.
pub fn make_token_stream(
    mut tokens: impl Iterator<Item = Token>,
) -> Result<TokenStream, &'static str> {
    let (stream, closing_token) = make_token_stream_inner(&mut tokens)?;
    match closing_token {
        Some(_) => Err("extra closing delimiter"),
        None => Ok(stream),
    }
}

fn make_token_stream_inner(
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<(TokenStream, Option<Token>), &'static str> {
    let mut trees = Vec::new();
    while let Some(token) = tokens.next() {
        trees.push(match token.kind {
            TokenKind::OpenParen => make_subtree(token, Delimiter::Parenthesis, tokens)?,
            TokenKind::OpenBrace => make_subtree(token, Delimiter::Brace, tokens)?,
            TokenKind::OpenBracket => make_subtree(token, Delimiter::Bracket, tokens)?,
            TokenKind::OpenInvisible(origin) => {
                make_subtree(token, Delimiter::Invisible(origin), tokens)?
            }
            TokenKind::CloseParen
            | TokenKind::CloseBrace
            | TokenKind::CloseBracket
            | TokenKind::CloseInvisible(_) => return Ok((TokenStream::new(trees), Some(token))),
            _ => TokenTree::Token(token, Spacing::Alone),
        });
    }
    Ok((TokenStream::new(trees), None))
}

fn make_subtree(
    token: Token,
    delimiter: Delimiter,
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<TokenTree, &'static str> {
    let (stream, Some(close_token)) = make_token_stream_inner(tokens)? else {
        return Err("missing close delimiter");
    };
    if close_token.kind != delimiter.as_close_token_kind() {
        return Err("wrong close delimiter");
    }
    Ok(TokenTree::Delimited(
        DelimSpan::from_pair(token.span, close_token.span),
        DelimSpacing::new(Spacing::Alone, Spacing::Alone),
        delimiter,
        stream,
    ))
}

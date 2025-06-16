//! Runs rustc's lexical analysis.
//!
//! This works by running the low-level and high-level lexers as far as making a `TokenStream`.
//! If rustc emits any error messages (or panics), we treat the input as rejected.
//!
//! Stringlike literal tokens are further run through `ast::LitKind::from_token_lit()`, to obtain the
//! "unescaped" value.
//!
//! The input string is fed through `SourceMap::new_source_file()`, which means that "normalisation"
//! (BOM-removal and CRLF-conversion) happen. Later shebang removal happens too. See the
//! [`cleaning`][`crate::cleaning`] module for how we make equivalent input for comparison.

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

// This compiles with
// rustc 1.88.0-nightly (10fa3c449 2025-04-26)

use std::sync::Arc;

use rustc_ast::{
    token::{Token, TokenKind},
    tokenstream::{TokenStream, TokenTree},
};
use rustc_span::{
    source_map::{FilePathMapping, SourceMap},
    FileName,
};

use crate::trees::{self, Forest, Tree};
use crate::{Edition, Lowering};

use self::error_accumulator::ErrorAccumulator;

mod error_accumulator;

/// Information we keep about a token from the rustc tokeniser.
///
/// Synthetic tokens aren't distinguished here, because I don't see a robust way to detect them.
pub struct RustcToken {
    /// The input characters which make up the token
    pub extent: String,
    /// The token kind, and any data we've extracted specific to this kind of token
    pub data: RustcTokenData,
    /// Human-readable description of the token
    pub summary: String,
}

impl std::fmt::Debug for RustcToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary)
    }
}

/// A rustc token's kind and attributes
pub enum RustcTokenData {
    DocComment {
        comment_kind: RustcCommentKind,
        style: RustcDocCommentStyle,
        body: String,
    },
    Punctuation {
        marks: &'static str,
    },
    Ident {
        style: RustcIdentIsRaw,
        identifier: String,
    },
    Lifetime {
        style: RustcIdentIsRaw,
        /// This includes the leading '
        symbol: String,
    },
    Lit {
        literal_data: RustcLiteralData,
    },
    Other,
}

/// A literal token's kind and attributes.
pub enum RustcLiteralData {
    /// Character literal with the "unescaped" character
    Character(char),

    /// Byte literal with the "unescaped" byte
    Byte(u8),

    /// String literal with the "unescaped" string
    String(String, RustcStringStyle),

    /// Byte-string literal with the "unescaped" bytes
    ByteString(Vec<u8>, RustcStringStyle),

    /// C-string literal with the "unescaped" bytes
    CString(Vec<u8>, RustcStringStyle),

    /// Integer literal with its suffix (which may be a suffix indicating float type)
    Integer(String),

    /// Float literal with its suffix
    Float(String),

    /// String-like literal with a suffix
    ForbiddenSuffix(String),

    /// A token that represented an ill-formed literal.
    ///
    /// This shouldn't appear unless analyse() reported an error.
    Error,
}

/// Line or block comment
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum RustcCommentKind {
    Line,
    Block,
}

/// Whether a doc-comment is an inner or outer doc-comment.
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum RustcDocCommentStyle {
    Inner,
    Outer,
}

/// Whether an identifier or lifetime/label was written in raw form.
pub enum RustcIdentIsRaw {
    No,
    Yes,
}

/// Whether a stringlike literal was written in raw form.
pub enum RustcStringStyle {
    NonRaw,
    Raw,
}

/// Runs rustc's lexical analysis on the specified input.
///
/// If the input is accepted, returns a [`Forest`] of tokens, in [`RustcToken`] form.
/// Otherwise returns at least one error message.
///
/// If rustc panics (ie, it would report an ICE), the panic message is sent to
/// standard error and this function returns CompilerError.
pub fn analyse(input: &str, edition: Edition, lowering: Lowering) -> Analysis {
    let error_list = ErrorAccumulator::new();

    let rustc_edition = match edition {
        Edition::E2015 => rustc_span::edition::Edition::Edition2015,
        Edition::E2021 => rustc_span::edition::Edition::Edition2021,
        Edition::E2024 => rustc_span::edition::Edition::Edition2024,
    };

    std::panic::catch_unwind(|| {
        match rustc_driver::catch_fatal_errors(|| {
            rustc_span::create_session_globals_then(rustc_edition, &[], None, || {
                run_lexer(input, lowering, error_list.clone())
            })
        }) {
            Ok(rustc_forest) => {
                let messages = error_list.extract();
                if messages.is_empty() {
                    // Lexing succeeded
                    Analysis::Accepts(rustc_forest)
                } else {
                    // Lexing reported a non-fatal error
                    Analysis::Rejects(rustc_forest, messages)
                }
            }
            Err(_) => {
                let mut messages = error_list.extract();
                messages.push("reported fatal error (panicked)".into());
                Analysis::Rejects(Forest::new(), messages)
            }
        }
    })
    .unwrap_or(Analysis::CompilerError)
}

/// Result of running lexical analysis on a string.
pub enum Analysis {
    /// Lexical analysis accepted the input.
    Accepts(Forest<RustcToken>),
    /// Lexical analysis rejected the input.
    ///
    /// The forest of tokens is what rustc would have passed on to the parser.
    /// Empty if there was a fatal error, or if there are unbalanced delimiters.
    ///
    /// The strings are error messages. There's always at least one message.
    Rejects(Forest<RustcToken>, Vec<String>),
    /// The input provoked an internal compiler error.
    CompilerError,
}

/// Runs rustc's lexical analysis on the specified input.
///
/// Panics if rustc would have reported a fatal error.
///
/// Otherwise:
///  - returns the lexed tokens, in RustcToken form
///  - doc-comments are desugared if requested by the 'lowering' parameter
///  - if rustc would have reported a non-fatal error, at least one message has
///    been added to error_list
///    - in this case, the returned tokens are what would have been passed on to
///      the parser (an empty list if token stream construction failed).
fn run_lexer(input: &str, lowering: Lowering, error_list: ErrorAccumulator) -> Forest<RustcToken> {
    let psess = make_parser_session(error_list.clone());
    let source_map = psess.source_map();
    let input = String::from(input);
    let filename = FileName::Custom("lex_via_rustc".into());
    let lexed = match rustc_parse::source_str_to_stream(&psess, filename, input, None) {
        Ok(mut token_stream) => {
            if lowering == Lowering::LowerDocComments {
                token_stream.desugar_doc_comments();
            }
            map_forest(&token_stream, source_map)
        }
        Err(diags) => {
            // Errors constructing the token stream are reported here
            // (ie, unbalanced delimiters).
            assert!(!diags.is_empty());
            for diag in diags {
                diag.emit();
            }
            Forest::<RustcToken>::new()
        }
    };
    // The lexer doesn't report errors itself when it sees emoji in 'identifiers'. Instead it leaves
    // a note in the ParseSess to be examined later. So we have to make this extra check.
    if !&psess.bad_unicode_identifiers.borrow_mut().is_empty() {
        error_list.push("bad unicode identifier(s)".into());
    }
    lexed
}

fn make_parser_session(error_list: ErrorAccumulator) -> rustc_session::parse::ParseSess {
    #[allow(clippy::arc_with_non_send_sync)]
    let sm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let dcx = error_list.into_diag_ctxt().disable_warnings();
    rustc_session::parse::ParseSess::with_dcx(dcx, sm)
}

/// Converts a rustc_ast `TokenStream` to our `TokenForest<RustcToken>`
fn map_forest(token_stream: &TokenStream, source_map: &SourceMap) -> Forest<RustcToken> {
    token_stream
        .iter()
        .map(|token_tree| match token_tree {
            TokenTree::Token(token, _) => {
                Tree::<RustcToken>::Token(token_from_ast_token(token, source_map))
            }
            &TokenTree::Delimited(delim_span, _, delimiter, ref token_stream) => {
                if let Ok(group_kind) = delimiter.try_into() {
                    Tree::<RustcToken>::Group(group_kind, map_forest(token_stream, source_map))
                } else {
                    // Shouldn't happen (invisible delimiter)
                    Tree::<RustcToken>::Token(RustcToken {
                        extent: source_map.span_to_snippet(delim_span.open).unwrap(),
                        data: RustcTokenData::Other,
                        summary: "((invisible group))".into(),
                    })
                }
            }
        })
        .collect()
}

fn token_from_ast_token(token: &Token, source_map: &SourceMap) -> RustcToken {
    let data = match token.kind {
        TokenKind::DocComment(comment_kind, style, symbol) => RustcTokenData::DocComment {
            comment_kind: comment_kind.into(),
            style: style.into(),
            body: symbol.to_string(),
        },
        TokenKind::Eq => RustcTokenData::Punctuation { marks: "=" },
        TokenKind::Lt => RustcTokenData::Punctuation { marks: "<" },
        TokenKind::Le => RustcTokenData::Punctuation { marks: "<=" },
        TokenKind::EqEq => RustcTokenData::Punctuation { marks: "==" },
        TokenKind::Ne => RustcTokenData::Punctuation { marks: "!=" },
        TokenKind::Ge => RustcTokenData::Punctuation { marks: ">=" },
        TokenKind::Gt => RustcTokenData::Punctuation { marks: ">" },
        TokenKind::AndAnd => RustcTokenData::Punctuation { marks: "&&" },
        TokenKind::OrOr => RustcTokenData::Punctuation { marks: "||" },
        TokenKind::Bang => RustcTokenData::Punctuation { marks: "!" },
        TokenKind::Tilde => RustcTokenData::Punctuation { marks: "~" },
        TokenKind::Plus => RustcTokenData::Punctuation { marks: "+" },
        TokenKind::Minus => RustcTokenData::Punctuation { marks: "-" },
        TokenKind::Star => RustcTokenData::Punctuation { marks: "*" },
        TokenKind::Slash => RustcTokenData::Punctuation { marks: "/" },
        TokenKind::Percent => RustcTokenData::Punctuation { marks: "%" },
        TokenKind::Caret => RustcTokenData::Punctuation { marks: "^" },
        TokenKind::And => RustcTokenData::Punctuation { marks: "&" },
        TokenKind::Or => RustcTokenData::Punctuation { marks: "|" },
        TokenKind::Shl => RustcTokenData::Punctuation { marks: "<<" },
        TokenKind::Shr => RustcTokenData::Punctuation { marks: ">>" },
        TokenKind::PlusEq => RustcTokenData::Punctuation { marks: "+=" },
        TokenKind::MinusEq => RustcTokenData::Punctuation { marks: "-=" },
        TokenKind::StarEq => RustcTokenData::Punctuation { marks: "*=" },
        TokenKind::SlashEq => RustcTokenData::Punctuation { marks: "/=" },
        TokenKind::PercentEq => RustcTokenData::Punctuation { marks: "%=" },
        TokenKind::CaretEq => RustcTokenData::Punctuation { marks: "^=" },
        TokenKind::AndEq => RustcTokenData::Punctuation { marks: "&=" },
        TokenKind::OrEq => RustcTokenData::Punctuation { marks: "|=" },
        TokenKind::ShlEq => RustcTokenData::Punctuation { marks: "<<=" },
        TokenKind::ShrEq => RustcTokenData::Punctuation { marks: ">>=" },
        TokenKind::At => RustcTokenData::Punctuation { marks: "@" },
        TokenKind::Dot => RustcTokenData::Punctuation { marks: "." },
        TokenKind::DotDot => RustcTokenData::Punctuation { marks: ".." },
        TokenKind::DotDotDot => RustcTokenData::Punctuation { marks: "..." },
        TokenKind::DotDotEq => RustcTokenData::Punctuation { marks: "..=" },
        TokenKind::Comma => RustcTokenData::Punctuation { marks: "," },
        TokenKind::Semi => RustcTokenData::Punctuation { marks: ";" },
        TokenKind::Colon => RustcTokenData::Punctuation { marks: ":" },
        TokenKind::PathSep => RustcTokenData::Punctuation { marks: "::" },
        TokenKind::RArrow => RustcTokenData::Punctuation { marks: "->" },
        TokenKind::LArrow => RustcTokenData::Punctuation { marks: "<-" },
        TokenKind::FatArrow => RustcTokenData::Punctuation { marks: "=>" },
        TokenKind::Pound => RustcTokenData::Punctuation { marks: "#" },
        TokenKind::Dollar => RustcTokenData::Punctuation { marks: "$" },
        TokenKind::Question => RustcTokenData::Punctuation { marks: "?" },
        TokenKind::SingleQuote => RustcTokenData::Punctuation { marks: "'" },
        TokenKind::OpenParen => RustcTokenData::Punctuation { marks: "(" },
        TokenKind::CloseParen => RustcTokenData::Punctuation { marks: ")" },
        TokenKind::OpenBrace => RustcTokenData::Punctuation { marks: "{" },
        TokenKind::CloseBrace => RustcTokenData::Punctuation { marks: "}" },
        TokenKind::OpenBracket => RustcTokenData::Punctuation { marks: "[" },
        TokenKind::CloseBracket => RustcTokenData::Punctuation { marks: "]" },
        TokenKind::Ident(symbol, style) => RustcTokenData::Ident {
            style: style.into(),
            identifier: symbol.to_string(),
        },
        TokenKind::Lifetime(symbol, style) => RustcTokenData::Lifetime {
            style: style.into(),
            symbol: symbol.to_string(),
        },
        TokenKind::Literal(rustc_ast::token::Lit {
            kind: rustc_ast::token::LitKind::Integer,
            suffix,
            ..
        }) => RustcTokenData::Lit {
            literal_data: RustcLiteralData::Integer(
                suffix.map(|s| s.to_string()).unwrap_or_else(String::new),
            ),
        },
        TokenKind::Literal(rustc_ast::token::Lit {
            kind: rustc_ast::token::LitKind::Float,
            suffix,
            ..
        }) => RustcTokenData::Lit {
            literal_data: RustcLiteralData::Float(
                suffix.map(|s| s.to_string()).unwrap_or_else(String::new),
            ),
        },
        TokenKind::Literal(lit) => {
            match lit.suffix {
                // from_token_lit() is what performs unescaping, but it will panic if it sees a
                // suffix
                None => {
                    let ast_lit = rustc_ast::ast::LitKind::from_token_lit(lit)
                        .expect("from_token_lit failed");
                    RustcTokenData::Lit {
                        literal_data: literal_data_from_ast_litkind(ast_lit),
                    }
                }
                Some(suffix) => RustcTokenData::Lit {
                    literal_data: RustcLiteralData::ForbiddenSuffix(suffix.to_string()),
                },
            }
        }
        // These shouldn't happen
        TokenKind::NtIdent(_, _) => RustcTokenData::Other,
        TokenKind::NtLifetime(_, _) => RustcTokenData::Other,
        TokenKind::Eof => RustcTokenData::Other,
        TokenKind::OpenInvisible(_) => RustcTokenData::Other,
        TokenKind::CloseInvisible(_) => RustcTokenData::Other,
    };
    RustcToken {
        extent: source_map.span_to_snippet(token.span).unwrap(),
        data,
        summary: format!("{:?}", token.kind.clone()),
    }
}

fn literal_data_from_ast_litkind(ast_lit: rustc_ast::ast::LitKind) -> RustcLiteralData {
    match ast_lit {
        rustc_ast::LitKind::Str(symbol, style) => {
            RustcLiteralData::String(symbol.to_string(), style.into())
        }
        rustc_ast::LitKind::ByteStr(bytes, style) => {
            RustcLiteralData::ByteString(bytes.as_ref().into(), style.into())
        }
        rustc_ast::LitKind::CStr(bytes, style) => {
            RustcLiteralData::CString(bytes.as_ref().into(), style.into())
        }
        rustc_ast::LitKind::Byte(byte) => RustcLiteralData::Byte(byte),
        rustc_ast::LitKind::Char(c) => RustcLiteralData::Character(c),
        _ => RustcLiteralData::Error,
    }
}

impl From<rustc_ast::token::IdentIsRaw> for RustcIdentIsRaw {
    fn from(value: rustc_ast::token::IdentIsRaw) -> Self {
        match value {
            rustc_ast::token::IdentIsRaw::No => RustcIdentIsRaw::No,
            rustc_ast::token::IdentIsRaw::Yes => RustcIdentIsRaw::Yes,
        }
    }
}

impl From<rustc_ast::token::CommentKind> for RustcCommentKind {
    fn from(value: rustc_ast::token::CommentKind) -> Self {
        match value {
            rustc_ast::token::CommentKind::Line => Self::Line,
            rustc_ast::token::CommentKind::Block => Self::Block,
        }
    }
}

impl From<rustc_ast::AttrStyle> for RustcDocCommentStyle {
    fn from(value: rustc_ast::AttrStyle) -> Self {
        match value {
            rustc_ast::AttrStyle::Outer => Self::Outer,
            rustc_ast::AttrStyle::Inner => Self::Inner,
        }
    }
}

impl From<rustc_ast::StrStyle> for RustcStringStyle {
    fn from(str_style: rustc_ast::StrStyle) -> Self {
        match str_style {
            rustc_ast::StrStyle::Cooked => Self::NonRaw,
            rustc_ast::StrStyle::Raw(_) => Self::Raw,
        }
    }
}

impl TryFrom<rustc_ast::token::Delimiter> for trees::GroupKind {
    type Error = ();

    fn try_from(value: rustc_ast::token::Delimiter) -> Result<Self, Self::Error> {
        match value {
            rustc_ast::token::Delimiter::Parenthesis => Ok(trees::GroupKind::Parenthesised),
            rustc_ast::token::Delimiter::Brace => Ok(trees::GroupKind::Braced),
            rustc_ast::token::Delimiter::Bracket => Ok(trees::GroupKind::Bracketed),
            rustc_ast::token::Delimiter::Invisible(_) => Err(()),
        }
    }
}

//! Runs rustc's lexical analysis.
//!
//! This works by running the low-level and high-level lexers as far as making a `TokenStream`, then
//! flattening the `TokenTree`s it contains back into a sequence of tokens in a similar way to
//! rustc's parser.
//! If rustc emits any error messages (or panics), we treat the input as rejected.
//!
//! Stringlike literal tokens are further run through ast::LitKind::from_token_lit(), to obtain the
//! "unescaped" value.
//!
//! The input string is fed through `SourceMap::new_source_file()`, which means that "normalisation"
//! (BOM-removal and CRLF-conversion) happen. Later shebang removal happens too. See the
//! [`cleaning`][`crate::cleaning`] module for how we make equivalent input for comparison.
//!
//! A limitation of this approach is that, because it constructs token trees, input with imbalanced
//! delimiters is rejected.

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

// This compiles with
// rustc 1.87.0-nightly (f8a913b13 2025-02-23)

use std::{
    mem,
    sync::{Arc, Mutex},
};

use rustc_ast::{
    token::{Token, TokenKind},
    tokenstream::{TokenStream, TokenTree},
};
use rustc_errors::{registry::Registry, DiagCtxt, LazyFallbackBundle};
use rustc_span::{
    source_map::{FilePathMapping, SourceMap},
    FileName,
};

use crate::Edition;

/// Information we keep about a token from the rustc tokeniser.
pub struct RustcToken {
    /// The input characters which make up the token
    pub extent: String,
    /// Spacing between this token and the next one
    pub spacing: RustcTokenSpacing,
    /// The token kind, and any data we've extracted specific to this kind of token
    pub data: RustcTokenData,
    /// Human-readable description of the token
    pub summary: String,
}

#[derive(Copy, Clone)]
pub enum RustcTokenSpacing {
    /// This token is followed by whitespace, a (non-doc) comment, or end-of-input.
    Alone,
    /// There is no space between this token and the next.
    Joint,
}

/// A rustc token's kind and attributes
pub enum RustcTokenData {
    DocComment {
        comment_kind: RustcCommentKind,
        style: RustcDocCommentStyle,
        body: String,
    },
    Punctuation,
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
/// If the input is accepted, returns a list of tokens, in [`RustcToken`] form.
/// Otherwise returns at least one error message.
///
/// If rustc panics (ie, it would report an ICE), the panic message is sent to
/// standard error and this function returns CompilerError.
pub fn analyse(input: &str, edition: Edition) -> Analysis {
    let error_list = Arc::new(Mutex::new(Vec::new()));
    fn extract_errors(error_list: ErrorAccumulator) -> Vec<String> {
        mem::take(&mut error_list.lock().unwrap())
    }

    let rustc_edition = match edition {
        Edition::E2015 => rustc_span::edition::Edition::Edition2015,
        Edition::E2021 => rustc_span::edition::Edition::Edition2021,
        Edition::E2024 => rustc_span::edition::Edition::Edition2024,
    };

    std::panic::catch_unwind(|| {
        match rustc_driver::catch_fatal_errors(|| {
            rustc_span::create_session_globals_then(rustc_edition, None, || {
                run_lexer(input, error_list.clone())
            })
        }) {
            Ok(rustc_tokens) => {
                let messages = extract_errors(error_list);
                if messages.is_empty() {
                    // Lexing succeeded
                    Analysis::Accepts(rustc_tokens)
                } else {
                    // Lexing reported a non-fatal error
                    Analysis::Rejects(rustc_tokens, messages)
                }
            }
            Err(_) => {
                let mut messages = extract_errors(error_list);
                messages.push("reported fatal error (panicked)".into());
                Analysis::Rejects(Vec::new(), messages)
            }
        }
    })
    .unwrap_or(Analysis::CompilerError)
}

/// Result of running lexical analysis on a string.
pub enum Analysis {
    /// Lexical analysis accepted the input.
    Accepts(Vec<RustcToken>),
    /// Lexical analysis rejected the input.
    ///
    /// The tokens are what rustc would have passed on to the parser.
    /// Empty if there was a fatal error, or if there are unbalanced delimiters.
    ///
    /// The strings are error messages. There's always at least one message.
    Rejects(Vec<RustcToken>, Vec<String>),
    /// The input provoked an internal compiler error.
    CompilerError,
}

/// Runs rustc's lexical analysis on the specified input.
///
/// Panics if rustc would have reported a fatal error.
///
/// Otherwise:
///  - returns the lexed tokens, in RustcToken form
///  - if rustc would have reported a non-fatal error, at least one message has
///    been added to error_list
///    - in this case, the returned tokens are what would have been passed on to
///      the parser (an empty list if token stream construction failed).
fn run_lexer(input: &str, error_list: ErrorAccumulator) -> Vec<RustcToken> {
    let psess = make_parser_session(error_list.clone());
    let source_map = psess.source_map();
    let input = String::from(input);
    let filename = FileName::Custom("lex_via_rustc".into());
    let lexed = match rustc_parse::source_str_to_stream(&psess, filename, input, None) {
        Ok(token_stream) => TokenStreamProcessor::process(&token_stream, &source_map),
        Err(diags) => {
            // Errors constructing the token stream are reported here
            // (ie, unbalanced delimiters).
            assert!(!diags.is_empty());
            for diag in diags {
                diag.emit();
            }
            Vec::new()
        }
    };
    // The lexer doesn't report errors itself when it sees emoji in 'identifiers'. Instead it leaves
    // a note in the ParseSess to be examined later. So we have to make this extra check.
    if !&psess.bad_unicode_identifiers.borrow_mut().is_empty() {
        psess.dcx().err("bad unicode identifier(s)");
    }
    lexed
}

type ErrorAccumulator = Arc<Mutex<Vec<String>>>;

struct ErrorEmitter {
    pub fallback_bundle: LazyFallbackBundle,
    seen: ErrorAccumulator,
}

impl ErrorEmitter {
    fn new(error_list: ErrorAccumulator) -> Self {
        let fallback_bundle = rustc_errors::fallback_fluent_bundle(
            rustc_driver::DEFAULT_LOCALE_RESOURCES.to_vec(),
            false,
        );
        ErrorEmitter {
            fallback_bundle,
            seen: error_list,
        }
    }
}

impl rustc_errors::translation::Translate for ErrorEmitter {
    fn fluent_bundle(&self) -> Option<&rustc_errors::FluentBundle> {
        None
    }

    fn fallback_fluent_bundle(&self) -> &rustc_errors::FluentBundle {
        &self.fallback_bundle
    }
}

impl rustc_errors::emitter::Emitter for ErrorEmitter {
    fn source_map(&self) -> Option<&SourceMap> {
        None
    }

    fn emit_diagnostic(&mut self, diag: rustc_errors::DiagInner, _: &Registry) {
        use rustc_error_messages::DiagMessage;
        if !diag.is_error() {
            return;
        }
        let mut seen = self.seen.lock().unwrap();
        if let Some(code) = diag.code {
            seen.push(format!("code: {}", code));
        } else if diag.messages.is_empty() {
            // I don't think this happens, but in case it does we store a
            // message so the caller knows to report failure.
            seen.push("error with no message".into());
        }
        for (msg, _style) in &diag.messages {
            let s = match msg {
                DiagMessage::Str(msg) => msg.to_string(),
                DiagMessage::Translated(msg) => msg.to_string(),
                DiagMessage::FluentIdentifier(fluent_id, _) => fluent_id.to_string(),
            };
            seen.push(s);
        }
    }
}

fn make_parser_session(error_list: ErrorAccumulator) -> rustc_session::parse::ParseSess {
    let emitter = ErrorEmitter::new(error_list);
    #[allow(clippy::arc_with_non_send_sync)]
    let sm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let emitter = Box::new(emitter);
    let dcx = DiagCtxt::new(emitter).disable_warnings();
    rustc_session::parse::ParseSess::with_dcx(dcx, sm)
}

/// Converts a rustc_ast `TokenStream` to a flat sequence of `RustcToken`s.
struct TokenStreamProcessor<'a> {
    source_map: &'a SourceMap,
    output: Vec<RustcToken>,
}

impl<'a> TokenStreamProcessor<'a> {
    fn process(token_stream: &TokenStream, source_map: &'a SourceMap) -> Vec<RustcToken> {
        let mut flattener = Self {
            source_map,
            output: Vec::new(),
        };
        flattener.add_tokens_from_stream(token_stream);
        flattener.output
    }

    fn add_tokens_from_stream(&mut self, token_stream: &TokenStream) {
        for token_tree in token_stream.iter() {
            self.add_tokens_from_tree(token_tree);
        }
    }

    fn add_tokens_from_tree(&mut self, token_tree: &TokenTree) {
        match token_tree {
            &TokenTree::Token(ref token, spacing) => {
                self.output
                    .push(token_from_ast_token(token, spacing, self.source_map))
            }
            &TokenTree::Delimited(delim_span, delim_spacing, delimiter, ref token_stream) => {
                self.output.push(token_from_ast_token(
                    &Token::new(TokenKind::OpenDelim(delimiter), delim_span.open),
                    delim_spacing.open,
                    self.source_map,
                ));
                self.add_tokens_from_stream(token_stream);
                self.output.push(token_from_ast_token(
                    &Token::new(TokenKind::CloseDelim(delimiter), delim_span.close),
                    delim_spacing.close,
                    self.source_map,
                ));
            }
        }
    }
}

fn token_from_ast_token(
    token: &Token,
    spacing: rustc_ast::tokenstream::Spacing,
    source_map: &SourceMap,
) -> RustcToken {
    let data = match token.kind {
        TokenKind::DocComment(comment_kind, style, symbol) => RustcTokenData::DocComment {
            comment_kind: comment_kind.into(),
            style: style.into(),
            body: symbol.to_string(),
        },
        TokenKind::Eq => RustcTokenData::Punctuation,
        TokenKind::Lt => RustcTokenData::Punctuation,
        TokenKind::Le => RustcTokenData::Punctuation,
        TokenKind::EqEq => RustcTokenData::Punctuation,
        TokenKind::Ne => RustcTokenData::Punctuation,
        TokenKind::Ge => RustcTokenData::Punctuation,
        TokenKind::Gt => RustcTokenData::Punctuation,
        TokenKind::AndAnd => RustcTokenData::Punctuation,
        TokenKind::OrOr => RustcTokenData::Punctuation,
        TokenKind::Not => RustcTokenData::Punctuation,
        TokenKind::Tilde => RustcTokenData::Punctuation,
        TokenKind::BinOp(_) => RustcTokenData::Punctuation,
        TokenKind::BinOpEq(_) => RustcTokenData::Punctuation,
        TokenKind::At => RustcTokenData::Punctuation,
        TokenKind::Dot => RustcTokenData::Punctuation,
        TokenKind::DotDot => RustcTokenData::Punctuation,
        TokenKind::DotDotDot => RustcTokenData::Punctuation,
        TokenKind::DotDotEq => RustcTokenData::Punctuation,
        TokenKind::Comma => RustcTokenData::Punctuation,
        TokenKind::Semi => RustcTokenData::Punctuation,
        TokenKind::Colon => RustcTokenData::Punctuation,
        TokenKind::PathSep => RustcTokenData::Punctuation,
        TokenKind::RArrow => RustcTokenData::Punctuation,
        TokenKind::LArrow => RustcTokenData::Punctuation,
        TokenKind::FatArrow => RustcTokenData::Punctuation,
        TokenKind::Pound => RustcTokenData::Punctuation,
        TokenKind::Dollar => RustcTokenData::Punctuation,
        TokenKind::Question => RustcTokenData::Punctuation,
        TokenKind::SingleQuote => RustcTokenData::Punctuation,
        TokenKind::OpenDelim(_) => RustcTokenData::Punctuation,
        TokenKind::CloseDelim(_) => RustcTokenData::Punctuation,
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
        TokenKind::Interpolated(_) => RustcTokenData::Other,
        TokenKind::NtIdent(_, _) => RustcTokenData::Other,
        TokenKind::NtLifetime(_, _) => RustcTokenData::Other,
        TokenKind::Eof => RustcTokenData::Other,
    };
    RustcToken {
        extent: source_map.span_to_snippet(token.span).unwrap(),
        spacing: spacing.into(),
        data,
        summary: format!("{:} {:?}", format_spacing(&spacing), token.kind.clone()),
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

fn format_spacing(spacing: &rustc_ast::tokenstream::Spacing) -> &str {
    match spacing {
        rustc_ast::tokenstream::Spacing::Alone => "alone",
        rustc_ast::tokenstream::Spacing::Joint => "joint",
        rustc_ast::tokenstream::Spacing::JointHidden => "-----",
    }
}

impl From<rustc_ast::tokenstream::Spacing> for RustcTokenSpacing {
    fn from(spacing: rustc_ast::tokenstream::Spacing) -> Self {
        match spacing {
            rustc_ast::tokenstream::Spacing::Alone => RustcTokenSpacing::Alone,
            rustc_ast::tokenstream::Spacing::Joint => RustcTokenSpacing::Joint,
            rustc_ast::tokenstream::Spacing::JointHidden => RustcTokenSpacing::Joint,
        }
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

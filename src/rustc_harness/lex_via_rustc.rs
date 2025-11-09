//! Runs rustc's lexical analysis.
//!
//! This works by running the low-level and high-level lexers as far as making a `TokenStream`.
//! That means input with unbalanced delimiters is rejected.
//! If rustc emits any error messages (including fatal errors), we treat the input as rejected.
//!
//! Stringlike literal tokens are further run through `ast::LitKind::from_token_lit()`, to obtain the
//! "unescaped" value.
//!
//! The input string is fed through `SourceMap::new_source_file()`, which means that "normalisation"
//! (BOM-removal and CRLF-conversion) always happen.
//!
//! Shebang removal and front-matter removal happen if requested by the CleaningMode. See
//! [`cleaning`][`crate::reimplementation::cleaning`] module for how we make equivalent input for
//! comparison.
//!
//! This normally uses `rustc_parse::source_str_to_stream` to run the lexers. But at present that
//! doesn't allow a choice of cleaning mode, so for modes other than CleanShebang this instead makes
//! a `rustc_parse::parser::Parser` then pulls tokens from it one by one in the same way as rustc's
//! parser does.
//!
//! The CleanShebangAndFrontmatter cleaning mode isn't stabilised yet. That doesn't matter here
//! because we don't check for feature gate errors.

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

use std::iter;
use std::sync::Arc;

use rustc_ast::{token::TokenKind, tokenstream::TokenStream};
use rustc_parse::{lexer::StripTokens, parser::Parser};
use rustc_session::parse::ParseSess;
use rustc_span::{
    FileName,
    source_map::{FilePathMapping, SourceMap},
};

use crate::trees::Forest;
use crate::{CleaningMode, Edition, Lowering};

use super::error_accumulator::ErrorAccumulator;
use super::rustc_tokens::{RustcToken, map_forest};
use super::rustc_tokenstreams::make_token_stream;

/// Runs rustc's lexical analysis on the specified input.
///
/// If the input is accepted, returns a [`Forest`] of tokens, in [`RustcToken`] form.
/// Otherwise returns at least one error message.
///
/// If rustc panics (ie, it would report an ICE), the panic message is sent to
/// standard error and this function returns CompilerError.
pub fn analyse(
    input: &str,
    edition: Edition,
    cleaning: CleaningMode,
    lowering: Lowering,
) -> Analysis {
    let error_list = ErrorAccumulator::new();

    let rustc_edition = match edition {
        Edition::E2015 => rustc_span::edition::Edition::Edition2015,
        Edition::E2021 => rustc_span::edition::Edition::Edition2021,
        Edition::E2024 => rustc_span::edition::Edition::Edition2024,
    };

    std::panic::catch_unwind(|| {
        match rustc_span::create_session_globals_then(rustc_edition, &[], None, || {
            run_lexer(input, cleaning, lowering, error_list.clone())
        }) {
            Ok(rustc_forest) => {
                let messages = error_list.extract();
                if messages.is_empty() {
                    Analysis::Accepts(rustc_forest)
                } else {
                    Analysis::Rejects(rustc_forest, messages)
                }
            }
            Err(msg) => Analysis::HarnessError(msg),
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
    /// There is a bug in this harness. The string is an error message.
    HarnessError(String),
}

/// Runs rustc's lexical analysis on the specified input.
///
/// An error return means there's a bug in this harness.
///
/// Panics if rustc would have reported an internal compiler error.
///
/// Otherwise:
///  - returns the lexed tokens, in RustcToken form
///  - doc-comments are desugared if requested by the 'lowering' parameter
///  - if rustc would have reported an error, at least one message has
///    been added to error_list
///    - in this case, the returned tokens are what would have been passed on to
///      the parser (empty if token stream construction failed or if rustc would
///      have reported a fatal error).
fn run_lexer(
    input: &str,
    cleaning: CleaningMode,
    lowering: Lowering,
    error_list: ErrorAccumulator,
) -> Result<Forest<RustcToken>, String> {
    use CleaningMode::*;
    let psess = make_parser_session(error_list.clone());
    let source_map = psess.source_map();
    let input = String::from(input);
    let filename = FileName::Custom("lex_via_rustc".into());
    let strip_tokens = match cleaning {
        NoCleaning => StripTokens::Nothing,
        CleanShebang => StripTokens::Shebang,
        CleanShebangAndFrontmatter => StripTokens::ShebangAndFrontmatter,
    };
    rustc_driver::catch_fatal_errors(|| {
        let mut token_stream = token_stream_from_string(&psess, input, filename, strip_tokens)?;
        // The lexer doesn't report errors itself when it sees emoji in 'identifiers'. Instead it leaves
        // a note in the ParseSess to be examined later. So we have to make this extra check.
        if !&psess.bad_unicode_identifiers.borrow_mut().is_empty() {
            error_list.push("bad unicode identifier(s)".into());
        }
        if lowering == Lowering::LowerDocComments {
            token_stream.desugar_doc_comments();
        }
        Ok(map_forest(&token_stream, source_map))
    })
    .unwrap_or_else(|_| {
        // Make sure error_list has at least one error
        error_list.push("reported fatal error (panicked)".into());
        Ok(Forest::new())
    })
}

/// Makes a TokenStream from the source input, via a Parser if necessary.
///
/// If the input has unbalanced delimiters, emits at least one diagnostic and returns an empty
/// TokenStream.
///
/// An error return means there's a bug in this harness.
///
/// Must be run inside catch_fatal_errors().
fn token_stream_from_string(
    psess: &ParseSess,
    input: String,
    filename: FileName,
    strip_tokens: StripTokens,
) -> Result<TokenStream, String> {
    match strip_tokens {
        StripTokens::Shebang => Ok(token_stream_directly_from_string(psess, filename, input)),
        StripTokens::Nothing | StripTokens::ShebangAndFrontmatter => {
            token_stream_from_string_via_parser(psess, filename, input, strip_tokens)
        }
    }
}

/// token_stream_from_string implementation for when we can use source_str_to_stream.
fn token_stream_directly_from_string(
    psess: &ParseSess,
    filename: FileName,
    input: String,
) -> TokenStream {
    rustc_parse::source_str_to_stream(psess, filename, input, None).unwrap_or_else(|diags| {
        // Errors constructing the token stream (ie, unbalanced delimiters) are reported here
        assert!(!diags.is_empty());
        for diag in diags {
            diag.emit();
        }
        TokenStream::new(vec![])
    })
}

/// token_stream_from_string implementation for when we can't use source_str_to_stream.
fn token_stream_from_string_via_parser(
    psess: &ParseSess,
    filename: FileName,
    input: String,
    strip_tokens: StripTokens,
) -> Result<TokenStream, String> {
    let parser = match rustc_parse::new_parser_from_source_str(psess, filename, input, strip_tokens)
    {
        Ok(parser) => parser,
        Err(diags) => {
            // Errors constructing the token stream (ie, unbalanced delimiters) are reported here
            assert!(!diags.is_empty());
            for diag in diags {
                diag.emit();
            }
            return Ok(TokenStream::new(vec![]));
        }
    };
    let ast_tokens = ast_tokens_from_parser(parser);
    // We have to reconstruct the TokenStream, to be parallel to source_str_to_stream and
    // in case we need to use desugar_doc_comments.
    make_token_stream(ast_tokens).map_err(|msg| {
        // Shouldn't happen, because an unbalanced stream would have been rejected previously.
        format!("failed to convert parser output to TokenStream: {msg}")
    })
}

/// Pulls tokens from the parser until Eof and returns them.
fn ast_tokens_from_parser(
    mut parser: Parser<'_>,
) -> impl Iterator<Item = rustc_ast::token::Token> + use<'_> {
    iter::from_fn(move || {
        let token = parser.token;
        if token.kind == TokenKind::Eof {
            None
        } else {
            parser.bump();
            Some(token)
        }
    })
}

fn make_parser_session(error_list: ErrorAccumulator) -> rustc_session::parse::ParseSess {
    #[allow(clippy::arc_with_non_send_sync)]
    let sm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let dcx = error_list.into_diag_ctxt().disable_warnings();
    rustc_session::parse::ParseSess::with_dcx(dcx, sm)
}

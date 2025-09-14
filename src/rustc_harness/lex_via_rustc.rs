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

extern crate rustc_driver;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

use std::sync::Arc;

use rustc_span::{
    source_map::{FilePathMapping, SourceMap},
    FileName,
};

use crate::trees::Forest;
use crate::{Edition, Lowering};

use super::error_accumulator::ErrorAccumulator;
use super::rustc_tokens::{map_forest, RustcToken};

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

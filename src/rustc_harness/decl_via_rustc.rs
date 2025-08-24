//! Runs rustc's lexical analysis by feeding the input to a declarative macro.
//!
//! This works by feeding rustc a source file which defines a declarative macro using the `tt`
//! fragment specifier, then invokes that macro on the input.
//!
//! The macro applies `stringify!()` to each individual leaf token in the input; we return a forest
//! containing `stringify!()`'s output. That means that this tells us how rustc divided the input up
//! into tokens, but (other than for delimiters) it doesn't tell us anything about how rustc
//! classfied the tokens into 'kinds'.
//!
//! This module uses `run_compiler()` to process the source file as far as name resolution and macro
//! expansion (which take place together).
//!
//! This way of handling things means that CRLF-conversion is observable, but BOM-removal and
//! shebang-removal aren't (because the input doesn't end up at the start of the file).
//!
//! If rustc emits any error messages (or panics), we treat the input as rejected.

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_expand;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_errors::registry;
use rustc_hash::FxHashMap;
use rustc_session::config;

use crate::trees::{Forest, GroupKind, Tree};
use crate::Edition;

use super::error_accumulator::ErrorAccumulator;

/// Information we retrieve from rustc about a token.
pub struct RustcDeclToken {
    /// The value that `stringify!` would produce for the token.
    ///
    /// For non-delimiters this holds the value of the literal expression produced by `stringify!`.
    ///
    /// For delimiters this holds the natural single-character string.
    pub stringified: String,
}

impl std::fmt::Debug for RustcDeclToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«{:}»", self.stringified)
    }
}

/// Runs rustc's lexical analysis by feeding the specified input to a declarative macro.
///
/// If the input is accepted, returns a [`Forest`] of tokens, in [`RustcDeclToken`] form.
/// Otherwise returns at least one error message.
///
/// If rustc panics (ie, it would report an ICE), the panic message is sent to
/// standard error and this function returns CompilerError.
///
/// If the input doesn't have properly balanced delimiters, it's possible that this function won't
/// correctly report `Rejects`. See comments under `TOKEN_RENDERER` below.
pub fn analyse(input: &str, edition: Edition) -> Analysis {
    let rustc_edition = match edition {
        Edition::E2015 => rustc_span::edition::Edition::Edition2015,
        Edition::E2021 => rustc_span::edition::Edition::Edition2021,
        Edition::E2024 => rustc_span::edition::Edition::Edition2024,
    };
    std::panic::catch_unwind(|| {
        let error_list = ErrorAccumulator::new();
        let psess_error_accumulator = error_list.clone();
        let config = rustc_interface::Config {
            opts: config::Options {
                edition: rustc_edition,
                ..config::Options::default()
            },
            crate_cfg: Vec::new(),
            crate_check_cfg: Vec::new(),
            input: config::Input::Str {
                name: rustc_span::FileName::Custom("main.rs".into()),
                input: token_renderer_program(input),
            },
            output_dir: None,
            output_file: None,
            file_loader: None,
            locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES.to_owned(),
            lint_caps: FxHashMap::default(),
            psess_created: Some(Box::new(|psess| {
                psess
                    .dcx()
                    .set_emitter(psess_error_accumulator.into_error_emitter());
            })),
            register_lints: None,
            override_queries: None,
            registry: registry::Registry::new(rustc_errors::codes::DIAGNOSTICS),
            make_codegen_backend: None,
            expanded_args: Vec::new(),
            ice_file: None,
            hash_untracked_state: None,
            using_internal_features: &rustc_driver::USING_INTERNAL_FEATURES,
            extra_symbols: Vec::new(),
        };

        match rustc_driver::catch_fatal_errors(|| {
            rustc_interface::run_compiler(config, |compiler| {
                let krate = rustc_interface::passes::parse(&compiler.sess);
                rustc_interface::create_and_enter_global_ctxt(compiler, krate, |tcx| {
                    // Despite the "lint" in the name, this is required for error checking:
                    // interface_emoji_identifier is covered here.
                    tcx.ensure_ok().early_lint_checks(());
                    let krate = &tcx.resolver_for_lowering().borrow().1;
                    if error_list.has_any_errors() {
                        Attempt::AnalysisRejected
                    } else {
                        recover_stringified_forest(krate)
                    }
                })
            })
        }) {
            Ok(Attempt::Recovered(forest)) => Analysis::Accepts(forest),
            Ok(Attempt::FailedRecovery(message)) => Analysis::FrameworkFailed(message),
            Ok(Attempt::AnalysisRejected) => Analysis::Rejects(error_list.extract()),
            Err(_) => {
                let mut messages = error_list.extract();
                messages.push("reported fatal error (panicked)".into());
                Analysis::Rejects(messages)
            }
        }
    })
    .unwrap_or(Analysis::CompilerError)
}

/// Result of running declarative-macro-based lexical analysis on a string.
pub enum Analysis {
    /// Lexical analysis accepted the input
    Accepts(Forest<RustcDeclToken>),

    /// Lexical analysis rejected the input.
    ///
    /// The strings are error messages. There's always at least one message.
    Rejects(Vec<String>),

    /// The macro-based framework failed to recover the tokens that rustc saw.
    FrameworkFailed(String),

    /// Something panicked: could be a rustc ICE or a bug in the macro-based framework.
    ///
    /// If this is a rustc ICE it will typically have printed a panic message to standard error.
    CompilerError,
}

/// Returns a program which can be expanded to tokenise the specified input.
fn token_renderer_program(input: &str) -> String {
    TOKEN_RENDERER.replace("@tokens@", input)
}

/// The program we feed to rustc, with a placeholder for the input tokens.
///
/// Note we need a newline after the `@tokens@` placeholder, to make sure a line commment doesn't
/// swallow too much of the program text.
///
/// If the input doesn't have properly balanced delimiters the `)` which is supposed to end the
/// macro invocation may not end up doing so. In almost all such cases rustc will still reject the
/// program as a whole, so we'll correctly report rejection. See the XFAIL testcases for an
/// exception.
///
/// Similarly if the input has an unterminated block comment it will swallow the rest of the
/// program. Again, rustc will reject the program as a whole so we'll correctly report rejection.
const TOKEN_RENDERER: &str = r#"

macro_rules! explain_tt {
    ( ( $($a:tt)* ) ) => {
        '(';
        explain!($($a)*);
        ')'
    };
    ( { $($a:tt)* } ) => {
        '{';
        explain!($($a)*);
        '}'
    };
    ( [ $($a:tt)* ] ) => {
        '[';
        explain!($($a)*);
        ']'
    };
    ( $a:tt ) => {
        stringify!($a)
    };
}

macro_rules! explain {
    ( $($a:tt)* ) => {
        $(explain_tt!($a));*
    };
}

const _: () = {
    explain!(@tokens@
);
};

"#;

/// The data we return from inside run_compiler().
enum Attempt {
    /// Lexical analysis accepted the input
    Recovered(Forest<RustcDeclToken>),
    /// Lexical analysis rejected the input
    AnalysisRejected,
    /// Either the input had unbalanced delimiters and "broke out of" the macro invocation or
    /// there's a bug in this module's machinery for extracting the lexical analysis,.
    FailedRecovery(String),
}

/// Extracts the tokenisation from the expanded source, as a forest.
///
/// If this returns an error, there's a bug in this module.
fn recover_stringified_forest(krate: &rustc_ast::Crate) -> Attempt {
    let flat = match recover_rendered_tokens(krate) {
        Ok(flat) => flat,
        Err(message) => return Attempt::FailedRecovery(message),
    };
    match construct_forest(&mut flat.into_iter(), None) {
        Ok(forest) => Attempt::Recovered(forest),
        Err(message) => Attempt::FailedRecovery(message),
    }
}

/// A token recovered from the expanded source.
enum RenderedToken {
    /// An open or close group delimiter
    Delimiter(char),
    /// Any non-delimiter token.
    ///
    /// The string is the value of the literal produced by `stringify!`.
    LeafToken(String),
}

/// Extracts the tokenisation from the expanded source, as a flat list.
///
/// Our `explain!()` macro turns each input token into a const item whose expression is a block
/// containing the stringified token as a literal expression (or for a delimiter, as a character
/// literal expression).
///
/// Returns the values of those literal expressions, as a flat list.
fn recover_rendered_tokens(krate: &rustc_ast::Crate) -> Result<Vec<RenderedToken>, String> {
    let mut tokens = Vec::new();
    for item in &krate.items {
        if let rustc_ast::ItemKind::Const(const_item) = &item.kind {
            let Some(expr) = const_item.expr.as_deref() else {
                return Err("const with no expression".to_string());
            };
            let rustc_ast::ExprKind::Block(block, ..) = &expr.kind else {
                return Err(format!("unexpected {expr:?}"));
            };
            for stmt in block.stmts.iter() {
                match &stmt.kind {
                    rustc_ast::StmtKind::Semi(expr) => {
                        let rustc_ast::ExprKind::Lit(token_lit) = &expr.kind else {
                            return Err(format!("stringify! didn't produce a literal: {expr:?}"));
                        };
                        // token_lit (for a non-delimiter) represents a string literal.
                        // from_token_lit() gives us the string that literal would evaluate to.
                        let Ok(ast_lit) = rustc_ast::ast::LitKind::from_token_lit(*token_lit)
                        else {
                            return Err(format!("from_token_lit failed for {token_lit:?}"));
                        };
                        let token = match ast_lit {
                            rustc_ast::LitKind::Char(c) => RenderedToken::Delimiter(c),
                            rustc_ast::LitKind::Str(symbol, ..) => {
                                RenderedToken::LeafToken(symbol.to_string())
                            }
                            _ => {
                                return Err(format!(
                                    "stringify! didn't use a nonraw string literal: {ast_lit}"
                                ))
                            }
                        };
                        tokens.push(token);
                    }
                    rustc_ast::StmtKind::Empty => {
                        // This appears if the explain! macro expanded to an empty sequence (ie, if
                        // the input is whitespace-only), or if there's an empty delimited group.
                    }
                    other => return Err(format!("unexpected statement in the block: {other:?}")),
                }
            }
        }
    }
    Ok(tokens)
}

/// Converts the output of recover_rendered_tokens() to a forest of stringified tokens.
fn construct_forest(
    tokens: &mut impl Iterator<Item = RenderedToken>,
    in_group: Option<GroupKind>,
) -> Result<Forest<RustcDeclToken>, String> {
    let mut constructed = Forest::<RustcDeclToken>::new();
    while let Some(token) = tokens.next() {
        let tree = match token {
            RenderedToken::Delimiter(mark) => {
                if let Some(group_kind) = GroupKind::for_open_char(mark) {
                    Tree::Group(group_kind, construct_forest(tokens, Some(group_kind))?)
                } else if let Some(group_kind) = GroupKind::for_close_char(mark) {
                    if Some(group_kind) == in_group {
                        return Ok(constructed);
                    } else {
                        return Err(format!("unexpected close delimiter: {mark}"));
                    }
                } else {
                    return Err(format!("bad delimiter: {mark}"));
                }
            }
            RenderedToken::LeafToken(stringified) => Tree::Token(RustcDeclToken { stringified }),
        };
        constructed.push(tree);
    }
    match in_group {
        Some(group_kind) => Err(format!(
            "missing close delimiter: {}",
            group_kind.close_char()
        )),
        None => Ok(constructed),
    }
}

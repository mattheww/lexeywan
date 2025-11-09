//! Parallel implementations of complete lexing pipelines, with comparable outputs.

pub mod decl_lexing;
pub mod direct_lexing;

/// The result of running a lexer.
pub enum Verdict<T: Eq> {
    /// The lexer accepted the input.
    ///
    /// Contains the lexer's output, in a form suitable for comparing implementations.
    Accepts(T),

    /// The lexer rejected the input.
    ///
    /// The strings describe why the input was rejected.
    Rejects(Vec<String>),

    /// The lexer reported a problem in its model or implementation.
    ModelError(Vec<String>),
}

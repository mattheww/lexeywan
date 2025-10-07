//! Step 1 (pretokenisation) of lexical analysis.

use crate::char_sequences::Charseq;
use crate::tokens_common::NumericBase;
use crate::Edition;

mod pest_pretokeniser;

#[derive(std::fmt::Debug)]
pub struct Pretoken {
    /// The pretoken's kind and attributes.
    pub data: PretokenData,

    /// The input characters which make up the token.
    pub extent: Charseq,
}

/// A pretoken's kind and attributes.
#[derive(std::fmt::Debug)]
pub enum PretokenData {
    Reserved,
    Whitespace,
    LineComment {
        comment_content: Charseq,
    },
    BlockComment {
        comment_content: Charseq,
    },
    Punctuation {
        mark: char,
    },
    Ident {
        ident: Charseq,
    },
    RawIdent {
        ident: Charseq,
    },
    LifetimeOrLabel {
        name: Charseq,
    },
    RawLifetimeOrLabel {
        name: Charseq,
    },
    CharacterLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    ByteLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    StringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    ByteStringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    CStringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    RawStringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    RawByteStringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    RawCStringLiteral {
        literal_content: Charseq,
        suffix: Option<Charseq>,
    },
    IntegerLiteral {
        base: NumericBase,
        digits: Charseq,
        suffix: Option<Charseq>,
    },
    FloatLiteral {
        body: Charseq,
        suffix: Option<Charseq>,
    },
}

/// Runs step 1 (pretokenisation) of lexical analysis on the specified input.
///
/// Returns an iterator which yields [`Outcome`]s.
///
/// The outcome usually provides a [`Pretoken`] or indicates that the input is unacceptable to the
/// lexer.
///
/// It may instead report a problem with lex_via_peg's model or implementation.
pub fn pretokenise(input: &[char], edition: Edition) -> impl Iterator<Item = Outcome> + use<'_> {
    Pretokeniser {
        edition,
        input,
        index: 0,
    }
}

/// Result of applying a single rule.
pub enum Outcome {
    /// Pretokenisation succeeded in extracting a pretoken.
    Found(Pretoken),

    /// Pretokenisation rejected the input as unacceptable to the lexer.
    ///
    /// The string describes the reason for rejection.
    Rejected(String),

    /// The input demonstrated a problem in lex_via_peg's model or implementation.
    ///
    /// The strings are a description of the problem (one string per line).
    ModelError(Vec<String>),
}

struct Pretokeniser<'a> {
    edition: Edition,
    input: &'a [char],
    index: usize,
}

impl<'a> Iterator for Pretokeniser<'a> {
    type Item = Outcome;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = &self.input[self.index..];
        if rest.is_empty() {
            return None;
        }
        match pest_pretokeniser::lex_one_pretoken(self.edition, rest) {
            pest_pretokeniser::LexOutcome::Lexed(pretoken) => {
                self.index += pretoken.extent.len();
                Some(Outcome::Found(pretoken))
            }
            pest_pretokeniser::LexOutcome::Failed => Some(Outcome::Rejected(
                "The edition's PRETOKEN nonterminal did not match".to_owned(),
            )),
            pest_pretokeniser::LexOutcome::ModelError(s) => Some(Outcome::ModelError(vec![s])),
        }
    }
}

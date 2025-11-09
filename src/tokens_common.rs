//! Common datatypes for fine-grained and coarse-grained tokens.

use crate::datatypes::char_sequences::Charseq;

/// Base (radix) of a numeric literal.
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum NumericBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

#[derive(Clone, std::fmt::Debug)]
/// Where a token came from.
pub enum Origin {
    /// The token was produced by lexical analysis.
    Natural {
        /// The input characters which make up the token.
        extent: Charseq,
    },
    /// The token was produced by other means.
    Synthetic {
        /// The extent of the natural token which was expanded to produce this one.
        ///
        /// This may not be meaningful for a coarse-grained token which was created by combination
        /// from at least one synthetic token (we don't care because that doesn't happen).
        lowered_from: Charseq,
        /// What stringify!() would return for the token.
        ///
        /// This isn't meaningful for a coarse-grained token which was created by combination
        /// from at least one synthetic token (we don't care because that doesn't happen).
        stringified: Charseq,
    },
}

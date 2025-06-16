//! Common datatypes for fine-grained and coarse-grained tokens.

/// Base (radix) of a numeric literal.
#[derive(Copy, Clone, std::fmt::Debug)]
pub enum NumericBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

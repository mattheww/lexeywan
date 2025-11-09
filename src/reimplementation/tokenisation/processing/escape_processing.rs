//! Escape processing
//!
//! Handles the conversion of an escape sequence to an escaped value.
//!
//! The functions here use processing::Error for convenience, but in principle when they return a
//! rejection this means only that the input isn't an instance of the relevant form of escape
//! sequence.

use crate::datatypes::char_sequences::Charseq;

use super::{Error, model_error, rejected};

/// Processes a _simple escape_ sequence, returning a byte.
///
/// Returns the escaped value converted to a byte, or rejects if **`\`** followed by `c` isn't a
/// well-formed simple escape.
pub fn interpret_simple_escape_as_byte(c: char) -> Result<u8, Error> {
    let represented_byte = match c {
        '0' => 0x00,
        't' => 0x09,
        'n' => 0x0a,
        'r' => 0x0d,
        '"' => 0x22,
        '\'' => 0x27,
        '\\' => 0x5c,
        _ => {
            return Err(rejected("not a simple escape"));
        }
    };
    Ok(represented_byte)
}

/// Processes a _simple escape_ sequence, returning a char.
///
/// Returns the escaped value, or rejects if **`\`** followed by `c` isn't a well-formed simple
/// escape.
pub fn interpret_simple_escape(c: char) -> Result<char, Error> {
    interpret_simple_escape_as_byte(c).map(|b| b.into())
}

/// Processes an _8-bit escape_ sequence, returning a byte.
///
/// Returns the escaped value converted to a byte, or rejects if **`\x`** followed by `digits` isn't
/// a well-formed 8-bit escape.
pub fn interpret_8_bit_escape_as_byte(digits: &[char]) -> Result<u8, Error> {
    if digits.len() != 2 {
        return Err(rejected("invalid 8-bit escape"));
    }
    let digits: String = digits.iter().collect();
    u8::from_str_radix(&digits, 16).map_err(|_| rejected("invalid 8-bit escape"))
}

/// Processes an _8-bit escape_ sequence, returning a char.
///
/// Returns the escaped value, or rejects if **`\x`** followed by `digits` isn't a well-formed 8-bit
/// escape.
pub fn interpret_8_bit_escape(digits: &[char]) -> Result<char, Error> {
    interpret_8_bit_escape_as_byte(digits).map(|b| b.into())
}

/// Processes a _7-bit escape_ sequence, returning a char.
///
/// Returns the escaped value, or rejects if **`\x`** followed by `digits` isn't a well-formed 7-bit
/// escape.
pub fn interpret_7_bit_escape(digits: &[char]) -> Result<char, Error> {
    if digits.len() != 2 {
        return Err(rejected("invalid 7-bit escape"));
    }
    let digits: String = digits.iter().collect();
    match u8::from_str_radix(&digits, 16) {
        Ok(byte) => {
            if byte >= 0x80 {
                Err(rejected("invalid 7-bit escape"))
            } else {
                Ok(byte.into())
            }
        }
        Err(_) => Err(rejected("invalid 7-bit escape")),
    }
}

/// Processes a _Unicode escape_ sequence, returning a char.
///
/// Returns the escaped value, or rejects if **`\u`** followed by `escape` isn't a well-formed
/// unicode escape.
pub fn interpret_unicode_escape(escape: &[char]) -> Result<char, Error> {
    let ['{', chars @ .., '}'] = escape else {
        return Err(rejected("unbraced unicode escape"));
    };
    if let Some('_') = chars.first() {
        return Err(rejected("leading underscore in unicode escape"));
    }
    let digits: Charseq = chars.iter().copied().filter(|c| *c != '_').collect();
    if digits.is_empty() {
        return Err(rejected("empty unicode escape"));
    }
    if digits.len() > 6 {
        return Err(rejected("overlong unicode escape"));
    }
    if !&digits.iter().all(|c| c.is_ascii_hexdigit()) {
        return Err(rejected("invalid char in unicode escape"));
    }
    match u32::from_str_radix(&digits.to_string(), 16) {
        Ok(scalar_value) => {
            char::from_u32(scalar_value).ok_or_else(|| rejected("invalid unicode escape"))
        }
        Err(_) => Err(model_error("unhandled invalid hex")),
    }
}

/// Says whether `c` is a whitespace character for the purpose of processing a _string continuation
/// escape_.
pub fn is_string_continuation_whitespace(c: char) -> bool {
    c == '\x09' || c == '\x0a' || c == '\x0d' || c == '\x20'
}

//! Step 2 (reprocessing) of lexical analysis.

use crate::char_sequences::Charseq;
use crate::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::tokens_common::{NumericBase, Origin};

use self::escape_processing::{
    interpret_7_bit_escape, interpret_8_bit_escape, interpret_8_bit_escape_as_byte,
    interpret_simple_escape, interpret_simple_escape_as_byte, interpret_unicode_escape,
    is_string_continuation_whitespace,
};

use super::pretokenisation::{Pretoken, PretokenData};

mod escape_processing;

/// Converts a single pretoken to a single fine-grained token.
///
/// Runs step 2 (reprocessing) of lexical analysis on a single [`Pretoken`] produced by step 1
/// (pretokenisation).
///
/// If the pretoken is accepted, returns a fine-grained token.
///
/// If the pretoken is rejected, distinguishes rejection from "model error".
pub fn reprocess(pretoken: &Pretoken) -> Result<FineToken, Error> {
    let token_data = match &pretoken.data {
        PretokenData::Reserved => {
            return Err(rejected("reserved form"));
        }
        PretokenData::Whitespace => FineTokenData::Whitespace,
        PretokenData::LineComment { comment_content } => lex_line_comment(comment_content)?,
        PretokenData::BlockComment { comment_content } => lex_block_comment(comment_content)?,
        PretokenData::Punctuation { mark } => FineTokenData::Punctuation { mark: *mark },
        PretokenData::Ident { identifier } => lex_nonraw_ident(identifier)?,
        PretokenData::RawIdent { identifier } => lex_raw_ident(identifier)?,
        PretokenData::LifetimeOrLabel { name } => {
            FineTokenData::LifetimeOrLabel { name: name.clone() }
        }
        PretokenData::RawLifetimeOrLabel { name } => lex_raw_lifetime_or_label(name)?,
        PretokenData::SingleQuotedLiteral {
            prefix,
            literal_content,
            suffix,
        } => lex_single_quote_literal(prefix, literal_content, suffix)?,
        PretokenData::DoubleQuotedLiteral {
            prefix,
            literal_content,
            suffix,
        } => lex_nonraw_double_quote_literal(prefix, literal_content, suffix)?,
        PretokenData::RawDoubleQuotedLiteral {
            prefix,
            literal_content,
            suffix,
        } => lex_raw_double_quote_literal(prefix, literal_content, suffix)?,
        PretokenData::IntegerLiteral {
            base,
            digits,
            suffix,
        } => lex_integer_literal(*base, digits, suffix)?,
        PretokenData::FloatLiteral { body, suffix } => lex_float_literal(body, suffix)?,
    };
    Ok(FineToken {
        data: token_data,
        origin: Origin::Natural {
            extent: pretoken.extent.clone(),
        },
    })
}

/// Error from an attempt to reprocess a pretoken.
pub enum Error {
    /// Reprocessing rejected the pretoken.
    ///
    /// The string describes the reason for rejection.
    Rejected(String),

    /// The input demonstrated a problem in lex_via_peg's model or implementation.
    ///
    /// The string describes the problem.
    ModelError(String),
}

fn model_error(s: &str) -> Error {
    Error::ModelError(s.to_owned())
}

fn rejected(s: &str) -> Error {
    Error::Rejected(s.to_owned())
}

/// Validates and interprets a line comment.
fn lex_line_comment(comment_content: &Charseq) -> Result<FineTokenData, Error> {
    let comment_content = comment_content.chars();
    let (style, body) = match comment_content {
        ['/', '/', ..] => (CommentStyle::NonDoc, &[] as &[char]),
        ['/', rest @ ..] => (CommentStyle::OuterDoc, rest),
        ['!', rest @ ..] => (CommentStyle::InnerDoc, rest),
        _ => (CommentStyle::NonDoc, &[] as &[char]),
    };
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains(&'\r') {
        return Err(rejected("CR in line doc comment"));
    }
    Ok(FineTokenData::LineComment {
        style,
        body: body.into(),
    })
}

/// Validates and interprets a block comment.
fn lex_block_comment(comment_content: &Charseq) -> Result<FineTokenData, Error> {
    let comment_content = comment_content.chars();
    let (style, body) = match comment_content {
        ['*', '*', ..] => (CommentStyle::NonDoc, &[] as &[char]),
        ['*', rest @ ..] if !rest.is_empty() => (CommentStyle::OuterDoc, rest),
        ['!', rest @ ..] => (CommentStyle::InnerDoc, rest),
        _ => (CommentStyle::NonDoc, &[] as &[char]),
    };
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains(&'\r') {
        return Err(rejected("CR in block doc comment"));
    }
    Ok(FineTokenData::BlockComment {
        style,
        body: body.into(),
    })
}

/// Validates and interprets a non-raw ident.
fn lex_nonraw_ident(identifier: &Charseq) -> Result<FineTokenData, Error> {
    Ok(FineTokenData::Ident {
        represented_identifier: identifier.nfc(),
    })
}

/// Validates and interprets a `r#...` raw ident.
fn lex_raw_ident(identifier: &Charseq) -> Result<FineTokenData, Error> {
    let represented_identifier = identifier.nfc();
    let s = represented_identifier.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw ident"));
    }
    Ok(FineTokenData::RawIdent {
        represented_identifier,
    })
}

/// Validates and interprets a `r#...` raw lifetime or label.
fn lex_raw_lifetime_or_label(name: &Charseq) -> Result<FineTokenData, Error> {
    let s = name.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw lifetime or label"));
    }
    Ok(FineTokenData::RawLifetimeOrLabel { name: name.clone() })
}

/// Validates and interprets a single-quoted (character or byte) literal.
fn lex_single_quote_literal(
    prefix: &Charseq,
    literal_content: &Charseq,
    suffix: &Option<Charseq>,
) -> Result<FineTokenData, Error> {
    let suffix = suffix.clone().unwrap_or_default();
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    match *prefix.chars() {
        [] => Ok(FineTokenData::CharacterLiteral {
            represented_character: unescape_single_quoted_character(literal_content)?,
            suffix: suffix.clone(),
        }),
        ['b'] => Ok(FineTokenData::ByteLiteral {
            represented_byte: unescape_single_quoted_byte(literal_content)?,
            suffix,
        }),
        _ => Err(model_error("impossible prefix")),
    }
}

/// Validates and interprets a non-raw double-quoted (string, byte, or C-string) literal.
fn lex_nonraw_double_quote_literal(
    prefix: &Charseq,
    literal_content: &Charseq,
    suffix: &Option<Charseq>,
) -> Result<FineTokenData, Error> {
    let suffix = suffix.clone().unwrap_or_default();
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    match *prefix.chars() {
        [] => Ok(FineTokenData::StringLiteral {
            represented_string: unescape_double_quoted_string(literal_content)?,
            suffix,
        }),
        ['b'] => Ok(FineTokenData::ByteStringLiteral {
            represented_bytes: unescape_double_quoted_byte_string(literal_content)?,
            suffix,
        }),
        ['c'] => Ok(FineTokenData::CStringLiteral {
            represented_bytes: unescape_c_string(literal_content)?,
            suffix,
        }),
        _ => Err(model_error("impossible prefix")),
    }
}

/// Validates and interprets a raw double-quoted (string, byte, or C-string) literal.
fn lex_raw_double_quote_literal(
    prefix: &Charseq,
    literal_content: &Charseq,
    suffix: &Option<Charseq>,
) -> Result<FineTokenData, Error> {
    let suffix = suffix.clone().unwrap_or_default();
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    match *prefix.chars() {
        ['r'] => Ok(FineTokenData::RawStringLiteral {
            represented_string: interpret_raw_string(literal_content)?,
            suffix,
        }),
        ['b', 'r'] => Ok(FineTokenData::RawByteStringLiteral {
            represented_bytes: interpret_raw_byte_string(literal_content)?,
            suffix,
        }),
        ['c', 'r'] => Ok(FineTokenData::RawCStringLiteral {
            represented_bytes: interpret_raw_c_string(literal_content)?,
            suffix,
        }),
        _ => Err(model_error("impossible prefix")),
    }
}

/// Validates and interprets an integer literal.
fn lex_integer_literal(
    base: NumericBase,
    digits: &Charseq,
    suffix: &Option<Charseq>,
) -> Result<FineTokenData, Error> {
    let suffix = suffix.clone().unwrap_or_default();
    if digits.iter().all(|c| *c == '_') {
        return Err(rejected("no digits"));
    }
    match base {
        NumericBase::Binary => {
            if !digits.iter().all(|c| *c == '_' || (*c >= '0' && *c < '2')) {
                return Err(rejected("invalid digit"));
            }
        }
        NumericBase::Octal => {
            if !digits.iter().all(|c| *c == '_' || (*c >= '0' && *c < '8')) {
                return Err(rejected("invalid digit"));
            }
        }
        _ => {}
    }
    Ok(FineTokenData::IntegerLiteral {
        base,
        digits: digits.clone(),
        suffix,
    })
}

/// Validates and interprets a floating-point literal.
fn lex_float_literal(body: &Charseq, suffix: &Option<Charseq>) -> Result<FineTokenData, Error> {
    let suffix = suffix.clone().unwrap_or_default();

    Ok(FineTokenData::FloatLiteral {
        body: body.clone(),
        suffix,
    })
}

/// Validates and interprets the content of a '' literal.
fn unescape_single_quoted_character(literal_content: &Charseq) -> Result<char, Error> {
    if literal_content.is_empty() {
        return Err(model_error("impossible character literal content: empty"));
    }
    if literal_content[0] == '\\' {
        let rest = &literal_content[1..];
        if rest.is_empty() {
            return Err(model_error(
                "impossible character literal content: backslash only",
            ));
        }
        if rest[0] == 'x' {
            return interpret_7_bit_escape(&rest[1..]);
        }
        if rest[0] == 'u' {
            return interpret_unicode_escape(&rest[1..]);
        }
        if rest.len() != 1 {
            return Err(rejected("unknown escape"));
        }
        match interpret_simple_escape(rest[0]) {
            Ok(escaped_value) => return Ok(escaped_value),
            Err(_) => return Err(rejected("unknown escape")),
        }
    }
    if literal_content.len() != 1 {
        return Err(model_error("impossible literal content: len != 1"));
    }
    let c = literal_content[0];
    if c == '\'' {
        return Err(model_error("impossible literal content: '"));
    }
    if c == '\n' || c == '\r' || c == '\t' {
        return Err(rejected("escape-only char"));
    }
    Ok(c)
}

/// Validates and interprets the content of a b'' literal.
fn unescape_single_quoted_byte(literal_content: &Charseq) -> Result<u8, Error> {
    if literal_content.is_empty() {
        return Err(model_error("impossible byte literal content: empty"));
    }
    if literal_content[0] == '\\' {
        let rest = &literal_content[1..];
        if rest.is_empty() {
            return Err(model_error(
                "impossible byte literal content: backslash only",
            ));
        }
        if rest[0] == 'x' {
            return interpret_8_bit_escape_as_byte(&rest[1..]);
        }
        if rest.len() != 1 {
            return Err(rejected("unknown escape"));
        }
        match interpret_simple_escape_as_byte(rest[0]) {
            Ok(b) => return Ok(b),
            Err(_) => return Err(rejected("unknown escape")),
        }
    }
    if literal_content.len() != 1 {
        return Err(model_error("impossible literal content: len != 1"));
    }
    let c = literal_content[0];
    if c == '\'' {
        return Err(model_error("impossible literal content: '"));
    }
    if c == '\n' || c == '\r' || c == '\t' {
        return Err(rejected("escape-only char"));
    }
    if c as u32 > 127 {
        return Err(rejected("non-ASCII character in byte literal"));
    }
    Ok(c.try_into().unwrap())
}

/// Validates and interprets the content of a "" literal.
fn unescape_double_quoted_string(literal_content: &Charseq) -> Result<Charseq, Error> {
    let mut chars = literal_content.iter().copied().peekable();
    let mut unescaped = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next().ok_or_else(|| model_error("empty escape"))? {
                'x' => {
                    let digits: Vec<_> = (0..2).filter_map(|_| chars.next()).collect();
                    unescaped.push(interpret_7_bit_escape(&digits)?);
                }
                'u' => {
                    let mut escape = Vec::new();
                    loop {
                        match chars.next() {
                            Some(c) => {
                                escape.push(c);
                                if c == '}' {
                                    break;
                                }
                            }
                            None => return Err(rejected("unterminated unicode escape")),
                        }
                    }
                    unescaped.push(interpret_unicode_escape(&escape)?);
                }
                '\n' => {
                    while let Some(c) = chars.peek() {
                        if is_string_continuation_whitespace(*c) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                c => match interpret_simple_escape(c) {
                    Ok(escaped_value) => unescaped.push(escaped_value),
                    Err(_) => return Err(rejected("unknown escape")),
                },
            },
            '\r' => return Err(rejected("CR in string literal")),
            _ => unescaped.push(c),
        }
    }
    Ok(Charseq::new(unescaped))
}

/// Validates and interprets the content of a b"" literal.
fn unescape_double_quoted_byte_string(literal_content: &Charseq) -> Result<Vec<u8>, Error> {
    let mut chars = literal_content.iter().copied().peekable();
    let mut unescaped = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next().ok_or_else(|| model_error("empty escape"))? {
                'x' => {
                    let digits: Vec<_> = (0..2).filter_map(|_| chars.next()).collect();
                    unescaped.push(interpret_8_bit_escape(&digits)?);
                }
                '\n' => {
                    while let Some(c) = chars.peek() {
                        if is_string_continuation_whitespace(*c) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                c => match interpret_simple_escape(c) {
                    Ok(escaped_value) => unescaped.push(escaped_value),
                    Err(_) => return Err(rejected("unknown escape")),
                },
            },
            '\r' => return Err(rejected("CR in byte string literal")),
            _ => {
                if c as u32 > 127 {
                    return Err(rejected("non-ASCII character in byte string literal"));
                }
                unescaped.push(c)
            }
        }
    }
    Ok(unescaped.iter().map(|c| (*c).try_into().unwrap()).collect())
}

/// Validates and interprets the content of a c"" literal.
fn unescape_c_string(literal_content: &Charseq) -> Result<Vec<u8>, Error> {
    let mut buf = [0; 4];
    let mut chars = literal_content.iter().copied().peekable();
    let mut unescaped = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next().ok_or_else(|| model_error("empty escape"))? {
                'x' => {
                    let digits: Vec<_> = (0..2).filter_map(|_| chars.next()).collect();
                    unescaped.push(interpret_8_bit_escape_as_byte(&digits)?);
                }
                'u' => {
                    let mut escape = Vec::new();
                    loop {
                        match chars.next() {
                            Some(c) => {
                                escape.push(c);
                                if c == '}' {
                                    break;
                                }
                            }
                            None => return Err(rejected("unterminated unicode escape")),
                        }
                    }
                    unescaped.extend(
                        interpret_unicode_escape(&escape)?
                            .encode_utf8(&mut buf)
                            .bytes(),
                    );
                }
                '\n' => {
                    while let Some(c) = chars.peek() {
                        if is_string_continuation_whitespace(*c) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                c => match interpret_simple_escape_as_byte(c) {
                    Ok(escaped_value) => unescaped.push(escaped_value),
                    Err(_) => return Err(rejected("unknown escape")),
                },
            },
            '\r' => return Err(rejected("CR in C string literal")),
            _ => unescaped.extend(c.encode_utf8(&mut buf).bytes()),
        }
    }
    if unescaped.contains(&0) {
        return Err(rejected("NUL in C string literal"));
    }
    Ok(unescaped)
}

/// Validates the content of a r"" literal.
fn interpret_raw_string(literal_content: &Charseq) -> Result<Charseq, Error> {
    if literal_content.contains(&'\r') {
        return Err(rejected("CR in raw string literal"));
    }
    Ok(literal_content.clone())
}

/// Validates and interprets the content of a br"" literal.
fn interpret_raw_byte_string(literal_content: &Charseq) -> Result<Vec<u8>, Error> {
    literal_content
        .chars()
        .iter()
        .copied()
        .map(|c| {
            if c == '\r' {
                Err(rejected("CR in raw byte string literal"))
            } else if c as u32 > 127 {
                Err(rejected("non-ASCII character in raw byte string literal"))
            } else {
                Ok(c.try_into().unwrap())
            }
        })
        .collect()
}

/// Validates and interprets the content of a cr"" literal.
fn interpret_raw_c_string(literal_content: &Charseq) -> Result<Vec<u8>, Error> {
    if literal_content.contains(&'\r') {
        return Err(rejected("CR in raw C string literal"));
    }
    let unescaped: Vec<u8> = literal_content.to_string().into();
    if unescaped.contains(&0) {
        return Err(rejected("NUL in raw C string literal"));
    }
    Ok(unescaped)
}

//! The "Processing a match" stage of extracting a fine-grained token.

use crate::datatypes::char_sequences::Charseq;
use crate::reimplementation::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::tokens_common::{NumericBase, Origin};

use super::tokens_matching::{Nonterminal, TokenKindMatch};

mod escape_processing;
use self::escape_processing::{
    interpret_7_bit_escape, interpret_8_bit_escape, interpret_8_bit_escape_as_byte,
    interpret_simple_escape, interpret_simple_escape_as_byte, interpret_unicode_escape,
    is_string_continuation_whitespace,
};

/// Converts a match to a fine-grained token, or rejects the match.
///
/// This is the "Processing a match" stage of extracting a fine-grained token.
///
/// If the match is accepted, returns a fine-grained token.
///
/// If the match is rejected, distinguishes rejection from "model error".
pub fn process(match_data: &TokenKindMatch) -> Result<FineToken, Error> {
    let token_data = match match_data.token_kind_nonterminal {
        Nonterminal::Whitespace => process_whitespace(match_data)?,
        Nonterminal::Line_comment => process_line_comment(match_data)?,
        Nonterminal::Block_comment => process_block_comment(match_data)?,
        Nonterminal::Character_literal => process_character_literal(match_data)?,
        Nonterminal::Byte_literal => process_byte_literal(match_data)?,
        Nonterminal::String_literal => process_string_literal(match_data)?,
        Nonterminal::Byte_string_literal => process_byte_string_literal(match_data)?,
        Nonterminal::C_string_literal => process_c_string_literal(match_data)?,
        Nonterminal::Raw_string_literal => process_raw_string_literal(match_data)?,
        Nonterminal::Raw_byte_string_literal => process_raw_byte_string_literal(match_data)?,
        Nonterminal::Raw_c_string_literal => process_raw_c_string_literal(match_data)?,
        Nonterminal::Float_literal => process_float_literal(match_data)?,
        Nonterminal::Integer_literal => process_integer_literal(match_data)?,
        Nonterminal::Raw_lifetime_or_label => process_raw_lifetime_or_label(match_data)?,
        Nonterminal::Lifetime_or_label => process_lifetime_or_label(match_data)?,
        Nonterminal::Raw_ident => process_raw_ident(match_data)?,
        Nonterminal::Ident => process_ident(match_data)?,
        Nonterminal::Punctuation => process_punctuation(match_data)?,
        Nonterminal::Unterminated_block_comment
        | Nonterminal::Unterminated_literal_2015
        | Nonterminal::Reserved_literal_2021
        | Nonterminal::Reserved_single_quoted_literal_2015
        | Nonterminal::Reserved_single_quoted_literal_2021
        | Nonterminal::Reserved_guard
        | Nonterminal::Reserved_float
        | Nonterminal::Reserved_lifetime_or_label_prefix
        | Nonterminal::Reserved_prefix_2015
        | Nonterminal::Reserved_prefix_2021 => {
            return Err(Error::Rejected(format!(
                "reserved form: {:?}",
                match_data.token_kind_nonterminal
            )));
        }
        _ => return Err(model_error("unhandled token-kind nonterminal")),
    };
    Ok(FineToken {
        data: token_data,
        origin: Origin::Natural {
            extent: match_data.consumed.clone(),
        },
    })
}

/// Error from an attempt to process a match.
pub enum Error {
    /// Processing rejected the match.
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

impl TokenKindMatch {
    /// Returns the characters consumed by the specified subsidiary nonterminal, or None if that
    /// nonterminal did not participate in the match.
    ///
    /// Reports ModelError if that nonterminal participated in the match more than once.
    fn maybe_consumed(&self, nonterminal: Nonterminal) -> Result<Option<&Charseq>, Error> {
        self.get_checked(nonterminal)
            .map_err(|_| Error::ModelError(format!("{nonterminal:?} participated more than once")))
    }

    /// Returns the characters consumed by the specified subsidiary nonterminal.
    ///
    /// Reports ModelError if that nonterminal did not participate in the match, or participated in
    /// the match more than once.
    fn consumed(&self, nonterminal: Nonterminal) -> Result<&Charseq, Error> {
        self.maybe_consumed(nonterminal)?.ok_or_else(|| {
            Error::ModelError(format!("{nonterminal:?} did not participate in the match"))
        })
    }

    /// Returns the characters consumed by the first participating match of the specified subsidiary
    /// nonterminal in the match.
    ///
    /// Reports ModelError if that nonterminal did not participate in the match.
    fn consumed_by_first_participating(&self, nonterminal: Nonterminal) -> Result<&Charseq, Error> {
        self.get_first(nonterminal).ok_or_else(|| {
            Error::ModelError(format!("{nonterminal:?} did not participate in the match"))
        })
    }

    /// Returns a clone of the characters consumed by the specified subsidiary nonterminal, or an
    /// empty character sequence if that nonterminal did not participate in the match.
    ///
    /// Reports ModelError if that nonterminal participated in the match more than once.
    fn consumed_or_empty(&self, nonterminal: Nonterminal) -> Result<Charseq, Error> {
        self.maybe_consumed(nonterminal)
            .map(|opt| opt.cloned().unwrap_or_default())
    }
}

fn process_whitespace(_m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    Ok(FineTokenData::Whitespace)
}

fn process_line_comment(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let comment_content = m.consumed(Nonterminal::LINE_COMMENT_CONTENT)?;
    let (style, body) = match comment_content.chars() {
        ['/', '/', ..] => (CommentStyle::NonDoc, &[] as &[char]),
        ['/', rest @ ..] => (CommentStyle::OuterDoc, rest),
        ['!', rest @ ..] => (CommentStyle::InnerDoc, rest),
        _ => (CommentStyle::NonDoc, &[] as &[char]),
    };
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains('\r') {
        return Err(rejected("CR in line doc comment"));
    }
    Ok(FineTokenData::LineComment {
        style,
        body: body.into(),
    })
}

fn process_block_comment(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let comment_content = m.consumed_by_first_participating(Nonterminal::BLOCK_COMMENT_CONTENT)?;
    let (style, body) = match comment_content.chars() {
        ['*', '*', ..] => (CommentStyle::NonDoc, &[] as &[char]),
        ['*', rest @ ..] if !rest.is_empty() => (CommentStyle::OuterDoc, rest),
        ['!', rest @ ..] => (CommentStyle::InnerDoc, rest),
        _ => (CommentStyle::NonDoc, &[] as &[char]),
    };
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains('\r') {
        return Err(rejected("CR in block doc comment"));
    }
    Ok(FineTokenData::BlockComment {
        style,
        body: body.into(),
    })
}

fn process_character_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::CharacterLiteral {
        represented_character: represented_character_for_character_literal(
            m.consumed(Nonterminal::SQ_CONTENT)?,
        )?,
        suffix,
    })
}

fn process_byte_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    let represented_character =
        represented_character_for_byte_literal(m.consumed(Nonterminal::SQ_CONTENT)?)?;
    let represented_byte: u8 = represented_character
        .try_into()
        .map_err(|_| model_error("represented_character_for_byte_literal is out of range"))?;
    Ok(FineTokenData::ByteLiteral {
        represented_byte,
        suffix,
    })
}

fn process_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::StringLiteral {
        represented_string: represented_string_for_string_literal(
            m.consumed(Nonterminal::DQ_CONTENT)?,
        )?,
        suffix,
    })
}

fn process_byte_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::ByteStringLiteral {
        represented_bytes: represented_bytes_for_byte_string(m.consumed(Nonterminal::DQ_CONTENT)?)?,
        suffix,
    })
}

fn process_c_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::CStringLiteral {
        represented_bytes: represented_bytes_for_c_string_literal(
            m.consumed(Nonterminal::DQ_CONTENT)?,
        )?,
        suffix,
    })
}

fn process_raw_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    if m.consumed(Nonterminal::HASHES)?.len() > 255 {
        return Err(rejected("too many hashes"));
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    let raw_dq_content = m.consumed(Nonterminal::RAW_DQ_CONTENT)?.clone();
    if raw_dq_content.contains('\r') {
        return Err(rejected("CR in raw string literal"));
    }
    Ok(FineTokenData::RawStringLiteral {
        represented_string: raw_dq_content,
        suffix,
    })
}

fn process_raw_byte_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    if m.consumed(Nonterminal::HASHES)?.len() > 255 {
        return Err(rejected("too many hashes"));
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    let raw_dq_content = m.consumed(Nonterminal::RAW_DQ_CONTENT)?;
    if raw_dq_content.scalar_values().any(|n| n > 127) {
        return Err(rejected("non-ASCII character in raw byte string literal"));
    }
    if raw_dq_content.contains('\r') {
        return Err(rejected("CR in raw byte string literal"));
    }
    let represented_bytes = raw_dq_content
        .scalar_values()
        .map(|c| c.try_into().unwrap())
        .collect();
    Ok(FineTokenData::RawByteStringLiteral {
        represented_bytes,
        suffix,
    })
}

fn process_raw_c_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    if m.consumed(Nonterminal::HASHES)?.len() > 255 {
        return Err(rejected("too many hashes"));
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    let raw_dq_content = m.consumed(Nonterminal::RAW_DQ_CONTENT)?;
    if raw_dq_content.contains('\r') {
        return Err(rejected("CR in raw C string literal"));
    }
    let represented_bytes: Vec<u8> = raw_dq_content.to_string().into();
    if represented_bytes.contains(&0) {
        return Err(rejected("NUL in raw C string literal"));
    }
    Ok(FineTokenData::RawCStringLiteral {
        represented_bytes,
        suffix,
    })
}

fn process_float_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    let body = match (
        m.maybe_consumed(Nonterminal::FLOAT_BODY_WITH_EXPONENT)?,
        m.maybe_consumed(Nonterminal::FLOAT_BODY_WITHOUT_EXPONENT)?,
        m.maybe_consumed(Nonterminal::FLOAT_BODY_WITH_FINAL_DOT)?,
    ) {
        (Some(consumed), None, None) => consumed,
        (None, Some(consumed), None) => consumed,
        (None, None, Some(consumed)) => consumed,
        _ => {
            return Err(model_error(
                "impossible participation for float body nonterminals",
            ));
        }
    };
    Ok(FineTokenData::FloatLiteral {
        body: body.clone(),
        suffix,
    })
}

fn process_integer_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    let digits = match (
        m.maybe_consumed(Nonterminal::LOW_BASE_TOKEN_DIGITS)?,
        m.maybe_consumed(Nonterminal::HEXADECIMAL_DIGITS)?,
        m.maybe_consumed(Nonterminal::DECIMAL_PART)?,
    ) {
        (Some(consumed), None, None) => consumed,
        (None, Some(consumed), None) => consumed,
        (None, None, Some(consumed)) => consumed,
        _ => {
            return Err(model_error(
                "impossible participation for integer digits nonterminals",
            ));
        }
    };
    if digits.iter().all(|c| c == '_') {
        return Err(rejected("no digits"));
    }
    let base = match (
        m.maybe_consumed(Nonterminal::INTEGER_BINARY_LITERAL)?,
        m.maybe_consumed(Nonterminal::INTEGER_OCTAL_LITERAL)?,
        m.maybe_consumed(Nonterminal::INTEGER_HEXADECIMAL_LITERAL)?,
        m.maybe_consumed(Nonterminal::INTEGER_DECIMAL_LITERAL)?,
    ) {
        (Some(_), None, None, None) => NumericBase::Binary,
        (None, Some(_), None, None) => NumericBase::Octal,
        (None, None, Some(_), None) => NumericBase::Hexadecimal,
        (None, None, None, Some(_)) => NumericBase::Decimal,
        _ => {
            return Err(model_error(
                "impossible participation for integer literal nonterminals",
            ));
        }
    };
    match base {
        NumericBase::Binary => {
            if !digits.iter().all(|c| c == '_' || ('0'..'2').contains(&c)) {
                return Err(rejected("invalid digit"));
            }
        }
        NumericBase::Octal => {
            if !digits.iter().all(|c| c == '_' || ('0'..'8').contains(&c)) {
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

fn process_raw_lifetime_or_label(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let name = m.consumed(Nonterminal::IDENT)?.clone();
    let s = name.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw lifetime or label"));
    }
    Ok(FineTokenData::RawLifetimeOrLabel { name })
}

fn process_lifetime_or_label(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let name = m.consumed(Nonterminal::IDENT)?.clone();
    Ok(FineTokenData::LifetimeOrLabel { name })
}

fn process_raw_ident(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let represented_ident = m.consumed(Nonterminal::IDENT)?.nfc();
    let s = represented_ident.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw ident"));
    }
    Ok(FineTokenData::RawIdent { represented_ident })
}

fn process_ident(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    Ok(FineTokenData::Ident {
        represented_ident: m.consumed(Nonterminal::IDENT)?.nfc(),
    })
}

fn process_punctuation(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let mark = match m.consumed.chars() {
        [c] => *c,
        _ => return Err(rejected("impossible Punctuation match")),
    };
    Ok(FineTokenData::Punctuation { mark })
}

/// Validates and interprets the SQ_CONTENT of a '' literal.
fn represented_character_for_character_literal(sq_content: &Charseq) -> Result<char, Error> {
    match sq_content.chars() {
        ['\''] | ['\\'] => Err(model_error("impossible SQ_CONTENT")),
        ['\n'] | ['\r'] | ['\t'] => Err(rejected("escape-only char")),
        [c] => Ok(*c),
        ['\\', 'x', rest @ ..] => interpret_7_bit_escape(rest),
        ['\\', 'u', rest @ ..] => interpret_unicode_escape(rest),
        ['\\', c] => interpret_simple_escape(*c).map_err(|_| rejected("unknown escape")),
        ['\\', ..] => Err(rejected("unknown escape")),
        _ => Err(model_error("impossible SQ_CONTENT")),
    }
}

/// Validates and interprets the SQ_CONTENT of a b'' literal as a character.
fn represented_character_for_byte_literal(sq_content: &Charseq) -> Result<char, Error> {
    match sq_content.chars() {
        ['\''] | ['\\'] => Err(model_error("impossible SQ_CONTENT")),
        ['\n'] | ['\r'] | ['\t'] => Err(rejected("escape-only char")),
        [c] if *c as u32 > 127 => Err(rejected("non-ASCII character in byte literal")),
        [c] => Ok(*c),
        ['\\', 'x', rest @ ..] => interpret_8_bit_escape(rest),
        ['\\', c] => interpret_simple_escape(*c).map_err(|_| rejected("unknown escape")),
        ['\\', ..] => Err(rejected("unknown escape")),
        _ => Err(model_error("impossible SQ_CONTENT")),
    }
}

/// Validates and interprets the DQ_CONTENT of a "" literal.
fn represented_string_for_string_literal(dq_content: &Charseq) -> Result<Charseq, Error> {
    let mut chars = dq_content.iter().peekable();
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

/// Validates and interprets the DQ_CONTENT of a b"" literal.
fn represented_bytes_for_byte_string(dq_content: &Charseq) -> Result<Vec<u8>, Error> {
    let mut chars = dq_content.iter().peekable();
    let mut represented_string = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next().ok_or_else(|| model_error("empty escape"))? {
                'x' => {
                    let digits: Vec<_> = (0..2).filter_map(|_| chars.next()).collect();
                    represented_string.push(interpret_8_bit_escape(&digits)?);
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
                    Ok(escaped_value) => represented_string.push(escaped_value),
                    Err(_) => return Err(rejected("unknown escape")),
                },
            },
            '\r' => return Err(rejected("CR in byte string literal")),
            _ => {
                if c as u32 > 127 {
                    return Err(rejected("non-ASCII character in byte string literal"));
                }
                represented_string.push(c)
            }
        }
    }
    Ok(represented_string
        .into_iter()
        .map(|c| c.try_into().unwrap())
        .collect())
}

/// Validates and interprets the DQ_CONTENT of a c"" literal.
fn represented_bytes_for_c_string_literal(dq_content: &Charseq) -> Result<Vec<u8>, Error> {
    let mut buf = [0; 4];
    let mut chars = dq_content.iter().peekable();
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

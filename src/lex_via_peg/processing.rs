//! The "Processing a match" stage of extracting a fine-grained token.

use crate::char_sequences::Charseq;
use crate::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::tokens_common::{NumericBase, Origin};

use super::token_matching::{MatchData, Nonterminal};

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
pub fn process(match_data: &MatchData) -> Result<FineToken, Error> {
    let token_data = match match_data.token_nonterminal {
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
                match_data.token_nonterminal
            )));
        }
        _ => return Err(model_error("unhandled token nonterminal")),
    };
    Ok(FineToken {
        data: token_data,
        origin: Origin::Natural {
            extent: match_data.extent.clone(),
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

impl MatchData {
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

    /// Returns the characters consumed by the outermost match of the specified subsidiary
    /// nonterminal.
    ///
    /// If that nonterminal participated in the match more than once, reports ModelError unless all
    /// the matches are nested inside one "outermost" match, in which case that match's characters
    /// are returned.
    fn outermost_consumed(&self, nonterminal: Nonterminal) -> Result<&Charseq, Error> {
        self.get_outermost(nonterminal)
            .map_err(|_| {
                Error::ModelError(format!(
                    "{nonterminal:?} participated in the match more than once without proper nesting"
                ))
            })?
            .ok_or_else(|| {
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

fn process_whitespace(_m: &MatchData) -> Result<FineTokenData, Error> {
    Ok(FineTokenData::Whitespace)
}

fn process_line_comment(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_block_comment(m: &MatchData) -> Result<FineTokenData, Error> {
    let comment_content = m.outermost_consumed(Nonterminal::BLOCK_COMMENT_CONTENT)?;
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

fn process_character_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_byte_literal(m: &MatchData) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::ByteLiteral {
        represented_byte: represented_byte_for_byte_literal(m.consumed(Nonterminal::SQ_CONTENT)?)?,
        suffix,
    })
}

fn process_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_byte_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return Err(rejected("underscore literal suffix"));
    }
    Ok(FineTokenData::ByteStringLiteral {
        represented_bytes: represented_bytes_for_byte_string(m.consumed(Nonterminal::DQ_CONTENT)?)?,
        suffix,
    })
}

fn process_c_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_raw_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_raw_byte_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_raw_c_string_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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

fn process_float_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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
            ))
        }
    };
    Ok(FineTokenData::FloatLiteral {
        body: body.clone(),
        suffix,
    })
}

fn process_integer_literal(m: &MatchData) -> Result<FineTokenData, Error> {
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
            ))
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
            ))
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

fn process_raw_lifetime_or_label(m: &MatchData) -> Result<FineTokenData, Error> {
    let name = m.consumed(Nonterminal::IDENT)?.clone();
    let s = name.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw lifetime or label"));
    }
    Ok(FineTokenData::RawLifetimeOrLabel { name })
}

fn process_lifetime_or_label(m: &MatchData) -> Result<FineTokenData, Error> {
    let name = m.consumed(Nonterminal::IDENT)?.clone();
    Ok(FineTokenData::LifetimeOrLabel { name })
}

fn process_raw_ident(m: &MatchData) -> Result<FineTokenData, Error> {
    let represented_ident = m.consumed(Nonterminal::IDENT)?.nfc();
    let s = represented_ident.to_string();
    if s == "_" || s == "crate" || s == "self" || s == "super" || s == "Self" {
        return Err(rejected("forbidden raw ident"));
    }
    Ok(FineTokenData::RawIdent { represented_ident })
}

fn process_ident(m: &MatchData) -> Result<FineTokenData, Error> {
    Ok(FineTokenData::Ident {
        represented_ident: m.consumed(Nonterminal::IDENT)?.nfc(),
    })
}

fn process_punctuation(m: &MatchData) -> Result<FineTokenData, Error> {
    let mark = match m.extent.chars() {
        [c] => *c,
        _ => return Err(rejected("impossible Punctuation match")),
    };
    Ok(FineTokenData::Punctuation { mark })
}

/// Validates and interprets the SQ_CONTENT of a '' literal.
fn represented_character_for_character_literal(sq_content: &Charseq) -> Result<char, Error> {
    if sq_content.is_empty() {
        return Err(model_error("impossible SQ_CONTENT: empty"));
    }
    if sq_content[0] == '\\' {
        let rest = &sq_content[1..];
        if rest.is_empty() {
            return Err(model_error("impossible SQ_CONTENT: backslash only"));
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
    if sq_content.len() != 1 {
        return Err(model_error("impossible SQ_CONTENT: len != 1"));
    }
    let c = sq_content[0];
    if c == '\'' {
        return Err(model_error("impossible SQ_CONTENT: '"));
    }
    if c == '\n' || c == '\r' || c == '\t' {
        return Err(rejected("escape-only char"));
    }
    Ok(c)
}

/// Validates and interprets the SQ_CONTENT of a b'' literal.
fn represented_byte_for_byte_literal(sq_content: &Charseq) -> Result<u8, Error> {
    if sq_content.is_empty() {
        return Err(model_error("impossible SQ_CONTENT: empty"));
    }
    if sq_content[0] == '\\' {
        let rest = &sq_content[1..];
        if rest.is_empty() {
            return Err(model_error("impossible SQ_CONTENT: backslash only"));
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
    if sq_content.len() != 1 {
        return Err(model_error("impossible SQ_CONTENT: len != 1"));
    }
    let c = sq_content[0];
    if c == '\'' {
        return Err(model_error("impossible SQ_CONTENT: '"));
    }
    if c == '\n' || c == '\r' || c == '\t' {
        return Err(rejected("escape-only char"));
    }
    if c as u32 > 127 {
        return Err(rejected("non-ASCII character in byte literal"));
    }
    let represented_character = c;
    Ok(represented_character.try_into().unwrap())
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

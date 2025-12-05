//! The "Processing a match" stage of extracting a fine-grained token.

use crate::datatypes::char_sequences::Charseq;
use crate::reimplementation::fine_tokens::{CommentStyle, FineToken, FineTokenData};
use crate::reimplementation::tokenisation::processing::escape_processing::{
    LiteralComponent, MaybeInterpretation::*, try_escape_interpretation,
    try_single_escape_interpretation,
};
use crate::tokens_common::{NumericBase, Origin};

use super::tokens_matching::{Nonterminal, TokenKindMatch};

mod escape_processing;

/// Converts a match to a fine-grained token, or rejects the match.
///
/// This is the "Processing a match" stage of extracting a fine-grained token.
///
/// If the match is accepted, returns a fine-grained token.
///
/// If the match is rejected, distinguishes rejection from "model error".
pub fn process(match_data: &TokenKindMatch) -> Result<FineToken, Error> {
    let token_data = match match_data.matched_nonterminal {
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
                match_data.matched_nonterminal
            )));
        }
        _ => return model_error("unhandled token-kind nonterminal"),
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

fn model_error<T>(s: &str) -> Result<T, Error> {
    Err(Error::ModelError(s.to_owned()))
}

fn rejected<T>(s: &str) -> Result<T, Error> {
    Err(Error::Rejected(s.to_owned()))
}

impl From<escape_processing::Error> for Error {
    fn from(error: escape_processing::Error) -> Self {
        use escape_processing::Error::*;
        let msg = match error {
            Undefined(msg) => format!("depends on undefined value: {msg}"),
            BadParse(msg) => format!("escape-processing grammar problem: {msg}"),
            Internal(msg) => format!("bug in escape-processing: {msg}"),
        };
        Error::ModelError(msg)
    }
}

impl TokenKindMatch {
    /// Returns the characters consumed by the specified subsidiary nonterminal, or None if that
    /// nonterminal did not participate in the match.
    ///
    /// Reports ModelError if that nonterminal participated in the match more than once.
    fn maybe_consumed(&self, nonterminal: Nonterminal) -> Result<Option<&Charseq>, Error> {
        self.consumed_by_only_participating_match(nonterminal)
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
        self.consumed_by_first_participating_match(nonterminal)
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
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains('\u{000d}') {
        return rejected("CR in line doc comment");
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
    if !matches!(style, CommentStyle::NonDoc) && comment_content.contains('\u{000d}') {
        return rejected("CR in block doc comment");
    }
    Ok(FineTokenData::BlockComment {
        style,
        body: body.into(),
    })
}

fn process_character_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    use LiteralComponent::*;
    let single_quoted_content = m.consumed(Nonterminal::SINGLE_QUOTED_CONTENT)?;
    let single_escape_interpretation =
        match try_single_escape_interpretation(single_quoted_content)? {
            HasInterpretation(interpretation) => interpretation,
            // rejected: "has no single-escape interpretation"
            HasNoInterpretation(reason) => return rejected(reason),
        };
    let Some(represented_character) = single_escape_interpretation.represented_character()? else {
        // rejected: "single-escape interpretation has no represented character"
        return rejected("no represented character");
    };
    if matches!(single_escape_interpretation, NonEscape { .. })
        && matches!(
            single_escape_interpretation.represented_character()?,
            Some('\u{000a}') | Some('\u{000d}') | Some('\u{0009}')
        )
    {
        // rejected: "non-escape whose represented character is LF, CR, or HT"
        return rejected("escape-only char");
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        // rejected: "suffix would consist of the single character _"
        return rejected("underscore literal suffix");
    }
    Ok(FineTokenData::CharacterLiteral {
        represented_character,
        suffix,
    })
}

fn process_byte_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    use LiteralComponent::*;
    let single_quoted_content = m.consumed(Nonterminal::SINGLE_QUOTED_CONTENT)?;
    let single_escape_interpretation =
        match try_single_escape_interpretation(single_quoted_content)? {
            HasInterpretation(interpretation) => interpretation,
            // rejected: "has no single-escape interpretation"
            HasNoInterpretation(reason) => return rejected(reason),
        };
    if matches!(single_escape_interpretation, NonEscape { .. })
        && matches!(
            single_escape_interpretation.represented_character()?,
            Some('\u{000a}') | Some('\u{000d}') | Some('\u{0009}')
        )
    {
        // rejected: "non-escape whose represented character is LF, CR, or HT"
        return rejected("escape-only char");
    }
    if matches!(single_escape_interpretation, UnicodeEscape { .. }) {
        // rejected: "Unicode escape"
        return rejected("unicode escape");
    }
    let Some(represented_byte) = single_escape_interpretation.represented_byte()? else {
        // rejected: "has no represented byte"
        return rejected("no represented byte");
    };
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        // rejected: "suffix would consist of the single character _"
        return rejected("underscore literal suffix");
    }
    Ok(FineTokenData::ByteLiteral {
        represented_byte,
        suffix,
    })
}

fn process_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    use LiteralComponent::*;
    let double_quoted_content = m.consumed(Nonterminal::DOUBLE_QUOTED_CONTENT)?;
    let escape_interpretation = match try_escape_interpretation(double_quoted_content)? {
        HasInterpretation(interpetation) => interpetation,
        // rejected: "has no escape interpretation"
        HasNoInterpretation(reason) => return rejected(reason),
    };
    let mut unescaped = Vec::new();
    for component in escape_interpretation.iter() {
        let Some(represented_character) = component.represented_character()? else {
            // rejected: "a component that has no represented character"
            return Err(Error::Rejected(format!(
                "component without represented character: {component:?}"
            )));
        };
        unescaped.push(represented_character);
        if matches!(component, NonEscape { .. })
            && component.represented_character()? == Some('\u{000d}')
        {
            // rejected: "a non-escape whose represented character is CR"
            return rejected("CR non-escape");
        }
    }
    let represented_string = Charseq::new(unescaped);
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        // rejected: "suffix would consist of the single character _"
        return rejected("underscore literal suffix");
    }
    Ok(FineTokenData::StringLiteral {
        represented_string,
        suffix,
    })
}

fn process_byte_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    use LiteralComponent::*;
    let double_quoted_content = m.consumed(Nonterminal::DOUBLE_QUOTED_CONTENT)?;
    let escape_interpretation = match try_escape_interpretation(double_quoted_content)? {
        HasInterpretation(interpetation) => interpetation,
        // rejected: "has no escape interpretation"
        HasNoInterpretation(reason) => return rejected(reason),
    };
    let mut represented_bytes = Vec::new();
    for component in escape_interpretation.iter() {
        if matches!(component, NonEscape { .. })
            && component.represented_character()? == Some('\u{000d}')
        {
            // rejected: "a non-escape whose represented character is CR"
            return rejected("CR non-escape");
        }
        if matches!(component, UnicodeEscape { .. }) {
            // rejected: "a Unicode escape"
            return rejected("unicode escape in byte string literal");
        }
        let Some(represented_byte) = component.represented_byte()? else {
            // rejected: "a component that has no represented byte"
            return Err(Error::Rejected(format!(
                "component without represented byte: {component:?}"
            )));
        };
        represented_bytes.push(represented_byte);
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        // rejected: "suffix would consist of the single character _"
        return rejected("underscore literal suffix");
    }
    Ok(FineTokenData::ByteStringLiteral {
        represented_bytes,
        suffix,
    })
}

fn process_c_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    use LiteralComponent::*;
    let double_quoted_content = m.consumed(Nonterminal::DOUBLE_QUOTED_CONTENT)?;
    let escape_interpretation = match try_escape_interpretation(double_quoted_content)? {
        HasInterpretation(interpetation) => interpetation,
        // rejected: "has no escape interpretation"
        HasNoInterpretation(reason) => return rejected(reason),
    };
    let mut buf = [0; 4];
    let mut represented_bytes = Vec::new();
    for component in escape_interpretation.iter() {
        if matches!(component, UnicodeEscape { .. }) && component.represented_character()?.is_none()
        {
            // rejected: "a Unicode escape which has no represented character"
            return rejected("out-of-range unicode escape");
        }
        if matches!(component, NonEscape { .. })
            && component.represented_character()? == Some('\u{000d}')
        {
            // rejected: "a non-escape whose represented character is CR"
            return rejected("CR non-escape");
        }
        match component {
            // "Each non-escape, simple escape, or Unicode escape contributes the UTF-8 encoding of
            //  its represented character"
            NonEscape { .. } | SimpleEscape { .. } | UnicodeEscape { .. } => {
                let Some(represented_character) = component.represented_character()? else {
                    return model_error("no represented character");
                };
                represented_bytes.extend(represented_character.encode_utf8(&mut buf).bytes());
            }
            // "Each hexadecimal escape contributes its represented byte"
            HexadecimalEscape { .. } => {
                let Some(represented_byte) = component.represented_byte()? else {
                    return model_error("no represented byte");
                };
                represented_bytes.push(represented_byte);
            }
            _ => return model_error("unhandled component"),
        }
    }
    if represented_bytes.contains(&0) {
        // rejected: "any of the token's represented bytes would be 0"
        return rejected("representation of NUL");
    }
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        // rejected: "suffix would consist of the single character _"
        return rejected("underscore literal suffix");
    }
    Ok(FineTokenData::CStringLiteral {
        represented_bytes,
        suffix,
    })
}

fn process_raw_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return rejected("underscore literal suffix");
    }
    let raw_double_quoted_content = m.consumed(Nonterminal::RAW_DOUBLE_QUOTED_CONTENT)?.clone();
    if raw_double_quoted_content.contains('\u{000d}') {
        return rejected("CR non-escape");
    }
    Ok(FineTokenData::RawStringLiteral {
        represented_string: raw_double_quoted_content,
        suffix,
    })
}

fn process_raw_byte_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return rejected("underscore literal suffix");
    }
    let raw_double_quoted_content = m.consumed(Nonterminal::RAW_DOUBLE_QUOTED_CONTENT)?;
    if raw_double_quoted_content.scalar_values().any(|n| n > 127) {
        return rejected("non-ASCII character");
    }
    if raw_double_quoted_content.contains('\u{000d}') {
        return rejected("CR in raw content");
    }
    let represented_bytes = raw_double_quoted_content
        .scalar_values()
        .map(|c| c.try_into().unwrap())
        .collect();
    Ok(FineTokenData::RawByteStringLiteral {
        represented_bytes,
        suffix,
    })
}

fn process_raw_c_string_literal(m: &TokenKindMatch) -> Result<FineTokenData, Error> {
    let suffix = m.consumed_or_empty(Nonterminal::SUFFIX)?;
    if suffix.chars() == ['_'] {
        return rejected("underscore literal suffix");
    }
    let raw_double_quoted_content = m.consumed(Nonterminal::RAW_DOUBLE_QUOTED_CONTENT)?;
    if raw_double_quoted_content.contains('\u{000d}') {
        return rejected("CR in raw content");
    }
    let represented_bytes: Vec<u8> = raw_double_quoted_content.to_string().into();
    if represented_bytes.contains(&0) {
        return rejected("NUL in raw content");
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
            return model_error("impossible participation for float body nonterminals");
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
            return model_error("impossible participation for integer digits nonterminals");
        }
    };
    if digits.iter().all(|c| c == '_') {
        return rejected("no digits");
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
            return model_error("impossible participation for integer literal nonterminals");
        }
    };
    match base {
        NumericBase::Binary => {
            if !digits.iter().all(|c| c == '_' || ('0'..'2').contains(&c)) {
                return rejected("invalid digit");
            }
        }
        NumericBase::Octal => {
            if !digits.iter().all(|c| c == '_' || ('0'..'8').contains(&c)) {
                return rejected("invalid digit");
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
        return rejected("forbidden raw lifetime or label");
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
        return rejected("forbidden raw ident");
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
        _ => return rejected("impossible Punctuation match"),
    };
    Ok(FineTokenData::Punctuation { mark })
}

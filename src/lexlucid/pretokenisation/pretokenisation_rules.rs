//! The rules for pretokenisation, and the per-edition rule lists.

// The regular expressions used here are for the Rust `regex` crate, with:
//   "verbose" (ignore-whitespace) representation
//   dot-matches-newline enabled

use regex::Captures;

use std::{collections::BTreeMap, sync::OnceLock};

use crate::{char_sequences::Charseq, Edition};

use super::{PretokenData, Rule};

pub fn list_rules(edition: Edition) -> &'static Vec<&'static Rule> {
    static EDITION_2015_RULES: OnceLock<Vec<&'static Rule>> = OnceLock::new();
    static EDITION_2021_RULES: OnceLock<Vec<&'static Rule>> = OnceLock::new();
    match edition {
        Edition::E2015 => EDITION_2015_RULES.get_or_init(|| make_rules(RULES_FOR_EDITION_2015)),
        Edition::E2021 => EDITION_2021_RULES.get_or_init(|| make_rules(RULES_FOR_EDITION_2021)),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum RuleName {
    Whitespace,
    LineComment,
    BlockComment,
    UnterminatedBlockComment,
    Punctuation,
    SingleQuotedLiteral,
    RawLifetimeOrLabel2021,
    ReservedLifetimeOrLabelPrefix2021,
    LifetimeOrLabel,
    DoublequotedNonrawLiteral2015,
    DoublequotedNonrawLiteral2021,
    DoublequotedHashlessRawLiteral2015,
    DoublequotedHashlessRawLiteral2021,
    DoublequotedHashedRawLiteral2015,
    DoublequotedHashedRawLiteral2021,
    FloatLiteralWithExponent,
    FloatLiteralWithoutExponent,
    FloatLiteralWithFinalDot,
    IntegerBinaryLiteral,
    IntegerOctalLiteral,
    IntegerHexadecimalLiteral,
    IntegerDecimalLiteral,
    RawIdentifier,
    UnterminatedLiteral2015,
    ReservedPrefixOrUnterminatedLiteral2021,
    NonrawIdentifier,
}

const RULES_FOR_EDITION_2015: &[RuleName] = [
    RuleName::Whitespace,
    RuleName::LineComment,
    RuleName::BlockComment,
    RuleName::UnterminatedBlockComment,
    RuleName::Punctuation,
    RuleName::SingleQuotedLiteral,
    RuleName::LifetimeOrLabel,
    RuleName::DoublequotedNonrawLiteral2015,
    RuleName::DoublequotedHashlessRawLiteral2015,
    RuleName::DoublequotedHashedRawLiteral2015,
    RuleName::FloatLiteralWithExponent,
    RuleName::FloatLiteralWithoutExponent,
    RuleName::FloatLiteralWithFinalDot,
    RuleName::IntegerBinaryLiteral,
    RuleName::IntegerOctalLiteral,
    RuleName::IntegerHexadecimalLiteral,
    RuleName::IntegerDecimalLiteral,
    RuleName::RawIdentifier,
    RuleName::UnterminatedLiteral2015,
    RuleName::NonrawIdentifier,
]
.as_slice();

const RULES_FOR_EDITION_2021: &[RuleName] = [
    RuleName::Whitespace,
    RuleName::LineComment,
    RuleName::BlockComment,
    RuleName::UnterminatedBlockComment,
    RuleName::Punctuation,
    RuleName::SingleQuotedLiteral,
    RuleName::RawLifetimeOrLabel2021,
    RuleName::ReservedLifetimeOrLabelPrefix2021,
    RuleName::LifetimeOrLabel,
    RuleName::DoublequotedNonrawLiteral2021,
    RuleName::DoublequotedHashlessRawLiteral2021,
    RuleName::DoublequotedHashedRawLiteral2021,
    RuleName::FloatLiteralWithExponent,
    RuleName::FloatLiteralWithoutExponent,
    RuleName::FloatLiteralWithFinalDot,
    RuleName::IntegerBinaryLiteral,
    RuleName::IntegerOctalLiteral,
    RuleName::IntegerHexadecimalLiteral,
    RuleName::IntegerDecimalLiteral,
    RuleName::RawIdentifier,
    RuleName::ReservedPrefixOrUnterminatedLiteral2021,
    RuleName::NonrawIdentifier,
]
.as_slice();

fn make_rules(wanted: &[RuleName]) -> Vec<&'static Rule> {
    static NAMED_RULES: OnceLock<BTreeMap<RuleName, Rule>> = OnceLock::new();
    let named_rules = NAMED_RULES.get_or_init(make_named_rules);
    wanted.iter().map(|name| &named_rules[name]).collect()
}

#[rustfmt::skip]
fn make_named_rules() -> BTreeMap<RuleName, Rule> {
    [

       // Whitespace
       (RuleName::Whitespace,
        Rule::new_regex(
            |_| PretokenData::Whitespace, r##"\A
                [ \p{Pattern_White_Space} ] +
            "##)),

       // Line comment
       (RuleName::LineComment,
        Rule::new_regex(
            |cp| PretokenData::LineComment{ comment_content: cp["comment_content"].into() }, r##"\A
                / /
                (?<comment_content>
                  [^ \n] *
                )
            "##)),

       // Block comment
       (RuleName::BlockComment,
        Rule::new_constrained_regex (
            |cp| PretokenData::BlockComment{ comment_content: cp["comment_content"].into() },
            block_comment_constraint, r##"\A
                / \*
            "##, r##"\A
                / \*
                (?<comment_content>
                  . *
                )
                \* /
            \z"##)),

       // Unterminated block comment
       (RuleName::UnterminatedBlockComment,
        Rule::new_regex(
            |_| PretokenData::Reserved, r##"\A
                / \*
            "##)),

       // Punctuation
       (RuleName::Punctuation,
        Rule::new_regex(
            |cp| PretokenData::Punctuation {
                mark: cp[0].chars().next().unwrap(),
            }, r##"\A
                [
                  ; , \. \( \) \{ \} \[ \] @ \# ~ \? : \$ = ! < > \- & \| \+ \* / ^ %
                ]
            "##)),

       // Single-quoted literal
       (RuleName::SingleQuotedLiteral,
        Rule::new_regex(
            |cp| PretokenData::SingleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, r##"\A
                (?<prefix>
                  b ?
                )
                '
                (?<literal_content>
                  [^ \\ ' ]
                |
                  \\ . [^']*
                )
                '
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Lifetime or label
       (RuleName::RawLifetimeOrLabel2021,
        Rule::new_regex(
            |cp| PretokenData::RawLifetimeOrLabel {
                name: cp["name"].into(),
            }, r##"\A
                ' r \#
                (?<name>
                  [ \p{XID_Start} _ ]
                  \p{XID_Continue} *
                )
            "##)),

       // Reserved lifetime or label prefix
       (RuleName::ReservedLifetimeOrLabelPrefix2021,
        Rule::new_regex(
            |_| PretokenData::Reserved, r##"\A
                '
                [ \p{XID_Start} _ ]
                \p{XID_Continue} *
                \#
            "##)),

       // Lifetime or label
       (RuleName::LifetimeOrLabel,
        Rule::new_regex_with_forbidden_follower(
            |cp| PretokenData::LifetimeOrLabel {
                name: cp["name"].into(),
            }, r##"\A
                '
                (?<name>
                  [ \p{XID_Start} _ ]
                  \p{XID_Continue} *
                )
            "##,
            |c| c == '\'')),

       // Double-quoted non-raw literal (Rust 2015 and 2018)
       (RuleName::DoublequotedNonrawLiteral2015,
        Rule::new_regex(
            |cp| PretokenData::DoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, r##"\A
                (?<prefix>
                  b ?
                )
                "
                (?<literal_content>
                  (?:
                    [^ \\ " ]
                  |
                    \\ .
                  ) *
                )
                "
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Double-quoted non-raw literal (Rust 2021)
       (RuleName::DoublequotedNonrawLiteral2021,
        Rule::new_regex(
            |cp| PretokenData::DoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, r##"\A
                (?<prefix>
                  [bc] ?
                )
                "
                (?<literal_content>
                  (?:
                    [^ \\ " ]
                  |
                    \\ .
                  ) *
                )
                "
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Double-quoted hashless raw literal (Rust 2015 and 2018)
       (RuleName::DoublequotedHashlessRawLiteral2015,
        Rule::new_regex (
            |cp| PretokenData::RawDoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, r##"\A
                (?<prefix>
                  r | br
                )
                "
                (?<literal_content>
                  [^"] *
                )
                "
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Double-quoted hashless raw literal (Rust 2021)
       (RuleName::DoublequotedHashlessRawLiteral2021,
        Rule::new_regex (
            |cp| PretokenData::RawDoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, r##"\A
                (?<prefix>
                  r | br | cr
                )
                "
                (?<literal_content>
                  [^"] *
                )
                "
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Double-quoted hashed raw literal (Rust 2015 and 2018)
       (RuleName::DoublequotedHashedRawLiteral2015,
        Rule::new_constrained_regex (
            |cp| PretokenData::RawDoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, |cp| {
              cp.name("hashes_1").unwrap().as_str() == cp.name("hashes_2").unwrap().as_str()
            }, r##"\A
                (
                  r | br
                )
                \#
            "##, r##"\A
                (?<prefix>
                  r | br
                )
                (?<hashes_1>
                  \# {1,255}
                )
                "
                (?<literal_content>
                  . *
                )
                "
                (?<hashes_2>
                  \# {1,255}
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            \z"##)),

       // Double-quoted hashed raw literal (Rust 2021)`
       (RuleName::DoublequotedHashedRawLiteral2021,
        Rule::new_constrained_regex (
            |cp| PretokenData::RawDoubleQuoteLiteral {
                prefix: cp["prefix"].into(),
                literal_content: cp["literal_content"].into(),
                suffix: cp["suffix"].into(),
            }, |cp| {
              cp.name("hashes_1").unwrap().as_str() == cp.name("hashes_2").unwrap().as_str()
            }, r##"\A
                (
                  r | br | cr
                )
                \#
            "##, r##"\A
                (?<prefix>
                  r | br | cr
                )
                (?<hashes_1>
                  \# {1,255}
                )
                "
                (?<literal_content>
                  . *
                )
                "
                (?<hashes_2>
                  \# {1,255}
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} _ ]
                    \p{XID_Continue} *
                  ) ?
                )
            \z"##)),


       // Float literal with exponent
       (RuleName::FloatLiteralWithExponent,
        Rule::new_regex(
            |cp| {
                PretokenData::FloatLiteral {
                    has_base: cp.name("based").is_some(),
                    body: cp["body"].into(),
                    exponent_digits: Some(cp["exponent_digits"].into()),
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                (?<body>
                  (?:
                    (?<based>
                      (?: 0b | 0o )
                      [ 0-9 _ ] *
                    )
                  |
                    [ 0-9 ]
                    [ 0-9 _ ] *
                  )
                  (?:
                    \.
                    [ 0-9 ]
                    [ 0-9 _ ] *
                  ) ?
                  [eE]
                  [+-] ?
                  (?<exponent_digits>
                    [ 0-9 _ ] *
                  )
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} ]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Float literal without exponent
       (RuleName::FloatLiteralWithoutExponent,
        Rule::new_regex(
            |cp| {
                PretokenData::FloatLiteral {
                    has_base: cp.name("based").is_some(),
                    body: cp["body"].into(),
                    exponent_digits: None,
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                (?<body>
                  (?:
                    (?<based>
                      (?: 0b | 0o )
                      [ 0-9 _ ] *
                    |
                      0x
                      [ 0-9 a-f A-F _ ] *
                    )
                  |
                    [ 0-9 ]
                    [ 0-9 _ ] *
                  )
                  \.
                  [ 0-9 ]
                  [ 0-9 _ ] *
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} -- eE]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Float literal with final dot
       (RuleName::FloatLiteralWithFinalDot,
        Rule::new_regex_with_forbidden_follower(
            |cp| {
                PretokenData::FloatLiteral {
                    has_base: cp.name("based").is_some(),
                    body: cp[0].into(),
                    exponent_digits: None,
                    suffix: Charseq::new(vec![]),
                }
            }, r##"\A
                (?:
                  (?<based>
                    (?: 0b | 0o )
                    [ 0-9 _ ] *
                  |
                    0x
                    [ 0-9 a-f A-F _ ] *
                  )
                |
                  [ 0-9 ]
                  [ 0-9 _ ] *
                )
                \.
            "##,
            |c| c == '_' || c == '.' || unicode_xid::UnicodeXID::is_xid_start(c))),

       // Integer binary literal
       (RuleName::IntegerBinaryLiteral,
        Rule::new_regex(
            |cp| {
                PretokenData::IntegerBinaryLiteral {
                    digits: cp["digits"].into(),
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                0b
                (?<digits>
                  [ 0-9 _ ] *
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} -- eE]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Integer octal literal
       (RuleName::IntegerOctalLiteral,
        Rule::new_regex(
            |cp| {
                PretokenData::IntegerOctalLiteral {
                    digits: cp["digits"].into(),
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                0o
                (?<digits>
                  [ 0-9 _ ] *
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} -- eE]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Integer hexadecimel literal
       (RuleName::IntegerHexadecimalLiteral,
        Rule::new_regex(
            |cp| {
                PretokenData::IntegerHexadecimalLiteral {
                    digits: cp["digits"].into(),
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                0x
                (?<digits>
                  [ 0-9 a-f A-F _ ] *
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} -- aAbBcCdDeEfF]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Integer decimal literal
       (RuleName::IntegerDecimalLiteral,
        Rule::new_regex(
            |cp| {
                PretokenData::IntegerDecimalLiteral {
                    digits: cp["digits"].into(),
                    suffix: cp["suffix"].into(),
                }
            }, r##"\A
                (?<digits>
                  [ 0-9 ]
                  [ 0-9 _ ] *
                )
                (?<suffix>
                  (?:
                    [ \p{XID_Start} -- eE]
                    \p{XID_Continue} *
                  ) ?
                )
            "##)),

       // Raw identifier
       (RuleName::RawIdentifier,
        Rule::new_regex(
            |cp| PretokenData::RawIdentifier {
                identifier: cp["identifier"].into(),
            }, r##"\A
                r \#
                (?<identifier>
                  [ \p{XID_Start} _ ]
                  \p{XID_Continue} *
                )
            "##)),

       // Unterminated literal (Rust 2015 and 2018)
       (RuleName::UnterminatedLiteral2015,
        Rule::new_regex(
            |_| PretokenData::Reserved, r##"\A
                ( r \# | b r \# | r " | b r " | b ' )
            "##)),

       // Reserved prefix or unterminated literal (Rust 2021)
       (RuleName::ReservedPrefixOrUnterminatedLiteral2021,
        Rule::new_regex(
            |_| PretokenData::Reserved, r##"\A
                [ \p{XID_Start} _ ]
                \p{XID_Continue} *
                ( \# | " | ' )
            "##)),

       // Non-raw identifier
       (RuleName::NonrawIdentifier,
        Rule::new_regex(
            |cp| PretokenData::Identifier {
                identifier: cp["identifier"].into(),
            }, r##"\A
                (?<identifier>
                  [ \p{XID_Start} _ ]
                  \p{XID_Continue} *
                )
            "##)),

    ].into_iter().collect()
}

/// Constraint rule for block comments.
pub fn block_comment_constraint(captures: &Captures) -> bool {
    let content = &captures[0];
    let mut depth = 0_isize;
    let mut after_slash = false;
    let mut after_star = false;
    for c in content.chars() {
        match c {
            '*' if after_slash => {
                depth += 1;
                after_slash = false;
            }
            '/' if after_star => {
                depth -= 1;
                // Depth doesn't drop below zero because a previous candidate would have succeeded.
                assert!(depth >= 0);
                after_star = false;
            }
            _ => {
                after_slash = c == '/';
                after_star = c == '*';
            }
        }
    }
    depth == 0
}

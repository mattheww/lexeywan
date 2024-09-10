//! Step 1 (pretokenisation) of lexical analysis.

use regex::{Captures, Regex};

use crate::{
    char_sequences::Charseq, lexlucid::pretokenisation::regex_utils::constrained_captures, Edition,
};
use regex_utils::pretokeniser_regex;

mod function_rules;
mod pretokenisation_rules;
mod regex_utils;

macro_rules! make_regex {
    ($re:literal $(,)?) => {{
        static RE: ::std::sync::OnceLock<regex::Regex> = ::std::sync::OnceLock::new();
        RE.get_or_init(|| crate::lexlucid::pretokenisation::regex_utils::pretokeniser_regex($re))
    }};
}
use make_regex;

#[derive(std::fmt::Debug)]
pub struct Pretoken {
    /// The pretoken's kind and attributes.
    pub data: PretokenData,

    /// The input characters which make up the token.
    pub extent: Charseq,
}

impl Pretoken {
    /// Returns the number of characters in the pretoken.
    fn char_length(&self) -> usize {
        self.extent.len()
    }
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
    Identifier {
        identifier: Charseq,
    },
    RawIdentifier {
        identifier: Charseq,
    },
    LifetimeOrLabel {
        name: Charseq,
    },
    RawLifetimeOrLabel {
        name: Charseq,
    },
    SingleQuoteLiteral {
        prefix: Charseq,
        literal_content: Charseq,
        suffix: Charseq,
    },
    DoubleQuoteLiteral {
        prefix: Charseq,
        literal_content: Charseq,
        suffix: Charseq,
    },
    RawDoubleQuoteLiteral {
        prefix: Charseq,
        literal_content: Charseq,
        suffix: Charseq,
    },
    IntegerDecimalLiteral {
        digits: Charseq,
        suffix: Charseq,
    },
    IntegerHexadecimalLiteral {
        digits: Charseq,
        suffix: Charseq,
    },
    IntegerBinaryLiteral {
        digits: Charseq,
        suffix: Charseq,
    },
    IntegerOctalLiteral {
        digits: Charseq,
        suffix: Charseq,
    },
    FloatLiteral {
        has_base: bool,
        body: Charseq,
        exponent_digits: Option<Charseq>,
        suffix: Charseq,
    },
}

/// Runs step 1 (pretokenisation) of lexical analysis on the specified input.
///
/// Returns an iterator which yields [`Outcome`]s.
///
/// The outcome usually provides a [`Pretoken`] or indicates that the input is unacceptable to the
/// lexer.
///
/// It may instead report a problem with lexlucid's model or implementation.
pub fn pretokenise(input: Charseq, edition: Edition) -> impl Iterator<Item = Outcome> {
    Pretokeniser {
        rules: pretokenisation_rules::list_rules(edition),
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

    /// The input demonstrated a problem in lexlucid's model or implementation.
    ///
    /// The strings are a description of the problem (one string per line).
    ModelError(Vec<String>),
}

struct Pretokeniser {
    rules: &'static Vec<&'static Rule>,
    input: Charseq,
    index: usize,
}

impl Iterator for Pretokeniser {
    type Item = Outcome;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = &self.input.chars()[self.index..];
        if rest.is_empty() {
            return None;
        }
        use Outcome::*;
        match lex_one_pretoken(self.rules, rest) {
            LexOutcome::Lexed(pretoken) => {
                self.index += pretoken.extent.len();
                Some(Outcome::Found(pretoken))
            }
            LexOutcome::NoRuleMatched => Some(Rejected("no rule matched".into())),
            LexOutcome::ForcedError(message) => Some(Rejected(message)),
            LexOutcome::PriorityViolation { best, violators } => {
                Some(ModelError(describe_priority_violations(best, violators)))
            }
        }
    }
}

/// Extracts the next pretoken from the input.
///
/// Applies each rule to the input.
///
/// If multiple rules succeed, uses the highest-priority (earliest-listed) rule.
///
/// Reports PriorityViolation if any lower-priority rule succeeded as many, or more, characters.
/// (This is checking that priority-based and longest-match-based formulations would be equivalent.)
fn lex_one_pretoken(rules: &Vec<&Rule>, rest: &[char]) -> LexOutcome {
    use LexOutcome::*;
    let mut matches = Vec::new();
    for rule in rules {
        match rule.apply(rest) {
            RuleOutcome::Success(token_length, data) => {
                matches.push(Pretoken {
                    data,
                    extent: rest[..token_length].into(),
                });
            }
            RuleOutcome::Failure => {}
            RuleOutcome::ForceError(message) => return ForcedError(message),
        }
    }
    if matches.is_empty() {
        NoRuleMatched
    } else {
        resolve(matches)
    }
}

enum LexOutcome {
    /// At least one rule matched, and there was no priority violation.
    Lexed(Pretoken),

    /// No rule matched.
    NoRuleMatched,

    /// Multiple rules matched, and the sequence matched by the highest priority rule wasn't longer
    /// than the sequence matched by all other rules.
    PriorityViolation {
        /// The pretoken generated by the highest-priority rules.
        best: Pretoken,
        /// The pretoken from lower-priority rules which were unexpectedly long.
        violators: Vec<Pretoken>,
    },

    /// A rule requested a forced lexer error (not currently used).
    ForcedError(String),
}

/// Returns the highest-priority match, or reports a priority violation.
fn resolve(matches: Vec<Pretoken>) -> LexOutcome {
    use LexOutcome::*;
    let mut iter = matches.into_iter();
    let best = iter.next().unwrap();
    let best_length = best.char_length();
    let violators: Vec<_> = iter
        .filter(|pretoken| pretoken.char_length() >= best_length)
        .collect();
    if violators.is_empty() || is_exception_to_longest_match_principle(&best, &violators) {
        Lexed(best)
    } else {
        PriorityViolation { best, violators }
    }
}

/// Returns true if we have a known exception to the "unique longest match" principle.
///
/// We want to be able to write that the priority-based system for choosing a successful rule gives
/// the same result as choosing the rule which matched the longest sequence of characters, with only
/// known exceptions.
///
/// At present the only exception is that an additional rule for a decimal integer literal may
/// succeed when the chosen rule is for a non-decimal float or integer literal (eg for `0x3` or
/// `0b1e2`).
///
/// 'best' is the pretoken from the highest-priority successful rule.
/// 'violators' are the pretokens from successful rules which are at least as long as 'best'.
fn is_exception_to_longest_match_principle(best: &Pretoken, violators: &Vec<Pretoken>) -> bool {
    fn is_decimal_integer_literal(pretoken: &Pretoken) -> bool {
        matches!(
            pretoken,
            Pretoken {
                data: PretokenData::IntegerDecimalLiteral { .. },
                ..
            }
        )
    }
    fn is_nondecimal_numeric_literal(pretoken: &Pretoken) -> bool {
        matches!(
            pretoken,
            Pretoken {
                data: PretokenData::IntegerBinaryLiteral { .. }
                    | PretokenData::IntegerOctalLiteral { .. }
                    | PretokenData::IntegerHexadecimalLiteral { .. }
                    | PretokenData::FloatLiteral { has_base: true, .. },
                ..
            }
        )
    }
    if is_nondecimal_numeric_literal(&best)
        && violators.len() == 1
        && is_decimal_integer_literal(&violators[0])
        && violators[0].char_length() == best.char_length()
    {
        return true;
    }
    false
}

fn describe_priority_violations(best: Pretoken, violators: Vec<Pretoken>) -> Vec<String> {
    let mut messages = vec![
        "matched multiple ways with surprising lengths".into(),
        "highest-priority match:".into(),
        format!("  {:?} {:?}", best.extent, &best.data),
        "other matches as long or longer:".into(),
    ];
    for pretoken in violators {
        messages.push(format!("  {:?} {:?}", pretoken.extent, pretoken.data));
    }
    messages
}

enum Rule {
    #[allow(unused)]
    Function(fn(&[char]) -> RuleOutcome),
    Regex {
        re: Regex,
        extract_data: fn(&Captures) -> PretokenData,
        forbidden_follower: Option<fn(char) -> bool>,
    },
    ConstrainedRegex {
        re: Regex,
        precheck_re: Regex,
        constraint: fn(&Captures) -> bool,
        extract_data: fn(&Captures) -> PretokenData,
    },
}

impl Rule {
    fn new_regex(extract_data: fn(&Captures) -> PretokenData, re: &str) -> Self {
        Self::Regex {
            re: pretokeniser_regex(re),
            extract_data,
            forbidden_follower: None,
        }
    }

    fn new_regex_with_forbidden_follower(
        extract_data: fn(&Captures) -> PretokenData,
        re: &str,
        forbidden_follower: fn(char) -> bool,
    ) -> Self {
        Self::Regex {
            re: pretokeniser_regex(re),
            extract_data,
            forbidden_follower: Some(forbidden_follower),
        }
    }

    fn new_constrained_regex(
        extract_data: fn(&Captures) -> PretokenData,
        constraint: fn(&Captures) -> bool,
        precheck_re: &str,
        re: &str,
    ) -> Self {
        Self::ConstrainedRegex {
            re: pretokeniser_regex(re),
            precheck_re: pretokeniser_regex(precheck_re),
            constraint,
            extract_data,
        }
    }

    fn apply(&self, input: &[char]) -> RuleOutcome {
        match self {
            Rule::Function(f) => f(input),
            Rule::Regex {
                re,
                extract_data,
                forbidden_follower,
            } => apply_regex_rule(re, *forbidden_follower, input, *extract_data),
            Rule::ConstrainedRegex {
                re,
                precheck_re,
                constraint,
                extract_data,
            } => apply_constrained_regex_rule(re, precheck_re, *constraint, input, *extract_data),
        }
    }
}

enum RuleOutcome {
    Success(usize, PretokenData),
    Failure,
    #[allow(unused)]
    ForceError(String),
}

/// Apply a "regex rule" (a rule which has no constraint) to the input.
///
/// If the rule succeeds, returns data extracted from the regex's captures,
/// as described by the `extract_data` callback.
///
/// The supplied regex must be anchored to the start of the haystack.
///
/// If a forbidden_follower function is provided and it accepts the character immediately following
/// successful regex match, the rule as a whole is considered not to succeed.
fn apply_regex_rule(
    re: &Regex,
    forbidden_follower: Option<fn(char) -> bool>,
    input: &[char],
    extract_data: fn(&Captures) -> PretokenData,
) -> RuleOutcome {
    let s: String = input.iter().collect();
    let Some(captures) = re.captures(&s) else {
        return RuleOutcome::Failure;
    };
    let mtch = captures.get(0).unwrap();
    assert!(mtch.start() == 0);
    let token_length = mtch.as_str().chars().count();
    if let Some(forbid) = forbidden_follower {
        if let Some(c) = input.get(token_length) {
            if forbid(*c) {
                return RuleOutcome::Failure;
            }
        }
    }
    RuleOutcome::Success(token_length, extract_data(&captures))
}

/// Applies a constrained regex rule to the input.
///
/// Finds the shortest maximal match (see writeup) of `re` in the input which satisfies the
/// constraint function.
///
/// First, checks the input matches precheck_re, and reports no-match if it doesn't.
/// (This is intended only to avoid slowness, and shouldn't affect the result.)
///
/// If the rule succeeds, returns data extracted from the regex's captures,
/// as described by the `extract_data` callback.
///
/// The supplied regex must be anchored to both ends of the haystack.
///
/// The constraint function is given the captures from a successful match of `re`. It must return
/// true iff the constraint is satisfied.
fn apply_constrained_regex_rule(
    re: &Regex,
    precheck_re: &Regex,
    constraint: fn(&Captures) -> bool,
    input: &[char],
    extract_data: fn(&Captures) -> PretokenData,
) -> RuleOutcome {
    let s: String = input.iter().collect();
    if !precheck_re.is_match(&s) {
        return RuleOutcome::Failure;
    }
    let Some(captures) = constrained_captures(re, constraint, &s) else {
        return RuleOutcome::Failure;
    };
    let mtch = captures.get(0).unwrap();
    assert!(mtch.start() == 0);
    let token_length = mtch.as_str().chars().count();
    RuleOutcome::Success(token_length, extract_data(&captures))
}

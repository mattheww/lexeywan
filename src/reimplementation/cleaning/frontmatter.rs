//! Find frontmatter using a PEG.
//!
//! See frontmatter.pest in this directory for the grammar this is using.

use std::ops::Range;

use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "reimplementation/cleaning/frontmatter.pest"]
struct FrontmatterParser;

/// Look for frontmatter at the start of the input.
///
/// May indicate that there is frontmatter to remove, or that the input should be rejected.
pub fn find_frontmatter(input: &[char]) -> FrontmatterOutcome {
    use FrontmatterOutcome::*;
    let s: String = input.iter().collect();
    match FrontmatterParser::parse(Rule::FRONTMATTER, &s) {
        Ok(mut pairs) => {
            let Some(pair) = pairs.next() else {
                return ModelError("FRONTMATTER matched with no contents");
            };
            let span = pair.as_span();
            if span.start() != 0 {
                return ModelError("FRONTMATTER matched after the start of the input");
            }
            Found(0..span.as_str().chars().count())
        }
        Err(_) => match FrontmatterParser::parse(Rule::RESERVED, &s) {
            Ok(_) => Reserved,
            Err(_) => NotFound,
        },
    }
}

/// Result of the search for frontmatter.
pub enum FrontmatterOutcome {
    /// Frontmatter wasn't found at the start of the input.
    NotFound,

    /// Frontmatter was found at the start of the input.
    ///
    /// The Range is the range of characters (not bytes) to remove.
    Found(Range<usize>),

    /// Frontmatter wasn't found at the start of the input, but a reserved frontmatter-like form was
    /// found.
    Reserved,

    /// The input demonstrated a problem in the reimplementation.
    ///
    /// The string is a description of the problem.
    ModelError(&'static str),
}

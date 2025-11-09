//! High-level support for comparing two analyses.

use crate::alignment::Verdict;

/// The result of comparing the output of two lexers.
pub enum Comparison {
    /// The two lexers produced equivalent verdicts.
    ///
    /// This means either they produced equal data (in practice, identical forests of tokens), or
    /// they both rejected the input (there's no attempt to check for equivalent reasons for
    /// rejection).
    Agree,

    /// The two lexers disagreed.
    ///
    /// This means either they produced different data, or one accepted the input but the other
    /// rejected it.
    Differ,

    /// One of the lexers reported a problem in its model or implementation.
    ModelErrors,
}

/// Compare the output of two lexers.
pub fn compare<T: Eq>(r1: &Verdict<T>, r2: &Verdict<T>) -> Comparison {
    use Comparison::*;
    use Verdict::*;
    match (r1, r2) {
        (Accepts(tokens1), Accepts(tokens2)) if tokens1 == tokens2 => Agree,
        (Accepts(_), Accepts(_)) => Differ,
        (Rejects(_), Rejects(_)) => Agree,
        (Accepts(_), Rejects(_)) => Differ,
        (Rejects(_), Accepts(_)) => Differ,
        _ => ModelErrors,
    }
}

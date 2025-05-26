//! Data type representing a sequence of characters.
//!
//! The debug representation indicates nonascii and control characters, in a way that won't be
//! confused with Rust escape notation.

use unicode_normalization::UnicodeNormalization;

/// A sequence of characters.
///
/// A `Charseq` can be indexed using any of the following forms:
///
///  - `charseq[idx]`
///  - `charseq[idx..]`
///  - `charseq[idx1..idx2]`
#[derive(PartialEq, Eq, Clone, Default)]
pub struct Charseq(Vec<char>);

impl Charseq {
    /// Returns a new `Charseq` representing the specified characters.
    pub fn new(chars: Vec<char>) -> Charseq {
        Charseq(chars)
    }

    /// Returns the number of characters in the sequence.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` iff the sequence is zero-length.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the sequence's characters.
    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.0.iter()
    }

    /// Returns `true` iff the specified character occurs in the sequence.
    pub fn contains(&self, c: &char) -> bool {
        self.0.contains(c)
    }

    /// Returns the sequence as a slice of `char`.
    pub fn chars(&self) -> &[char] {
        self.0.as_slice()
    }

    /// Converts to  Unicode Normalisation Form C.
    ///
    /// Returns a new character sequence.
    pub fn nfc(&self) -> Self {
        self.iter().copied().nfc().collect()
    }

    /// Removes the characters in the specified range from the sequence.
    pub fn remove_range<R>(&mut self, range: R)
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.0.drain(range);
    }
}

impl std::fmt::Display for Charseq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().collect::<String>())
    }
}

impl FromIterator<char> for Charseq {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<&[char]> for Charseq {
    fn from(chars: &[char]) -> Self {
        Self(chars.into())
    }
}

impl From<&str> for Charseq {
    fn from(s: &str) -> Self {
        Self(s.chars().collect())
    }
}

impl From<String> for Charseq {
    fn from(s: String) -> Self {
        Self(s.chars().collect())
    }
}

impl From<char> for Charseq {
    fn from(c: char) -> Self {
        Self(vec![c])
    }
}

impl<I> std::ops::Index<I> for Charseq
where
    I: std::slice::SliceIndex<[char]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl std::fmt::Debug for Charseq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«")?;
        for c in self.0.iter().copied() {
            if c.is_ascii_graphic() || c == ' ' || c == '·' {
                write!(f, "{c}")?;
            } else if (c as u32) < 256 {
                write!(f, "‹{:02X}›", c as u32)?;
            } else {
                write!(f, "‹{:04X}›", c as u32)?;
            }
        }
        write!(f, "»")?;
        Ok(())
    }
}

/// Returns a new `Charseq` containing the characters of `l1` followed by the characters of `l2`.
pub fn concat_charseqs(l1: &Charseq, l2: &Charseq) -> Charseq {
    let mut chars = l1.0.clone();
    chars.extend(l2.iter());
    Charseq(chars)
}

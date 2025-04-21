# Definitions

### Byte

For the purposes of this document, <dfn>byte</dfn> means the same thing as Rust's `u8`
(corresponding to a natural number in the range 0 to 255 inclusive).


### Character

For the purposes of this document, <dfn>character</dfn> means the same thing as Rust's `char`.
That means, in particular:

- there's exactly one character for each [Unicode scalar value]
- the things that Unicode calls "[noncharacters]" are characters
- there are no characters corresponding to surrogate code points

[Unicode scalar value]: https://unicode.org/glossary/#unicode_scalar_value
[noncharacters]: https://unicode.org/glossary/#noncharacter


### Sequence

When this document refers to a <dfn>sequence</dfn> of items, it means a finite, but possibly empty, ordered list of those items.

"character sequence" and "sequence of characters" are different ways of saying the same thing.


### NFC normalisation

References to <dfn>NFC-normalised</dfn> strings are talking about Unicode's Normalization Form C, defined in [Unicode Standard Annex #15][UAX15].


[UAX15]: https://www.unicode.org/reports/tr15/tr15-53.html

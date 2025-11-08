# Definitions

##### Table of contents
<!-- toc -->

### Unicode

References to <dfn>Unicode</dfn> in this document refer to the [Unicode standard, version 16.0][unicode-core].

References to the <dfn>Unicode character database</dfn> refer to [version 16.0.0][UCD].


#### NFC normalisation { #nfc }

References to <dfn>NFC-normalised</dfn> strings are talking about Unicode's Normalization Form C, defined in [Unicode Standard Annex #15][UAX15].


### Byte

For the purposes of this document, <dfn>byte</dfn> means the same thing as Rust's `u8`
(corresponding to a natural number in the range 0 to 255 inclusive).


### Character

For the purposes of this document, <dfn>character</dfn> means the same thing as Rust's `char`.
That means, in particular:

- there's exactly one character for each [Unicode scalar value]
- the things that Unicode calls "[noncharacters]" are characters
- there are no characters corresponding to surrogate code points
- there is a character for each unassigned code point

[Unicode scalar value]: https://unicode.org/glossary/#unicode_scalar_value
[noncharacters]: https://unicode.org/glossary/#noncharacter

#### Notation for characters

This document identifies characters in the following ways:

Printable ASCII characters other than space are represented by themselves
using highlighting like <b>a</b>.
For example <b>\\</b> represents character `U+005C` (REVERSE SOLIDUS).

ASCII control characters and space are represented as follows:

|          |                |
|----------|----------------|
| `U+0000` | <kbd>NUL</kbd> |
| `U+000A` | <kbd>LF</kbd>  |
| `U+000D` | <kbd>CR</kbd>  |
| `U+0009` | <kbd>HT</kbd>  |
| `U+0020` | <kbd>SP</kbd>  |

Other characters are identified by hexadecimal scalar value and name,
for example `U+FEFF` (BYTE ORDER MARK).


### Sequence

When this document refers to a <dfn>sequence</dfn> of items, it means a finite, but possibly empty, ordered list of those items.

"character sequence" and "sequence of characters" are different ways of saying the same thing.


[unicode-core]: https://www.unicode.org/versions/Unicode16.0.0/core-spec/
[UCD]: https://www.unicode.org/Public/16.0.0/ucd/
[UAX15]: https://www.unicode.org/reports/tr15/tr15-56.html

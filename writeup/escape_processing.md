## Escape processing

##### Table of contents
<!-- toc -->

### The escape-processing grammar { #escape-grammar }

The <dfn>escape-processing grammar</dfn> is the following [Parsing Expression Grammar](pegs.md):

```
{{#include escape_processing_anchored.pest:main}}
```

### Classifying escapes

A match of `LITERAL_COMPONENT` is:
 - a _non-escape_ if `ESCAPE_BODY` did not participate in the match
 - a _simple escape_ if `SIMPLE_ESCAPE_BODY` participated in the match
 - a _Unicode escape_ if `UNICODE_ESCAPE_BODY` participated in the match
 - a _hexadecimal escape_ if `HEXADECIMAL_ESCAPE_BODY` participated in the match
 - a _string continuation escape_ if `STRING_CONTINUATION_ESCAPE_BODY` participated in the match.

It follows from the definitions of `LITERAL_COMPONENT` AND `ESCAPE_BODY`
that each match of `LITERAL_COMPONENT` is exactly one of the above forms.


#### Non-escapes

The <dfn>represented character</dfn> of a non-escape is the single character consumed by the non-escape.

The <dfn>represented byte</dfn> of a non-escape
whose represented character has a [Unicode scalar value] that is less than 128
is that Unicode scalar value.
Other non-escapes have no represented byte.

> Note: this means a non-escape has a represented byte exactly when
> the UTF-8 encoding of its represented character is a single byte.


#### Simple escapes

> A simple escape is a form like `\n` or `\"`.
> Simple escapes are used to represent common control characters and characters that have special meaning in the tokenisation grammar.

The <dfn>represented character</dfn> of a simple escape is determined from the character consumed by the match of `SIMPLE_ESCAPE_BODY` that participated in the escape,
according to the table below.

| Simple escape body | Represented character    |
|--------------------|--------------------------|
| <b>0</b>           | U+0000 <kbd>NUL</kbd>    |
| <b>t</b>           | U+0009 <kbd>HT</kbd>     |
| <b>n</b>           | U+000A <kbd>LF</kbd>     |
| <b>r</b>           | U+000D <kbd>CR</kbd>     |
| <b>\"</b>          | U+0022 (QUOTATION MARK)  |
| <b>\'</b>          | U+0027 (APOSTROPHE)      |
| <b>\\</b>          | U+005C (REVERSE SOLIDUS) |

The <dfn>represented byte</dfn> of a simple escape is the [Unicode scalar value] of its represented character.


#### Unicode escapes

> A Unicode escape is a form like `\u{211d}` or `\u{01_F9_80}`.
> A Unicode escape can represent any single character.

The <dfn>digits</dfn> of a Unicode escape are
the characters consumed by the sequence of participating matches of `HEXADECIMAL_DIGIT` in the escape.

The <dfn>numeric value</dfn> of a Unicode escape is the result of interpreting the escape's digits as a hexadecimal integer,
as if by [`u32::from_str_radix`] with radix 16.

If a Unicode escape's numeric value is a [Unicode scalar value],
the <dfn>represented character</dfn> of the escape is the character with that Unicode scalar value.
Otherwise the Unicode escape has no represented character.


#### Hexadecimal escapes

> A hexadecimal escape is a form like `\xA0` or `\x1b`.
> In byte, byte-string, and C-string literals, a hexadecimal escape can represent any single byte.
> In character and string literals, a hexadecimal escape can represent any single ASCII character.

The <dfn>digits</dfn> of a hexadecimal escape are
the characters consumed by the sequence of participating matches of `HEXADECIMAL_DIGIT` in the escape.

The <dfn>represented byte</dfn> of a hexadecimal escape is the result of interpreting the escape's digits as a hexadecimal integer,
as if by [`u8::from_str_radix`] with radix 16.

The <dfn>represented character</dfn> of a hexadecimal escape
whose represented byte is less than 128
is the character whose [Unicode scalar value] is the escape's represented byte.
Other hexadecimal escapes have no represented character.

> Note: this means a hexadecimal escape has a represented character exactly when
> its represented byte is the UTF-8 encoding of a character.


#### String continuation escapes

> A string continuation escape is <b>\\</b> followed immediately by <kbd>LF</kbd>,
> optionally followed by some forms of additional whitespace
> (see [`STRING_CONTINUATION_ESCAPE_BODY`](#escape-grammar)).
> The escape is effectively removed from the literal content.
>
> The Reference says the whitespace-removal behaviour may change in future;
> see [String continuation escapes].


### Escape interpretations


##### Single-escape interpretation

If an attempt to match the `LITERAL_COMPONENT` nonterminal against a character sequence succeeds and consumes the entire sequence,
and the match is not a string continuation escape,
the <dfn>single-escape interpretation</dfn> of that character sequence is the resulting match.

Otherwise the character sequence has no single-escape interpretation.

> This means a single-escape interpretation is one of the forms described under [Classifying escapes] above,
> other than a string continuation escape.


##### Escape interpretation

If an attempt to match the `LITERAL_COMPONENTS` nonterminal against a character sequence succeeds and consumes the entire sequence,
the <dfn>escape interpretation</dfn> of that character sequence is
the [sequence of participating matches][participating] of `LITERAL_COMPONENT` in the resulting match, omitting any string continuation escapes.

Otherwise, the character sequence has no escape interpretation.

The individual matches in an escape interpretation are referred to as its <dfn>components</dfn>.

> This means the escape interpretation is a sequence of components,
> each of which has one of the forms described under [Classifying escapes] above,
> and it doesn't include any string continuation escapes.


[Classifying escapes]: #classifying-escapes

[participating]: pegs.md#participating

[Reference #1042]: https://github.com/rust-lang/reference/pull/1042
[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value
[`u8::from_str_radix`]: https://doc.rust-lang.org/std/primitive.u8.html#method.from_str_radix
[`u32::from_str_radix`]: https://doc.rust-lang.org/std/primitive.u32.html#method.from_str_radix
[String continuation escapes]: rustc_oddities.md#string-continuation-escapes


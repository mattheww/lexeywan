### Escape processing

##### Table of contents
<!-- toc -->

The descriptions of processing string and character literals make use of several forms of <dfn>escape</dfn>.

> The following table summarises which forms of escape are accepted in each kind of string or byte literal (raw literals don't use any forms of escape).
>
> | Kind  | Simple | 8-bit | 7-bit | Unicode | String continuation |
> |-------|--------|-------|-------|---------|---------------------|
> | `''`  | ✓      |       | ✓     | ✓       |                     |
> | `b''` | ✓      | ✓     |       |         |                     |
> | `""`  | ✓      |       | ✓     | ✓       | ✓                   |
> | `b""` | ✓      | ✓     |       |         | ✓                   |
> | `c""` | ✓      | ✓     |       | ✓       | ✓                   |


Each form of escape is characterised by:
- an <dfn>escape sequence</dfn>: a sequence of characters, which always begins with <b>\\</b>
- an <dfn>escaped value</dfn>: either a single character or an empty sequence of characters

In the definitions of escapes below:
- An <dfn>octal digit</dfn> is any of the characters in the range <b>0</b>..=<b>7</b>.
- A <dfn>hexadecimal digit</dfn> is any of the characters in the ranges <b>0</b>..=<b>9</b>, <b>a</b>..=<b>f</b>, or <b>A</b>..=<b>F</b>.


#### Simple escapes

Each sequence of characters occurring in the first column of the following table is an escape sequence.

In each case, the escaped value is the character given in the corresponding entry in the second column.

| Escape sequence | Escaped value          |
|-----------------|------------------------|
| <b>\0</b>       | U+0000 <kbd>NUL</kbd>  |
| <b>\t</b>       | U+0009 <kbd>HT</kbd>   |
| <b>\n</b>       | U+000A <kbd>LF</kbd>   |
| <b>\r</b>       | U+000D <kbd>CR</kbd>   |
| <b>\\"</b>      | U+0022 QUOTATION MARK  |
| <b>\\'</b>      | U+0027 APOSTROPHE      |
| <b>\\\\</b>     | U+005C REVERSE SOLIDUS |

> Note: The escaped value therefore has a [Unicode scalar value] which can be represented in a byte.


#### 8-bit escapes

The escape sequence consists of <b>\x</b> followed by two hexadecimal digits.

The escaped value is the character whose [Unicode scalar value] is the result of interpreting the final two characters in the escape sequence as a hexadecimal integer,
as if by [`u8::from_str_radix`] with radix 16.

> Note: The escaped value therefore has a [Unicode scalar value] which can be represented in a byte.


#### 7-bit escapes

The escape sequence consists of <b>\x</b> followed by an octal digit then a hexadecimal digit.

The escaped value is the character whose [Unicode scalar value] is the result of interpreting the final two characters in the escape sequence as a hexadecimal integer,
as if by [`u8::from_str_radix`] with radix 16.


#### Unicode escapes

The escape sequence consists of <b>\u{</b>,
followed by a hexadecimal digit,
followed by a sequence of characters each of which is a hexadecimal digit or <b>_</b>,
followed by <b>}</b>,
with the following restrictions:

 - there are no more than six hexadecimal digits in the entire escape sequence; and
 - the result of interpreting the hexadecimal digits contained in the escape sequence as a hexadecimal integer,
   as if by [`u32::from_str_radix`] with radix 16,
   is a [Unicode scalar value].

The escaped value is the character with that Unicode scalar value.


#### String continuation escapes

The escape sequence consists of <b>\\</b> followed immediately by <kbd>LF</kbd>,
and all following whitespace characters before the next non-whitespace character.

For this purpose, the whitespace characters are
<kbd>HT</kbd>, <kbd>LF</kbd>, <kbd>CR</kbd>, and <kbd>SP</kbd>.

The escaped value is an empty sequence of characters.

> The Reference says this behaviour may change in future; see [String continuation escapes].


[Reference #1042]: https://github.com/rust-lang/reference/pull/1042
[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value
[`u8::from_str_radix`]: https://doc.rust-lang.org/std/primitive.u8.html#method.from_str_radix
[`u32::from_str_radix`]: https://doc.rust-lang.org/std/primitive.u32.html#method.from_str_radix
[String continuation escapes]: open_questions.md#string-continuation-escapes


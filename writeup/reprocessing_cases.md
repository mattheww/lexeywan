##### Table of contents

[`Reserved`](#reserved)\
[`Whitespace`](#whitespace)\
[`LineComment`](#linecomment)\
[`BlockComment`](#blockcomment)\
[`Punctuation`](#punctuation)\
[`Identifier`](#identifier)\
[`RawIdentifier`](#rawidentifier)\
[`LifetimeOrLabel`](#lifetimeorlabel)\
[`SingleQuoteLiteral`](#singlequoteliteral)\
[`DoubleQuoteLiteral`](#doublequoteliteral)\
[`RawDoubleQuoteLiteral`](#rawdoublequoteliteral)\
[`IntegerLiteral`](#integerliteral)\
[`FloatLiteral`](#floatliteral)


### The list of of reprocessing cases

The list below has an entry for each kind of [pretoken],
describing what kind of [fine-grained token] it produces,
how the fine-grained token's attributes are determined,
and the circumstances under which a pretoken is rejected.

When an attribute value is given below as "copied",
it has the same value as the pretoken's attribute with the same name.


#### `Reserved` { .rcase }

A `Reserved` pretoken is always rejected.


#### `Whitespace` { .rcase }

Fine-grained token kind produced:
`Whitespace`

A `Whitespace` pretoken is always accepted.


#### `LineComment` { .rcase }

Fine-grained token kind produced:
`LineComment`

##### Attributes

<var>style</var> and <var>body</var> are determined from the pretoken's <var>comment content</var> as follows:

- if the <var>comment content</var> begins with <b>//</b>:
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

- otherwise, if the <var>comment content</var> begins with <b>/</b>,
  - <var>style</var> is **outer doc**
  - <var>body</var> is the characters from the <var>comment content</var> after that <b>/</b>

- otherwise, if the <var>comment content</var> begins with <b>!</b>,
  - <var>style</var> is **inner doc**
  - <var>body</var> is the characters from the <var>comment content</var> after that <b>!</b>

- otherwise
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty


The pretoken is rejected if (and only if):
- the <var>style</var> determined above is **inner doc** or **outer doc**; and
- the pretoken's <var>comment content</var> includes a <kbd>CR</kbd> character

> Note: the body of a non-doc comment is ignored


#### `BlockComment` { .rcase }

Fine-grained token kind produced:
`BlockComment`

##### Attributes

<var>style</var> and <var>body</var> are determined from the pretoken's <var>comment content</var> as follows:


- if the <var>comment content</var> begins with `**`:
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

- otherwise, if the <var>comment content</var> begins with `*` and contains at least one further character,
  - <var>style</var> is **outer doc**
  - <var>body</var> is the characters from the <var>comment content</var> after that `/`

- otherwise, if the <var>comment content</var> begins with `!`,
  - <var>style</var> is **inner doc**
  - <var>body</var> is the characters from the <var>comment content</var> after that `!`

- otherwise
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty


The pretoken is rejected if (and only if):
- the <var>style</var> determined above is **inner doc** or **outer doc**; and
- the pretoken's <var>comment content</var> includes a <kbd>CR</kbd> character

> Note: it follows that `/**/` and `/***/` are not doc-comments

> Note: the body of a non-doc comment is ignored


#### `Punctuation` { .rcase }

Fine-grained token kind produced:
`Punctuation`

A `Punctuation` pretoken is always accepted.


##### Attributes
<var>mark</var>: copied


#### `Identifier` { .rcase }

Fine-grained token kind produced:
`Identifier`

An `Identifier` pretoken is always accepted.


##### Attributes
<var>represented identifier</var>: NFC-normalised form of the pretoken's <var>identifier</var>


#### `RawIdentifier` { .rcase }

Fine-grained token kind produced:
`RawIdentifier`


##### Attributes
<var>represented identifier</var>: NFC-normalised form of the pretoken's <var>identifier</var>

The pretoken is rejected if (and only if) the <var>represented identifier</var> is one of the following sequences of characters:

- <b>_</b>
- <b>crate</b>
- <b>self</b>
- <b>super</b>
- <b>Self</b>


#### `LifetimeOrLabel` { .rcase }

Fine-grained token kind produced:
`LifetimeOrLabel`

A `LifetimeOrLabel` pretoken is always accepted.

##### Attributes
<var>name</var>: copied

> Note that the name is not NFC-normalised.
> See [NFC normalisation for lifetime/label].


#### `SingleQuoteLiteral` { .rcase }

The pretokeniser guarantees the pretoken's <var>prefix</var> attribute is one of the following:
- empty, in which case it is reprocessed as described under [Character literal](#character-literal)
- the single character <b>b</b>, in which case it is reprocessed as described under [Byte literal](#byte-literal).

In either case, the pretoken is rejected if its <var>suffix</var> consists of the single character <b>_</b>.

##### Character literal { .subcase }

Fine-grained token kind produced:
`CharacterLiteral`

##### Attributes

The <var>represented character</var> is derived from the pretoken's <var>literal content</var> as follows:

- If the <var>literal content</var> is one of the following forms of escape sequence,
  the <var>represented character</var> is the escape sequence's escaped value:
  - [Simple escapes]
  - [7-bit escapes]
  - [Unicode escapes]

- If the <var>literal content</var> begins with a <b>\\</b> character which did not introduce one of the above forms of escape,
the pretoken is rejected.

- Otherwise, if the single character that makes up the <var>literal content</var> is <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>TAB</kbd>,
the pretoken is rejected.

- Otherwise the <var>represented character</var> is the single character that makes up the <var>literal content</var>.

<var>suffix</var>: copied

> Note: The protokeniser guarantees the pretoken's <var>literal content</var> is either a single character,
> or a character sequence beginning with <b>\\</b>.


##### Byte literal { .subcase }

Fine-grained token kind produced:
`ByteLiteral`

##### Attributes

Define a <dfn>represented character</dfn>, derived from the pretoken's <var>literal content</var> as follows:

- If the literal content is one of the following forms of escape sequence,
  the represented character is the escape sequence's escaped value:
  - [Simple escapes]
  - [8-bit escapes]

- If the <var>literal content</var> begins with a <b>\\</b> character which did not introduce one of the above forms of escape,
the pretoken is rejected.

- Otherwise, if the single character that makes up the <var>literal content</var> is <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>TAB</kbd>,
the pretoken is rejected.

- Otherwise, if the single character that makes up the <var>literal content</var> has a unicode scalar value greater than 127,
the pretoken is rejected.


- Otherwise the represented character is the single character that makes up the literal content.

The <var>represented byte</var> is the represented character's [Unicode scalar value].

<var>suffix</var>: copied


> Note: The protokeniser guarantees the pretoken's <var>literal content</var> is either a single character,
> or a character sequence beginning with <b>\\</b>.


#### `DoubleQuoteLiteral` { .rcase }

The pretokeniser guarantees the pretoken's <var>prefix</var> attribute is one of the following:
- empty, in which case it is reprocessed as described under [String literal](#string-literal)
- the single character <b>b</b>, in which case it is reprocessed as described under [Byte-string literal](#byte-string-literal)
- the single character <b>c</b>, in which case it is reprocessed as described under [C-string literal](#c-string-literal)

In each case, the pretoken is rejected if its <var>suffix</var> consists of the single character <b>_</b>.


##### String literal { .subcase }

Fine-grained token kind produced:
`StringLiteral`

##### Attributes

The <var>represented string</var> is derived from the pretoken's <var>literal content</var> by
replacing each escape sequence of any of the following forms occurring in the <var>literal content</var> with the escape sequence's escaped value.
- [Simple escapes]
- [7-bit escapes]
- [Unicode escapes]
- [String continuation escapes]

These replacements take place in left-to-right order.
For example, the pretoken with extent `"\\x41"` is converted to the characters <b>\\</b> <b>x</b> <b>4</b> <b>1</b>.

If a <b>\\</b> character appears in the <var>literal content</var> but is not part of one of the above forms of escape,
the pretoken is rejected.

If a <kbd>CR</kbd> character appears in the <var>literal content</var> and is not part of a string continuation escape,
the pretoken is rejected.

<var>suffix</var>: copied

> See [Wording for string unescaping]


##### Byte-string literal { .subcase }

Fine-grained token kind produced:
`ByteStringLiteral`

If any character whose unicode scalar value is greater than 127 appears in the <var>literal content</var>, the pretoken is rejected.

##### Attributes

Define a <dfn>represented string</dfn> (a sequence of characters) derived from the pretoken's <var>literal content</var> by
replacing each escape sequence of any of the following forms occurring in the <var>literal content</var> with the escape sequence's escaped value.
- [Simple escapes]
- [8-bit escapes]
- [String continuation escapes]

These replacements take place in left-to-right order.
For example, the pretoken with extent `b"\\x41"` is converted to the characters <b>\\</b> <b>x</b> <b>4</b> <b>1</b>.

If a <b>\\</b> character appears in the <var>literal content</var> but is not part of one of the above forms of escape,
the pretoken is rejected.

If a <kbd>CR</kbd> character appears in the <var>literal content</var> and is not part of a string continuation escape,
the pretoken is rejected.

The <var>represented bytes</var> are the sequence of [Unicode scalar values] of the characters in the represented string.

<var>suffix</var>: copied

> See [Wording for string unescaping]


##### C-string literal { .subcase }

Fine-grained token kind produced:
`CStringLiteral`

##### Attributes

The pretoken's <var>literal content</var> is treated as a sequence of items,
each of which is either a single Unicode character other than <b>\\</b> or an [escape].

The sequence of items is converted to the <var>represented bytes</var> as follows:
- Each single Unicode character contributes its UTF-8 representation.
- Each [simple escape] contributes a single byte containing the [Unicode scalar value] of its escaped value.
- Each [8-bit escape] contributes a single byte containing the [Unicode scalar value] of its escaped value.
- Each [unicode escape] contributes the UTF-8 representation of its escaped value.
- Each [string continuation escape] contributes no bytes.

If a <b>\\</b> character appears in the <var>literal content</var> but is not part of one of the above forms of escape,
the pretoken is rejected.

If a <kbd>CR</kbd> character appears in the <var>literal content</var> and is not part of a string continuation escape,
the pretoken is rejected.

If any of the resulting <var>represented bytes</var> have value 0, the pretoken is rejected.

<var>suffix</var>: copied

> See [Wording for string unescaping]


#### `RawDoubleQuoteLiteral` { .rcase }


The pretokeniser guarantees the pretoken's <var>prefix</var> attribute is one of the following:
- the single character <b>r</b>, in which case it is reprocessed as described under [Raw string literal](#raw-string-literal)
- the characters <b>br</b>, in which case it is reprocessed as described under [Raw byte-string literal](#raw-byte-string-literal)
- the characters <b>cr</b>, in which case it is reprocessed as described under [Raw C-string literal](#raw-c-string-literal)

In each case, the pretoken is rejected if its <var>suffix</var> consists of the single character <b>_</b>.


##### Raw string literal { .subcase }

Fine-grained token kind produced:
`RawStringLiteral`

The pretoken is rejected if (and only if) a <kbd>CR</kbd> character appears in the <var>literal content</var>.


##### Attributes

<var>represented string</var>: the pretoken's <var>literal content</var>

<var>suffix</var>: copied


##### Raw byte-string literal { .subcase }

Fine-grained token kind produced:
`RawByteStringLiteral`

If any character whose unicode scalar value is greater than 127 appears in the <var>literal content</var>, the pretoken is rejected.

If a <kbd>CR</kbd> character appears in the <var>literal content</var>,
the pretoken is rejected.


##### Attributes

<var>represented bytes</var>: the sequence of [Unicode scalar values] of the characters in the pretoken's <var>literal content</var>

<var>suffix</var>: copied


##### Raw C-string literal { .subcase }

Fine-grained token kind produced:
`RawCStringLiteral`

If a <kbd>CR</kbd> character appears in the <var>literal content</var>,
the pretoken is rejected.


##### Attributes

<var>represented_bytes</var>: the UTF-8 encoding of the pretoken's <var>literal content</var>

<var>suffix</var>: copied

If any of the resulting <var>represented bytes</var> have value 0, the pretoken is rejected.


#### `IntegerLiteral` { .rcase }

Fine-grained token kind produced:
`IntegerLiteral`

The pretoken is rejected if its <var>digits</var> attribute consists entirely of <b>_</b> characters.

##### Attributes

<var>base</var>: determined from the pretoken's <var>base</var> attribute as follows:

- `0b` -> **binary**
- `0o` -> **octal**
- `0x` -> **hexadecimal**
- **none** -> **decimal**

<var>digits</var>: copied

<var>suffix</var>: copied

If the resulting <var>base</var> is **binary** and <var>digits</var> contains any character other than <b>0</b>, <b>1</b>, or <b>_</b>,
the pretoken is rejected.

If the resulting <var>base</var> is **octal** and <var>digits</var> contains any character other than <b>0</b>, <b>1</b>, <b>2</b>, <b>3</b>, <b>4</b>, <b>5</b>, <b>6</b>, <b>7</b>, or <b>_</b>,
the pretoken is rejected.

> Note: in particular, a `FloatLiteral` whose <var>digits</var> is empty is rejected.


#### `FloatLiteral` { .rcase }

Fine-grained token kind produced:
`FloatLiteral`

The pretoken is rejected if (and only if)
 - its <var>has base</var> attribute is **true**; or
 - its <var>exponent digits</var> attribute is a character sequence which consists entirely of <b>_</b> characters.

##### Attributes

<var>body</var>: copied

<var>suffix</var>: copied

> Note: in particular, a `FloatLiteral` whose <var>exponent digits</var> is empty is rejected.


[fine-grained token]: fine_grained_tokens.md
[pretoken]: pretokens.md
[escape]: escape_processing.md

[Simple escape]: escape_processing.md#simple-escapes
[Simple escapes]: escape_processing.md#simple-escapes
[8-bit escape]: escape_processing.md#8-bit-escapes
[8-bit escapes]: escape_processing.md#8-bit-escapes
[7-bit escape]: escape_processing.md#7-bit-escapes
[7-bit escapes]: escape_processing.md#7-bit-escapes
[Unicode escape]: escape_processing.md#unicode-escapes
[Unicode escapes]: escape_processing.md#unicode-escapes
[String continuation escape]: escape_processing.md#string-continuation-escapes
[String continuation escapes]: escape_processing.md#string-continuation-escapes

[Wording for string unescaping]: open_questions.md#wording-for-string-unescaping
[NFC normalisation for lifetime/label]: rustc_oddities.md#nfc-lifetime

[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value


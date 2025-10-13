## String and byte literal tokens

##### Table of contents
<!-- toc -->

### Single-quoted literals

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include tokenise_anchored.pest:single_quoted_literals_common}}

{{#include tokenise_anchored.pest:suffix}}
{{#include tokenise_anchored.pest:idents}}
```

#### Character literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:character_literal}}
```

##### Definitions

Define a <dfn>represented character</dfn>, derived from <u>SQ_CONTENT</u> as follows:

- If <u>SQ_CONTENT</u> is the single character <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>TAB</kbd>,
  the match is rejected.

- If <u>SQ_CONTENT</u> is any other single character,
  the represented character is that character.

- If <u>SQ_CONTENT</u> is one of the following forms of escape sequence,
  the represented character is the escape sequence's escaped value:
  - [Simple escapes]
  - [7-bit escapes]
  - [Unicode escapes]

- Otherwise the match is rejected

##### Attributes

The token's <var>represented character</var> is the represented character.

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>SQ_CONTENT</u> is unacceptable, as described in the definition of the represented character above; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Byte literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:byte_literal}}
```

##### Definitions

Define a <dfn>represented character</dfn>, derived from <u>SQ_CONTENT</u> as follows:

- If <u>SQ_CONTENT</u> is the single character <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>TAB</kbd>,
  the match is rejected.

- If <u>SQ_CONTENT</u> is a single character with [Unicode scalar value] greater than 127,
  the match is rejected.

- If <u>SQ_CONTENT</u> is any other single character,
  the represented character is that character.

- If <u>SQ_CONTENT</u> is one of the following forms of escape sequence,
  the represented character is the escape sequence's escaped value:
  - [Simple escapes]
  - [8-bit escapes]

- Otherwise the match is rejected

##### Attributes

The token's <var>represented byte</var> is the represented character's [Unicode scalar value].
(This is well defined because the definition above ensures that value is less than 256.)

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>SQ_CONTENT</u> is unacceptable, as described in the definition of the represented character above; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


### (Non-raw) double-quoted literals

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include tokenise_anchored.pest:double_quoted_literals_common}}

{{#include tokenise_anchored.pest:suffix}}
{{#include tokenise_anchored.pest:idents}}
```

#### String literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:string_literal}}
```

##### Attributes

The token's <var>represented string</var> is derived from <u>DQ_CONTENT</u> by
replacing each escape sequence of any of the following forms with the escape sequence's escaped value:
- [Simple escapes]
- [7-bit escapes]
- [Unicode escapes]
- [String continuation escapes]

These replacements take place in left-to-right order.
For example, a match against the characters `"\\x41"` is converted to the characters <b>\\</b> <b>x</b> <b>4</b> <b>1</b>.

> See [Wording for string unescaping]

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - a <b>\\</b> character appears in <u>DQ_CONTENT</u> and is not part of one of the above forms of escape; or
 - a <kbd>CR</kbd> character appears in <u>DQ_CONTENT</u> and is not part of a string continuation escape; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Byte-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:byte_string_literal}}
```

##### Definitions

Define a <dfn>represented string</dfn> (a sequence of characters) derived from <u>DQ_CONTENT</u> by
replacing each escape sequence of any of the following forms with the escape sequence's escaped value:
- [Simple escapes]
- [8-bit escapes]
- [String continuation escapes]

These replacements take place in left-to-right order.
For example, a match against the characters `b"\\x41"` is converted to the characters <b>\\</b> <b>x</b> <b>4</b> <b>1</b>.

> See [Wording for string unescaping]

##### Attributes

The token's <var>represented bytes</var> are the sequence of [Unicode scalar values] of the characters in the represented string.
(This is well defined because of the first rejection case below.)

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.


##### Rejection

The match is rejected if:
 - any character whose unicode scalar value is greater than 127 appears in <u>DQ_CONTENT</u>; or
 - a <b>\\</b> character appears in <u>DQ_CONTENT</u> and is not part of one of the above forms of escape; or
 - a <kbd>CR</kbd> character appears in <u>DQ_CONTENT</u> and is not part of a string continuation escape; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### C-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:c_string_literal}}
```

##### Attributes

<u>DQ_CONTENT</u> is treated as a sequence of items,
each of which is either a single Unicode character other than <b>\\</b> or an [escape].

The token's <var>represented bytes</var> are derived from that sequence of items in the following way:
- Each single Unicode character contributes its UTF-8 representation.
- Each [simple escape] or [8-bit escape] contributes a single byte containing the [Unicode scalar value] of its escaped value.
- Each [unicode escape] contributes the UTF-8 representation of its escaped value.
- Each [string continuation escape] contributes no bytes.

> See [Wording for string unescaping]

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - a <b>\\</b> character appears in <u>DQ_CONTENT</u> and is not part of one of the above forms of escape; or
 - a <kbd>CR</kbd> character appears in <u>DQ_CONTENT</u> and is not part of a string continuation escape; or
 - any of the token's <var>represented bytes</var> would have value 0; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


### Raw double-quoted literals { #rdql }

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include tokenise_anchored.pest:raw_double_quoted_literals_common}}

{{#include tokenise_anchored.pest:suffix}}
{{#include tokenise_anchored.pest:idents}}
```

These definitions require an extension to the Parsing Expression Grammar formalism:
each of the expressions marked as `HASHES²` fails unless the text it matches is the same as the text matched by the (only) successful match using the expression marked as `HASHES¹` in the same attempt to match the current token-kind nonterminal.

> See [Grammar for raw string literals](raw_strings.md) for a discussion of alternatives to this extension.


#### Raw string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:raw_string_literal}}
```

##### Attributes

The token's <var>represented string</var> is <u>RAW_DQ_CONTENT</u>.

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - a <kbd>CR</kbd> character appears in <u>RAW_DQ_CONTENT</u>; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Raw byte-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:raw_byte_string_literal}}
```

##### Attributes

The token's <var>represented bytes</var> are the [Unicode scalar values] of the characters in <u>RAW_DQ_CONTENT</u>.
(This is well defined because of the first rejection case below.)

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - any character whose unicode scalar value is greater than 127 appears in <u>RAW_DQ_CONTENT</u>; or
 - a <kbd>CR</kbd> character appears in <u>RAW_DQ_CONTENT</u>; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Raw C-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:raw_c_string_literal}}
```

##### Attributes

The token's <var>represented bytes</var> are the UTF-8 encoding of <u>RAW_DQ_CONTENT</u>

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - a <kbd>CR</kbd> character appears in <u>RAW_DQ_CONTENT</u>; or
 - any of the token's <var>represented bytes</var> would have value 0; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


### Reserved forms

#### Reserved or unterminated literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:unterminated_literal}}
```

##### Rejection

All matches are rejected.

> Note: I believe in the `Unterminated_literal_2015` definition only the `b'` form is strictly needed:
> if that definition matches using one of the other subexpressions
> then the input will be rejected eventually anyway
> (given that the corresponding string literal nonterminal didn't match).

> Note: `Reserved_literal_2021` catches both reserved forms and unterminated `b'` literals.


#### Reserved single-quoted literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:reserved_single_quoted_literal}}
```

##### Rejection

All matches are rejected.

> Note: This reservation is to catch forms like `'aaa'bbb`,
> so this definition must come before `Lifetime_or_label`.


#### Reserved guard (Rust 2024) { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:reserved_guard}}
```

##### Rejection

All matches are rejected.

> Note: This definition is listed here near the double-quoted string literals because these forms were reserved during discussions about introducing string literals formed like `#"…"#`.


[fine-grained token]: fine_grained_tokens.md
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

[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value


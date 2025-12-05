## Quoted literal tokens

##### Table of contents
<!-- toc -->

> Each kind of quoted literal represents a character, byte, sequence of characters, or sequence of bytes.
>
> These representations are obtained by interpreting the literal content
> (the consumed characters between <b>'</b> <b>'</b> or <b>"</b> <b>"</b>),
> which may contain several forms of _escape_
> (character sequences beginning with <b>\\</b>).
>
> The descriptions of processing the non-raw literals below are based on the [single-escape interpretation] and [escape interpretation] of the literal content,
> which are defined in [Escape processing].


### Summary

> The following table summarises which forms of character and escape are accepted in each kind of quoted literal.

<div class=escapes-table>

| Literal | Forbidden                                       | Simple | Unicode    | Hexadecimal | String continuation |
|---------|-------------------------------------------------|--------|------------|-------------|---------------------|
| `''`    | <kbd>CR</kbd> <kbd>LF</kbd> <kbd>HT</kbd>       | ✓      | ✓          | ✓ (<= 127)  |                     |
| `b''`   | <kbd>CR</kbd> <kbd>LF</kbd> <kbd>HT</kbd> > 127 | ✓      |            | ✓           |                     |
| `""`    | <kbd>CR</kbd>                                   | ✓      | ✓          | ✓ (<= 127)  | ✓                   |
| `b""`   | <kbd>CR</kbd> > 127                             | ✓      |            | ✓           | ✓                   |
| `c""`   | <kbd>CR</kbd>                                   | ✓      | ✓          | ✓           | ✓                   |
| `r""`   | <kbd>CR</kbd>                                   |        |            |             |                     |
| `br""`  | <kbd>CR</kbd> > 127                             |        |            |             |                     |
| `cr""`  | <kbd>CR</kbd>                                   |        |            |             |                     |
| Eg      |                                                 | `\n`   | `\u{2014}` | `\x1b`      |                     |
</div>

> The "Forbidden" column indicates which characters may not appear directly in the literal content;
> "> 127" means any character whose Unicode scalar value is greater than 127.
>
> The remaining columns indicate which [forms of escape][classifying escapes] are accepted.
>
> The "(<= 127)" annotation means that hexadecimal escapes whose first hexadecimal digit is greater than 7 aren't accepted.
>
> In raw literals the <b>\\</b> character represents itself; otherwise a <b>\\</b> that doesn't introduce an escape is forbidden.


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

##### Attributes

The token's <var>represented character</var> is the represented character of <u>SQ_CONTENT</u>'s [single-escape interpretation].

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>SQ_CONTENT</u> has no single-escape interpretation; or
 - <u>SQ_CONTENT</u>'s single-escape interpretation has no represented character; or
 - <u>SQ_CONTENT</u>'s single-escape interpretation is a [non-escape]
   whose represented character is <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>HT</kbd>; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Byte literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:byte_literal}}
```

##### Attributes

The token's <var>represented byte</var> is the represented byte of <u>SQ_CONTENT</u>'s [single-escape interpretation].

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>SQ_CONTENT</u> has no single-escape interpretation; or
 - <u>SQ_CONTENT</u>'s single-escape interpretation is any of the following:
   - a [non-escape] whose represented character is <kbd>LF</kbd>, <kbd>CR</kbd>, or <kbd>HT</kbd>
   - a [Unicode escape]; or
 - <u>SQ_CONTENT</u>'s single-escape interpretation has no represented byte; or
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

The token's <var>represented string</var> is the sequence made up of
the represented character of each component of <u>DQ_CONTENT</u>'s [escape interpretation].

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>DQ_CONTENT</u> has no escape interpretation; or
 - <u>DQ_CONTENT</u>'s escape interpretation contains any of the following
   - a component that has no represented character
   - a [non-escape] whose represented character is <kbd>CR</kbd>; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### Byte-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:byte_string_literal}}
```

##### Attributes

The token's <var>represented bytes</var> are
the represented byte of each component of <u>DQ_CONTENT</u>'s [escape interpretation].

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>DQ_CONTENT</u> has no escape interpretation; or
 - <u>DQ_CONTENT</u>'s escape interpretation contains any of the following:
   - a [non-escape] whose represented character is <kbd>CR</kbd>
   - a [Unicode escape]
   - a component that has no represented byte; or
 - the token's <var>suffix</var> would consist of the single character <b>_</b>.


#### C-string literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:c_string_literal}}
```

##### Attributes

The token's <var>represented bytes</var> are derived from <u>DQ_CONTENT</u>'s [escape interpretation] in the following way:
- Each [non-escape], [simple escape], or [Unicode escape] contributes the UTF-8 encoding of its represented character.
- Each [hexadecimal escape] contributes its represented byte.

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:
 - <u>DQ_CONTENT</u> has no escape interpretation; or
 - <u>DQ_CONTENT</u>'s escape interpretation contains any of the following:
   - a Unicode escape that has no represented character
   - a [non-escape] whose represented character is <kbd>CR</kbd>; or
 - any of the token's <var>represented bytes</var> would be 0; or
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
an attempt to match one of the parsing expressions marked as `HASHES²` fails
unless the characters it consumes are the same as the characters consumed by the (only) match of the expression marked as `HASHES¹` under the same match attempt of a token-kind nonterminal.

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
 - any character whose Unicode scalar value is greater than 127 appears in <u>RAW_DQ_CONTENT</u>; or
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
 - any of the token's <var>represented bytes</var> would be 0; or
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


[byte]: definitions.md#byte
[fine-grained token]: fine_grained_tokens.md
[escape processing]: escape_processing.md

[single-escape interpretation]: escape_processing.md#single-escape-interpretation
[escape interpretation]: escape_processing.md#escape-interpretation
[classifying escapes]: escape_processing.md#classifying-escapes

[non-escape]: escape_processing.md#non-escapes
[Simple escape]: escape_processing.md#simple-escapes
[Simple escapes]: escape_processing.md#simple-escapes
[hexadecimal escape]: escape_processing.md#hexadecimal-escapes
[hexadecimal escapes]: escape_processing.md#hexadecimal-escapes
[Unicode escape]: escape_processing.md#unicode-escapes
[Unicode escapes]: escape_processing.md#unicode-escapes

[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value


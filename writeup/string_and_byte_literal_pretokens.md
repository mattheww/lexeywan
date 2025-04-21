### String and byte literal pretokens

#### Single-quoted literal { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:single_quoted_literals}}
```

##### Pretoken kind
`SingleQuotedLiteral`

##### Attributes
|                            |                                 |
|:---------------------------|:--------------------------------|
| <var>prefix</var>          | from `SQ_PREFIX`                |
| <var>literal content</var> | from `SQ_CONTENT`               |
| <var>suffix</var>          | from `SUFFIX` (may be **none**) |


#### (Non-raw) double-quoted literal { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:double_quoted_literals}}
```

##### Pretoken kind
`DoubleQuotedLiteral`

##### Attributes
|                            |                                           |
|:---------------------------|:------------------------------------------|
| <var>prefix</var>          | from `DQ_PREFIX_2015` or `DQ_PREFIX_2021` |
| <var>literal content</var> | from `DQ_CONTENT`                         |
| <var>suffix</var>          | from `SUFFIX` (may be **none**)           |


#### Raw double-quoted literal { .rule #rdql }

##### Grammar
```
{{#include pretokenise_anchored.pest:raw_double_quoted_literals_top}}

RAW_DQ_REMAINDER = {
    HASHES¹ ~
    "\"" ~ RAW_DQ_CONTENT ~ "\"" ~
    HASHES² ~
    SUFFIX ?
}
RAW_DQ_CONTENT = {
    ( !("\"" ~ HASHES²) ~ ANY ) *
}
HASHES = { "#" {0, 255} }
```

These definitions require an extension to the Parsing Expression Grammar formalism:
each of the expressions marked as `HASHES²` fails unless the text it matches is the same as the text matched by the (only) successful match using the expression marked as `HASHES¹` in the same attempt to match the current pretoken nonterminal.

> See [Grammar for raw string literals](raw_strings.md) for a discussion of alternatives to this extension.


##### Pretoken kind
`RawDoubleQuotedLiteral`

##### Attributes
|                            |                                                   |
|:---------------------------|:--------------------------------------------------|
| <var>prefix</var>          | from `RAW_DQ_PREFIX_2015` or `RAW_DQ_PREFIX_2021` |
| <var>literal content</var> | from `RAW_DQ_CONTENT`                             |
| <var>suffix</var>          | from `SUFFIX` (may be **none**)                   |


#### Reserved or unterminated literal { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:unterminated_literal}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)

> Note: I believe in the `Unterminated_literal_2015` definition only the `b'` form is strictly needed:
> if that definition matches using one of the other subexpressions
> then the input will be rejected eventually anyway
> (given that the corresponding string literal nonterminal didn't match).

> Note: `Reserved_literal_2021` catches both reserved forms and unterminated `b'` literals.


#### Reserved guard (Rust 2024) { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:reserved_guard}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)

> Note: This definition is listed here near the double-quoted string literals because these forms were reserved during discussions about introducing string literals formed like `#"…"#`.


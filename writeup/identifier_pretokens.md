### Ident, lifetime, and label pretokens

This writeup uses the term <dfn>ident</dfn> to refer to a token that lexically has the form of an identifier,
including keywords and lone underscore.

> Note: the procedural macros system uses the name `Ident` to refer to what this writeup calls `Ident` and `RawIdent`.

Recall that the `IDENT` nonterminal is defined as follows:

##### Grammar
```
{{#include pretokenise_anchored.pest:ident}}
```

> Note: This is following the specification in [Unicode Standard Annex #31][UAX31] for Unicode version 16.0, with the addition of permitting underscore as the first character.


#### Raw lifetime or label (Rust 2021 and 2024) { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:raw_lifetime_or_label_2021}}
```

##### Pretoken kind
`RawLifetimeOrLabel`

##### Attributes
|                 |              |
|:----------------|:-------------|
| <var>name</var> | from `IDENT` |
|                 |              |


#### Reserved lifetime or label prefix (Rust 2021 and 2024) { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:reserved_lifetime_or_label_prefix_2021}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)


#### (Non-raw) lifetime or label { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:lifetime_or_label}}
```

##### Pretoken kind
`LifetimeOrLabel`

##### Attributes
|                 |              |
|:----------------|:-------------|
| <var>name</var> | from `IDENT` |

> Note: The `Reserved_single_quoted_literal` definitions make sure that forms like `'aaa'bbb` are not accepted.

> See [Modelling lifetimes and labels] for a discussion of why this model doesn't simply treat `'` as punctuation.

[Modelling lifetimes and labels]: rationale.md#modelling-lifetimes-and-labels


#### Raw ident { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:raw_ident}}
```

##### Pretoken kind
`RawIdent`

##### Attributes
|                       |              |
|:----------------------|:-------------|
| <var>ident</var> | from `IDENT` |


#### Reserved prefix { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:reserved_prefix}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)

> Note: This definition must appear here in priority order.
> Tokens added in future which match these reserved forms wouldn't necessarily be forms of identifier.


#### (Non-raw) ident { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:ident}}
```

##### Pretoken kind
`Ident`

##### Attributes
|                       |              |
|:----------------------|:-------------|
| <var>ident</var> | from `IDENT` |

> Note: The Reference adds the following when discussing identifiers:
> "Zero width non-joiner (ZWNJ U+200C) and zero width joiner (ZWJ U+200D) characters are not allowed in identifiers."
> Those characters don't have `XID_Start` or `XID_Continue`, so that's only informative text, not an additional constraint.

[UAX31]: https://www.unicode.org/reports/tr31/tr31-41.html

## Ident, lifetime, and label tokens

This writeup uses the term <dfn>ident</dfn> to refer to a token that lexically has the form of an identifier,
including keywords and lone underscore.

> Note: the procedural macros system uses the name `Ident` to refer to what this writeup calls `Ident` and `RawIdent`.

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include tokenise_anchored.pest:idents}}
```

> Note: This is following the specification in [Unicode Standard Annex #31][UAX31] for Unicode version 16.0, with the addition of permitting underscore as the first character.


#### Raw lifetime or label (Rust 2021 and 2024) { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:raw_lifetime_or_label_2021}}
```

##### Token kind produced
`RawLifetimeOrLabel`


##### Attributes

The token's <var>name</var> is <u>IDENT</u>.

> Note that the name is not NFC-normalised.
> See [NFC normalisation for lifetime/label].

##### Rejection

The match is rejected if <u>IDENT</u> is one of the following sequences of characters:

- <b>_</b>
- <b>crate</b>
- <b>self</b>
- <b>super</b>
- <b>Self</b>


#### Reserved lifetime or label prefix (Rust 2021 and 2024) { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:reserved_lifetime_or_label_prefix_2021}}
```

##### Rejection

All matches are rejected.


#### (Non-raw) lifetime or label { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:lifetime_or_label}}
```

##### Token kind produced
`LifetimeOrLabel`

> Note: The `Reserved_single_quoted_literal` definitions make sure that forms like `'aaa'bbb` are not accepted.

> See [Modelling lifetimes and labels] for a discussion of why this model doesn't simply treat `'` as punctuation.

[Modelling lifetimes and labels]: rationale.md#modelling-lifetimes-and-labels

##### Attributes

The token's <var>name</var> is <u>IDENT</u>.

> Note that the name is not NFC-normalised.
> See [NFC normalisation for lifetime/label].

##### Rejection

No matches are rejected.


#### Raw ident { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:raw_ident}}
```

##### Token kind produced
`RawIdent`

##### Attributes

The token's <var>represented ident</var> is the NFC-normalised form of <u>IDENT</u>.

##### Rejection

The match is rejected if the token's <var>represented ident</var> would be one of the following sequences of characters:

- <b>_</b>
- <b>crate</b>
- <b>self</b>
- <b>super</b>
- <b>Self</b>


#### Reserved prefix { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:reserved_prefix}}
```

##### Rejection

All matches are rejected.

> Note: This definition must appear here in priority order.
> Tokens added in future which match these reserved forms wouldn't necessarily be forms of identifier.


#### (Non-raw) ident { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:ident}}
```

##### Token kind produced
`Ident`

> Note: The Reference adds the following when discussing identifiers:
> "Zero width non-joiner (ZWNJ U+200C) and zero width joiner (ZWJ U+200D) characters are not allowed in identifiers."
> Those characters don't have `XID_Start` or `XID_Continue`, so that's only informative text, not an additional constraint.

##### Attributes

The token's <var>represented ident</var> is the NFC-normalised form of <u>IDENT</u>

##### Rejection

No matches are rejected.



[UAX31]: https://www.unicode.org/reports/tr31/tr31-41.html

[NFC normalisation for lifetime/label]: rustc_oddities.md#nfc-lifetime

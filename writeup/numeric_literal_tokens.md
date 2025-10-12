## Numeric literal tokens

##### Table of contents
<!-- toc -->

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include tokenise_anchored.pest:numeric_common}}

{{#include tokenise_anchored.pest:suffix}}
{{#include tokenise_anchored.pest:idents}}
```

#### Float literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:float_literal}}
```

> Note: The `! "."` subexpression makes sure that forms like `1..2` aren't treated as starting with a float.
> The `! IDENT_START` subexpression makes sure that forms like `1.some_method()` aren't treated as starting with a float.

##### Token kind produced
`FloatLiteral`

##### Attributes

The token's <var>body</var> is <u>FLOAT_BODY_WITH_EXPONENT</u>, <u>FLOAT_BODY_WITHOUT_EXPONENT</u>, or <u>FLOAT_BODY_WITH_FINAL_DOT</u>, whichever one participated in the match.

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

No matches are rejected.


#### Reserved float { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:reserved_float}}
```

##### Rejection

All matches are rejected.


#### Integer literal { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:integer_literals}}
```

> Note: See [rfc0879] for the reason we accept all decimal digits in binary and octal tokens;
> the inappropriate digits cause the token to be rejected.

> Note: The `INTEGER_DECIMAL_LITERAL` nonterminal is listed last in the `Integer_literal` definition in order to resolve ambiguous cases like the following:
> - `0b1e2` (which isn't `0` with suffix `b1e2`)
> - `0b0123` (which is rejected, not accepted as `0` with suffix `b0123`)
> - `0xy` (which is rejected, not accepted as `0` with suffix `xy`)
> - `0x·` (which is rejected, not accepted as `0` with suffix `x·`)


##### Token kind produced
`IntegerLiteral`


##### Attributes

The token's <var>base</var> is looked up in the following table,
depending on which nonterminal participated in the match:

|                               |                 |
|:------------------------------|:----------------|
| `INTEGER_BINARY_LITERAL`      | **binary**      |
| `INTEGER_OCTAL_LITERAL`       | **octal**       |
| `INTEGER_HEXADECIMAL_LITERAL` | **hexadecimal** |
| `INTEGER_DECIMAL_LITERAL`     | **decimal**     |

The token's <var>digits</var> are
<u>LOW_BASE_TOKEN_DIGITS</u>, <u>HEXADECIMAL_DIGITS</u>, or <u>DECIMAL_PART</u>,
whichever one participated in the match.

The token's <var>suffix</var> is <u>SUFFIX</u>, or empty if <u>SUFFIX</u> did not participate in the match.

##### Rejection

The match is rejected if:

- the token's <var>digits</var> would consist entirely of <b>_</b> characters; or
- the token's <var>base</var> would be **binary** and
  its <var>digits</var> would contain any character other than <b>0</b>, <b>1</b>, or <b>_</b>; or
- the token's <var>base</var> would be **octal** and
  its <var>digits</var> would contain any character other than <b>0</b>, <b>1</b>, <b>2</b>, <b>3</b>, <b>4</b>, <b>5</b>, <b>6</b>, <b>7</b>, or <b>_</b>.

> Note: In particular, a match which would make an `IntegerLiteral` with empty <var>digits</var> is rejected.


[rfc0879]: https://github.com/rust-lang/rfcs/pull/0879

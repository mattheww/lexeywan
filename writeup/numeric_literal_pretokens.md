### Numeric literal pretokens

The following nonterminals are common to the definitions below:

##### Grammar
```
{{#include pretokenise_anchored.pest:numeric_common}}
```

#### Float literal { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:float_literal}}
```

##### Pretoken kind
`FloatLiteral`

##### Attributes

|                   |                                                                                               |
|:------------------|:----------------------------------------------------------------------------------------------|
| <var>body</var>   | from `FLOAT_BODY_WITH_EXPONENT`,`FLOAT_BODY_WITHOUT_EXPONENT`, or `FLOAT_BODY_WITH_FINAL_DOT` |
| <var>suffix</var> | from `SUFFIX` (may be **none**)                                                               |

> Note: The `! "."` subexpression makes sure that forms like `1..2` aren't treated as starting with a float.
> The `! IDENT_START` subexpression makes sure that forms like `1.some_method()` aren't treated as starting with a float.


#### Reserved float { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:reserved_float}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)


#### Integer literal { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:integer_literals}}
```

##### Pretoken kind
`IntegerLiteral`

##### Attributes
|                   |                                                                          |
|:------------------|:-------------------------------------------------------------------------|
| <var>base</var>   | See below                                                                |
| <var>digits</var> | from `LOW_BASE_PRETOKEN_DIGITS`, `HEXADECIMAL_DIGITS`, or `DECIMAL_PART` |
| <var>suffix</var> | from `SUFFIX` (may be **none**)                                          |

The <var>base</var> attribute is determined from the following table, depending on which nonterminal participated in the match:

|                               |                 |
|:------------------------------|:----------------|
| `INTEGER_BINARY_LITERAL`      | **binary**      |
| `INTEGER_OCTAL_LITERAL`       | **octal**       |
| `INTEGER_HEXADECIMAL_LITERAL` | **hexadecimal** |
| `INTEGER_DECIMAL_LITERAL`     | **decimal**     |



> Note: See [rfc0879] for the reason we accept all decimal digits in binary and octal pretokens;
> the inappropriate digits are rejected in reprocessing.

> Note: The `INTEGER_DECIMAL_LITERAL` nonterminal is listed last in the `Integer_literal` definition in order to resolve ambiguous cases like the following:
> - `0b1e2` (which isn't `0` with suffix `b1e2`)
> - `0b0123` (which is rejected, not accepted as `0` with suffix `b0123`)
> - `0xy` (which is rejected, not accepted as `0` with suffix `xy`)
> - `0x·` (which is rejected, not accepted as `0` with suffix `x·`)

[rfc0879]: https://github.com/rust-lang/rfcs/pull/0879

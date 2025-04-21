### Whitespace and comment pretokens

#### Whitespace { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:whitespace}}
```

##### Pretoken kind
`Whitespace`

##### Attributes
(none)


#### Line comment { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:line_comment}}
```

##### Pretoken kind
`LineComment`

##### Attributes
|                            |                             |
|:---------------------------|:----------------------------|
| <var>comment content</var> | from `LINE_COMMENT_CONTENT` |
|                            |                             |


#### Block comment { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:block_comment}}
```

##### Pretoken kind
`BlockComment`

##### Attributes
|                            |                              |
|:---------------------------|:-----------------------------|
| <var>comment content</var> | from `BLOCK_COMMENT_CONTENT` |
|                            |                              |

> Note: See [Nested block comments] for discussion of the `!"/*"` subexpression.


#### Unterminated block comment { .rule }

##### Grammar
```
{{#include pretokenise_anchored.pest:unterminated_block_comment}}
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)

> Note: This definition makes sure that an unterminated block comment isn't accepted as punctuation (<b>*</b> followed by <b>/</b>).

[Nested block comments]: rustc_oddities.md#nested-block-comments

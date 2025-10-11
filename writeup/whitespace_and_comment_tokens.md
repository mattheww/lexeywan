## Whitespace and comment tokens

#### Whitespace { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:whitespace}}
```

##### Token kind produced
`Whitespace`

##### Attributes
(none)

##### Rejection

No matches are rejected.


#### Line comment { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:line_comment}}
```

##### Token kind produced
`LineComment`

##### Attributes

The token's <var>style</var> and <var>body</var> are determined from <u>LINE_COMMENT_CONTENT</u> as follows:

- if <u>LINE_COMMENT_CONTENT</u> begins with <b>//</b>:
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

- otherwise, if <u>LINE_COMMENT_CONTENT</u> begins with <b>/</b>,
  - <var>style</var> is **outer doc**
  - <var>body</var> is the characters from <u>LINE_COMMENT_CONTENT</u> after that <b>/</b>

- otherwise, if <u>LINE_COMMENT_CONTENT</u> begins with <b>!</b>,
  - <var>style</var> is **inner doc**
  - <var>body</var> is the characters from <u>LINE_COMMENT_CONTENT</u> after that <b>!</b>

- otherwise
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

> Note: The body of a non-doc comment is ignored by the rest of the compilation process

##### Rejection

The match is rejected if the token's <var>body</var> would include a <kbd>CR</kbd> character.


#### Block comment { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:block_comment}}
```

##### Token kind produced
`BlockComment`

> Note: See [Nested block comments] for discussion of the `!"/*"` subexpression.


##### Attributes

The token's <var>style</var> and <var>body</var> are determined from <u>BLOCK_COMMENT_CONTENT</u> as follows:


- if <u>BLOCK_COMMENT_CONTENT</u> begins with `**`:
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

- otherwise, if <u>BLOCK_COMMENT_CONTENT</u> begins with `*` and contains at least one further character,
  - <var>style</var> is **outer doc**
  - <var>body</var> is the characters from <u>BLOCK_COMMENT_CONTENT</u> after that `*`

- otherwise, if <u>BLOCK_COMMENT_CONTENT</u> begins with `!`,
  - <var>style</var> is **inner doc**
  - <var>body</var> is the characters from <u>BLOCK_COMMENT_CONTENT</u> after that `!`

- otherwise
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty


> Note: It follows that `/**/` and `/***/` are not doc-comments

> Note: The body of a non-doc comment is ignored by the rest of the compilation process

##### Rejection

The match is rejected if the token's <var>body</var> would include a <kbd>CR</kbd> character.


#### Unterminated block comment { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:unterminated_block_comment}}
```

##### Rejection

All matches are rejected.

> Note: This definition makes sure that an unterminated block comment isn't accepted as punctuation (<b>*</b> followed by <b>/</b>).

[Nested block comments]: rustc_oddities.md#nested-block-comments

## Whitespace and comment tokens

##### Table of contents
<!-- toc -->

#### Whitespace { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:whitespace}}
```

> See [Special terminals] for the definition of `PATTERN_WHITE_SPACE`.

[Special terminals]: grammars.md#special-terminals


##### Attributes
(none)

##### Rejection

No matches are rejected.


#### Line comment { .processing }

##### Grammar
```
{{#include tokenise_anchored.pest:line_comment}}
```

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
> Note: See [Nested block comments] for discussion of the `!"/*"` subexpression.


##### Attributes

The <dfn>comment content</dfn> is the sequence of characters consumed by the [first participating match][participating] (that is, the outermost match)
of `BLOCK_COMMENT_CONTENT` in the match.

The token's <var>style</var> and <var>body</var> are determined from the block comment content as follows:

- if the comment content begins with `**`:
  - <var>style</var> is **non-doc**
  - <var>body</var> is empty

- otherwise, if the comment content begins with `*` and contains at least one further character,
  - <var>style</var> is **outer doc**
  - <var>body</var> is the characters from the comment content after that `*`

- otherwise, if the comment content begins with `!`,
  - <var>style</var> is **inner doc**
  - <var>body</var> is the characters from the comment content after that `!`

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


[participating]: pegs.md#participating
[Nested block comments]: rustc_oddities.md#nested-block-comments

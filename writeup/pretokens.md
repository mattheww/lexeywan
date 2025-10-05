## Pretokens

Each pretoken has an *extent*, which is a sequence of characters taken from the input.

Each pretoken has a *kind*, and possibly also some attributes, as described in the tables below.

| Kind                     | Attributes                                                       |
|:-------------------------|:-----------------------------------------------------------------|
| `Reserved`               |                                                                  |
| `Whitespace`             |                                                                  |
| `LineComment`            | <var>comment content</var>                                       |
| `BlockComment`           | <var>comment content</var>                                       |
| `Punctuation`            | <var>mark</var>                                                  |
| `Ident`                  | <var>ident</var>                                                 |
| `RawIdent`               | <var>ident</var>                                                 |
| `LifetimeOrLabel`        | <var>name</var>                                                  |
| `RawLifetimeOrLabel`     | <var>name</var>                                                  |
| `SingleQuotedLiteral`    | <var>prefix</var>, <var>literal content</var>, <var>suffix</var> |
| `DoubleQuotedLiteral`    | <var>prefix</var>, <var>literal content</var>, <var>suffix</var> |
| `RawDoubleQuotedLiteral` | <var>prefix</var>, <var>literal content</var>, <var>suffix</var> |
| `IntegerLiteral`         | <var>base</var>, <var>digits</var>, <var>suffix</var>            |
| `FloatLiteral`           | <var>body</var>, <var>suffix</var>                               |

These attributes have the following types:

| Attribute                  | Type                                                   |
|:---------------------------|:-------------------------------------------------------|
| <var>base</var>            | **binary** / **octal** / **decimal** / **hexadecimal** |
| <var>body</var>            | sequence of characters                                 |
| <var>comment content</var> | sequence of characters                                 |
| <var>digits</var>          | sequence of characters                                 |
| <var>ident</var>           | sequence of characters                                 |
| <var>literal content</var> | sequence of characters                                 |
| <var>mark</var>            | single character                                       |
| <var>name</var>            | sequence of characters                                 |
| <var>prefix</var>          | sequence of characters                                 |
| <var>suffix</var>          | either a sequence of characters or **none**            |


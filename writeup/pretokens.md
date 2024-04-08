### Pretokens

Each pretoken has an *extent*, which is a sequence of characters taken from the input.

Each pretoken has a *kind*, and possibly also some attributes, as described in the tables below.

| Kind                    | Attributes                                                                          |
|:------------------------|:------------------------------------------------------------------------------------|
| `Reserved`              |                                                                                     |
| `Whitespace`            |                                                                                     |
| `LineComment`           | <var>comment content</var>                                                          |
| `BlockComment`          | <var>comment content</var>                                                          |
| `Punctuation`           | <var>mark</var>                                                                     |
| `Identifier`            | <var>identifier</var>                                                               |
| `RawIdentifier`         | <var>identifier</var>                                                               |
| `LifetimeOrLabel`       | <var>name</var>                                                                     |
| `SingleQuoteLiteral`    | <var>prefix</var>, <var>literal content</var>, <var>suffix</var>                    |
| `DoubleQuoteLiteral`    | <var>prefix</var>, <var>literal content</var>, <var>suffix</var>                    |
| `RawDoubleQuoteLiteral` | <var>prefix</var>, <var>literal content</var>, <var>suffix</var>                    |
| `IntegerLiteral`        | <var>base</var>, <var>digits</var>, <var>suffix</var>                               |
| `FloatLiteral`          | <var>has base</var>, <var>body</var>, <var>exponent digits</var>, <var>suffix</var> |

These attributes have the following types:

| Attribute                  | Type                                         |
|:---------------------------|:---------------------------------------------|
| <var>base</var>            | either a sequence of characters, or **none** |
| <var>body</var>            | sequence of characters                       |
| <var>digits</var>          | sequence of characters                       |
| <var>exponent digits</var> | either a sequence of characters, or **none** |
| <var>has base</var>        | **true** or **false**                        |
| <var>identifier</var>      | sequence of characters                       |
| <var>literal content</var> | sequence of characters                       |
| <var>comment content</var> | sequence of characters                       |
| <var>mark</var>            | single character                             |
| <var>name</var>            | sequence of characters                       |
| <var>prefix</var>          | sequence of characters                       |
| <var>suffix</var>          | sequence of characters                       |


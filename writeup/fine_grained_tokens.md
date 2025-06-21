## Fine-grained tokens

Reprocessing produces <dfn>fine-grained tokens</dfn>.

Each fine-grained token has a <dfn>kind</dfn>, and possibly also some attributes, as described in the tables below.

| Kind                   | Attributes                                            |
|:-----------------------|:------------------------------------------------------|
| `Whitespace`           |                                                       |
| `LineComment`          | <var>style</var>, <var>body</var>                     |
| `BlockComment`         | <var>style</var>, <var>body</var>                     |
| `Punctuation`          | <var>mark</var>                                       |
| `Identifier`           | <var>represented identifier</var>                     |
| `RawIdentifier`        | <var>represented identifier</var>                     |
| `LifetimeOrLabel`      | <var>name</var>                                       |
| `RawLifetimeOrLabel`   | <var>name</var>                                       |
| `CharacterLiteral`     | <var>represented character</var>, <var>suffix</var>   |
| `ByteLiteral`          | <var>represented byte</var>, <var>suffix</var>        |
| `StringLiteral`        | <var>represented string</var>, <var>suffix</var>      |
| `RawStringLiteral`     | <var>represented string</var>, <var>suffix</var>      |
| `ByteStringLiteral`    | <var>represented bytes</var>, <var>suffix</var>       |
| `RawByteStringLiteral` | <var>represented bytes</var>, <var>suffix</var>       |
| `CStringLiteral`       | <var>represented bytes</var>, <var>suffix</var>       |
| `RawCStringLiteral`    | <var>represented bytes</var>, <var>suffix</var>       |
| `IntegerLiteral`       | <var>base</var>, <var>digits</var>, <var>suffix</var> |
| `FloatLiteral`         | <var>body</var>, <var>suffix</var>                    |

These attributes have the following types:

| Attribute                         | Type                                                   |
|:----------------------------------|:-------------------------------------------------------|
| <var>base</var>                   | **binary** / **octal** / **decimal** / **hexadecimal** |
| <var>body</var>                   | sequence of characters                                 |
| <var>digits</var>                 | sequence of characters                                 |
| <var>mark</var>                   | single character                                       |
| <var>name</var>                   | sequence of characters                                 |
| <var>represented byte</var>       | single byte                                            |
| <var>represented bytes</var>      | sequence of bytes                                      |
| <var>represented character</var>  | single character                                       |
| <var>represented identifier</var> | sequence of characters                                 |
| <var>represented string</var>     | sequence of characters                                 |
| <var>style</var>                  | **non-doc** / **inner doc** / **outer doc**            |
| <var>suffix</var>                 | sequence of characters                                 |


### Notes:

At this stage:

- Both <b>_</b> and keywords are treated as instances of `Identifier`.
- There are explicit tokens representing whitespace and comments.
- Single-character tokens are used for all punctuation.
- A lifetime (or label) is represented as a single token
  (which includes the leading <b>'</b>).


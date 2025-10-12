## Fine-grained tokens

Tokenising produces <dfn>fine-grained tokens</dfn>.

Each fine-grained token has a <dfn>kind</dfn>,
which is the name of one of the token-kind nonterminals.
Most kinds of fine-grained token also have <dfn>attributes</dfn>,
as described in the tables below.

| Kind                      | Attributes                                            |
|:--------------------------|:------------------------------------------------------|
| `Whitespace`              |                                                       |
| `Line_comment`            | <var>style</var>, <var>body</var>                     |
| `Block_comment`           | <var>style</var>, <var>body</var>                     |
| `Punctuation`             | <var>mark</var>                                       |
| `Ident`                   | <var>represented ident</var>                          |
| `Raw_ident`               | <var>represented ident</var>                          |
| `Lifetime_or_label`       | <var>name</var>                                       |
| `Raw_lifetime_or_label`   | <var>name</var>                                       |
| `Character_literal`       | <var>represented character</var>, <var>suffix</var>   |
| `Byte_literal`            | <var>represented byte</var>, <var>suffix</var>        |
| `String_literal`          | <var>represented string</var>, <var>suffix</var>      |
| `Raw_string_literal`      | <var>represented string</var>, <var>suffix</var>      |
| `Byte_string_literal`     | <var>represented bytes</var>, <var>suffix</var>       |
| `Raw_byte_string_literal` | <var>represented bytes</var>, <var>suffix</var>       |
| `C_string_literal`        | <var>represented bytes</var>, <var>suffix</var>       |
| `Raw_c_string_literal`    | <var>represented bytes</var>, <var>suffix</var>       |
| `Integer_literal`         | <var>base</var>, <var>digits</var>, <var>suffix</var> |
| `Float_literal`           | <var>body</var>, <var>suffix</var>                    |

> Note: Some token-kind nonterminals do not appear in this table.
> These are the <i>reserved forms</i>, whose matches are always rejected.
> The names of reserved forms begin with `Reserved_` or `Unterminated_`.


These attributes have the following types:

| Attribute                        | Type                                                   |
|:---------------------------------|:-------------------------------------------------------|
| <var>base</var>                  | **binary** / **octal** / **decimal** / **hexadecimal** |
| <var>body</var>                  | sequence of characters                                 |
| <var>digits</var>                | sequence of characters                                 |
| <var>mark</var>                  | single character                                       |
| <var>name</var>                  | sequence of characters                                 |
| <var>represented byte</var>      | single byte                                            |
| <var>represented bytes</var>     | sequence of bytes                                      |
| <var>represented character</var> | single character                                       |
| <var>represented ident</var>     | sequence of characters                                 |
| <var>represented string</var>    | sequence of characters                                 |
| <var>style</var>                 | **non-doc** / **inner doc** / **outer doc**            |
| <var>suffix</var>                | sequence of characters                                 |


> Note: At this stage
>
> - Both <b>_</b> and keywords are treated as instances of `Ident`.
> - There are explicit tokens representing whitespace and comments.
> - Single-character tokens are used for all punctuation.
> - A lifetime (or label) is represented as a single token
>   (which includes the leading <b>'</b>).


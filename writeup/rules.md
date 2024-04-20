##### Table of contents

[Whitespace](#whitespace)\
[Line comment](#line-comment)\
[Block comment](#block-comment)\
[Unterminated block comment](#unterminated-block-comment)\
[Punctuation](#punctuation)\
[Single-quoted literal](#single-quoted-literal)\
[Lifetime or label](#lifetime-or-label)\
[Double-quoted non-raw literal (Rust 2015 and 2018)](#double-quoted-non-raw-literal-rust-2015-and-2018)\
[Double-quoted non-raw literal (Rust 2021)](#double-quoted-non-raw-literal-rust-2021)\
[Double-quoted hashless raw literal (Rust 2015 and 2018)](#double-quoted-hashless-raw-literal-rust-2015-and-2018)\
[Double-quoted hashless raw literal (Rust 2021)](#double-quoted-hashless-raw-literal-rust-2021)\
[Double-quoted hashed raw literal (Rust 2015 and 2018)](#double-quoted-hashed-raw-literal-rust-2015-and-2018)\
[Double-quoted hashed raw literal (Rust 2021)](#double-quoted-hashed-raw-literal-rust-2021)\
[Float literal with exponent](#float-literal-with-exponent)\
[Float literal without exponent](#float-literal-without-exponent)\
[Float literal with final dot](#float-literal-with-final-dot)\
[Integer binary literal](#integer-binary-literal)\
[Integer octal literal](#integer-octal-literal)\
[Integer hexadecimal literal](#integer-hexadecimal-literal)\
[Integer decimal literal](#integer-decimal-literal)\
[Raw identifier](#raw-identifier)\
[Unterminated literal (Rust 2015 and 2018)](#unterminated-literal-rust-2015-and-2018)\
[Reserved prefix or unterminated literal (Rust 2021)](#reserved-prefix-or-unterminated-literal-rust-2021)\
[Non-raw identifier](#non-raw-identifier)


### The list of pretokenisation rules

The list of pretokenisation rules is given below.

Rules whose names indicate one or more editions are included in the list only when one of those editions is in effect.

Unless otherwise stated, a rule has no constraint and has an empty set of forbidden followers.

When an attribute value is given below as "captured characters",
the value of that attribute is the sequence of characters captured by the capture group in the pattern whose name is the same as the attribute's name.


#### Whitespace { .rule }

##### Pattern
```
[ \p{Pattern_White_Space} ] +
```

##### Pretoken kind
`Whitespace`

##### Attributes
(none)


#### Line comment { .rule }

##### Pattern
```
/ /
(?<comment_content>
  [^ \n] *
)
```
##### Pretoken kind
`LineComment`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>comment content</var> | captured characters |


#### Block comment { .rule }

##### Pattern
```
/ \*
(?<comment_content>
  . *
)
\* /
```

##### Constraint

The constraint is satisfied if (and only if) the following block of Rust code evaluates to `true`,
when `character_sequence` represents an iterator over the sequence of characters being tested against the constraint.

````rust
{
    let mut depth = 0_isize;
    let mut after_slash = false;
    let mut after_star = false;
    for c in character_sequence {
        match c {
            '*' if after_slash => {
                depth += 1;
                after_slash = false;
            }
            '/' if after_star => {
                depth -= 1;
                after_star = false;
            }
            _ => {
                after_slash = c == '/';
                after_star = c == '*';
            }
        }
    }
    depth == 0
}
````

##### Pretoken kind
`BlockComment`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>comment content</var> | captured characters |


> See also [Defining the block-comment constraint][block-comment-constraint]


#### Unterminated block comment { .rule }

##### Pattern
```
/ \*
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)


#### Punctuation { .rule }

##### Pattern
```
[
  ; , \. \( \) \{ \} \[ \] @ \# ~ \? : \$ = ! < > \- & \| \+ \* / ^ %
]
```

##### Pretoken kind
`Punctuation`

##### Attributes
|                 |                                             |
|:----------------|:--------------------------------------------|
| <var>mark</var> | the single character matched by the pattern |

> Note: When this pattern matches, the matched character sequence is necessarily one character long.


#### Single-quoted literal { .rule }

##### Pattern
```
(?<prefix>
  b ?
)
'
(?<literal_content>
  [^ \\ ' ]
|
  \\ . [^']*
)
'
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`SingleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |


#### Lifetime or label { .rule }

##### Pattern
```
'
(?<%name>
 [ \p{XID_Start} _ ]
 \p{XID_Continue} *
)
```

Forbidden followers:

- The character <b>'</b>

##### Pretoken kind
`LifetimeOrLabel`

##### Attributes
|                 |                     |
|:----------------|:--------------------|
| <var>name</var> | captured characters |

> Note: the forbidden follower here makes sure that forms like `aaa'bbb` are not accepted.


#### Double-quoted non-raw literal (Rust 2015 and 2018) { .rule }

##### Pattern
```
(?<prefix>
  b ?
)
"
(?<literal_content>
  (?:
    [^ \\ " ]
  |
    \\ .
  ) *
)
"
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`DoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |


#### Double-quoted non-raw literal (Rust 2021) { .rule }

##### Pattern
```
(?<prefix>
  [bc] ?
)
"
(?<literal_content>
  (?:
    [^ \\ " ]
  |
    \\ .
  ) *
)
"
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`DoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |

> Note: the difference between the 2015/2018 and 2021 patterns is that the 2021 pattern allows `c` as a prefix.


#### Double-quoted hashless raw literal (Rust 2015 and 2018) { .rule }

##### Pattern
```
(?<prefix>
  r | br
)
"
(?<literal_content>
  [^"] *
)
"
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`RawDoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |


#### Double-quoted hashless raw literal (Rust 2021) { .rule }

##### Pattern
```
(?<prefix>
  r | br | cr
)
"
(?<literal_content>
  [^"] *
)
"
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`RawDoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |

> Note: the difference between the 2015/2018 and 2021 patterns is that the 2021 pattern allows `cr` as a prefix.

> Note: we can't treat the hashless rule as a special case of the hashed one because the "shortest maximal match" rule doesn't work without hashes (consider `r"x""`).


#### Double-quoted hashed raw literal (Rust 2015 and 2018) { .rule }

##### Pattern
```
(?<prefix>
  r | br
)
(?<hashes_1>
  \# {1,255}
)
"
(?<literal_content>
  . *
)
"
(?<hashes_2>
  \# {1,255}
)
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Constraint

The constraint is satisfied if (and only if) the character sequence captured by the `hashes_1` capture group is equal to the character sequence captured by the `hashes_2` capture group.

##### Pretoken kind
`RawDoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |


#### Double-quoted hashed raw literal (Rust 2021) { .rule }

##### Pattern
```
(?<prefix>
  r | br | cr
)
(?<hashes_1>
  \# {1,255}
)
"
(?<literal_content>
  . *
)
"
(?<hashes_2>
  \# {1,255}
)
(?<suffix>
  (?:
    [ \p{XID_Start} _ ]
    \p{XID_Continue} *
  ) ?
)
```

##### Constraint

The constraint is satisfied if (and only if) the character sequence captured by the `hashes_1` capture group is equal to the character sequence captured by the `hashes_2` capture group.

##### Pretoken kind
`RawDoubleQuoteLiteral`

##### Attributes
|                            |                     |
|:---------------------------|:--------------------|
| <var>prefix</var>          | captured characters |
| <var>literal content</var> | captured characters |
| <var>suffix</var>          | captured characters |

> Note: the difference between the 2015/2018 and 2021 patterns is that the 2021 pattern allows `cr` as a prefix.


#### Float literal with exponent { .rule }

##### Pattern
```
(?<body>
  (?:
    (?<based>
      (?: 0b | 0o )
      [ 0-9 _ ] *
    )
  |
    [ 0-9 ]
    [ 0-9 _ ] *
  )
  (?:
    \.
    [ 0-9 ]
    [ 0-9 _ ] *
  )?
  [eE]
  [+-] ?
  (?<exponent_digits>
    [ 0-9 _ ] *
  )
)
(?<suffix>
  (?:
    [ \p{XID_Start} ]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`FloatLiteral`

##### Attributes

|                            |                                                                                         |
|:---------------------------|:----------------------------------------------------------------------------------------|
| <var>has base</var>        | **true** if the `based` capture group participates in the match,<br>**false** otherwise |
| <var>body</var>            | captured characters                                                                     |
| <var>exponent digits</var> | captured characters                                                                     |
| <var>suffix</var>          | captured characters                                                                     |


#### Float literal without exponent { .rule }

##### Pattern
```
(?<body>
  (?:
    (?<based>
      (?: 0b | 0o )
      [ 0-9 _ ] *
    |
      0x
      [ 0-9 a-f A-F _ ] *
    )
  |
    [ 0-9 ]
    [ 0-9 _ ] *
  )
  \.
  [ 0-9 ]
  [ 0-9 _ ] *
)
(?<suffix>
  (?:
    [ \p{XID_Start} -- eE]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`FloatLiteral`

##### Attributes
|                            |                                                                                         |
|:---------------------------|:----------------------------------------------------------------------------------------|
| <var>has base</var>        | **true** if the `based` capture group participates in the match,<br>**false** otherwise |
| <var>body</var>            | captured characters                                                                     |
| <var>exponent digits</var> | **none**                                                                                |
| <var>suffix</var>          | captured characters                                                                     |


#### Float literal with final dot { .rule }

##### Pattern
```
(?:
  (?<based>
    (?: 0b | 0o )
    [ 0-9 _ ] *
  |
    0x
    [ 0-9 a-f A-F _ ] *
  )
|
  [ 0-9 ]
  [ 0-9 _ ] *
)
\.
```

Forbidden followers:

- The character <b>_</b>
- The character <b>.</b>
- The characters with the Unicode property `XID_start`

##### Pretoken kind
`FloatLiteral`

##### Attributes
|                            |                                                                                         |
|:---------------------------|:----------------------------------------------------------------------------------------|
| <var>has base</var>        | **true** if the `based` capture group participates in the match,<br>**false** otherwise |
| <var>body</var>            | the entire character sequence matched by the pattern                                    |
| <var>exponent digits</var> | **none**                                                                                |
| <var>suffix</var>          | empty character sequence                                                                |


#### Integer binary literal { .rule }

##### Pattern
```
0b
(?<digits>
  [ 0-9 _ ] *
)
(?<suffix>
  (?:
    [ \p{XID_Start} -- eE]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`IntegerBinaryLiteral`

##### Attributes
|                   |                     |
|:------------------|:--------------------|
| <var>digits</var> | captured characters |
| <var>suffix</var> | captured characters |


#### Integer octal literal { .rule }

##### Pattern
```
0o
(?<digits>
  [ 0-9 _ ] *
)
(?<suffix>
  (?:
    [ \p{XID_Start} -- eE]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`IntegerOctalLiteral`

##### Attributes
|                   |                     |
|:------------------|:--------------------|
| <var>digits</var> | captured characters |
| <var>suffix</var> | captured characters |


#### Integer hexadecimal literal { .rule }

##### Pattern
```
0x
(?<digits>
  [ 0-9 a-f A-F _ ] *
)
(?<suffix>
  (?:
    [ \p{XID_Start} -- aAbBcCdDeEfF]
    \p{XID_Continue} *
  ) ?
)
```

##### Pretoken kind
`IntegerHexadecimalLiteral`

##### Attributes
|                   |                     |
|:------------------|:--------------------|
| <var>digits</var> | captured characters |
| <var>suffix</var> | captured characters |


#### Integer decimal literal { .rule }

##### Pattern
```
(?<digits>
  [ 0-9 ]
  [ 0-9 _ ] *
)
(?<suffix>
  (?:
    [ \p{XID_Start} -- eE]
    \p{XID_Continue} *
  ) ?
)
```
|                   |                     |
|:------------------|:--------------------|
| <var>digits</var> | captured characters |
| <var>suffix</var> | captured characters |


##### Pretoken kind
`IntegerDecimalLiteral`

##### Attributes


> Note: it is important that this rule has lower priority than the other numeric literal rules.
> See [Integer literal base-vs-suffix ambiguity][base-vs-suffix].


#### Raw identifier { .rule }

##### Pattern
```
r \#
(?<identifier>
  [ \p{XID_Start} _ ]
  \p{XID_Continue} *
)
```

##### Pretoken kind
`RawIdentifier`

##### Attributes
|                       |                     |
|:----------------------|:--------------------|
| <var>identifier</var> | captured characters |


#### Unterminated literal (Rust 2015 and 2018) { .rule }

##### Pattern
```
( r \# | b r \# | r " | b r " | b ' )
```

> Note: I believe the double-quoted forms here aren't strictly needed: if this rule is chosen when its pattern matched via one of those forms then the input must be rejected eventually anyway.


##### Pretoken kind
`Reserved`

##### Attributes
(none)


#### Reserved prefix or unterminated literal (Rust 2021) { .rule }

##### Pattern
```
(
  [ \p{XID_Start} _ ]
  \p{XID_Continue} *
)
( \# | " | ')
```

##### Pretoken kind
`Reserved`

##### Attributes
(none)


#### Non-raw identifier { .rule }

##### Pattern
```
(?<identifier>
  [ \p{XID_Start} _ ]
  \p{XID_Continue} *
)
```

##### Pretoken kind
`Identifier`

##### Attributes
|                       |                     |
|:----------------------|:--------------------|
| <var>identifier</var> | captured characters |

> Note: this is following the specification in [Unicode Standard Annex #31][UAX31] for Unicode version 15.0, with the addition of permitting underscore as the first character.



[base-vs-suffix]: open_questions.md#base-vs-suffix
[block-comment-constraint]: open_questions.md#block-comment-constraint
[UAX31]: https://www.unicode.org/reports/tr31/tr31-37.html

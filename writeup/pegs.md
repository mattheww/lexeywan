## Parsing Expression Grammars

This document relies on two parsing expression grammars
(one for tokenising and one for recognising frontmatter).

Parsing Expression Grammars are described informally in §2 of [Ford 2004][peg-paper].


### Grammar notation

The notation used in this document is the [variant used by][pest-grammar] the [Pest] Rust library,
so that it's easy to keep in sync with the comparable implementation.

In particular:

 - the sequencing operator is written explicitly, as `~`
 - the ordered choice operator is `|`
 - `?`, `*`, and `+` have their usual senses (as expression suffixes)
 - `{0, 255}` is a repetition suffix, meaning "from 0 to 255 repetitions"
 - the not-predicate (for negative lookahead) is `!` (as an expression prefix)
 - a terminal matching an individual character is written like `"x"`
 - a terminal matching a sequence of characters is written like `"abc"`
 - a terminal matching a range of characters is written like `'0'..'9'`
 - `"\""` matches a single <b>"</b> character
 - `"\\"` matches a single <b>\\</b> character
 - `"\n"` matches a single <kbd>LF</kbd> character

The ordered choice operator `|` has the lowest precedence, so
```
a ~ b | c ~ d
```
is equivalent to
```
( a ~ b ) | ( c ~ d )
```

The sequencing operator `~` has the next-lowest precedence, so
```
!"." ~ SOMETHING
```
is equivalent to
```
(!".") ~ SOMETHING
```

"Any character except" is written using the not-predicate and `ANY`, for example
```
( !"'" ~ ANY )
```
matches any single character except <b>'</b>.

See [Grammar for raw string literals](raw_strings.md) for a discussion of extensions used to model raw string literals and frontmatter fences.



### Special terminals

The following named terminals are available in all grammars in this document.

##### Grammar
```
EOI
ANY
PATTERN_WHITE_SPACE
XID_START
XID_CONTINUE
```

`EOI` matches only when the sequence remaining to be matched is empty,
without consuming any characters

`ANY` matches any Unicode [character].

`PATTERN_WHITE_SPACE` matches any character which has the `Pattern_White_Space` Unicode property.
These characters are:

|        |                         |
|:-------|:------------------------|
| U+0009 | (horizontal tab, '\t')  |
| U+000A | (line feed, '\n')       |
| U+000B | (vertical tab)          |
| U+000C | (form feed)             |
| U+000D | (carriage return, '\r') |
| U+0020 | (space, ' ')            |
| U+0085 | (next line)             |
| U+200E | (left-to-right mark)    |
| U+200F | (right-to-left mark)    |
| U+2028 | (line separator)        |
| U+2029 | (paragraph separator)   |

> Note: This set doesn't change in updated Unicode versions.


`XID_START` matches any character which has the `XID_Start` Unicode property
(as of Unicode 16.0.0).


`XID_CONTINUE` matches any character which has the `XID_Continue` Unicode property
(as of Unicode 16.0.0).



[character]: definitions.md#character
[Pest]: https://pest.rs/book/grammars/syntax.html
[pest-grammar]: https://docs.rs/pest_derive/latest/pest_derive/#grammar
[peg-paper]: https://pdos.csail.mit.edu/papers/parsing:popl04.pdf


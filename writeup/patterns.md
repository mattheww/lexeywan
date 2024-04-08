### Patterns

A pattern has two functions:

- To answer the question "does this sequence of characters match the pattern?"
- When the answer is yes, to *capture* zero or more named groups of characters.

The patterns in this document use the notation from the well-known Rust [`regex` crate].

Specifically, the notation is to be interpreted in verbose mode (`ignore_whitespace`)
and with `.` allowed to match newlines (`dot_matches_new_line`).

> See open question [Pattern notation].

Patterns are always used to match against a fixed-length sequence of characters
(as if the pattern was anchored at both ends).

> Other than for constrained pattern matches, the comparable implementation anchors to the start but not the end, relying on `Regex::find()` to find the longest matching prefix.

Named capture groups (eg `(?<suffix> … )` are used in the patterns to supply character sequences used to determine attribute values.


#### Sets of characters

In particular, the following notations are used to specify sets of Unicode characters:

```
\p{Pattern_White_Space}
```

refers to the set of characters which have the `Pattern_White_Space` Unicode property, which are:

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


```
\p{XID_Start}
```

refers to the set of characters which have the `XID_Start` Unicode property.

```
\p{XID_Continue}
```

refers to the set of characters which have the `XID_Continue` Unicode property.


> The Reference adds the following when discussing identifiers:
> "Zero width non-joiner (ZWNJ U+200C) and zero width joiner (ZWJ U+200D) characters are not allowed in identifiers."
> Those characters don't have `XID_Start` or `XID_Continue`, so that's only informative text, not an additional constraint.


[Pattern notation]: open_questions.md#pattern-notation

[`regex` crate]: https://docs.rs/regex/1.10.4/regex/


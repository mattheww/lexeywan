### Common definitions

Some grammar definitions which are needed on the following pages appear below.


#### Sets of characters

The following special terminals specify sets of Unicode characters:

##### Grammar
```
ANY
PATTERN_WHITE_SPACE
XID_START
XID_CONTINUE
```

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


#### Identifier-like forms

##### Grammar
```
{{#include pretokenise_anchored.pest:ident}}
{{#include pretokenise_anchored.pest:suffix}}
```

[character]:definitions.md#character

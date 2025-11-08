## Machine-readable frontmatter grammar

The machine-readable Pest grammar for frontmatter is presented here for convenience.

See [Parsing Expression Grammars](pegs.md) for an explanation of the notation.

This version of the grammar uses Pest's [`PUSH`, `PEEK`, and `POP`](raw_strings.md#pests-stack-extension) for the matched fences.

`ANY`, `EOI`, `PATTERN_WHITE_SPACE`, `XID_START`, and `XID_CONTINUE` are built in to Pest and so not defined below.

`LF` and `TAB` are treated as special terminals in this writeup,
but they are not built in to Pest so they have definitions below using character-sequence terminals which include escapes.

```
{{#include frontmatter.pest}}
```

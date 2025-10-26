## Machine-readable frontmatter grammar

The machine-readable Pest grammar for frontmatter is presented here for convenience.

See [Parsing Expression Grammars](pegs.md) for an explanation of the notation.

This version of the grammar uses Pest's [`PUSH`, `PEEK`, and `POP`](raw_strings.md#pests-stack-extension) for the matched fences.

`ANY`, `EOI`, `PATTERN_WHITE_SPACE`, `XID_START`, and `XID_CONTINUE` are built in to Pest and so not defined below.

`LF` is treated as a special terminal in this writeup,
but it is not built in to Pest so it has a definition below using a character-sequence terminal which includes an escape.

```
{{#include frontmatter.pest}}
```

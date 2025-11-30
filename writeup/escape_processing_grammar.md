## Machine-readable escape-processing grammar

The machine-readable Pest grammar for escape processing is presented here for convenience.

See [Parsing Expression Grammars](pegs.md) for an explanation of the notation.

`ANY`, `EOI`, `PATTERN_WHITE_SPACE`, `XID_START`, and `XID_CONTINUE` are built in to Pest and so not defined below.

`TAB`, `CR`, `LF`, `DOUBLEQUOTE`, and `BACKSLASH` are treated as special terminals in this writeup,
but they are not built in to Pest so they have definitions below using character-sequence terminals which include escapes.

```
{{#include escape_processing.pest}}
```

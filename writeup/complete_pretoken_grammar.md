### The complete pretokenisation grammar

The machine-readable Pest grammar for pretokenisation is presented here for convenience.

See [Parsing Expression Grammar notation](pegs.md) for an explanation of the notation.

This version of the grammar uses Pest's [`PUSH`, `PEEK`, and `POP`](raw_strings.md#pests-stack-extension) for the `Raw_double_quoted_literal` definitions.

`ANY`, `PATTERN_WHITE_SPACE`, `XID_START`, and `XID_CONTINUE` are built in to Pest and so not defined below.

```
{{#include pretokenise.pest}}
```

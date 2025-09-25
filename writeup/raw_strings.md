## Grammar for raw string literals

I believe the PEG formalism can't naturally describe Rust's rule for matching the number of `#` characters in raw string literals.

I can think of the following ways to handle this:


### Ad-hoc extension

This writeup uses an [ad-hoc extension][rdql-pretoken] to the formalism,
along similar lines to the stack extension described below
(but without needing a full stack).


### Pest's stack extension

Pest provides a [stack extension][pest-stack] which is a good fit for this requirement,
and is used in the comparable implementation.

It looks like this:
```
RAW_DQ_REMAINDER = {
    PUSH(HASHES) ~
    "\"" ~ RAW_DQ_CONTENT ~ "\"" ~
    POP ~
    SUFFIX ?
}
RAW_DQ_CONTENT = {
    ( !("\"" ~ PEEK) ~ ANY ) *
}
HASHES = { "#" {0, 255} }
```

The notion of matching an expression is extended to include a _context stack_ (a stack of strings):
each match attempt takes the stack as an additional input and produces an updated stack as an additional output.

The stack is initially empty.

There are three additional forms of expression: `PUSH(e)`, `PEEK(e)`, and `POP(e)`, where _e_ is an arbitrary expression.

`PUSH(e)` behaves in the same way as the expression _e_;
if it succeeds, it additionally pushes the text consumed by _e_ onto the stack.

`PEEK(e)` behaves in the same way as a literal string expression, where the string is the top entry of the stack.
If the stack is empty, `PEEK(e)` fails.

`POP(e)` behaves in the same way as `PEEK(e)`.
If it succeeds, it then pops the top entry from the stack.

All other expressions leave the stack unmodified.


### Scheme of definitions

Because raw string literals have a limit of 255 `#` characters,
it is in principle possible to model them using a PEG with 256 (pairs of) definitions.

So writing this out as a "scheme" of definitions might be thinkable:

```
RDQ_0 = {
    "\"" ~ RDQ_0_CONTENT ~ "\"" ~
}
RDQ_0_CONTENT = {
    ( !("\"") ~ ANY ) *
}

RDQ_1 = {
    "#"{1} ~
    "\"" ~ RDQ_1_CONTENT ~ "\"" ~
    "#"{1} ~
}
RDQ_1_CONTENT = {
    ( !("\"" ~ "#"{1}) ~ ANY ) *
}

RDQ_2 = {
    "#"{2} ~
    "\"" ~ RDQ_2_CONTENT ~ "\"" ~
    "#"{2} ~
}
RDQ_2_CONTENT = {
    ( !("\"" ~ "#"{2}) ~ ANY ) *
}

…

RDQ_255 = {
    "#"{255} ~
    "\"" ~ RDQ_255_CONTENT ~ "\"" ~
    "#"{255} ~
}
RDQ_255_CONTENT = {
    ( !("\"" ~ "#"{255}) ~ ANY ) *
}

```


[rdql-pretoken]: string_and_byte_literal_pretokens.html#rdql
[pest-stack]: https://docs.rs/pest/2.8.0/pest/#special-rules

## Grammar for raw string literals

I believe the PEG formalism can't naturally describe Rust's rule for matching the number of `#` characters in raw string literals.

(The same limitations apply to matching the number of `-` characters in frontmatter fences.)

I can think of the following ways to handle this:


### Ad-hoc extension

This writeup uses an [ad-hoc extension][rdql-token] to the formalism,
along similar lines to the stack extension described below
(but without needing a full stack).

This extension isn't formalised in the [appendix on PEGs](pegs.md).
I don't think a formalisation would be simpler than formalising the stack extension described below.


### Pest's stack extension

<div class=pegs-description>

Pest provides a [stack extension][pest-stack] which is a good fit for this requirement,
and is used in the reimplementation.

It looks like this:
```
RAW_DOUBLE_QUOTED_FORM = {
    PUSH(HASHES) ~
    "\"" ~ RAW_DOUBLE_QUOTED_CONTENT ~ "\"" ~
    POP ~
    SUFFIX ?
}
RAW_DOUBLE_QUOTED_CONTENT = {
    ( !("\"" ~ PEEK) ~ ANY ) *
}
HASHES = { "#" {0, 255} }
```

The notion of attempting to match a parsing expression is extended to include a _context stack_ (a stack of character sequences):
each match attempt takes the stack as an additional input and produces an updated stack as an additional part of the outcome.

The stack is initially empty.

There are three additional forms of parsing expression:
<code>PUSH(<var>e</var>)</code>, `PEEK`, and `POP`,
where <var>e</var> is an arbitrary parsing expression.

<code>PUSH(<var>e</var>)</code> behaves in the same way as the parsing expression <var>e</var>.
If it succeeds, it additionally pushes the text consumed by <var>e</var> onto the stack.

An attempt to match `PEEK` against a character sequence <var>s</var> succeeds if and only if the top entry of the stack is a prefix of <var>s</var>.
If the stack is empty, `PEEK` fails.

`POP` behaves in the same way as `PEEK`.
Additionally, if it succeeds it then pops the top entry from the stack.

All other parsing expressions leave the stack unmodified.

</div>

### Scheme of definitions

Because raw string literals have a limit of 255 `#` characters,
it is in principle possible to model them using a PEG with 256 (pairs of) definitions.

So writing this out as a "scheme" of definitions might be thinkable:

```
RDQ_0 = {
    DOUBLEQUOTE ~ RDQ_0_CONTENT ~ DOUBLEQUOTE ~
}
RDQ_0_CONTENT = {
    ( !(DOUBLEQUOTE) ~ ANY ) *
}

RDQ_1 = {
    "#"{1} ~
    DOUBLEQUOTE ~ RDQ_1_CONTENT ~ DOUBLEQUOTE ~
    "#"{1} ~
}
RDQ_1_CONTENT = {
    ( !(DOUBLEQUOTE ~ "#"{1}) ~ ANY ) *
}

RDQ_2 = {
    "#"{2} ~
    DOUBLEQUOTE ~ RDQ_2_CONTENT ~ DOUBLEQUOTE ~
    "#"{2} ~
}
RDQ_2_CONTENT = {
    ( !(DOUBLEQUOTE ~ "#"{2}) ~ ANY ) *
}

…

RDQ_255 = {
    "#"{255} ~
    DOUBLEQUOTE ~ RDQ_255_CONTENT ~ DOUBLEQUOTE ~
    "#"{255} ~
}
RDQ_255_CONTENT = {
    ( !(DOUBLEQUOTE ~ "#"{255}) ~ ANY ) *
}

```


[rdql-token]: quoted_literal_tokens.html#rdql
[pest-stack]: https://docs.rs/pest/2.8.0/pest/#special-rules

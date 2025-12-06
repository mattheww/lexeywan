## Grammar for raw string literals

##### Table of contents
<!-- toc -->

I believe the PEG formalism can't naturally describe Rust's rule for matching the number of `#` characters in raw string literals.

(The same limitations apply to matching the number of `-` characters in frontmatter fences.)

I can think of the following ways to handle this:


### Corresponding nonterminal extension { #corresponding-nonterminal }

This writeup uses an [ad-hoc extension][rdql-token] to the formalism,
along similar lines to the stack extension described below
(but without needing a full stack).

It's described as follows:

 > an attempt to match one of the parsing expressions marked as HASHES² fails unless the characters it consumes are the same as the characters consumed by the (only) match of the expression marked as HASHES¹ under the same match attempt of a token-kind nonterminal.

This extension isn't formalised in the [appendix on PEGs].

It could be formalised in a similar way to the [mark/check] extension below,
with the addition of some notion of a _scoping nonterminal_ which uses an empty context for its sub-attempt.


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


### Mark/check extension { #mark-check }

This extension uses the same notation as the [corresponding nonterminal] extension.
It might be described along the following lines:

<div class=pegs-description>

<div class=sketch>
An attempt to match a parsing expression marked with ² fails
unless the characters it consumes are the same as the characters consumed by the previous match of an expression marked as ¹.
</div>

A formalisation of this extension in the style used in the [appendix on PEGs] is sketched below.

Treat ¹ and ² as operators, defining a _mark expression_ and a _check expression_ respectively.

Extend the characterisation of a match attempt to include a _context_, which is a sequence of matches
(this formalises a notion of the matches preceding the attempt).

Alter the description of most kinds of expression to consider a context and use the same context for each sub-attempt,
for example:

<div class=sketch>
An attempt <var>A</var> to match a nonterminal against <var>s</var> in context <var>c</var> succeeds if and only if
an attempt <var>A′</var> to match the nonterminal's expression against <var>s</var> in context <var>c</var> succeeds.
</div>

Alter the description of sequencing expressions to use an updated context when attempting the right-hand side:

<div class=sketch>
The outcome of an attempt <var>A</var> to match a <dfn>sequencing expression</dfn> <code><var>e₁</var> ~ <var>e₂</var></code> against <var>s</var> in context <var>c</var> is as follows:

 - If an attempt <var>A₁</var> to match the expression <var>e₁</var> against <var>s</var> in context <var>c</var> fails,
   <var>A</var> fails.
 - Otherwise, <var>A</var> succeeds if and only if
   an attempt <var>A₂</var> to match <var>e₂</var> against <var>s′</var> in context <var>c′</var> succeeds,
   where <var>s′</var> is the sequence of characters obtained by removing the prefix consumed by <var>A₁</var> from <var>s</var>,
   and <var>c′</var> is <var>c</var> followed by the elaboration of <var>A₁</var>.
</div>

Include mark expressions in the elaboration:

<div class=sketch>
An attempt <var>A</var> to match a <dfn>mark expression</dfn> <code><var>e¹</var></code> against <var>s</var> in context <var>c</var> succeeds
if and only if an attempt <var>A′</var> to match <var>e</var> against <var>s</var> in context <var>c</var> succeeds.

If <var>A</var> is successful,
it consumes the characters consumed by <var>A′</var>
and its elaboration is <var>A</var> followed by the elaboration of <var>A′</var>.
</div>

Describe a check expression as failing unless the characters its subexpression consumes are the same as the characters consumed by the last mark expression in its context:

<div class=sketch>
An attempt <var>A</var> to match a <dfn>check expression</dfn> <code><var>e²</var></code> against <var>s</var> in context <var>c</var> succeeds if

 - an attempt <var>A′</var> to match <var>e</var> against <var>s</var> in context <var>c</var> succeeds; and
 - <var>c</var> includes at least one mark expression; and
 - the characters consumed by <var>A′</var> are the same as the characters consumed by the last mark expression in <var>c</var>.

Otherwise <var>A</var> fails.
</div>

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

[appendix on PEGs]: pegs.md
[mark/check]: #mark-check
[corresponding nonterminal]: #corresponding-nonterminal

[rdql-token]: quoted_literal_tokens.html#rdql
[pest-stack]: https://docs.rs/pest/2.8.0/pest/#special-rules

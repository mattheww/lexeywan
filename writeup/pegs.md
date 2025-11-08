## Parsing Expression Grammars

Parsing Expression Grammars were introduced in [Ford 2004][peg-paper].

The notation used in this document is based on the [variant used by][pest-grammar] the [Pest] Rust library.

This page describes a subset of the formalism that is sufficient for the grammars used in this writeup.

See [Grammars] above for a less formal treatment.


##### Table of contents
<!-- toc -->

<div class=pegs-description>

### Nonterminal definitions

A Parsing Expression Grammar is made up of a sequence of <dfn>nonterminal definitions</dfn> of the form

```
NAME = { … }
```

The order of the nonterminal definitions is not significant.

The name on the left hand side of the `=` in a nonterminal definition is a <dfn>nonterminal</dfn>.

The part of the nonterminal definition that appears between `{` and `}` is that nonterminal's <dfn>expression</dfn>.
It is a [Parsing expression] as defined below.

No nonterminal appears more than once on the left hand side of a definition.


### Parsing expressions

Parsing expressions have the following forms, where
 - <var>e</var>, <var>e₁</var>, and <var>e₂</var> represent arbitrary parsing expressions
 - <var>n</var> represents an arbitrary positive integer, written in decimal.

|                                             |                                     |
|---------------------------------------------|-------------------------------------|
| __Terminals__                               |                                     |
| eg `"abc"`                                  | Character-sequence terminal         |
| eg `'a'..'f'`                               | Character-range terminal            |
| `ANY`                                       | Any character                       |
| `DOUBLEQUOTE`                               | <b>"</b>                            |
| `BACKSLASH`                                 | <b>\\</b>                           |
| `LF`                                        | Line feed                           |
| `TAB`                                       | Tab                                 |
| `PATTERN_WHITE_SPACE`                       |                                     |
| `XID_START`                                 |                                     |
| `XID_CONTINUE`                              |                                     |
| `EOI`                                       | End of input                        |
| `EMPTY`                                     | Empty match                         |
| __Nonterminals__                            |                                     |
| A defined nonterminal                       |                                     |
| __Compound expressions__                    |                                     |
| <code><var>e₁</var> ~ <var>e₂</var></code>  | Sequencing expression               |
| <code><var>e₁</var> \| <var>e₂</var></code> | Prioritised choice expression       |
| <code><var>e</var> ?</code>                 | Option suffix expression            |
| <code><var>e</var> *</code>                 | Zero-or-more repetitions expression |
| <code><var>e</var> +</code>                 | One-or-more repetitions expression  |
| <code><var>e</var> {0, <var>n</var>}</code> | limited repetitions expression      |
| <code>! <var>e</var></code>                 | Negative lookahead expression       |
| __Grouping__                                |                                     |
| <code>( <var>e</var> )</code>               |                                     |

The symbols `~`, `|`, `?`, `*`, `+`, `!`, and the form <code>{0, <var>n</var>}</code>,
are called <dfn>parsing operators</dfn>.

Each nonterminal which appears in a parsing expression has a definition in the grammar.

> The `EMPTY` terminal doesn't appear in any of grammars in this writeup;
> it's used below in the descriptions of matching option and repetition expressions.


### Grouping, precedence, and association

The definition of matching below assumes that each parsing expression has a known interpretation as
a tree of compound expressions, nonterminals, and terminals.

This section describes how to resolve ambiguities in the written form of a parsing expression
to produce such a tree.

A subexpression in parentheses `(` and `)` is treated as a separate unit.

The prioritised choice parsing operator `|` has the lowest precedence, so for example
<code><var>e₁</var> ~ <var>e₂</var> | <var>e₃</var> ~ <var>e₄</var></code>
is interpreted as
<code>( <var>e₁</var> ~ <var>e₂</var> ) | ( <var>e₃</var> ~ <var>e₄</var> )</code>.

The sequencing parsing operator `~` has the next-lowest precedence, so
for example <code>!<var>e₁</var> ~ <var>e₂</var></code>
is interpreted as
<code>(!<var>e₁</var>) ~ <var>e₂</var></code>, and
<code><var>e₁</var> ~ <var>e₂</var>?</code>
is interpreted as
<code><var>e₁</var> ~ (<var>e₂</var>?)</code>.

The grammars used in this writeup do not rely on a defined precedence between the unary parsing operators.

The binary parsing operators `~` and `|` are left-associative:

- <code><var>e₁</var> ~ <var>e₂</var> ~ <var>e₃</var></code>
  is interpreted as
  <code>(<var>e₁</var> ~ <var>e₂</var>) ~ <var>e₃</var></code>
- <code><var>e₁</var> | <var>e₂</var> | <var>e₃</var></code>
  is interpreted as
  <code>(<var>e₁</var> | <var>e₂</var>) | <var>e₃</var></code>

> The associativity doesn't matter in practice:
> <code>(<var>e₁</var> ~ <var>e₂</var>) ~ <var>e₃</var></code>
> and
> <code><var>e₁</var> ~ (<var>e₂</var> ~ <var>e₃</var>)</code>
> have identical outcomes, and
> <code>(<var>e₁</var> | <var>e₂</var>) | <var>e₃</var></code>
> and
> <code><var>e₁</var> | (<var>e₂</var> | <var>e₃</var>)</code>
> have identical outcomes.


### Matching

A <dfn>match attempt</dfn> is characterised by
a grammar,
a parsing expression <var>e</var>, and
a character sequence <var>s</var>. In this document the grammar is always implicit.

A match attempt is identified using the form "a match attempt of <var>e</var> against <var>s</var>" or "an attempt to match <var>e</var> against <var>s</var>".
On this page,
a match attempt may be referred to simply as an <dfn>attempt</dfn>.

The descriptions of terminals, nonterminals, and compound expressions below, taken together,
define the outcome of any match attempt.

> It is possible to write a grammar under which the definition of outcome below is not well-founded,
> because of direct or indirect left recursion in the definitions of nonterminals.
> The grammars used in this writeup do not have this complication,
> so we may assume all match attempts have a well-defined outcome.

The <dfn>outcome</dfn> of a match attempt against <var>s</var> is one of:
 - success, together with
   - a prefix of <var>s</var> which was <dfn>consumed</dfn> by the attempt
   - a sequence of matches (the attempt's <dfn>elaboration</dfn>)
 - failure.

We say a match attempt <dfn>succeeds</dfn> or <dfn>is successful</dfn> if its outcome is success,
and <dfn>fails</dfn> if its outcome is failure.

A successful match attempt can be referred to as a <dfn>match</dfn>.

Note that any nonterminal is a parsing expression on its own,
so it is meaningful to talk about an attempt to match a nonterminal against a character sequence.

For the purposes of this section, a <dfn>prefix</dfn> of a sequence is
the first <var>n</var> characters of the sequence, for some <var>n</var>.
The prefix may be empty, or the entire sequence.

In the descriptions below, <var>s</var> represents a character sequence.


#### Terminals

An attempt to match a <dfn>character-sequence terminal</dfn> <code>"c₁…cₙ"</code> against <var>s</var>
succeeds if and only if the character sequence c₁…cₙ is a prefix of <var>s</var>,
and (if it succeeds) consumes c₁…cₙ.
Here, c₁…cₙ represents an arbitrary sequence of characters other than <b>"</b>
(in practice, they are printable ASCII characters).

An attempt to match a <dfn>character-range terminal</dfn> <code>'c₁'..'c₂'</code> against <var>s</var>
succeeds if and only <var>s</var> begins with a character whose [Unicode scalar value] is between the Unicode scalar value of c₁ and the Unicode scalar value of c₂ (inclusive),
and (if it succeeds) consumes that character.
Here, c₁ and c₂ represent arbitrary single characters other than <b>'</b> (in practice, ASCII digits or letters).

An attempt to match `ANY` against <var>s</var> succeeds if and only if <var>s</var> is not empty,
and (if it succeeds) consumes the first character in <var>s</var>.

An attempt to match `DOUBLEQUOTE` against <var>s</var> succeeds if and only if <var>s</var> begins with the character <b>"</b>,
and (if it succeeds) consumes that character.

An attempt to match `BACKSLASH` against <var>s</var> succeeds if and only if <var>s</var> begins with the character <b>\\</b>,
and (if it succeeds) consumes that character.

An attempt to match `LF` against <var>s</var> succeeds if and only if <var>s</var> begins with the character <kbd>LF</kbd>,
and (if it succeeds) consumes that character.

An attempt to match `TAB` against <var>s</var> succeeds if and only if <var>s</var> begins with the character <kbd>HT</kbd>,
and (if it succeeds) consumes that character.

An attempt to match `PATTERN_WHITE_SPACE` succeeds if and only if <var>s</var> begins with a character which has the `Pattern_White_Space` Unicode character property,
as defined in `PropList.txt` in the [Unicode character database][UCD],
and (if it succeeds) consumes that character.

An attempt to match `XID_START` succeeds if and only if <var>s</var> begins with a character which has the `XID_Start` Unicode character property,
as defined in `DerivedCoreProperties.txt` in the [Unicode character database][UCD],
and (if it succeeds) consumes that character.

An attempt to match `XID_CONTINUE` succeeds if and only if <var>s</var> begins with a character which has the `XID_Continue` Unicode character property,
as defined in `DerivedCoreProperties.txt` in the [Unicode character database][UCD],
and (if it succeeds) consumes that character.

An attempt to match `EOI` against <var>s</var> succeeds if and only if <var>s</var> is empty,
and (if it succeeds) consumes an empty character sequence.

An attempt to match `EMPTY` always succeeds,
and (if it succeeds) consumes an empty character sequence.

All matches of terminals have an empty elaboration.


#### Nonterminals

An attempt <var>A</var> to match a nonterminal against <var>s</var> succeeds if and only if
an attempt <var>A′</var> to match the nonterminal's expression against <var>s</var> succeeds.

If <var>A</var> is successful,
it consumes the characters consumed by <var>A′</var>
and its elaboration is <var>A</var> followed by the elaboration of <var>A′</var>.


#### Compound expressions

In the descriptions below,
a statement that an expression <var>e₁</var> <dfn>reduces to</dfn> an expression <var>e₂</var> means that
the outcome of an attempt to match <var>e₁</var> against <var>s</var>
is the outcome of an attempt to match <var>e₂</var> against <var>s</var>.


##### Sequencing expressions (`~`) { #sequencing-expressions }

The outcome of an attempt <var>A</var> to match a <dfn>sequencing expression</dfn> <code><var>e₁</var> ~ <var>e₂</var></code> against <var>s</var> is as follows:
 - If an attempt <var>A₁</var> to match the expression <var>e₁</var> against <var>s</var> fails,
   <var>A</var> fails.
 - Otherwise, <var>A</var> succeeds if and only if
   an attempt <var>A₂</var> to match <var>e₂</var> against <var>s′</var> succeeds,
   where <var>s′</var> is the sequence of characters obtained by removing the prefix consumed by <var>A₁</var> from <var>s</var>.

If <var>A</var> succeeds:
 - It consumes the characters consumed by <var>A₁</var> followed by the characters consumed by <var>A₂</var>.
 - Its elaboration is the elaboration of <var>A₁</var> followed by the elaboration of <var>A₂</var>.


##### Prioritised choice expressions (`|`)

The outcome of an attempt <var>A</var> to match a <dfn>prioritised choice</dfn> expression <code><var>e₁</var> | <var>e₂</var></code> against <var>s</var> is as follows:
 - If an attempt <var>A₁</var> to match <var>e₁</var> against <var>s</var> succeeds,
   the outcome of <var>A</var> is the outcome of <var>A₁</var>.
 - Otherwise, the outcome of <var>A</var> is the outcome of an attempt to match <var>e₂</var> against <var>s</var>.


##### Option expressions (`?`)

The <dfn>option expression</dfn> <code><var>e</var>?</code> reduces to
<code><var>e</var> | EMPTY</code>.


##### Repetition expressions (`*`, `+`, and <code>{0,<var>n</var>}</code>)

A <dfn>zero-or-more repetitions expression</dfn> <code><var>e</var>\*</code> reduces to
<code>( <var>e</var> ~ <var>e</var>* ) | EMPTY</code>.

A <dfn>one-or-more repetitions expression</dfn> <code><var>e</var>+</code> reduces to
<code><var>e</var> ~ <var>e</var>*</code>.

A <dfn>limited repetition expression</dfn> of the form <code><var>e</var>{0, 1}</code> reduces to
<code><var>e</var>?</code>.

A <dfn>limited repetition expression</dfn> of the form <code><var>e</var>{0, <var>n</var>}</code>, for <var>n</var> > 1, reduces to
<code><var>e</var>? ~ <var>e</var>{0, <var>n</var>-1}</code>.


##### Negative lookahead expressions (`!`)

An attempt to match a <dfn>negative lookahead expression</dfn> <code>!<var>e</var></code> against <var>s</var> succeeds
if and only if an attempt to match <var>e</var> against <var>s</var> fails.

If the attempt succeeds, it consumes no characters and has an empty elaboration.


### Participating in a match { #participating }

A match is a <dfn>participating match</dfn> of a nonterminal <var>N</var> in a match <var>A</var>
if it is a match of <var>N</var> which appears in the elaboration of <var>A</var>.

A nonterminal <var>N</var> <dfn>participates in</dfn> a match <var>A</var> if
there is a participating match of <var>N</var> in <var>A</var>.

If a nonterminal <var>N</var> participates in a match <var>A</var>,
the <dfn>first participating match</dfn> of <var>N</var> in <var>A</var> is
the first match of <var>N</var> in the elaboration of <var>A</var>.

If <var>𝑵</var> is a class of nonterminals,
the <dfn>sequence of participating matches</dfn> of <var>𝑵</var> in a match <var>A</var>
is the sequence obtained by restricting the elaboration of <var>A</var> to matches of members of <var>𝑵</var>.

</div>

[UCD]: definitions.md#unicode
[Grammars]: grammars.md

[Parsing expression]: #parsing-expressions
[Matching]: #matching

[token-kind nonterminals]: tokenising.md#token-kind-nonterminals
[character]: definitions.md#character
[characters]: definitions.md#character

[Pest]: https://pest.rs/book/grammars/syntax.html
[pest-grammar]: https://docs.rs/pest_derive/latest/pest_derive/#grammar
[peg-paper]: https://pdos.csail.mit.edu/papers/parsing:popl04.pdf
[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value


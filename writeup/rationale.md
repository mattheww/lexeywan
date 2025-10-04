## Rationale for this model

##### Table of contents

[Pretokenising and reprocessing](#pretokenising-and-reprocessing)\
[Using a Parsing Expression Grammar](#using-a-parsing-expression-grammar)\
[Modelling lifetimes and labels](#modelling-lifetimes-and-labels)\
[Producing tokens with attributes](#producing-tokens-with-attributes)


### Pretokenising and reprocessing

The split into pretokenisation and reprocessing is primarily a way to make the grammar simpler.

The main advantage is dealing with character, byte, and string literals,
where we have to reject invalid escape sequences at lexing time.

In this model, the lexer finds the extent of the token using simple grammar definitions,
and then checks whether the escape sequences are valid in a separate "reprocessing" operation.
So the grammar "knows" that a backslash character indicates an escape sequence, but doesn't model escapes in any further detail.

In contrast the Reference gives grammar productions which try to describe the available escape sequences in each kind of string-like literal,
but this isn't enough to characterise which forms are accepted
(for example `"\u{d800}"` is rejected at lexing time, because there is no character with scalar value D800).

Given that we have this separate operation, we can use it to simplify other parts of the grammar too,
including:

- distinguishing doc-comments
- rejecting <kbd>CR</kbd> in comments
- rejecting the reserved keywords in raw identifiers, eg `r#crate`
- rejecting no-digit forms like `0x_`
- rejecting the variants of numeric literals reserved in [rfc0879], eg `0b123`
- rejecting literals with a single `_` as a suffix

This means we can avoid adding many "reserved form" definitions.
For example, if we didn't accept `_` as a suffix in the main string-literal grammar definitions,
we'd have to have another `Reserved_something` definition to prevent the `_` being accepted as a separate token.

Given the choice to use locally greedy matching (see below),
I think an operation which rejects pretokens after parsing them is necessary to deal with a case like `0b123`,
to avoid analysing it as `0b1` followed by `23`.


### Using a Parsing Expression Grammar

I think a PEG is a good formalism for modelling Rust's lexer
(though probably not for the rest of the grammar)
for several reasons.


#### Resolving ambiguities

The lexical part of Rust's grammar is necessarily full of ambiguities.

For example:
 - `ab` could be a single identifier, or `a` followed by `b`
 - `1.2` could be a floating-point literal, or `1` followed by `.` followed by `2`
 - `r"x"` could be a raw string literal, or `r` followed by `"x"`

The Reference doesn't explicitly state what formalism it's using,
or what rule it uses to disambiguate such cases.

There are two common approaches:
to choose the longest available match (as Lex traditionally does),
or to explicitly list rules in priority order and specify locally "greedy" repetition-matching (as PEGs do).

With the "longest available" approach, additional rules are needed if multiple rules can match with equal length.

The [2024 version of this model][lexeywan 2024] characterised the cases where rustc doesn't choose the longest available match,
and where (given its choice of rules) there are multiple longest-matching rules.

For example, the Reference's lexer rules for input such as `0x3` allow two interpretations, matching the same extent:
 - as a hexadecimal integer literal: `0x3` with no suffix
 - as a decimal integer literal: `0` with a suffix of `x3`

We want to choose the former interpretation.
(We could say that these are both the same kind of token and re-analyse it later to decide which part was the suffix, but we'd still have to understand the distinction inside the lexer in order to reject forms like `0b0123`.)

Examples where rustc chooses a token which is shorter than the longest available match are rare. In the model used by the Reference, `0x·` is one:
rustc treats this as a "reserved number" (`0x`), rather than `0` with suffix `x·`.
(Note that <b>·</b> has the `XID_Continue` property but not `XID_Start`.)

I think that in 2025 it's clear that a priority-based system is a better fit for Rust's lexer.
In particular, if [pr131656] is accepted (allowing forms like `1em`),
the new ambiguities will be resolved naturally because the floating-point literal definitions have priority over the integer literal definitions.

Generative grammars don't inherently have prioritisation, but parsing expression grammars do.


#### Ambiguities that must be resolved as errors

There are a number of forms which are errors at lexing time, even though in principle they could be analysed as multiple tokens.

Many cases can be handled in reprocessing, as described above.

Other cases can be handled naturally using a PEG, by writing medium-priority rules rules to match them, for example:

- the [rfc3101] "reserved prefixes" (in Rust 2021 and newer): `k#abc`,  `f"..."`, or `f'...'`
- unterminated block comments such as `/* abc`
- forms that look like floating-point literals with a base indicator, such as `0b1.0`

In this model, these additional rules produce `Reserved` pretokens, which are rejected at reprocessing time.


#### Lookahead

There are two cases where the Reference currently describes the lexer's behaviour using lookahead:
- for (possibly raw) lifetime-or-label, to prevent `'ab'c'` being analysed as `'ab` followed by `'c`
- for floating-point literals, to make sure that `1.a` is analysed as `1` `.` `a` rather than `1.` `a`

These are easily modelled using PEG predicates
(though this writeup prefers a reserved form for the former).


#### Handling raw strings

The biggest weakness of using the PEG formalism is that it can't naturally describe the rule for matching the number of `#` characters in raw string literals.

See [Grammar for raw string literals](raw_strings.md) for discussion.


#### Adopting language changes

Rustc's lexer is made up of hand-written imperative code, largely using `match` statements.
It often peeks ahead for a few characters, and it tries to avoid backtracking.

This is a close fit for the behaviour modelled by PEGs,
so there's good reason to suppose that it will be easy to update this model for future versions of Rust.


### Modelling lifetimes and labels

Like the Reference, this model has a separate kind of token for lifetime-or-label.

It would be nice to be able to treat them as two fine-grained tokens (`'` followed by an identifier),
like they are treated in procedural macro input,
but I think it's impractical.

The main difficulty is dealing with cases like `'r"abc"`.
Rust accepts that as a lifetime-or-label `'r` followed by a string literal `"abc"`.
A model which treats `'` as a complete token would analyse this as `'` followed by a raw string literal `r"abc"`.
This problem can occur with any prefix (including a reserved prefix).


### Producing tokens with attributes

This model makes the lexing process responsible for a certain amount of 'interpretation' of the tokens,
rather than simply describing how the input source is sliced up and assigning a 'kind' to each resulting token.

The main motivation for this is to deal with string-like literals:
it means we don't need to separate the description of the result of "unescaping" strings from the description of which strings contain well-formed escapes.

In particular, describing unescaping at lexing time makes it easy to describe the rule about rejecting NULs in C-strings, even if they were written using an escape.

For numeric literals, the way the suffix is identified isn't always simple (see [Resolving ambiguities](#resolving-ambiguities) above);
I think it's best to make the lexer responsible for doing it,
so that the description of numeric literal expressions doesn't have to.

For identifiers, many parts of the spec will need a notion of equivalence
(both for handling raw identifiers and for dealing with NFC normalisation),
and some restrictions depend on the normalised form (see [ASCII identifiers]).
I think it's best for the lexer to handle this by defining the <var>represented identifier</var>.

This document treats the lexer's "output" as a stream of tokens which have concrete attributes,
but of course it would be equivalent (and I think more usual for a spec) to treat each attribute as an independent defined term,
and write things like "the <dfn>represented character</dfn> of a character literal token is…".


[rfc0879]: https://rust-lang.github.io/rfcs/0879-small-base-lexing.html
[rfc3101]: https://rust-lang.github.io/rfcs/3101-reserved_prefixes.html

[pr131656]: https://github.com/rust-lang/rust/pull/131656

[ASCII identifiers]: open_questions.md#ascii-identifiers

[lexeywan 2024]: https://mjw.woodcraft.me.uk/2024-lexeywan/open_questions.html#rule-priority

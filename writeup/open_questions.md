## Open questions

##### Table of contents

[Terminology](#terminology)\
[Pattern notation](#pattern-notation)\
[Rule priority](#rule-priority)\
[Token kinds and attributes](#token-kinds-and-attributes)\
[Defining the block-comment constraint](#block-comment-constraint)\
[Wording for string unescaping](#wording-for-string-unescaping)\
[How to model shebang removal](#how-to-model-shebang-removal)\
[String continuation escapes](#string-continuation-escapes)


### Terminology

Some of the terms used in this document are taken from pre-existing documentation or rustc's error output,
but many of them are new (and so can freely be changed).

Here's a partial list:

| Term                       | Source                           |
|:---------------------------|:---------------------------------|
| pretoken                   | New                              |
| reprocessing               | New                              |
| fine-grained token         | New                              |
| compound token             | New                              |
| literal content            | Reference (recent)               |
| simple escape              | Reference (recent)               |
| escape sequence            | Reference                        |
| escaped value              | Reference (recent)               |
| string continuation escape | Reference (as `STRING_CONTINUE`) |
| string representation      | Reference (recent)               |
| represented byte           | New                              |
| represented character      | Reference (recent)               |
| represented bytes          | Reference (recent)               |
| represented string         | Reference (recent)               |
| represented identifier     | New                              |
| style (of a comment)       | rustc internal                   |
| body (of a comment)        | Reference                        |

Terms listed as "Reference (recent)" are ones I introduced in PRs merged in January 2024,
so it's not very likely that they've been picked up more widely.



### Pattern notation

This document is relying on the [`regex` crate] for its pattern notation.

This is convenient for checking that the writeup is the same as the comparable implementation,
but it's presumably not suitable for the spec.

The usual thing for specs seems to be to define their own notation from scratch.


#### Requirements for patterns

I've tried to keep the patterns used here as simple as possible.

There's no use of non-greedy matches.

I think all the uses of alternation are obviously unambiguous.

In particular, all uses of alternation inside repetition have disjoint sets of accepted first characters.

I believe all uses of repetition in the unconstrained patterns have unambiguous termination.
That is, anything permitted to follow the repeatable section would not be permitted to start a new repetition.
In these cases, the distinction between greedy and non-greedy matches doesn't matter.


#### Naming sub-patterns

The patterns used in this document are inconveniently repetitive,
particularly for the edition-specific rule variants and for numeric literals.

Of course the usual thing is to have a way to define reusable named sub-patterns.
So I think addressing this should be treated as part of choosing a pattern notation.


### Rule priority

At present this document gives the pretokenisation rules explicit priorities,
used to determine which rule is chosen in positions where more than one rule matches.

I believe that in almost all cases it would be equivalent to say that the rule which matches the longest extent is chosen
(in particular, if multiple rules match then one has a longer extent than any of the others).

See [Integer literal base-vs-suffix ambiguity][base-vs-suffix] below for the exception.

This document uses the order in which the rules are presented as the priority,
which has the downside of forcing an unnatural presentation order
(for example, [Raw identifier] and [Non-raw identifier] are separated).

Perhaps it would be better to say that longest-extent is the primary way to disambiguate,
and add a secondary principle to cover the exceptional cases.

The comparable implementation reports (as "model error") any cases
(other than the Integer literal base-vs-suffix ambiguity)
where the priority principle doesn't agree with the longest-extent principle,
or where there wasn't a unique longest match.


#### Integer literal base-vs-suffix ambiguity { #base-vs-suffix }

The Reference's lexer rules for input such as `0x3` allow two interpretations, matching the same extent:
- as a hexadecimal integer literal: `0x3` with no suffix
- as a decimal integer literal: `0` with a suffix of `x3`

If the output of the lexer is simply a token with a kind and an extent, this isn't a problem:
both cases have the same kind.

But if we want to make the lexer responsible for identifying which part of the input is the suffix,
we need to make sure it gets the right answer (ie, the one with no suffix).

Further, there are cases where we need to reject input which matches the rule for a decimal integer literal `0` with a suffix,
for example `0b1e2`, `0b0123` (see [rfc0879]), or `0x·`.

(Note that <b>·</b> has the `XID_Continue` property but not `XID_Start`.)

In these cases we can't avoid dealing with the base-vs-suffix ambiguity in the lexer.

This model uses a separate rule for integer decimal literals,
with lower priority than all other numeric literals,
to make sure we get these results.

Note that in the `0x·` example the extent matched by the lower priority rule is longer than the extent matched by the chosen rule.

If relying on priorities like this seems undesirable,
I think it would be possible to rework the rules to avoid it.
It might work to allow the difficult cases to pretokenise as decimal integer literals,
and have reprocessing reject decimal literal pretokens which begin with a base indicator.


### Token kinds and attributes

What kinds and attributes should fine-grained tokens have?


#### Distinguishing raw and non-raw forms

The current table distinguishes raw from non-raw forms as different top-level "kinds".

I think this distinction will be needed in some cases,
but perhaps it would be better represented using an attributes on unified kinds
(like `rustc_ast::StrStyle` and `rustc_ast::token::IdentIsRaw`).

As an example of where it might be wanted: proc-macros `Display` for raw identifers includes the `r#` prefix for raw identifiers, but I think simply using the source extent isn't correct because the `Display` output is NFC-normalised.


#### Hash count

Should there be an attribute recording the number of hashes in a raw string or byte-string literal?
Rustc has something of the sort.


#### ASCII identifiers

Should there be an attribute indicating whether an identifier is all ASCII?
The Reference lists several places where identifiers have this restriction,
and it seems natural for the lexer to be responsible for making this check.

The list in the Reference is:
- `extern crate` declarations
- External crate names referenced in a path
- Module names loaded from the filesystem without a `path` attribute
- `no_mangle` attributed items
- Item names in external blocks

I believe this restriction is applied after NFC-normalisation,
so it's best thought of as a restriction on the <var>represented identifier</var>.


#### Represented bytes for C strings

At present this document says that the sequence of "represented bytes" for C string literals doesn't include the added NUL.

That's following the way the Reference currently uses the term "represented bytes",
but `rustc` includes the NUL in its equivalent piece of data.


### Defining the block-comment constraint { #block-comment-constraint }

This document currently uses imperative Rust code to define the [Block comment] constraint
(ie, to say that `/*` and `*/` must be properly nested inside a candidate comment).

It would be nice to do better;
the options might depend on what pattern notation is chosen.

I don't think there's any very elegant way to describe the constraint in English
(note that the constraint is asymmetrical; for example `/* /*/ /*/ */` is rejected).

Perhaps the natural continuation of this writeup's approach would be to define a mini-tokeniser to use inside the constraint,
but that would be a lot of words for a small part of the spec.

Or perhaps this part could borrow some definitions from whatever formalisation the spec ends up using for Rust's grammar,
and use the traditional sort of context-free-grammar approach.


### Wording for string unescaping

The description of reprocessing for [String literals] and [C-string literals] was originally drafted for the Reference.
Should there be a more formal definition of unescaping processes than the current "left-to-right order" and
"contributes" wording?

I believe that any literal content which will be accepted can be written uniquely as a sequence of (escape-sequence or non-<b>\\</b>-character),
but I'm not sure that's obvious enough that it can be stated without justification.

This is a place where the comparable implementation isn't closely parallel to the writeup.


### How to model shebang removal

This part of the Reference text isn't trying to be rigorous:

> As an exception, if the `#!` characters are followed (ignoring intervening comments or whitespace) by a `[` token, nothing is removed. This prevents an inner attribute at the start of a source file being removed.

`rustc` implements the "ignoring intervening comments or whitespace" part by
running its lexer for long enough to see whether the `[` is there or not,
then discarding the result (see [#70528] and [#71487] for history).

So should the spec define this in terms of its model of the lexer?


### String continuation escapes

`rustc` has a warning that the behaviour of [String continuation escapes][string-continuation]
(when multiple newlines are skipped)
may change in future.

The Reference has a [note][ref-string-continuation] about this,
and points to [#1042][Ref#1042] for more information.
Should the spec say anything?


[base-vs-suffix]: #base-vs-suffix

[Block comment]: rules.md#block-comment
[Raw identifier]: rules.md#raw-identifier
[Non-raw identifier]: rules.md#non-raw-identifier

[String literals]: reprocessing_cases.md#string-literal
[C-string literals]: reprocessing_cases.md#c-string-literal

[string-continuation]: escape_processing.md#string-continuation-escapes

[rfc0879]: https://github.com/rust-lang/rfcs/pull/0879

[#70528]: https://github.com/rust-lang/rust/issues/70528
[#71487]: https://github.com/rust-lang/rust/pull/71487

[Ref#1042]: https://github.com/rust-lang/reference/pull/1042
[ref-string-continuation]: https://doc.rust-lang.org/nightly/reference/expressions/literal-expr.html#string-continuation-escapes

[`regex` crate]: https://docs.rs/regex/1.10.4/regex/


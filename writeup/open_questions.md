## Open questions

##### Table of contents

[Terminology](#terminology)\
[Presenting reprocessing as a separate pass](#reprocessing-as-a-pass)\
[Raw string literals](#raw-string-literals)\
[Token kinds and attributes](#token-kinds-and-attributes)\
[How to indicate captured text](#how-to-indicate-captured-text)\
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

Terms listed as "Reference (recent)" are ones I introduced in PRs merged in January 2024.



### Presenting reprocessing as a separate pass { #reprocessing-as-a-pass }

This writeup presents pretokenisation and reprocessing in separate sections,
with separate but similar definitions for a "Pretoken" and a "Fine-grained token".

That's largely because I wanted the option to have further processing between those two stages which might split or join tokens,
as some [earlier models][CAD97 spec] have done.

But in this version of the model that flexibility isn't used:
one pretoken always corresponds to one fine-grained token
(unless the input is rejected).

So it might be possible to drop the distinction between those two types altogether.

In any case I don't think it's necessary to describe reprocessing as a second pass:
the conditions for rejecting each type of pretoken,
and the definitions of the things which are currently attributes of fine-grained tokens,
could be described in the same place as the description of how the pretoken is produced.


### Raw string literals

How should raw string literals be documented?
See [Grammar for raw string literals](raw_strings.md) for some options.


### Token kinds and attributes

What kinds and attributes should fine-grained tokens have?


#### Distinguishing raw and non-raw forms

The current table distinguishes raw from non-raw forms as different top-level "kinds".

I think this distinction will be needed in some cases,
but perhaps it would be better represented using an attributes on unified kinds
(like `rustc_ast::StrStyle` and `rustc_ast::token::IdentIsRaw`).

As an example of where it might be wanted: proc-macros `Display` for raw identifiers includes the `r#` prefix for raw identifiers, but I think simply using the source extent isn't correct because the `Display` output is NFC-normalised.


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

Should this writeup change to match rustc?


### How to indicate captured text

Some of the nonterminals in the grammar exist only to identify text to be "captured",
for example `LINE_COMMENT_CONTENT` here:

```
{{#include pretokenise_anchored.pest:line_comment}}
```

Would it be better to extend the notation to allow annotating part of an expression without separating out a nonterminal?
Pest's ["Tags" extension][pest-tags] would allow doing this, but it's not a standard feature of PEGs.


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

[#136600] asks whether this is intentional.


[String literals]: reprocessing_cases.md#string-literal
[C-string literals]: reprocessing_cases.md#c-string-literal

[string-continuation]: escape_processing.md#string-continuation-escapes

[#70528]: https://github.com/rust-lang/rust/issues/70528
[#71487]: https://github.com/rust-lang/rust/pull/71487
[#136600]: https://github.com/rust-lang/rust/issues/136600

[Ref#1042]: https://github.com/rust-lang/reference/pull/1042
[ref-string-continuation]: https://doc.rust-lang.org/nightly/reference/expressions/literal-expr.html#string-continuation-escapes

[CAD97 spec]: https://github.com/CAD97/rust-lexical-spec

[pest-tags]: https://pest.rs/book/grammars/syntax.html#tags

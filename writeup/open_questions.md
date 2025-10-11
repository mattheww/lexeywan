## Open questions

##### Table of contents

[Terminology](#terminology)\
[Raw string literals](#raw-string-literals)\
[Token kinds and attributes](#token-kinds-and-attributes)\
[How to indicate captured text](#how-to-indicate-captured-text)\
[Wording for string unescaping](#wording-for-string-unescaping)\
[String continuation escapes](#string-continuation-escapes)


### Terminology

Some of the terms used in this document are taken from pre-existing documentation or rustc's error output,
but many of them are new (and so can freely be changed).

Here's a partial list:

| Term                       | Source                           |
|:---------------------------|:---------------------------------|
| processing                 | New                              |
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
| represented ident          | New                              |
| style (of a comment)       | rustc internal                   |
| body (of a comment)        | Reference                        |

Terms listed as "Reference (recent)" are ones I introduced in PRs merged in January 2024.



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
so it's best thought of as a restriction on the <var>represented ident</var>.


#### Represented bytes for C strings

At present this document says that the sequence of "represented bytes" for C string literals doesn't include the added NUL.

That's following the way the Reference currently uses the term "represented bytes",
but `rustc` includes the NUL in its equivalent piece of data.

Should this writeup change to match rustc?


### How to indicate captured text

Some of the nonterminals in the grammar exist only to identify text to be "captured",
for example `LINE_COMMENT_CONTENT` here:

```
{{#include tokenise_anchored.pest:line_comment}}
```

Would it be better to extend the notation to allow annotating part of an expression without separating out a nonterminal?
Pest's ["Tags" extension][pest-tags] would allow doing this, but it's not a standard feature of PEGs.


### Wording for string unescaping

The description of processing for [String literals], [Byte-string literals], and [C-string literals] was originally drafted for the Reference.
Should there be a more formal definition of unescaping processes than the current "left-to-right order" and
"contributes" wording?

I believe that any literal content which will be accepted can be written uniquely as a sequence of (escape-sequence or non-<b>\\</b>-character),
but I'm not sure that's obvious enough that it can be stated without justification.

This is a place where the reimplementation isn't closely parallel to the writeup.


### String continuation escapes

`rustc` has a warning that the behaviour of [String continuation escapes][string-continuation]
(when multiple newlines are skipped)
may change in future.

The Reference has a [note][ref-string-continuation] about this,
and points to [#1042][Ref#1042] for more information.

[#136600] asks whether this is intentional.


[String literals]: string_and_byte_literal_tokens.md#string-literal
[Byte-string literals]: string_and_byte_literal_tokens.md#byte-string-literal
[C-string literals]: string_and_byte_literal_tokens.md#c-string-literal

[string-continuation]: escape_processing.md#string-continuation-escapes

[#136600]: https://github.com/rust-lang/rust/issues/136600

[Ref#1042]: https://github.com/rust-lang/reference/pull/1042
[ref-string-continuation]: https://doc.rust-lang.org/nightly/reference/expressions/literal-expr.html#string-continuation-escapes

[pest-tags]: https://pest.rs/book/grammars/syntax.html#tags

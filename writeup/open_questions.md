## Open questions

##### Table of contents
<!-- toc -->


### Terminology

Some of the terms used in this document are taken from pre-existing documentation or rustc's error output,
but many of them are new (and so can freely be changed).

Here's a partial list:

| Term                       | Source                                |
|:---------------------------|:--------------------------------------|
| processing                 | New                                   |
| fine-grained token         | New                                   |
| compound token             | New                                   |
| literal content            | Reference                             |
| non-escape                 | New                                   |
| simple escape              | Reference                             |
| hexadecimal escape         | rustc error message (as "hex escape") |
| escape sequence            | Reference                             |
| escaped value              | Reference                             |
| string continuation escape | Reference (as `STRING_CONTINUE`)      |
| string representation      | Reference                             |
| represented byte           | New                                   |
| represented character      | Reference                             |
| represented bytes          | Reference                             |
| represented string         | Reference                             |
| represented ident          | New                                   |
| style (of a comment)       | rustc internal                        |
| body (of a comment)        | Reference                             |


### Raw string literals

How should raw string literals be documented,
and in particular how should any necessary extension to PEGs be formalised?

See [Grammar for raw string literals](raw_strings.md) for some options.


### Token kinds and attributes

What kinds and attributes should fine-grained tokens have?


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


### Wording for string unescaping

The description of building up the represented bytes for [C-string literals] still uses the "contributes" wording from the Reference.
Is it worth having something more formal?


[String literals]: quoted_literal_tokens.md#string-literal
[Byte-string literals]: quoted_literal_tokens.md#byte-string-literal
[C-string literals]: quoted_literal_tokens.md#c-string-literal


# Introduction

This document contains a description of `rustc`'s lexer,
which is aiming to be both correct and verifiable.

It's accompanied by a reimplementation of the lexer in Rust based on that description
(called the "comparable implementation" below),
and a framework for comparing its output to `rustc`'s.

## Scope

### Rust language version

This document describes Rust version 1.77, with the addition of
[rfc3593] (reserving `#"..."#` string literals in the 2024 edition).

That means it describes `c""` literals, but not
[rfc3349] (*Mixed UTF-8 literals*)

Other statements in this document are intended to be true as of April 2024.

The comparable implementation is intended to be compiled against (and compared against)\
`rustc 1.78.0-nightly (7d3702e47 2024-03-06)`

(But for the 2024 edition support on this branch, [pr123951] is also needed.)

[pr123951]: https://github.com/rust-lang/rust/pull/123951


### Editions

This document describes the editions supported by Rust 1.77:
- 2015
- 2018
- 2021

Plus some features from the in-development edition:
- 2024

There are no differences in lexing behaviour between the 2015 and 2018 editions.

In the comparable implementation, "2015" is used to refer to the common behaviour of Rust 2015 and Rust 2018.


### Accepted input

This description aims to accept input exactly if `rustc`'s lexer would.

Specifically, it aims to model what's accepted as input to a function-like macro
(a procedural macro or a by-example macro using the `tt` fragment specifier).

> See [emoji-in-unknown-prefix] for an exception

It's not attempting to accurately model `rustc`'s "reasons" for rejecting input,
or to provide enough information to reproduce error messages similar to `rustc`'s.

It's not attempting to describe `rustc`'s "recovery" behaviour
(where input which will be reported as an error provides tokens to later stages of the compiler anyway).


#### Size limits

This description doesn't attempt to characterise `rustc`'s limits on the size of the input as a whole.

> As far as I know, `rustc` has no limits on the size of individual tokens beyond its limits on the input as a whole.
> But I haven't tried to test this.


### Output form

This document only goes as far as describing how to produce a "least common denominator" stream of tokens.

Further writing will be needed to describe how to convert that stream to forms that fit the (differing) needs of the grammar and the macro systems.

In particular, this representation may be unsuitable for direct use by a description of the grammar because:

- there's no distinction between identifiers and keywords;
- there's a single "kind" of token for all punctuation;
- sequences of punctuation such as `::` aren't glued together to make a single token.

(The comparable implementation includes code to make compound punctuation tokens so they can be compared with `rustc`'s, but that process isn't described here.)


### Licence

This document and the accompanying lexer implementation are released under the terms of both the [MIT license] and the [Apache License (Version 2.0)].

[MIT license]: https://github.com/mattheww/lexeywan/blob/main/LICENSE-MIT
[Apache License (Version 2.0)]: https://github.com/mattheww/lexeywan/blob/main/LICENSE-APACHE


### Authorship and source access

© Matthew Woodcraft 2024

The source code for this document and the accompanying lexer implementation is available at <https://github.com/mattheww/lexeywan>


[rfc3349]: https://rust-lang.github.io/rfcs/3349-mixed-utf8-literals.html
[rfc3593]: https://github.com/rust-lang/rfcs/pull/3593

[emoji-in-unknown-prefix]: rustc_oddities.md#emoji-in-unknown-prefix


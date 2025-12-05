## Rustc oddities

##### Table of contents
<!-- toc -->

### NFC normalisation for lifetime/label { #nfc-lifetime }

Identifiers are normalised to [NFC],
which means that `Kelvin` and `Kelvin` are treated as representing the same identifier.
See [rfc2457].

But this doesn't happen for lifetimes or labels, so `'Kelvin` and `'Kelvin` are different as lifetimes or labels.

For example, [this][playground-lifetime] compiles without warning in Rust 1.91, while [this][playground-ident] doesn't.

In this writeup, the <var>represented ident</var> attribute of `Ident` and `Raw_ident` fine-grained tokens is in NFC,
and the <var>name</var> attribute of `Lifetime_or_label` and `Raw_lifetime_or_label` tokens isn't.

I think this behaviour is a promising candidate for provoking the
"Wait...that's what we currently do? We should fix that."
reaction to being given a spec to review.

Filed as rustc [#126759].


### Nested block comments

The Reference says "Nested block comments are supported".

Rustc implements this by counting occurrences of `/*` and `*/`, matching greedily.
That means it rejects forms like `/* xyz /*/`.

This writeup includes a `!"/*"` subexpression in the `BLOCK_COMMENT_CONTENT` definition to match rustc's behaviour.

The grammar production in the Reference seems to be written to assume that these forms should be accepted
(but I think it's garbled anyway: it accepts `/* /* */`).

I haven't seen any discussion of whether this rustc behaviour is considered desirable.


### String continuation escapes

`rustc` has a warning that the behaviour of [String continuation escapes][string-continuation]
(when multiple newlines are skipped)
may change in future.

The Reference has a [note][ref-string-continuation] about this,
and points to [#1042][Ref#1042] for more information.

[#136600] asks whether this is intentional.


[NFC]: definitions.md#nfc

[playground-lifetime]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=31fc06e4d678e1a38d8d39f521e8a11c
[playground-ident]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=aad27eb75b2774f16fc6b0981b770d56

[ref-string-continuation]: https://doc.rust-lang.org/nightly/reference/expressions/literal-expr.html#string-continuation-escapes
[string-continuation]: escape_processing.md#string-continuation-escapes

[rfc2457]: https://rust-lang.github.io/rfcs/2457-non-ascii-idents.html
[#126759]: https://github.com/rust-lang/rust/issues/126759
[#136600]: https://github.com/rust-lang/rust/issues/136600
[Ref#1042]: https://github.com/rust-lang/reference/pull/1042


## Rustc oddities

### emoji-in-unknown-prefix in Rust 2015/2018 { #emoji-in-unknown-prefix }

Rust 2015 and 2018 accept a form like `🦁#`
(as input to a function-like macro),
treating it as an identifier `🦁` followed by punctuation `#`.

This is surprising, because `🦁` would normally be rejected as an identifier.

I assume this is just a bug.

TO DO: file a bug report

This document doesn't attempt to characterise this behaviour,
and the comparable implementation doesn't attempt to imitate it.


### NFC normalisation for lifetime/label { #nfc-lifetime }

Identifiers are normalised to NFC,
which means that `Kelvin` and `Kelvin` are treated as representing the same identifier.
See [rfc2457].

But this doesn't happen for lifetimes or labels, so `'Kelvin` and `'Kelvin` are different as lifetimes or labels.

For example, [this][playground-lifetime] compiles without warning in Rust 1.77, while [this][playground-ident] doesn't.

In this writeup, the <var>represented identifier</var> attribute of `Identifier` and `RawIdentifier` fine-grained tokens is in NFC,
and the <var>name</var> attribute of `LifetimeOrLabel` tokens isn't.

I think this behaviour is a promising candidate for provoking the
"Wait...that's what we currently do? We should fix that."
reaction to being given a spec to review.


[playground-lifetime]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=31fc06e4d678e1a38d8d39f521e8a11c
[playground-ident]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=aad27eb75b2774f16fc6b0981b770d56

[rfc2457]: https://rust-lang.github.io/rfcs/2457-non-ascii-idents.html


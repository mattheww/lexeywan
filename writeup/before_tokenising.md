# Processing that happens before tokenising

##### Table of contents
<!-- toc -->

This document's description of tokenising takes a sequence of characters as input.

That sequence of characters is derived from an input sequence of bytes by performing the steps listed below in order.

It is also possible for one of the steps below to determine that the input should be rejected,
in which case tokenising does not take place.

> Normally the input sequence of bytes is the contents of a single source file.


## Decoding

The input sequence of bytes is interpreted as a sequence of characters represented using the [UTF-8] Unicode [encoding scheme].

If the input sequence of bytes is not a well-formed UTF-8 code unit sequence, the input is rejected.


## Byte order mark removal

If the first character in the sequence is `U+FEFF` (BYTE ORDER MARK), it is removed.


## CRLF normalisation

Each pair of characters <kbd>CR</kbd> immediately followed by <kbd>LF</kbd> is replaced by a single <kbd>LF</kbd> character.

> Note: It's not possible for two such pairs to overlap, so this operation is unambiguously defined.

> Note: Other occurrences of the character <kbd>CR</kbd> are left in place.
> It's still possible for the sequence <kbd>CR</kbd><kbd>LF</kbd> to be passed on to the tokeniser:
> that will happen if the input contained the sequence <kbd>CR</kbd><kbd>CR</kbd><kbd>LF</kbd>.


## Shebang removal

Shebang removal is performed if:

 - the remaining sequence begins with the characters <b>#!</b>; and
 - the result of [finding the first non-whitespace token] with the characters following the <b>#!</b> as input is not a `Punctuation` token whose <var>mark</var> is the <b>[</b> character.

If shebang removal is performed:
- the characters up to and including the first <kbd>LF</kbd> character are removed from the sequence
- if the sequence did not contain a <kbd>LF</kbd> character, all characters are removed from the sequence.

> Note: The check for <b>[</b> prevents an inner attribute at the start of the input being removed.
> See [#70528] and [#71487] for history.


## Frontmatter removal

> Stability: As of Rust 1.90 frontmatter removal is unstable.
> Under stable rustc 1.90, and under nightly rustc without the `frontmatter` feature flag,
> input which would undergo frontmatter removal is rejected.

If an attempt to match the `FRONTMATTER` nonterminal defined in the frontmatter grammar against the remaining sequence succeeds,
the characters consumed by that match are removed from the sequence.

Otherwise, if an attempt to match the `RESERVED` nonterminal defined in the frontmatter grammar against the remaining sequence succeeds,
the input is rejected.

The frontmatter grammar is the following [Parsing Expression Grammar](pegs.md):

```
{{#include frontmatter_anchored.pest:main}}
```

> See [Special terminals] for the definition of `PATTERN_WHITE_SPACE`.

These definitions require an extension to the Parsing Expression Grammar formalism:
each of the parsing expressions marked as `FENCE²` fails
unless the characters it consumes are the same as the characters consumed by the (only) match of the expression marked as `FENCE¹`.

> See [Grammar for raw string literals](raw_strings.md) for a discussion of alternatives to this extension.

> Note: If there are any `WHITESPACE_ONLY_LINE`s, rustc emits a single whitespace token to represent them.
> But I think that token isn't observable by Rust programs, so it isn't modelled here.


[UTF-8]: https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G31703
[encoding scheme]: https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G28070

[Special terminals]: grammars.md#special-terminals
[finding the first non-whitespace token]: tokenising.md#find-first-nw-token

[#70528]: https://github.com/rust-lang/rust/issues/70528
[#71487]: https://github.com/rust-lang/rust/pull/71487

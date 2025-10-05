# Lowering doc-comments

This phase of processing converts an input sequence of [fine-grained tokens] to a new sequence of fine-grained tokens.

The new sequence is the same as the input sequence,
except that each `LineComment` or `BlockComment` token whose <var>style</var> is **inner doc** or **outer doc** is replaced with the following sequence:

- `Punctuation` with <var>mark</var> <b>#</b>
- `Whitespace`
- `Punctuation` with <var>mark</var> <b>!</b> (omitted if the comment token's <var>style</var> is **outer doc**)
- `Punctuation` with <var>mark</var> <b>[</b>
- `Ident` with <var>represented identifier</var> <b>doc</b>
- `Punctuation` with <var>mark</var> <b>=</b>
- `Whitespace`
- `RawStringLiteral` with the comment token's <var>body</var> as the <var>represented string</var> and empty <var>suffix</var>
- `Punctuation` with <var>mark</var> <b>]</b>

> Note: the whitespace tokens aren't observable by anything currently described in this writeup,
> but they explain the `spacing` in the tokens that proc-macros see.

[fine-grained tokens]: fine_grained_tokens.md

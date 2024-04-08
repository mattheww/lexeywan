# Processing that happens before tokenising

This document's description of tokenising takes a sequence of characters as input.

`rustc` obtains that sequence of characters as follows:

> This description is taken from the *[Input format]* chapter of the Reference.


## Source encoding

Each source file is interpreted as a sequence of Unicode characters encoded in UTF-8.
It is an error if the file is not valid UTF-8.

## Byte order mark removal

If the first character in the sequence is `U+FEFF` (BYTE ORDER MARK), it is removed.

## CRLF normalization

Each pair of characters `U+000D` <kbd>CR</kbd> immediately followed by `U+000A` <kbd>LF</kbd> is replaced by a single `U+000A` <kbd>LF</kbd>.

Other occurrences of the character `U+000D` <kbd>CR</kbd> are left in place (they are treated as whitespace).

> Note: this document's description of tokenisation doesn't assume that the sequence <kbd>CR</kbd><kbd>LF</kbd> never appears in its input;
> that makes it more general than necessary, but should do no harm.
>
> In particular, in places where the Reference says that tokens may not contain "lone CR", this description just says that any <kbd>CR</kbd> is rejected.


## Shebang removal

If the remaining sequence begins with the characters <b>#!</b>, the characters up to and including the first `U+000A` <kbd>LF</kbd> are removed from the sequence.

For example, the first line of the following file would be ignored:

```rust,ignore
#!/usr/bin/env rustx

fn main() {
    println!("Hello!");
}
```

As an exception, if the <b>#!</b> characters are followed (ignoring intervening comments or whitespace) by a `[` punctuation token, nothing is removed.
This prevents an inner attribute at the start of a source file being removed.

> See open question: [How to model shebang removal]

[Input format]: https://doc.rust-lang.org/nightly/reference/input-format.html
[How to model shebang removal]: open_questions.md#how-to-model-shebang-removal

# Processing that happens before tokenising

This document's description of tokenising takes a sequence of characters as input.

`rustc` obtains that sequence of characters as follows:

> This description is taken from the *[Input format]* chapter of the Reference.


## Source encoding

Each source file is interpreted as a sequence of Unicode characters encoded in UTF-8.
It is an error if the file is not valid UTF-8.

## Byte order mark removal

If the first character in the sequence is `U+FEFF` (BYTE ORDER MARK), it is removed.

## CRLF normalisation

Each pair of characters `U+000D` <kbd>CR</kbd> immediately followed by `U+000A` <kbd>LF</kbd> is replaced by a single `U+000A` <kbd>LF</kbd>.

Other occurrences of the character `U+000D` <kbd>CR</kbd> are left in place (they are treated as whitespace).

> Note: It's still possible for the sequence <kbd>CR</kbd><kbd>LF</kbd> to be passed on to the tokeniser:
> that will happen if the source file contained the sequence <kbd>CR</kbd><kbd>CR</kbd><kbd>LF</kbd>.


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

# Overview

The following processes might be considered to be part of Rust's lexer:

- [Decode]: interpret UTF-8 input as a sequence of Unicode characters
- [Clean]:
  - Byte order mark removal
  - CRLF normalisation
  - Shebang removal
  - Frontmatter removal
- [Tokenise]: interpret the characters as ("fine-grained") tokens
- [Lower doc-comments]: convert doc-comments into attributes
- Build trees: organise tokens into delimited groups
- Combine: convert fine-grained tokens to compound tokens (for declarative macros)
- Prepare proc-macro input: convert fine-grained tokens to the form used for proc-macros
- Remove whitespace: remove whitespace tokens

> This document attempts to completely describe the "Decode", "Clean", "Tokenise", and "Lower doc-comments" processes.

[Decode]: before_tokenising.md#decoding
[Clean]: before_tokenising.md#byte-order-mark-removal
[Tokenise]: tokenising.md
[Lower doc-comments]: doc_comments.md

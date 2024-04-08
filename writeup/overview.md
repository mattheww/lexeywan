# Overview

The following processes might be considered to be part of Rust's lexer:

- Decode: interpret UTF-8 input as a sequence of Unicode characters
- [Clean]:
  - Byte order mark removal
  - CRLF normalisation
  - Shebang removal
- [Tokenise]: interpret the characters as ("fine-grained") tokens
- Further processing: to fit the needs of later parts of the spec
  - For example, convert fine-grained tokens to compound tokens
  - possibly different for the grammar and the two macro implementations

> This document attempts to completely describe the "Tokenise" process.

[Clean]: before_tokenising.md
[Tokenise]: tokenising.md

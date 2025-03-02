This repository includes:

* a detailed description of the Rust 1.85 lexer (in `writeup`)
* a Rust reimplementation of the lexer based on that description (in `src`)
* a manual list of testcases
* a harness for running `rustc`'s lexer in-process (via `rustc_private`)
* strategies for comparing the implementation with `rustc`'s using [`proptest`]

> **This branch also documents the behaviour of [pr131656]\
> _lexer: Treat more floats with empty exponent as valid tokens_\
> as of 2025-03-02**

[pr131656]: https://github.com/rust-lang/rust/pull/131656


[`proptest`]: https://proptest-rs.github.io/proptest/intro.html


See also the [rendered description][1].

[1]: https://mjw.woodcraft.me.uk/2025-lexeywan-e-suffix/


## Running the tests

To see what's available from the CLI:

```
cargo run -- --help
```

Note the provided `rust-toolchain.toml` will cause this to install the required nightly version of `rustc`.

At present three tests should fail:
two where the comparable implementation's approximation to rustc's shebang removal isn't good enough,
and one because rustc declines to lex input with unbalanced delimiters.


## Building the description

```
mdbook build
```

The output will appear in `book/`



## License

This document is distributed under the terms of both the MIT license and the Apache license (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

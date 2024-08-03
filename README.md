This repository includes:

* a detailed description of the Rust 1.80 lexer (in `writeup`)
* a Rust reimplementation of the lexer based on that description (in `src`)
* a manual list of testcases
* a harness for running `rustc`'s lexer in-process (via `rustc_private`)
* strategies for comparing the implementation with `rustc`'s using [`proptest`]

[`proptest`]: https://proptest-rs.github.io/proptest/intro.html


See also the [rendered description][1].

[1]: https://mjw.woodcraft.me.uk/2024-lexeywan/


## Running the tests

To see what's available from the CLI:

```
cargo run -- --help
```

Note the provided `rust-toolchain.toml` will cause this to install the required nightly version of `rustc`.


## Building the description

```
mdbook build
```

The output will appear in `book/`



## License

This document is distributed under the terms of both the MIT license and the Apache license (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

This repository includes:

* a detailed description of the Rust 1.91 lexer (in `writeup`)
* a Rust reimplementation of the lexer based on that description (in `src/reimplementation`)
* a manual list of testcases (in `src/framework/testcases.rs`)
* harnesses for running `rustc`'s lexer in-process via `rustc_private` (in `src/rustc_harness`:
  * one comparing the reimplementation to the `rustc_parse` high-level lexer
  * one comparing the reimplementation to what declarative macros see
* strategies for comparing the reimplementation with `rustc_parse`'s using [`proptest`]

[`proptest`]: https://proptest-rs.github.io/proptest/intro.html


See also the [rendered description][1].

[1]: https://mjw.woodcraft.me.uk/2025-lexeywan/


## Running the tests

See [Reimplementation command-line interface] for what's available from the CLI:

[Reimplementation command-line interface]: https://mjw.woodcraft.me.uk/2025-lexeywan//reimplementation_cli.html

Note the provided `rust-toolchain.toml` will cause `cargo run` to install the required nightly version of `rustc`.


## Building the description

Install [`mdbook`] and [`mdbook-toc`]:

```
cargo install mdbook mdbook-toc
```

Then run

```
scripts/build_writeup
```

The output will appear in `book/`


[`mdbook`]: https://github.com/rust-lang/mdBook
[`mdbook-toc`]: https://github.com/badboy/mdbook-toc


## License

The contents of this repository are distributed under the terms of both the MIT license and the Apache license (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

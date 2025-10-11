## Command-line interface for the reimplementation

The [repository containing this writeup] also contains a binary crate which contains the reimplementation and a command line program for comparing the reimplementation against rustc
(linking against the rustc implementation via `rustc_private`).

Run it in the usual way, from a working copy:

```
cargo run -- <subcommand> [options]
```

Note the repository includes a `rust-toolchain.toml` file
which will cause `cargo run` to install the required nightly version of `rustc`.


### Summary usage

```
Usage: lexeywan [<subcommand>] [...options]

Subcommands:
 *test          [suite-opts]
  compare       [suite-opts] [comparison-opts] [dialect-opts]
  decl-compare  [suite-opts] [comparison-opts] [--edition=2015|2021|*2024]
  inspect       [suite-opts] [dialect-opts]
  coarse        [suite-opts] [dialect-opts]
  identcheck
  proptest      [--count] [--strategy=<name>] [--print-failures|--print-all]
                [dialect-opts]

suite-opts (specify at most one):
  --short: run the SHORTLIST rather than the LONGLIST
  --xfail: run the tests which are expected to fail

dialect-opts:
  --edition=2015|2021|*2024
  --cleaning=none|*shebang|shebang-and-frontmatter
  --lower-doc-comments

comparison-opts:
  --failures-only: don't report cases where the lexers agree
  --details=always|*failures|never

* -- default
```

### Subcommands

The following subcommands are available:

#### `test`

This is the main way to check the whole system for disagreements.

`test` is the default subcommand.

For each known edition, it runs the following for the [requested testcases]:

- comparison of `rustc_parse`'s lexer and the reimplementation, like for `compare` with options
   - `--cleaning=none`
   - `--cleaning=shebang`
   - `--cleaning=shebang-and-frontmatter`
   - `--cleaning=shebang --lower-doc-comments`
- comparison via declarative macros, like for `decl-compare`

For each comparison it reports whether the implementations agreed for all testcases,
without further detail.


#### `compare`

This is the main way to run the testsuite and see results for individual testcases.

It analyses the [requested testcases] with the `rustc_parse` lexer and the reimplementation,
and compares the output.

The analysis uses a single [dialect].

For each testcase, the comparison _agrees_ if either:

- both implementations accept the input and produce the same forest of regularised tokens; or
- both implementations reject the input.

Otherwise the comparison _disagrees_.

See `regular_tokens.rs` for what regularisation involves.

The comparison may also mark a testcase as a _model error_.
This happens if rustc panics or the reimplementation reports an internal error.


##### Example output

```
‼ R:✓ L:✗ «//comment»
✔ R:✓ L:✓ «'label»
```
Here, the first line says that rustc (`R`) accepted the input `//comment` but the reimplementation (`L`) rejected it.
The initial `‼` indicates the disagreement.

The second line says that both rustc and the reimplementation accepted the input `'label`.
The initial `✔` indicates the agreement.


##### Output control

By default `compare` prints a line (of the sort shown in the example above) for each testcase.
Pass `--failures-only` to only print lines for the cases where the implementations disagree.

The `compare` subcommand can also report further detail for a testcase:

- if the input is accepted, the forest of regularised tokens
- if the input is rejected, the rustc error message or the reimplementation's reason for rejection

Further detail is controlled as follows:

|                                |                                         |
|--------------------------------|-----------------------------------------|
| `--details=always`             | Report detail for all testcases         |
| `--details=failures` (default) | Report detail for disagreeing testcases |
| `--details=never`              | Report detail for no testcases          |


#### `inspect`

This shows more detail than `compare`, but doesn't report on agreement.

Analyses the [requested testcases] using the `rustc_parse` lexer and the reimplementation,
and prints each lexer's analysis.

Uses a specified [dialect].

Unlike `compare`, this shows the tokens before regularisation.

For the reimplementation, it shows details about what the grammar matched,
and fine-grained tokens.

If rustc rejects the input (and the rejection wasn't a fatal error),
it reports the tokens rustc would have passed on to the parser.

If the reimplementation rejects the input, reports what has been tokenised so far.
If the rejection comes from processing,
describes the rejected match and reports any matches and fine-grained tokens from before the rejection.


#### `coarse`

This shows the reimplementation's coarse-grained tokens.

Analyses the [requested testcases] using the reimplementation only,
including combination of fine-grained tokens into coarse-grained tokens,
and prints a representation of the analysis.

Uses a specified [dialect].


#### `decl-compare`

This is a second way to test the observable behaviour of Rust's lexer,
which doesn't depend so much on `rustc_parse`'s internals.

It analyses the [requested testcases] via declarative macros,
and compares the result to what the reimplementation expects.

The analysis works by defining a macro using the `tt` fragment specifier which applies `stringify!` to each parameter,
embedding the testcase in an invocation of that macro,
running rustc's macro expander and parser,
and inspecting the results.

See the comments in `decl_via_rustc.rs` for details.

The reimplementation includes a model of what `stringify!` does.

It uses the selected edition.
Doc-comments are always lowered.
The only [Processing that happens before tokenising] step performed is CRLF normalisation.

The `--details` and `--failures-only` options work in the same way as for `compare`;
"details" shows the stringified form of each token.


#### `identcheck`

This checks that the `rustc_parse` lexer and the reimplementation agree which characters are permitted in identifiers.

For each Unicode character `C` this constructs a string containing `C aC`,
and checks the reimplementation and `rustc_parse` agree on its analysis.

It reports the number of agreements and disagreements.

It will notice if the Unicode version changes in one of the implementations
(rustc's Unicode version comes from its `unicode-xid` dependency,
 and the reimplementation's comes from its `pest` dependency).

It won't notice if the Unicode version used for NFC normalisation is out of sync
(for both the reimplementation and rustc, this comes from the `unicode-normalization` dependency).

`identcheck` always uses the latest available edition.


#### `proptest`

This performs randomised testing.

It uses [proptest] to generate random strings,
analyses them with the `rustc_parse` lexer and the reimplementation,
and compares the output.
The analysis and comparison is the same as for `compare` above,
for a specified [dialect].

If this process finds a string which results in disagreement (or a model error),
[proptest] simplifies it as much as it can while still provoking the problem,
then testing stops.

The `--count` argument specifies how many strings to generate
(the default is 5000).

The `--strategy` argument specified how to generate the strings.
See `SIMPLE_STRATEGIES` in `strategies.rs` for the list of available strategies.
The `mix` strategy is the default.


##### Output control

By default `proptest` prints a single reduced disagreement, if it finds any.

If `--print-all` is passed it prints each string it generates.

If `--print-failures` is passed it prints each disagreeing testcase it generates,
so you can see the simplification process.


### Dialects

The `compare`, `inspect`, `coarse`, and `proptest` subcommands accept the following options to control the lexers' behaviour:

- `--edition=2015|2021|2024`
- `--cleaning=none|shebang|shebang-and-frontmatter`
- `--lower-doc-comments`

The `decl-compare` subcommand accepts only `--edition`.

The options apply both to rustc and the reimplementation.

`--edition` controls which Rust edition's lexing semantics are used.
It defaults to the most recent known edition.
There's no `2018` option because there were no lexing changes between the 2015 and 2018 editions.

`--cleaning` controls which of the steps described in [Processing that happens before tokenising] are performed.
It defaults to `shebang`.
Byte order mark removal and CRLF normalisation are always performed.
(The reimplementation doesn't model the "Decoding" step,
 because the hard-coded testcases are provided as Rust string literals and so are already UTF-8.)

If `--lower-doc-comments` is passed,
doc-comments are converted to attributes as described in [Lowering doc-comments].


### Choosing the testcases to run { #testcases }

By default, subcommands which need a list of testcases use the list hard-coded as `LONGLIST` in `testcases.rs`.

Pass `--short` to use the list hard-coded as `SHORTLIST` instead.

Pass `--xfail` to use the list hard-coded as `XFAIL` instead.
This list includes testcases which are expected to fail or disagree with at least one subcommand and set of options.


### Exit status

Each subcommand which compares the reimplementation to rustc reports exit status 0 if all comparisons agreed,
or exit status 3 if any comparison disagreed or any model errors were observed.

For all subcommands, exit status 1 indicates an unhandled error.


[Processing that happens before tokenising]: before_tokenising.md
[Lowering doc-comments]: doc_comments.md
[repository containing this writeup]: https://github.com/mattheww/lexeywan
[requested testcases]: #testcases
[dialect]: #dialects
[Dialects]: #dialects
[proptest]: https://proptest-rs.github.io/proptest/intro.html

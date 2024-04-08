## Rationale for this model

### Pretokenising

The main difference between the model described in this document and the way the Reference (as of Rust 1.77) describes lexing is the split into pretokenisation and reprocessing.

There are a number of forms which are errors at lexing time, even though in principle they could be analysed as multiple tokens.

Examples include

- the [rfc3101] "reserved prefixes" (in Rust 2021 and newer): `k#abc`,  `f"..."`, or `f'...'`.
- the [rfc0879] variants of numeric literals reserved in , eg `0x1.2` or `0b123`
- adjacent-lifetime-like forms such as `'ab'c`
- stringlike literals with a single `_` as a suffix
- byte or C strings with unacceptable contents that would be accepted in plain strings, eg `b"€"`, `b"\u{00a0}"`, or `c"\x00"`

The Reference tries to account for some of these cases by adding rules which match the forms that cause errors, while keeping the forms matched by those rules disjoint from the forms matched by the non-error-causing rules.

The resulting rules for reserved prefixes and numeric literals are quite complicated (and still have mistakes).
Rules of this sort haven't been attempted stringlike literals.

The rules are simpler in a model with a 'pretokenising' step which can match a form such as `c"\x00"` (preventing it being matched as `c` followed by `"\x00"`), leaving it to a later stage to decide whether it's a valid token or a lexing-time error.

This separation also gives us a natural way to lex doc and non-doc comments uniformly,
and inspect their contents later to make the distinction,
rather than trying to write non-overlapping lexer rules as the Reference does.


### Lookahead

The model described in this document uses one-character lookahead (beyond the token which will be matched) in the prelexing step, in two cases:

- the lifetime-or-label rule, to prevent `'ab'c'` being analysed as `'ab` followed by `'c`
- the rule for float literals ending in `.`, to make sure that `1.a` is analysed as `1` `.` `a` rather than `1.` `a`

I think some kind of lookahead is unavoidable in these cases.

I think the lookahead could be done after prelexing instead, by adding a pass that could reject pretokens or join them together, but I think that would be less clear.
(In particular, the float rule would end up using a list of pretoken kinds that start with an identifier character, which seems worse than just looking for such a character.)


### Constraints and imperative code

There are two kinds of token which are hard to deal with using a "regular" lexer:
raw-string literals (where the number of `#` characters need to match),
and block comments (where the `/*` and `*/` delimiters need to be balanced).

Raw-string literals can in principle fit into a regular language because there's a limit of 255 `#` symbols, but it seems hard to do anything useful with that.

Nested comments can in principle be described using non-regular rules (as the Reference does).

The model described in this document deals with these cases by allowing rules to define constraints beyond the simple pattern match, effectively intervening in the "find the longest match" part of pattern matching.

The constraint for raw strings is simple, but the one for block comments has ended up using imperative code, which doesn't seem very satisfactory.
See [Defining the block-comment constraint][block-comment-constraint].


### Producing tokens with attributes

The "output" of the lexing process described in this document is a stream of tokens which have attributes, of their own, rather than simply representing slices of the input source.

The main motivation for this is to deal with stringlike literals:
it means we don't need to separate the description of the result of "unescaping" strings from the description of which strings contain well-formed escapes.

In particular, describing unescaping at lexing time makes it easy to describe the rule about rejecting NULs in C-strings, even if they were written using an escape.

But in any case I hope that working with tokens which have attributes will be the best starting point for describing what the "clients" of these tokens (the grammar and the two macro implementations) do,
and of course working with tokens with attributes is what actually happens in rustc.


[rfc0879]: https://rust-lang.github.io/rfcs/0879-small-base-lexing.html
[rfc3101]: https://rust-lang.github.io/rfcs/3101-reserved_prefixes.html

[block-comment-constraint]: open_questions.md#block-comment-constraint

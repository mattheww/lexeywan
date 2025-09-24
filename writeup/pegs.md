## Parsing Expression Grammar notation

Parsing Expression Grammars are described informally in §2 of [Ford 2004][peg-paper].

The notation used in this document is the [variant used by][pest-grammar] the [Pest] Rust library,
so that it's easy to keep in sync with the comparable implementation.

In particular:

 - the sequencing operator is written explicitly, as `~`
 - the ordered choice operator is `|`
 - `?`, `*`, and `+` have their usual senses (as expression suffixes)
 - `{0, 255}` is a repetition suffix, meaning "from 0 to 255 repetitions"
 - the not-predicate (for negative lookahead) is `!` (as an expression prefix)
 - a terminal matching an individual character is written like `"x"`
 - a terminal matching a sequence of characters is written like `"abc"`
 - a terminal matching a range of characters is written like `'0'..'9'`
 - `"\""` matches a single <b>"</b> character
 - `"\\"` matches a single <b>\\</b> character
 - `"\n"` matches a single <kbd>LF</kbd> character

The ordered choice operator `|` has the lowest precedence, so
```
a ~ b | c ~ d
```
is equivalent to
```
( a ~ b ) | ( c ~ d )
```

The sequencing operator `~` has the next-lowest precedence, so
```
!"." ~ SOMETHING
```
is equivalent to
```
(!".") ~ SOMETHING
```

"Any character except" is written using the not-predicate and `ANY`, for example
```
( !"'" ~ ANY )
```
matches any single character except <b>'</b>.

See [Grammar for raw string literals](raw_strings.md) for a discussion of extensions used to model raw string literals and frontmatter fences.


[Pest]: https://pest.rs/book/grammars/syntax.html
[pest-grammar]: https://docs.rs/pest_derive/latest/pest_derive/#grammar
[peg-paper]: https://pdos.csail.mit.edu/papers/parsing:popl04.pdf


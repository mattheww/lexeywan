# Grammars used in this writeup

This document relies on two _parsing expression grammars_:
one for tokenising and one for recognising frontmatter.

This page summarises how these grammars work.
See the [Parsing Expression Grammars][pegs] appendix for a more formal treatment.

See [Frontmatter grammar] and [Complete tokenisation grammar] for the grammars themselves.

There is no standardised notation for parsing expression grammars.
This writeup is based on the [variant used by][pest-grammar] the [Pest] Rust library,
so that it's easy to keep in sync with the reimplementation.

See [Grammar for raw string literals](raw_strings.md) for a discussion of extensions used to model raw string literals and frontmatter fences.
Those extensions are not described on this page.


##### Table of contents
<!-- toc -->

<div class=pegs-description>

## Grammars

Here is an example of a grammar:

```
DIGITS = { '0'..'9' + }
NUMBER = { DIGITS ~ "." ~ DIGITS }
VARIABLE = { ( 'a'..'z' | "_" ) + }
VALUE = { NUMBER | VARIABLE }
```

The name on the left hand side of each definition is a <dfn>nonterminal</dfn>.

The right hand side of each definition contains a <dfn>parsing expression</dfn> (between `{` and `}`)
which describes what that nonterminal matches.


## Matching

Given a grammar, we can <dfn>attempt to match</dfn> a nonterminal against an input sequence of characters.

If the start of the input matches what the nonterminal's parsing expression requires,
we say the attempt <dfn>succeeds</dfn> and <dfn>consumes</dfn> the matched characters.
Otherwise we say the attempt <dfn>fails</dfn>.

We describe the result of a successful attempt as a <dfn>match</dfn>.

The table below summarises the forms a parsing expression can take,
and describes what each form matches.


## Participating in a match

The process of matching a nonterminal often involves matching further nonterminals against a part of the input.
We say that those further nonterminals <dfn>participated in</dfn> the match,
and their matches are <dfn>participating matches</dfn>.

> __Examples__
>
> If `VALUE` in the example grammar is matched against <b>abc</b>,
> `VARIABLE` participates in the match,
> and the participating match consumes the characters <b>abc</b>.
>
> If `NUMBER` in the example grammar is matched against <b>123.456</b>,
> there are two participating matches of the `DIGITS` nonterminal,
> the first consuming <b>123</b> and the second consuming <b>456</b>.


## Parsing expressions

The following forms of parsing expression are available:

| Form                                                   | Matching                                                                               |
|--------------------------------------------------------|----------------------------------------------------------------------------------------|
| eg `"abc"`                                             | Match the exact string provided                                                        |
| eg `'a'..'f'`                                          | Match one [character] from the provided (inclusive) range                              |
| A nonterminal                                          | Match the nonterminal's parsing expression                                             |
| <code><var>e₁</var> ~ <var>e₂</var></code>             | First match <var>e₁</var>, then match <var>e₂</var>                                    |
| <code><var>e₁</var> \| <var>e₂</var></code>            | Match either <var>e₁</var> or <var>e₂</var>, with <var>e₁</var> having higher priority |
| <code><var>e</var> ?</code>                            | Match <var>e</var> if possible                                                         |
| <code><var>e</var> *</code>                            | Match as many repetitions of <var>e</var> as possible (possibly zero)                  |
| <code><var>e</var> +</code>                            | Match as many repetitions of <var>e</var> as possible (at least one)                   |
| <code><var>e</var> {<var>m</var>, <var>n</var>}</code> | Match between <var>m</var> and <var>n</var> (inclusive) repetitions of <var>e</var>    |
| <code>! <var>e</var></code>                            | Fail if <var>e</var> would match at this point                                         |
| <code>( <var>e</var> )</code>                          | Match <var>e</var>, overriding the usual precedence                                    |

Here, <var>e</var>, <var>e₁</var>, and <var>e₂</var> can be any parsing expression,
and <var>m</var> and <var>n</var> can be any positive whole number.


### Special terminals

In addition, the following named terminals are available in all grammars in this document:

| Terminal              | Matches                                                       |
|-----------------------|---------------------------------------------------------------|
| `ANY`                 | Any single Unicode [character]                                |
| `DOUBLEQUOTE`         | A <b>"</b> [character]                                        |
| `BACKSLASH`           | A <b>\\</b> [character]                                       |
| `CR`                  | A <kbd>CR</kbd> [character]                                  |
| `LF`                  | An <kbd>LF</kbd> [character]                                  |
| `TAB`                 | An <kbd>HT</kbd> [character]                                  |
| `PATTERN_WHITE_SPACE` | A [character] with the Unicode `Pattern_White_Space` property |
| `XID_START`           | A [character] with the Unicode `XID_Start` property           |
| `XID_CONTINUE`        | A [character] with the Unicode `XID_Continue` property        |
| `EOI`                 | The end of input                                              |

`EOI` only matches when the remaining input is empty.

The `Pattern_White_Space` Unicode character property is defined in `PropList.txt` in the [Unicode character database][UCD].
The `XID_Start` and `XID_Continue` Unicode character properties are defined in `DerivedCoreProperties.txt` in the [Unicode character database][UCD].

> Note: The characters with the `PATTERN_WHITE_SPACE` Unicode character property are:
>
> |        |                           |
> |:-------|:--------------------------|
> | U+0009 | CHARACTER TABULATION (HT) |
> | U+000A | LINE FEED (LF)            |
> | U+000B | LINE TABULATION (VT)      |
> | U+000C | FORM FEED (FF)            |
> | U+000D | CARRIAGE RETURN (CR)      |
> | U+0020 | SPACE                     |
> | U+0085 | NEXT LINE (NEL)           |
> | U+200E | LEFT-TO-RIGHT MARK        |
> | U+200F | RIGHT-TO-LEFT MARK        |
> | U+2028 | LINE SEPARATOR            |
> | U+2029 | PARAGRAPH SEPARATOR       |
>
> This set doesn't change in updated Unicode versions.


### Prioritised choice

The prioritised choice operator `|` is the distinctive feature of parsing expression grammars.

An attempt to match `ONE | TWO` first attempts to match `ONE`,
and if that succeeds it never considers `TWO`.
If the match of `ONE` fails it attempts to match `TWO` instead.

> __Example__
>
> Matching `"aa" | "aaa"` against <b>aaa</b> consumes the characters <b>aa</b>,
> not <b>aaa</b>.


### Repetition and backtracking

The repetition operators `*` and `+` always match as many repetitions as possible.
If they are used as part of a larger match attempt which later fails,
the matching process does not backtrack to see if the whole match can succeed if the repetition expression consumes fewer repetitions.

For example, matching `"a"* ~ "ab"` against <b>aaab</b> fails.

Similarly <code>{<var>m</var>, <var>n</var>}</code> and `?` match as much as they can when they are first attempted,
and there is no backtracking.

> __Examples__
>
> Matching `"ab"? ~ "abc"` against <b>abc</b> fails.
>
> Matching `"ab" ~ "abc" | "abc"`  against <b>abc</b> succeeds.

> __Example__
>
> With the following grammar
> ```
> LETTER = { 'a'..'z' }
> LETTER_OR_DOT = { 'a'..'z' | "." }
> ENDS_WITH_LETTER = { LETTER_OR_DOT * ~ LETTER }
> ```
> matching `ENDS_WITH_LETTER` against <b>abcde</b> fails.
>
> With this grammar it succeeds:
>
> ```
> LETTER = { 'a'..'z' }
> LETTER_OR_DOT = { 'a'..'z' | "." }
> ENDS_WITH_LETTER = { LETTER_OR_DOT ~ ENDS_WITH_LETTER | LETTER }
> ```

### Precedence

The prioritised choice operator `|` has the lowest precedence, so for example
```
ONE ~ TWO | THREE ~ FOUR
```
is equivalent to
```
( ONE ~ TWO ) | ( THREE ~ FOUR )
```

The sequencing operator `~` has the next-lowest precedence, so for example
```
!"." ~ SOMETHING
```
is equivalent to
```
(!".") ~ SOMETHING
```

Both the `|` and `~` operators can be used repeatedly without parentheses, so for example

```
ONE | TWO | THREE
```
means "match `ONE`, `TWO`, or `THREE`, in descending order of priority", and

```
ONE ~ TWO ~ THREE
```
means "first match `ONE`, then match `TWO`, then match `THREE`".



## Common idioms

"Any character except" is written using the negative lookup operator and `ANY`.

> __Example__
>
> ```
> ( !"'" ~ ANY )
> ```
> matches any single character except <b>'</b>.

</div>

[UCD]: definitions.md#unicode
[pegs]: pegs.md
[Frontmatter grammar]: frontmatter_grammar.md
[Complete tokenisation grammar]: complete_token_grammar.md

[character]: definitions.md#character
[characters]: definitions.md#character

[UAX31]: https://www.unicode.org/reports/tr31/tr31-41.html
[Pest]: https://pest.rs/book/grammars/syntax.html
[pest-grammar]: https://docs.rs/pest_derive/latest/pest_derive/#grammar

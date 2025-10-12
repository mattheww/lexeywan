# Tokenising

##### Table of contents
<!-- toc -->

This phase of processing takes a character sequence (the *input*), and either:

- produces a sequence of [fine-grained tokens]; or
- reports that lexical analysis failed

The analysis depends on the Rust edition which is in effect when the input is processed.

> So strictly speaking, the edition is a second parameter to the process described here.

The process is to repeatedly [extract a fine-grained token from the start of the input]
until lexical analysis fails or the input is empty.

If lexical analysis does not fail,
those extracted tokens make up the sequence of tokens produced by this phase.

## The grammar

Tokenisation is described by a [Parsing Expression Grammar](pegs.md) which describes how to match a single fine-grained token.

> The grammar isn't strictly a Parsing Expression Grammar.
> See [Grammar for raw string literals](raw_strings.md)

The grammar defines an _edition nonterminal_ for each Rust edition:

| Edition      | Edition nonterminal |
|--------------|---------------------|
| 2015 or 2018 | `TOKEN_2015`        |
| 2021         | `TOKEN_2021`        |
| 2024         | `TOKEN_2024`        |

Each edition nonterminal is defined as a choice expression, each of whose subexpressions is a single nonterminal (a _token nonterminal_).

##### Grammar
```
{{#include tokenise_anchored.pest:tokens}}
```

The token nonterminals are distinguished in the grammar as having names in `Title_case`.

The rest of the grammar is presented in the following pages in this section.
The definitions of some nonterminals are repeated on multiple pages for convenience.
The full grammar is also available on a [single page](complete_token_grammar.md).

The token nonterminals are presented in an order consistent with their appearance in the edition nonterminals.
That means they appear in priority order (highest priority first).


## Extracting fine-grained tokens

To *extract a fine-grained token from the start of the input*:

- Attempt to match the edition's edition nonterminal at the start of the input.
- If the match fails, lexical analysis fails.
- If the match succeeds, process the match as described below.
  - Then if the match is rejected, lexical analysis fails.
  - Otherwise the processing determines the extracted token.
- Remove the characters consumed by the edition nonterminal from the start of the input.

> Strictly speaking we have to justify the assumption that the match will always either fail or succeed,
> which basically means observing that the grammar has no left recursion.


### Processing a match

It follows from the definitions of the edition nonterminals above that a single token nonterminal participates in any successful match.

For each token nonterminal there is a subsection on the following pages
which describes how to process a match in which that token nonterminal participated.

Each of these subsections specifies which matches are rejected.
For matches which are not rejected,
the subsection specifies the kind and attributes of the token that is produced.

If for any match the subsection doesn't either say that the match is rejected or uniquely specify the produced token's kind and the value for each of that token kind's attributes,
it's a bug in this writeup.

##### Referring to matched characters

In these subsections, notation of the form <u>NTNAME</u> denotes the sequence of characters consumed by the nonterminal named `NTNAME` which participated in the match.

If this notation is used for a nonterminal which might participate more than once in the match,
it's a bug in this writeup.


## Finding the first non-whitespace token { #find-first-nw-token }

> This section defines a variant of the tokenisation process which is used in the definition of [Shebang removal].

The process of _finding the first non-whitespace token_ in a character sequence (the _input_) is:
1. if the input is empty, the result is **no token**
2. [extract a fine-grained token from the start of the input]
3. if the extraction determines that lexical analysis should fail, the result is **no token**.
4. if the extracted fine-grained token is not a token representing whitespace, the result is that token
5. otherwise, return to step 1


For this purpose a <dfn>token representing whitespace</dfn> is any of:
 - a `Whitespace` token
 - a `LineComment` token whose <var>style</var> is **non-doc**
 - a `BlockComment` token whose <var>style</var> is **non-doc**


[extract a fine-grained token from the start of the input]: #extracting-fine-grained-tokens
[Shebang removal]: before_tokenising.md#shebang-removal
[fine-grained tokens]: fine_grained_tokens.md


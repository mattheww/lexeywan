# Tokenising

##### Table of contents
<!-- toc -->

## The tokenisation grammar

The <dfn>tokenisation grammar</dfn> is a [Parsing Expression Grammar](pegs.md)
which describes how to divide the input into [fine-grained tokens].

> The tokenisation grammar isn't strictly a Parsing Expression Grammar.
> See [Grammar for raw string literals](raw_strings.md)

The tokenisation grammar defines a <dfn>tokens nonterminal</dfn> and a <dfn>token nonterminal</dfn> for each Rust edition:

| Edition      | Tokens nonterminal | Token nonterminal |
|--------------|--------------------|-------------------|
| 2015 or 2018 | `TOKENS_2015`      | `TOKEN_2015`      |
| 2021         | `TOKENS_2021`      | `TOKEN_2021`      |
| 2024         | `TOKENS_2024`      | `TOKEN_2024`      |

Their definitions are presented in [Token nonterminals](token_nonterminals.md) below.

Each tokens nonterminal allows any number of repetitions of the corresponding token nonterminal.

Each token nonterminal is defined as a prioritised choice parsing expression,
each of whose subexpressions is a single nonterminal (a <dfn>token-kind nonterminal</dfn>).

The token-kind nonterminals are distinguished in the grammar as having names in `Title_case`.

The rest of the grammar is presented in the following pages in this section.
The definitions of some nonterminals are repeated on multiple pages for convenience.
The full grammar is also available on a [single page](complete_token_grammar.md).

The token-kind nonterminals are presented in an order consistent with their appearance in the token nonterminals.
That means they appear in priority order (highest priority first).

## Tokenisation { #token-kind-nonterminals }

Tokenisation takes a character sequence (the <dfn>input</dfn>), and either
produces a sequence of [fine-grained tokens] or
reports that lexical analysis failed.

The analysis depends on the Rust edition which is in effect when the input is processed.

> So strictly speaking, the edition is a second parameter to the process described here.

First, a match of the edition's tokens nonterminal is attempted against the input.
If the attempt does not succeed and consume the complete input, lexical analysis fails.

Otherwise, each member of the [sequence of participating matches][participating] of token-kind nonterminals in that attempt is processed as described below,
giving the sequence of fine-grained tokens.

If any match is rejected during that processing, lexical analysis fails.

### Processing a token-kind nonterminal match { #processing }

This operation considers a match of a token-kind nonterminal against part of the input,
and either produces a [fine-grained token] or rejects the match.

The following pages describe how to process a match of each token-kind nonterminal,
underneath the presentation of that nonterminal's section of the tokenisation grammar.

Each description specifies which matches are rejected.
For matches which are not rejected,
a token is produced whose kind mirrors the name of the token-kind nonterminal.
The description specifies the token's attributes.

> If for any match the description doesn't either say that the match is rejected or specify a well-defined value for each attribute needed for the token's kind,
> it's a bug in this writeup.

In these descriptions, notation of the form <u>NTNAME</u> denotes the sequence of characters consumed by the single participating match of `NTNAME` in the token-kind nonterminal match.

> If this notation is used for a nonterminal which might not participate in the match,
> without saying what happens in that case,
> it's a bug in this writeup.
>
> If this notation is used for a nonterminal which might participate more than once in the match,
> it's a bug in this writeup.


## Finding the first non-whitespace token { #find-first-nw-token }

> This section defines a variant of the tokenisation process which is used in the definition of [Shebang removal].

The operation of _finding the first non-whitespace token_ in a character sequence (the _input_) is:

Match the edition's tokens nonterminal against the input,
giving a [sequence of participating matches][participating] of token-kind nonterminals.

Consider the sequence of tokens obtained by [processing] each of those matches,
stopping as soon as any match is rejected.

The operation's result is the first token in that sequence which does not represent whitespace,
or **no token** if there is no such token.

For this purpose a token <dfn>represents whitespace</dfn> if it is any of:
 - a `Whitespace` token
 - a `Line_comment` token whose <var>style</var> is **non-doc**
 - a `Block_comment` token whose <var>style</var> is **non-doc**


[fine-grained token]: fine_grained_tokens.md
[fine-grained tokens]: fine_grained_tokens.md
[Shebang removal]: before_tokenising.md#shebang-removal
[processing]: #processing

[participating]: pegs.md#participating

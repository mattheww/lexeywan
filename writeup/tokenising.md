# Tokenising

This phase of processing takes a character sequence (the *input*), and either:

- produces a sequence of [fine-grained tokens]; or
- reports that lexical analysis failed

The analysis depends on the Rust edition which is in effect when the input is processed.

> So strictly speaking, the edition is a second parameter to the process described here.

Tokenisation is described using two operations:

- [Pretokenising] extracts [pretokens] from the character sequence.
- [Reprocessing] converts pretokens to [fine-grained tokens].

Either operation can cause lexical analysis to fail.


## Process

The process is to repeat the following steps until the input is empty:
1. [extract a pretoken from the start of the input][pretokenising]
2. [reprocess][reprocessing] that pretoken

If no step determines that lexical analysis should fail,
the output is the sequence of fine-grained tokens produced by the repetitions of the second step.

> Note: Each fine-grained token corresponds to one pretoken, representing exactly the same characters from the input;
> reprocessing doesn't involve any combination or splitting.

> Note: It doesn't make any difference whether we treat this as one pass with interleaved pretoken-extraction and reprocessing, or as two passes.
> The comparable implementation uses a single interleaved pass, which means when it reports an error it describes the earliest part of the input which caused trouble.


## Finding the first non-whitespace token { #find-first-nw-token }

> This section defines a variant of the tokenisation process which is used in the definition of [Shebang removal].

The process of _finding the first non-whitespace token_ in a character sequence (the _input_) is:
1. if the input is empty, the result is **no token**
2. [extract a pretoken from the start of the input][pretokenising]
3. [reprocess][reprocessing] that pretoken
4. if the resulting fine-grained token is not a _token representing whitespace_, the result is that token
5. otherwise, return to step 1

If any step determines that lexical analysis should fail, the result is **no token**.

For this purpose a _token representing whitespace_ is any of:
 - a `Whitespace` token
 - a `LineComment` token whose <var>style</var> is **non-doc**
 - a `BlockComment` token whose <var>style</var> is **non-doc**


[Pretokenising]: pretokenising.md#extracting-pretokens
[Reprocessing]: reprocessing.md
[Shebang removal]: before_tokenising.md#shebang-removal
[pretokens]: pretokens.md
[fine-grained tokens]: fine_grained_tokens.md

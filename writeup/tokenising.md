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

> Note: If lexical analysis succeeds, concatenating the extents of the produced tokens produces an exact copy of the input.


## Process

The process is to repeat the following steps until the input is empty:
1. [extract a pretoken from the start of the input][pretokenising]
2. [reprocess][reprocessing] that pretoken

If no step determines that lexical analysis should fail,
the output is the sequence of fine-grained tokens produced by the repetitions of the second step.

> Note: Each fine-grained token corresponds to one pretoken, representing exactly the same characters from the input;
> reprocessing doesn't involve any combination or splitting.

> Note: it doesn't make any difference whether we treat this as one pass with interleaved pretoken-extraction and reprocessing, or as two passes.
> The comparable implementation uses a single interleaved pass, which means when it reports an error it describes the earliest part of the input which caused trouble.


[Pretokenising]: pretokenising.md
[Reprocessing]: reprocessing.md
[pretokens]: pretokens.md
[fine-grained tokens]: fine_grained_tokens.md


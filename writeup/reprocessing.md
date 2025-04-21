## Reprocessing

Reprocessing examines a pretoken, and either accepts it (producing a [fine-grained token]),
or rejects it (causing lexical analysis to fail).

> Note: Reprocessing behaves in the same way in all Rust editions.

The effect of reprocessing each kind of pretoken is given in [List of reprocessing cases].

[fine-grained token]: fine_grained_tokens.md
[List of reprocessing cases]: reprocessing_cases.md

## Tokenisation grammar

As [explained above],
the grammar defines a <dfn>tokens nonterminal</dfn> for each Rust edition.
They are presented below.

The rest of the grammar is presented in the following pages in this section.
The definitions of some nonterminals are repeated on multiple pages for convenience.

The definitions of the tokenisation nonterminals are presented an order consistent with their appearances in the choice expressions below.
That means they appear in priority order (higher priority earlier).

The full grammar is also available on a [single page](complete_token_grammar.md).


[explained above]: tokenising.md#the-tokenisation-grammar

##### Grammar
```
{{#include tokenise_anchored.pest:tokens}}
```


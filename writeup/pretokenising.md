## Pretokenising

Pretokenisation is described by a [Parsing Expression Grammar](pegs.md) which describes how to match a single pretoken (not a sequence of pretokens).

> The grammar isn't strictly a PEG.
> See [Grammar for raw string literals](raw_strings.md)

The grammar defines an _edition nonterminal_ for each Rust edition:

| Edition | Edition nonterminal |
|---------|---------------------|
| 2015    | `PRETOKEN_2015`     |
| 2021    | `PRETOKEN_2021`     |
| 2024    | `PRETOKEN_2024`     |

Each edition nonterminal is defined as a choice expression, each of whose subexpressions is a single nonterminal (a _pretoken nonterminal_).

##### Grammar
```
{{#include pretokenise_anchored.pest:pretokens}}
```

The pretoken nonterminals are distinguished in the grammar as having names in `Title_case`.

The rest of the grammar is presented in the following pages in this section.
It's also available on a [single page](complete_pretoken_grammar.md).

The pretoken nonterminals are presented in an order consistent with their appearance in the edition nonterminals.
That means they appear in priority order (highest priority first).
There is one exception, for floating-point literals and their related reserved forms (see [Float literal]).


### Extracting pretokens

To *extract a pretoken from the input*:

- Attempt to match the edition's edition nonterminal at the start of the input.
- If the match fails, lexical analysis fails.
- If the match succeeds, the extracted pretoken has:
   - extent: the characters consumed by the nonterminal's expression
   - kind and attributes: determined by the pretoken nonterminal used in the match, as described below.
- Remove the extracted pretoken's extent from the start of the input.

> Strictly speaking we have to justify the assumption that the match will always either fail or succeed,
> which basically means observing that the grammar has no left recursion.


### Determining the pretoken kind and attributes

Each pretoken nonterminal produces a single kind of pretoken.

In most cases a given kind of pretoken is produced only by a single pretoken nonterminal.
The exceptions are:
- Several pretoken nonterminals produce `Reserved` pretokens.
- There are two pretoken nonterminals producing `FloatLiteral` pretokens.
- In some cases there are variant pretoken nonterminals for different editions.

Each pretoken nonterminal (or group of edition variants) has a subsection on the following pages,
which lists the pretoken kind and provides a table of that pretoken kind's attributes.

In most cases an attribute value is "captured" by a named definition from the grammar:

- If an attributes table entry says "from `NONTERMINAL`",
the attribute's value is the sequence of characters consumed by that nonterminal,
which will appear in one of the pretoken nonterminal's subexpressions
(possibly via the definitions of additional nonterminals).

- Some attributes table entries list multiple nonterminals,
eg "from `NONTERMINAL1` or `NONTERMINAL2`".
In these cases the grammar ensures that at most one of those nonterminals may be matched,
so that the attribute is unambiguously defined.

- If no listed nonterminal is matched
(which can happen if they all appear before `?` or inside choice expressions),
the attribute's value is **none**.
The table says "(may be **none**)" in these cases.

If for any input the above rules don't result in a unique well-defined attribute value,
it's a bug in this specification.

In other cases the attributes table entry defines the attribute value explicitly,
depending on the characters consumed by the pretoken nonterminal or on which subexpression of the pretoken nonterminal matched.


[Float literal]: numeric_literal_pretokens.md#float-literal

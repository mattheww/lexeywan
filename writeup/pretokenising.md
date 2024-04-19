## Pretokenising

Pretokenisation works from an ordered list of *rules*.

See *[pretokenisation rules]* for the list (which depends on the Rust edition which is in effect).

To *extract a pretoken from the input*:

- *Apply* each rule to the input.

- If no rules succeed, lexical analysis fails.

- Otherwise, the extracted pretoken's extent, kind, and attributes are determined by the successful rule which appears earliest in the list, as described below.

- Remove the extracted pretoken's extent from the start of the input.

> Note: If lexical analysis succeeds, concatenating the extents of the pretokens extracted during the analysis produces an exact copy of the input.

> See open question [Rule priority]


### Rules

Each *rule* has a *pattern* (see [patterns])
and a set of *forbidden followers* (a set of characters).

A rule may also have a *constraint* (see [constrained pattern matches]).


The result of applying a rule to a character sequence is either:

- the rule fails; or
- the rule succeeds, and reports
  - an *extent*, which is a prefix of the character sequence
  - a pretoken kind
  - values for the attributes appropriate to that kind of pretoken

> Note: a given rule always reports the same pretoken kind, but some pretoken kinds are reported by multiple rules.


### Applying rules

To *apply* a rule to a character sequence:

- Attempt to match the rule's pattern against each prefix of the sequence.
- If no prefix matches the pattern, the rule fails.
- Otherwise the extent is the longest prefix which matches the pattern.
  - But if the rule has a constraint,
    see [constrained pattern matches] instead for how the extent is determined.
- If the extent is not the entire character sequence,
  and the character in the sequence which immediately follows the extent is in the rule's set of forbidden followers,
  the rule fails.
- Otherwise the rule succeeds.

The description of each rule below says how the pretoken kind and attribute values are determined when the rule succeeds.



#### Constrained pattern matches

Each rule which has a constraint defines what is required for a sequence of characters to *satisfy* its constraint.

> Or more formally: a constraint is a predicate function defined on sequences of characters.

When a rule which has a constraint is applied to a character sequence,
the resulting extent is the *shortest maximal match*, defined as follows:

- The *candidates* are the prefixes of the character sequence which match the rule's pattern and also satisfy the constraint.

- The *successor* of a prefix of the character sequence is the prefix which is one character longer
  (the prefix which is the entire sequence has no successor).

- The *shortest maximal match* is the shortest candidate whose successor is not a candidate
  (or which has no successor)

> Note: constraints are used only for block comments and for raw string literals with hashes.
>
> For the block comments rule it would be equivalent to say that the shortest match becomes the extent.
>
> For raw string literals, the "shortest maximal match" behaviour is a way to get the mix of non-greedy and greedy matching we need:
> the rule as a whole has to be non-greedy so that it doesn't jump to the end of a later literal, but the suffix needs to be matched greedily.


[constrained pattern matches]: #constrained-pattern-matches

[pretokenisation rules]: rules.md
[patterns]: patterns.md
[Rule priority]: open_questions.md#rule-priority

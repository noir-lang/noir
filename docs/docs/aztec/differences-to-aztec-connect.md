## Heuristics that differ from Aztec Connect

**A contract cannot manipulate another contract's state**

In A.C. we had multiple circuits that could each create/destroy a unified set of value notes. This was acceptable because all A.C. circuits had a single author (us!).

In A3 our architecture must process arbitrary circuits written by potentially dishonest actors. Contract state must therefore be siloed at the architecture level similar to Ethereum. Fortunately, this does not require splitting up the anonymity set.

**Anonymity set must be shared across all contracts**

In A.C. observers knew when different note types were being created (value note, account note etc). This cannot be the case in A3 as we want to provide strong privacy gaurantees to all private contracts even if they have few transactions interacting with their contract.

**Support for call semantics**

If a contract can only modify its own state, we need a way for a contract to "call" another contract to request state modifications.

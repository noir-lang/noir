# noir-edwards

Optimized implementation of Twisted Edwards curves.

Uses lookup tables and maximally efficient representations of group operations (for width-4 noir) to efficiently implement scalar multiplication and multiscalar multiplication.

Cost of 1 scalar mul = 2232 gates. Marginal cost of additional muls in an msm = 972 gates.

For example usage see `test.nr`

List of potential optimizations to improve performance:

1. update barretenberg backend to identify when memory lookups always come in pairs. e.g. two MEM operations, different ids, read index is the same. backend can convert into 1 memory table with 2 values instead of 2 memory tables with 1 value

```
MEM (id: 1, read at: x1, value: x125)
MEM (id: 2, read at: x1, value: x126)
```

2. fix barretenberg bug where range checks for values <2^{14} create an unneccessary addition gate.

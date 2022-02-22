# Rollup circuit

Takes two snarks, checks they're correct, and does a 'subtree' of state updating for the various trees. Eventually, 2^k kernel snarks will have been verified in a 'rollup tree' of height k, where the root of the rollup tree is a single rollup snark representing 2^k transactions.

There are two kinds of rollup circuit:
- 'Base' rollup circuit: Verify two kernel snarks.
- 'Merge' rollup circuit: Verify two rollup snarks (can be either two 'base' snarks or two 'merge' snarks), 'merging' them into 1 rollup snark.
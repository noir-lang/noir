# L1 Results Copier Circuit

See the [L2 --> L1 calls](../contracts/l1-calls.md#l2----l1-calls) section for motivation.

This circuit copies results of L1 calls (which were executed by the _previous_ rollup) to the `l1ResultsTree` on L2. It will effectively be updated _before_ any other state changes of this next rollup, so that the results are available for txs (callbacks) in this rollup to use.

This is a generalisation of how the root rollup circuit adds defi interaction notes to the defi tree.

> TODO: consider whether we can safely make execution of this circuit _optional_, so that several sequential rollups can be computed in parallel (meaning much faster rollup throughput). I.e. if we don't force the very next rollup to copy over results data, then the very next rollup (and the one after that, and so on) can start to be computed before the L1 txs are finalised, for mega fast throughput. Every now and then, a rollup provider could include an 'L1 results copy proof'... we just need to incentivise it so that copying actually happens, whenever possible.

> Note: I've separated this circuit for ease of reading, and because (as the above note suggests) we might not need to do this copying for _every_ rollup.


## Private Inputs ABI

```js
subtreeMembershipWitness: {
    siblingPath,
},
pendingResultLeafValues: [...],
l1Results: [...],
```

## Public Inputs ABI

```js
start: {
    l1ResultsTree: {
        root,
        firstPendingResultLeafIndex,
    },
},
end: {
    l1ResultsTree: {
        root,
        firstPendingResultLeafIndex,
    },
},
l1ResultsHash,
```


## Execution Logic

- `assert(l1Results.length == pendingResultLeafValues.length)`.
- Calculate the 'subroot' of the batch of 'pending' leaves (using `pendingResultLeafValues`) before updating them.
- Recalculate the `start.l1ResultsTree.root` using the calculated subroot and the membership witness `subtreeMembershipWitness.siblingPath` and `start.l1ResultsTree.firstPendingResultLeafIndex`, and assert that the calculated root matches `start.l1ResultsTree.root`.
  - This step confirms that the pending leaf hashes (`pendingResultLeafValues`) are correct.
- 'Finalise' each pending leaf:
  - Let `updatedLeaves[i] := hash(l1Results[i].l1ResultHash, pendingResultLeafValues[i])`
  - Note: `l1ResultHash = 0` for any failed L1 calls.
- Update the tree with the batch of completed leaves using the `subtreeMembershipWitness.siblingPath` and `start.l1ResultsTree.firstPendingResultLeafIndex`.
- Let `end.l1ResultsTree.firstPendingResultLeafIndex = start.l1ResultsTree.firstPendingResultLeafIndex + l1Results.length`
- Hash all of the results (using the expensive sha256 hash) so that it may be checked on-chain:
  - `l1ResultsHash = sha256(...l1Results.map(r => r.l1ResultHash));`

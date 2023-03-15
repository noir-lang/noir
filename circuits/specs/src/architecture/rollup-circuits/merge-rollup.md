# THIS PAGE NEEDS SOME UPDATES

# Merge rollup circuit

## Private inputs ABI

```js
// two input rollup snarks (can be EITHER a BASE or a MERGE snark).
rollupSnarks: [
    {
        proof,
        publicInputs: {
            // public inputs of a ROLLUP snark (BASE or MERGE - they have the same layout)
        },
        rollupType, // enum: base/merge
        vk, // must be a rollup snark vk
    },
    // and again for the 2nd rollup snark being passed into this base rollup circuit.
],
```

## Public inputs ABI

Exactly the same as the BASE rollup snark.
```js
start: {
    nullifierTree: {
        root,
        nextAvailableLeafIndex,
    },
    privateDataTree: {
        root,
        nextAvailableLeafIndex,
    },
    privateDataTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    },
    publicDataTree: {
        root,
    },
    contractTree: {
        root,
        nextAvailableLeafIndex,
    },
    contractTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    },
    l1ResultsTree: {
        root,
        nextAvailableLeafIndex,
    },
    l1ResultsTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    },
},
end: {
    nullifierTree: {
        root,
        nextAvailableLeafIndex,
    },
    privateDataTree: {
        root,
        nextAvailableLeafIndex,
    },
    privateDataTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    },
    publicDataTree: {
        root,
    },
    contractTree: {
        root,
        nextAvailableLeafIndex,
    },
    contractTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    },
    l1ResultsTree: {
        root,
        nextAvailableLeafIndex,
    },
    l1ResultsTreeRootsTree: {
        root,
        nextAvailableLeafIndex,
    }, 
},
constants: {
    privateKernelVKTreeRoot,
    publicKernelVKTreeRoot,
    baseRollupVKHash,
    mergeRollupVKHash,
},
l1CallStack: [...], // combined from both input kernel snarks' public inputs
optionallyRevealedData: [{
    callStackItemHash,
    functionData,
    emittedPublicInputs: [_, _, _, _],
    vkHash,
    portalContractAddress,
    <bools>,
},...], // combined from both input kernel snarks' public inputs
proverRecords: [{
    proverAddress,
    totalNumberOfGates, 
},...], // combined from both input kernel snarks' public inputs
proverAddress,
minMinTimestamp, // the min of the minTimestamps passed to this circuit
maxMinTimestamp, // the max of the minTimestamps passed to this circuit
```


## Execution Logic

We'll do a few things in the circuit:
- Verify the two input ROLLUP snarks
- Merge some arrays from both inputs
- Check the public inputs of both rollup snarks based on the ordering:
    - `rollupSnarks[0]` happened first
    - `rollupSnarks[1]` happened second
    - I.e. the `end` data of `rollupSnarks[0]` must line up with the `start` data of `rollupSnarks[1]`

Here we go...

- Validate that either both inputs are a BASE snark or both inputs are a MERGE snark. You can't mix the two.
    - `assert((rollupSnarks[0].rollupType == 'base' && rollupSnarks[1].rollupType == 'base') ^  (rollupSnarks[0].rollupType == 'merge' && rollupSnarks[1].rollupType == 'merge')`:
- For `i in [0, 1]`:
    - Verify `(rollupSnarks[i].proof, rollupSnarks[i].publicInputs)` against the vk `rollupSnarks[i].vk`.
    - Validate that the `rollupSnarks[i].vk` is a valid base/merge snark:
        - Let `vkHash := hash(rollupSnarks[i].vk)`
        - If `rollupSnarks[i].rollupType == 'base'`:
            - `assert(vkHash == constants.baseRollupVKHash)`
        - Else (`rollupSnarks[i].rollupType == 'merge'`):
            - `assert(vkHash == constants.mergeRollupVKHash)`

Ensure constants are kept constant.
- Assert that all items in `constant` are the same between the two input rollup snarks:
    - `assert(rollupSnarks[0].publicInputs.constants == rollupSnarks[1].publicInputs.constants)`
- Assert that the input `constant` are the same as the output `constant`
    - `assert(rollupSnarks[0].publicInputs.constants == constants)`

Check that the `end` data of `rollupSnarks[0]` lines up with the `start` data of `rollupSnarks[1]`
- `assert(rollupSnarks[0].end == rollupSnarks[1].start)`


Merge input stacks/arrays to create single output stacks/arrays:
- `assert(l1CallStack == [...rollupSnarks[0].publicInputs.l1CallStack, ...rollupSnarks[1].publicInputs.l1CallStack]);`
- `assert(optionallyRevealedData == [...rollupSnarks[0].publicInputs.optionallyRevealedData, ...rollupSnarks[1].publicInputs.optionallyRevealedData]);`
- `assert(deployedContractData == [...rollupSnarks[0].publicInputs.deployedContractData, ...rollupSnarks[1].publicInputs.deployedContractData]);`

* Calculate new data to push to `proverRecords`:
    - Calculate new prover record data from both input rollupSnarks, using the `proverAddress` and `vk` of the rollup snarks' `publicInputs` (see detailed logic in the kernel snarks' 'execution logic' sections).
    - Push that data onto the `proverRecords` array.
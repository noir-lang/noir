# THIS PAGE NEEDS SOME UPDATES

# Base rollup circuit

## Private inputs ABI

```js
// two input kernel snarks
kernelSnarks: [
    {
        proof,
        publicInputs: {
            end: {
                aggregatedProof,
                callCount: 0,
                privateCallStack: [], // empty
                publicCallStack: [], // empty
                contractDeploymentCallStack: [], // empty
                l1CallStack: [...],
                callbackStack: [...],
                optionallyRevealedData: [...],
                deployedContractData: [...],
                proverRecords: [...],
                outputCommitments: [...],
                inputNullifiers: [...],
                publicDataTreeRoot,
                contractTreeRoot,
                nextAvailableContractTreeLeafIndex,
            },
            constants: {
                oldTreeRoots: {
                    privateDataTree,
                    contractTree,
                    l1ResultsTree,
                },
                privateKernelVKTreeRoot,
                publicKernelVKTreeRoot,
                initialPublicDataTreeRoot,
                initialContractTreeRoot,
                initialNextAvailableContractTreeLeafIndex,
                isConstructorRecursion,
                isCallbackRecursion,
                executedCallback: {
                    l1ResultHash,
                    l1ResultsTreeLeafIndex,
                }
                globals: {
                    minTimestamp,
                    block.timestamp,
                    block.number,
                    prevBlock.ethTimestamp,
                    prevBlock.ethNumber,
                },
            },
            proverAddress,
            isPrivate,
            isPublic,
            isContractDeployment,
        },
        kernelType, // enum: private/public/contractDeployment
        vk, // must be a kernel snark vk
        vkIndex, // in the kernel vk tree
        vkPath, // for proving membership of the kernel vk in the kernel vk tree
    },
    // and again for the 2nd kernel snark being passed into this base rollup circuit.
],
membershipWitnesses: {
    lowNullifiers: [{
        // the leaf data of nullifiers which are 'one smaller' than the
        // new nullifiers we'll be inserting.
        leafIndex,
        leafValue: {
            thisValue,
            nextValue,
            nextIndex,
        },
        siblingPath
    }, ...],
    inputNullifiersBatch: {
        // leafValues provided in 'inputNullifiers' input
        // leafIndex provided in 'start.nullifierTree.nextAvailableLeafIndex'
        siblingPath, // The 'frontier'. The sibling path of the next available leaf
                     // index.
    },
    outputCommitmentsBatch: {
        // leafValues provided in 'inputCommitments' input
        // leafIndex provided in 'start.privateDataTree.nextAvailableLeafIndex'
        siblingPath, // The 'frontier'. The sibling path of the next available leaf
                     // index.
    },
    callbackStackBatch: {
        // We'll insert any new callbacks for any new L1 calls.
        // leafValues provided in the 'callbackStack' input
        // leafIndex provided in 'start.l1ResultsTree.nextAvailableLeafIndex'
        siblingPath, // The 'frontier'. The sibling path of the next available leaf
                     // index.
    },
    executedCallbacks: [
        // A witness for each kernel snark.
        // If a kernel snark `isCallbackRecursion`, then it'll be referencing some
        // L1 Result. This witness allows that purported result to be validated.
        {
            siblingPath,
            callbackStackItem: {
                // We need to provide this preimage of a callbackStackItemHash,
                // because if a nonzero callbackPublicKey is included,
                // then this callback must be validated only by the private
                // kernel circuit (not by this circuit).
                // Also, we need to validate the function data of the callback.
                callbackPublicKey,
                successCallback,
                failureCallback,
            },
            // The leafIndex is already provided in the kernel snark's public inputs,
            // to help the rollup provider to find the correct leaf (and to force the
            // correct leaf to be used).
        },
        {
            siblingPath,
            callbackStackItem: {...},
        }
    ],
    oldPrivateDataTreeRoots: [
        // Two old private data tree roots were given: one for each kernel snark:
        {
            leafIndex, // of the oldPrivateDataTreeRoot in the privateDataTreeRootsTree
            siblingPath,
        },
        {
            leafIndex,
            siblingPath,
        },
    ],
    oldContractTreeRoots: [
        // Two old contract tree roots were given: one for each kernel snark:
        {
            leafIndex, // of the oldContractTreeRoot in the contractTreeRootsTree
            siblingPath,
        },
        {
            leafIndex,
            siblingPath,
        },
    ],
    oldL1ResultsTreeRoots: [
        // Two old l1 results tree roots were given: one for each kernel snark:
        {
            leafIndex, // of the oldL1ResultsTreeRoot in the l1ResultsTreeRootsTree
            siblingPath,
        },
        {
            leafIndex,
            siblingPath,
        },
    ],
    endPrivateDataTreeRoot: {
        siblingPath,
    },
    endContractTreeRoot: {
        siblingPath,
    },
},
```

## Public inputs ABI

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
- Verify the two kernel snarks
- Merge some arrays from both inputs
- Check some tree stuff
- Check the public inputs of both kernel snarks based on the ordering:
    - `kernelSnarks[0]` happened first
    - `kernelSnarks[1]` happened second

So:
- the base rollup circuit's `start` tree data must line up with the `initial...` tree data of `kernelSnarks[0]`.
- the `end` tree data of `kernelSnarks[0]` must line up with the `initial...` tree data of `kernelSnarks[1]`.
- the `end` tree data of `kernelSnarks[1]` must line up with the base rollup circuit's `end` tree data.

Here we go...

- For `i in [0, 1]`:
    - If `kernelSnarks[i].publicInputs.constants.isConstructorRecursion == true`:
        - Revert. A constructor kernel can only be verified by the contractDeployment kernel snark.
    - Verify `(kernelSnarks[i].proof, kernelSnarks[i].publicInputs)` against the vk `kernelSnarks[i].vk`.
    - Validate that the `kernelSnarks[i].vk` is a valid kernel snark of the correct `kernelSnarks[i].kernelType` with a membership check of the relevent kernel snark vk tree:
        - Calculate `kernelVKHash := hash(kernelSnarks[i].vk)`;
        - Compute `root` using the `kernelVKHash`, `kernelSnarks[i].vkPath` and `kernelSnarks[i].vkIndex`.
        - Validate that `root == constants.<kernelType>KernelVKTreeRoot`.

Validate that the callStacks of both kernel snarks were all completed (are all empty):
- For `i in [0, 1]`:
    - `require(callCount == 0);`
    - Require all values of `privateCallStack`, `publicCallStack`, `contractDeploymentCallStack` to be `0`.

Check the old tree roots (that were referred-to by the users' circuits) have actually existed (at some point in history) as the root of their tree:
- For `i in [0, 1]`:
    - Check the old private data tree root once existed:
        - Calculate `root` using: `leafValue: kernelSnarks[i].publicInputs.constants.oldTreeRoots.privateDataTree, leafIndex: membershipWitnesses.oldPrivateDataTreeRoots[i].leafIndex, siblingPath: membershipWitnesses.oldPrivateDataTreeRoots[i].siblingPath`.
        - `assert(root == start.privateDataTreeRootsTree.root`
    - Check the old contract tree root once existed:
        - Calculate `root` using: `leafValue: kernelSnarks[i].publicInputs.constants.oldTreeRoots.contractTree, leafIndex: membershipWitnesses.oldContractTreeRoots[i].leafIndex, siblingPath: membershipWitnesses.oldContractTreeRoots[i].siblingPath`.
        - `assert(root == start.contractTreeRootsTree.root`
    - Check the old L1 results tree root once existed:
        - Calculate `root` using: `leafValue: kernelSnarks[i].publicInputs.constants.oldTreeRoots.l1ResultsTree, leafIndex: membershipWitnesses.oldL1ResultsTreeRoots[i].leafIndex, siblingPath: membershipWitnesses.oldL1ResultsTreeRoots[i].siblingPath`.
        - `assert(root == start.contractTreeRootsTree.root`

Check the alignment of the start and end states of the two trees which were updated _within the kernel snarks_ (the publicDataTree and the contractTree). This ensures the kernel snarks didn't make any unexpected changes to those trees:

- `start -> kernelSnarks[0]`:
    - `require(start.publicDataTree.root == kernelSnarks[0].publicInputs.constants.initialPublicDataTreeRoot);`
    - `require(start.contractTree.root == kernelSnarks[0].publicInputs.constants.initialContractTreeRoot);`
    - `require(start.contractTree.nextAvailableLeafIndex == kernelSnarks[0].publicInputs.constants.initialNextAvailableContractTreeLeafIndex);`
- `kernelSnarks[0] -> kernelSnarks[1]`:
    - `require(kernelSnarks[0].publicInputs.end.publicDataTreeRoot == kernelSnarks[1].publicInputs.constants.initialPublicDataTreeRoot);`
    - `require(kernelSnarks[0].publicInputs.end.contractTreeRoot == kernelSnarks[1].publicInputs.constants.initialContractTreeRoot);`
    - `require(kernelSnarks[0].publicInputs.end.nextAvailableContractTreeLeafIndex == kernelSnarks[1].publicInputs.constants.initialNextAvailableContractTreeLeafIndex);`
- `kernelSnarks[1] -> end`:
    - `require(kernelSnarks[1].publicInputs.end.publicDataTreeRoot == end.publicDataTree.root);`
    - `require(kernelSnarks[1].publicInputs.end.contractTreeRoot == end.contractTree.root);`
    - `require(kernelSnarks[1].publicInputs.end.nextAvailableContractTreeLeafIndex == end.contractTree.nextAvailableLeafIndex);`

Merge input stacks/arrays to create single output stacks/arrays:
- `assert(l1CallStack == [...kernelSnarks[0].publicInputs.end.l1CallStack, ...kernelSnarks[1].publicInputs..end.l1CallStack]);`
- `assert(optionallyRevealedData == [...kernelSnarks[0].publicInputs..end.optionallyRevealedData, ...kernelSnarks[1].publicInputs.end.optionallyRevealedData]);`
- `assert(deployedContractData == [...kernelSnarks[0].publicInputs..end.deployedContractData, ...kernelSnarks[1].publicInputs.end.deployedContractData]);`


If either of the kernel snarks was a callback execution, check that their purported L1 Result is correct (i.e. that it exists in the L1 results tree).
We do this here, in the base rollup circuit rather than the public kernel circuit, because this only needs to be done at most once per tx, so it'd be wasteful to repeat this calc in every kernel recursion. Having said that, the _private_ kernel circuit does also offer this functionality, so that a user may hide which callback they're calling (and even the fact they're executing a callback at all).
This logic MUST come before the 'nullifier insertion' logic, because we add a new nullifier to the set here.
- Let `callbackNullifiers = []`
- For `i in [0, 1]`:
    - If `kernelSnarks[i].publicInputs.constants.isCallbackRecursion`
        - `require(l1ResultHash != 0)` to prevent a user executing a callback which refers to a _pending_ result.
        - Extract `{ l1ResultHash, l1ResultsTreeLeafIndex } = kernelSnarks[i].publicInputs.constants.executedCallback;`
        - Extract `{ callbackStackItem: { callbackPublicKey, successCallback, failureCallbac } } = membershipWitnesses.executedCallbacks[i];`
        - Extract the function data of the callback which was called (this will always be at the 0th position of the optionallyRevealedData):
            - Let `executedCallbackFunctionData = kernelSnarks[i].publicInputs.end.optionallyRevealedData[0].functionSelector`.
            - `require(executedCallbackFunctionData != 0)`
            - Let `executedCallbackEmittedPublicInputs = kernelSnarks[i].publicInputs.end.optionallyRevealedData[0].emittedPublicInputs`.
        - If `l1ResultHash != 0` (success):
            - Check the successCallback's `functionData` matches the kernel snark's:
            - `require(successCallbackHash.functionData == executedCallbackFunctionData)`
            - In fact, check the entire successCallbackCallHash
        - Else:
            - Check the failureCallbackHash's `functionData` matches the kernel snark's:
            - `require(failureCallbackHash.functionData == executedCallbackFunctionData)`
            - In fact, check the entire successCallbackCallHash
        - `require(callbackPublicKey == 0)`, otherwise this callback should have been processed in the private kernel snark.
        - Let `callbackStackItemHash := hash(callbackPublicKey, callbackStackItem.successCallbackHash, callbackStackItem.failureCallbackHash)`
        - Let `leafValue = hash(l1ResultHash, callbackStackItemHash);`
        - Check membership of the `leafValue` in the `l1ResultsTree` using:
            - `l1ResultsTreeLeafIndex`
            - `membershipWitnesses.executedCallbacks[i].siblingPath`
            - `kernelSnarks[i].publicInputs.constants.oldTreeRoots.l1ResultsTree`
        - Set `callbackNullifiers[i] := hash(callbackStackItemHash, callbackPrivateKey = 0)` to prevent this callback from being executed again.
            - We don't include the whole `leafValue` in the preimage of the nullifier, to prevent a user maliciously calling the callback twice: once for 'success' and once for 'fail' (although that shouldn't be possible if the 'L1 results copier' circuit works correctly).
            - The `callbackPrivateKey` only gets used if the callback is executed privately (in which case all of the checks of this section are done in the private kernel circuit. The private kernel circuit would then toggle the `isCallbackRecursion` to `false`, to hide the fact that it's a callback, and so that this section is skipped within this circuit).

Add new pending callbacks to the l1ResultsTree:
- Let `pendingResultLeafValues = [];`
- For `i in [0, 1]`
  - For each `callbackStackItem in kernelSnarks[i].publicInputs.end.callbackStack`.
  - Set `pendingResultLeafValues.push( callbackStackItemHash )` ("pendingResult", because "finalisation" of the leaf happens once the L1 results are added in the next rollup).
- Batch-insert the `pendingResultLeafValues`, using:
    - `start.l1ResultsTree.nextAvailableLeafIndex`
    - `start.l1ResultsTree.root` (to reconcile the starting root before adding more leaves)
    - `end.l1ResultsTree.nextAvailableLeafIndex`
    - `end.l1ResultsTree.root`
    - `membershipWitnesses.callbackStackBatch`



Add the inputNullifiers and outputCommitments to their trees:
- Merge the arrays from the two input kernel snarks:
    - Let `inputNullifiers := [...kernelSnarks[0].publicInputs.end.inputNullifiers, ...kernelSnarks[1].publicInputs.end.inputNullifiers]`;
    - Let `outputCommitments := [...kernelSnarks[0].publicInputs.end.outputCommitments, ...kernelSnarks[1].publicInputs.end.outputCommitments]`;

Nullifiers:
- Let `inputNullifierLeafIndex := start.nullifierTree.nextAvailableLeafIndex`
- Let `rootBeforeInsertion := start.nullifierTree.root`
- For each `inputNullifier` in `inputNullifiers` (index `i`):        
    - Do a non-membership check, using:
        - `membershipWitnesses.lowNullifiers[i]` (to validate `rootBeforeInsertion`)
        - `inputNullifiers[i]` to validate non-membership.
        - See the hackmd for the new nullifier tree info.
    - Update all of the `lowNullifiers[i].leafValue.nextValue` and `lowNullifiers[i].leafValue.nextIndex` values, using:
        - `membershipWitnesses.lowNullifiers[i]` 
        - `inputNullifiers[i]` to update the 'next' values.
    - Batch-insert the new `inputNullifiers`, using:
        - `start.nullifierTree.nextAvailableLeafIndex`
        - `start.nullifierTree.root` (to reconcile the starting root before adding more leaves)
        - `end.nullifierTree.nextAvailableLeafIndex`
        - `end.nullifierTree.root`
        - `membershipWitnesses.inputNullifiersBatch`

Commitments:
- Batch-insert `outputCommitments` (and validate start and end tree states) using:
    - `start.privateDataTree.nextAvailableLeafIndex`
    - `start.privateDataTree.root` (to reconcile the starting root before adding more leaves)
    - `end.privateDataTree.nextAvailableLeafIndex`
    - `end.privateDataTree.root`
    - `membershipWitnesses.outputCommitmentsBatch`


Update the (difficultly-named) trees whose leaves are the roots of historic trees. I.e. add the newly-calculated roots as the 'latest' roots of the historic trees (`privateDataTreeRootsTree`, `contractTreeRootsTree`, `l1ResultsTreeRootsTree`).

- Insert the newly-calculated `end.privateDataTree.root` as a new leaf in the `privateDataTreeRootsTree` using:
    - `membershipWitnesses.endPrivateDataTreeRoot.siblingPath`
    - `start.privateDataTreeRootsTree.nextAvailableLeafIndex`
    - `start.privateDataTreeRootsTree.root`
    - `end.privateDataTreeRootsTree.nextAvailableLeafIndex`
    - `end.privateDataTreeRootsTree.root`

- Insert the newly-calculated `end.contractTree.root` as a new leaf in the `contractTreeRootsTree` using:
    - `membershipWitnesses.endContractTreeRoot.siblingPath`
    - `start.contractTreeRootsTree.nextAvailableLeafIndex`
    - `start.contractTreeRootsTree.root`
    - `end.contractTreeRootsTree.nextAvailableLeafIndex`
    - `end.contractTreeRootsTree.root`

- Insert the newly-calculated `end.l1ResultsTree.root` as a new leaf in the `l1ResultsTreeRootsTree` using:
    - `membershipWitnesses.endl1ResultsTreeRoot.siblingPath`
    - `start.l1ResultsTreeRootsTree.nextAvailableLeafIndex`
    - `start.l1ResultsTreeRootsTree.root`
    - `end.l1ResultsTreeRootsTree.nextAvailableLeafIndex`
    - `end.l1ResultsTreeRootsTree.root`

* Calculate new data to push to `proverRecords`:
    - Calculate new prover record data from both input kernelSnarks, using the `proverAddress` and `vk` of the kernel snarks' `publicInputs` (see detailed logic in the kernel snarks' 'execution logic' sections).
    - Push that data onto the `proverRecords` array.
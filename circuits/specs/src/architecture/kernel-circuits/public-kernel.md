# THIS PAGE IS OUT OF DATE

# Public Kernel Circuit

**IGNORE this page. We now think we'll need a VM for public functions, so this is massively out of date. (Even without the need for a VM, the content of this page has fallen significantly behind the naming and architecture of other pages).**

The public kernel circuit has a very similar structure to that of the private kernel circuit.

The public circuit tracks its own call depth. If `start.publicCallCount == 0` , one of two conditions must be met:

* A valid ECDSA signature is provided by a user (where the message is the callStackItemHash). In this case `msg.sender = user`.
* A valid private kernel proof is provided where `end.privateCallStack.length == 0`.

Public kernel proofs are generated recursively until `end.publicCallStack.length == 0`. At this point the proof can be included within a rollup circuit.

## Private inputs ABI

| Data Type | Description |
| -------- | -------- |
| `signature` | ECDSA signature signed by user (if no private kernel proof has been provided && `start.publicCallCount == 0`, otherwise empty) |
| `start: {` | |
| `- aggregatedProof` | The current aggregated proof state (if any) |
| `- publicCallCount` | How many _public_ calls have been recursively executed so far? |
| `- publicCallStack` | Starting state of public call stack (max depth 64) |
| `- contractDeploymentCallStack` |  Starting state of contract deployment call stack (max depth 4?) |
| `- l1CallStack` | Starting state of l1 call stack (max depth 64?) |
| `- callbackStack` | See breakdown in the public inputs ABI in the next table |
| `- optionallyRevealedData` | See breakdown in the public inputs ABI in the next table |
| `- proverRecords` | |
| `- outputCommitments` | Starting state of commitments to be added to `privateDataTree`.<br>Notice that a public circuit is allowed to add commitments to the privateDataTree. Imagine, for example, a defi-bridge between private L2 and public L2. Then the rollup processor can complete partial commitments (partial commitments can be added to the _public_ data tree), and the completed commitments can be added straight to the private data tree. |
| `- inputNullifiers` | Values being passed-through from the private circuit executions (must be kept constant). Although this MUST be kept constant in the public kernel recursion, shouldn't go in the `constants` object of this ABI, because a more important requirement is for _all_ kernel snarks to have the same public inputs layout, so this needs to live here in `start`/`end` instead. There's a check in each kernel snark to ensure arrays are append-only (and hence unchanged at earlier indices), so constant-ness is ensured. |
| `- publicDataTreeRoot` | The publicDataTree's root _before_ we make any state transitions (as requested in the publicCall being verified within this circuit). |
| `}` | |
| `previousKernel: {` | |
| `- proof` | |
| <pre>- publicInputs = {<br/>&nbsp;&nbsp;end: start, // copy within circuit.<br/>&nbsp;&nbsp;constants: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;oldTreeRoots: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;privateDataTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;contractTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;&nbsp;&nbsp;privateKernelVKTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;publicKernelVKTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;initialPublicDataTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isConstructorRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isCallbackRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;executedCallback: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;l1ResultHash,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;l1ResultsTreeLeafIndex,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;&nbsp;&nbsp;globals: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;minTimestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;block.timestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;block.number,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;prevBlock.ethTimestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;prevBlock.ethNumber,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;}<br/>&nbsp;&nbsp;proverAddress,<br/>&nbsp;&nbsp;isPrivate,<br/>&nbsp;&nbsp;isPublic,<br/>&nbsp;&nbsp;isContractDeployment,<br/>}</pre> | |
| `- vk` | The VK that should be used to verify the `previousKernel.proof`. |
| `- vkIndex` | The leaf index of the `previousKernel.vk`, to accompany the `previousKernel.vkPath` in doing a membership check. |
| `- vkPath` | We can support multiple 'sizes' of kernel circuit (i.e. kernel circuits with varying numbers of public inputs) if we store all of the VKs of such circuits as vkHashes in a little Merkle tree. This path is the sibling path from the `previousKernel.vk`'s leaf to the root of such a tree. |
| `}` | |
| `publicCall: {` | Data relating to the next public callstack item that we're going to pop off the callstack and verify within this snark. Recall we're representing items in the callstack as a `callStackItemHash`. We'll pass the unpacked data here, for validation within this kernel circuit. |
| `- functionData,` | 64-bits (see earlier section) |
| `- publicInputs: {...},` | Rather than pass in the `publicInputsHash`, we pass in its preimage here (we'll hash it within this kernel circuit). This is _all_ of the data listed in the Public Circuit ABI's table of public inputs (see earlier section). |
| `- callContext: {...},` |  |
| `- isDelegateCall,` | |
| `- isStaticCall,` | |
| `- proof` | The proof for this item of the callstack. We'll be verifying this within this kernel circuit. |
| `- vk` | The full verification key of the public call proof that will be verified within this circuit. (I.e. the proof `start.publicCallStack.pop().proof`) |
| `- vkPath` | The sibling path of the `vk`'s `vkHash` within the aztec contract's 'mini merkle tree'. |
| `- portalContractAddress` | Needed to reconstruct the contract's leaf of the `contractTree`. |
| `- contractLeafIndex` | The leaf index of the contracts tree where this contract's vkTree is stored |
| `- contractPath` | The sibling path from the contract's root (a leaf of the contract tree) to the `oldContractTreeRoot` |
| `- stateReadPaths: [<one for each state read]` | Sibling paths to allow pure reads from the public data tree. |
| `- stateTransitionPaths: [<one for each state transition]` | :question: We could alternatively do public state tree updates in the rollup circuit. I'm doing it here to get rid of as much data as possible as early as possible (we'll be left with the new tree root). |
| `}` | |

## Public inputs ABI

| Data type | Description |
| --- | --- |
| `end: {` | |
| `- aggregatedProof` | Output aggregated proof |
| `- publicCallCount` | How many calls have been recursively executed at end of circuit execution? (either `start.publicCallCount + 1` or `0` iff `end.publicCallStack` is empty |
| `- publicCallStack` | Output state of public call stack (max depth 64) |
| `- contractDeploymentCallStack` |  Output state of contract deployment call stack (max depth 4?) |
| `- l1CallStack` | Output state of l1 call stack (max depth 64?) |
| `- callbackStack: [{` | Data to add to the `l1ResultsTree`. See [here](../contracts/l1-calls.md) |
| `-  - callbackPublicKey` | |
| `-  - successCallbackCallHash,` | of the callback function to execute upon success of the L1 call. |
| `-  - failureCallbackCallHash` | of the callback function to execute upon failure of the L1 call. |
| `-  - successResultArgMapAcc` | |
| `}],` | |
| `- optionallyRevealedData: [{` | Some values from a public call can be optionally revealed to the Contract Deployment kernel circuit / L1, depending on bools of the public circuit ABI. For some/every public call, each 'object' in this 'array' contains the following fields (some of which might be 0, depending on the bools) (note: there might be more efficient ways to encode this data - this is just for illustration): |
| `-  - callStackItemHash,` | Serves as a 'lookup key' of sorts. |
| `-  - functionData,` | |
| `-  - emittedPublicInputs: [_, _, _, _],` | |
| `-  - vkHash,` | |
| `-  - portalContractAddress,` | |
| `-  - <bools>` | :question: Discussion needed. We might also need to reveal all the bools to the public kernel snark, so that it may filter out data that doesn't need to be revealed to L1. |
| `- }, ...]` | :question: Should a callStackItemHash contain a salt, to hide the nature of the call? |
| `- proverRecords` | We need to record info about who generated public proofs and who generated public kernel snarks, so that they may be rewarded for their work. It's not as simple as there being one prover. Proving might be delegated to various people by the rollup processor, so we need to track an array (obvs of some bounded size) with an element for each prover. Each prover record entry in `proverRecords` will be of the form `[proverAddress, totalNumberOfGates]`, where we track the `totalNumberOfGates` this prover has generated proofs over, as an approximation of the work they've done. |
| `- outputCommitments` | Output state of commitments to be added to `privateDataTree`.<br/><br/>Notice that a public circuit is allowed to add commitments to the privateDataTree. Imagine, for example, a defi-bridge between private L2 and public L2. Then the rollup processor can complete partial commitments (partial commitments can be added to the _public_ data tree), and the completed commitments can be added straight to the private data tree. |
| `- inputNullifiers` | Values being passed-through from the private circuit executions (must be kept constant). Although this MUST be kept constant in the public kernel recursion, shouldn't go in the `constants` object of this ABI, because a more important requirement is for _all_ kernel snarks to have the same public inputs layout, so this needs to live here in `start`/`end` instead. There's a check in each kernel snark to ensure arrays are append-only (and hence unchanged at earlier indices), so constant-ness is ensured. |
| `- publicDataTreeRoot` | The root of the public data tree _after_ performing the stateTransitions requested by the publicCall being verified within this circuit.  |
| `}` | |
| `constants: {` | |
| `- oldTreeRoots: {` | |
| `- - privateDataTree` | must equal the value used in the final private kernel proof |
| `- - contractTree` | A recent root of the contract tree (for vk membership checks). Must also equal the value used in the final private kernel proof. |
| `- - l1ResultsTree,` | A recent root of the L1 results tree. |
| `- }` | |
| `- privateKernelVKTreeRoot` | must equal the value used in the final private kernel proof |
| `- publicKernelVKTreeRoot` | The root of a little Merkle tree whose leaves are hashes of the public kernel circuit VKs, allowing support for multuple 'sizes' of kernel circuit (i.e. with varying numbers of public inputs). |
| `- initialPublicDataTreeRoot` | The root of the public data tree before _any_ of the calls from the publicCallStack were executed. We need this to demonstrate that we started our recursion using the correct tree. This will be kept the same throughout the recursion. |
| `- isConstructorRecursion`, | Flags whether this entire recursion began with a constructor function. |
| `- isCallbackRecursion`, | Flags whether this entire recursion began with a callback function. |
| `- executedCallback: {` | Only populated (if at all) by the first call in a tx, if that call is a callback AND if the callback's execution doesn't want to be 'hidden'. This gives details of the callback that's been executed, so the callback and result can be cross-checked against the l1ResultsTree eventually in the Base Rollup Circuit.<br/> We assume the rollup provider will have a record of the contents of each leaf of the l1ResultsTree, so we don't need to provide the data here; the rollup provider can pass it as a private input to the Base Rollup snark. |
| `-  - l1ResultHash`, | This will eventually be compared against a leaf of the L1 Callback Tree in the Base Rollup Circuit. |
| `-  - l1ResultsTreeLeafIndex`, | Communicates to the rollup provider the correct leaf to prove membership against in the l1ResultsTree. This will eventually be used to lookup a leaf of the L1 Callback Tree in the Base Rollup Circuit. |
| `- },` | |
| `- globals: {` | |
| `-  - minTimestamp` | must equal value used in private call proof |
| `-  - block.timestamp` | aztec block timestamp |
| `-  - block.number` | aztec block number | 
| `-  - prevBlock.ethTimestamp` | Ethereum timestamp of block that contained previous Aztec 3 block |
| `-  - prevBlock.ethNumber` | block number of Ethereum block that contained previous Aztec 3 block |
| `- }` | |
| `}` | |
| `proverAddress` | Who is going to be generating the proof for this kernel circuit (any value can be injected here by the prover). This is used to apportion gas rewards. Note: not needed for the private kernel snarks, because they'll always be generated by the fee payer. |
| `isPrivate = false` | Bool. Tells the next kernel circuit that this is not a private kernel snark. |
| `isPublic = true` | Bool. Tells the next kernel circuit that this is a public kernel snark. |
| `isContractDeployment = false` | Bool. Tells the next kernel circuit that this is not a contract deployment kernel snark. |

## Execution Logic

:heavy_exclamation_mark: It would be ideal if the number of public inputs of a private kernel and public kernel circuit are the same, so that the below base case and recursion case can use the same verification code path.

- `require(previousKernel.publicInputs.isPrivate == true NAND previousKernel.publicInputs.isPublic == true)`
- `require(previousKernel.publicInputs.isContractDeployment == false`
- `require(start.publicCallCount == 0)`
* `require(start.publicCallStack.length > 0)`
Base case (verifying a private kernel snark):
*  if `(previousKernel.proof && previousKernel.publicInputs.isPrivate)`
    - `require(privateCall.functionData.isConstructor == false && privateCall.functionData.isCallback == false)`:
        - only the first call in the kernel recursion can be a constructor or a callback.
    * Verify the (`previousKernel.proof`, `previousKernel.publicInputs`) using the `previousKernel.vk`.
    * Validate that the `previousKernel.vk` is a valid _private_ kernel VK with a membership check:
        * Calculate `previousKernelVKHash := hash(previousKernel.vk)`;
        * Compute `root` using the `previousKernelVKHash`, `previousKernel.vkPath` and `previousKernel.vkIndex`.
        * Validate that `root == privateKernelTreeRoot`.
    * Validate consistency of 'start' and 'previous end' values:
        - verify that the `start...` values match the `previousKernel.publicInputs.end...` equivalents.
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit):
        * ensure this kernel circuit's `constant` public inputs match the `previousKernelProof`'s public inputs.
            * E.g. old tree roots.
    - Also ensure that any 'append-only' stacks or arrays have the same entries as the previous kernel proof, before pushing more data onto them!
- else:
    - This is the first call of the tx.
    * Require previous kernel data to be empty.
    * Validate that `start.publicCallStack.length == 1 && start.contractDeploymentCallStack.length == 0 && start.l1CallStack.length == 0`
        - TBD: to allow the option of a fee payment, we might require `start.publicCallStack.length` to be "1" or "2, where one tx has an `isFeePayment` indicator". We could even allow any number of initial private calls on the stack, but that's a pretty big deviation from the ethereum tx model.
    * Pop the only (TBD) `publicCallStackItemHash` off the `start.publicCallStack`.
        - If `publicCall.functionData.isConstructor == true`:
            - then we don't need a signature from the user, since this entire 'callstack' has been instantiated by a Contract Deployment kernel snark (which itself will have been signed by the user).
            - Set `isConstructorRecursion := true` - This public input will percolate to, and be checked by. the Contract Deployment Kernel Circuit which calls this constructor. This check is required to prevent a person from circumventing the ECDSA signature check by simply setting `isConstructor = true` when making a private call. If this aggregated kernel snark reaches the rollup circuit without this flag being reset to `false` by the Contract Deployment Kernel Circuit (to say "yes, this kernel was indeed a constructor for a Contract Deployment Kernel Circuit"), then the entire tx will be rejected by the rollup circuit.
        - Else:
            - Set `isConstructorRecursion := false`
            * Verify the ECDSA `signature`, with `message := publicCallStackItemHash` and `signer := publicCall.callContext.msgSender`.
            - Validate the `callContext`. Usually the correctness of a callContext is checked between the `publicCall` and all the new calls it makes (see later in this logic). That means for this 'Base case', those checks haven't been done for this `publicCall` (since there was no prior iteration of this kernel circuit to make those checks).
                - If `publicCall.isDelegateCall == true || publicCall.isStaticCall == true`:
                    - Revert - a user cannot make a delegateCall or staticCall.
                - Else:
                    - Assert `publicCall.callContext.storageContractAddress == publicCall.functionData.contractAddress`
        - If `publicCall.functionData.isCallback == true`:
            - Set `isCallbackRecursion = true;` - this can only be set in the _first_ call of a recursion.
            - Assert `l1ResultHash != 0`
            - Copy over the `publicCall.publicInputs.executedCallback` data to this snark's output: `constants.executedCallback`.
        - Else:
            - Set `isCallbackRecursion = false;`
            - Assert `l1ResultHash == 0`
- Set `initialPublicDataTreeRoot := start.publicDataTreeRoot`.


Recursion (of public kernel snarks):
* If `previousKernel.publicInputs.isPublic && start.publicCallCount > 0`:
    - `require(privateCall.functionData.isConstructor == false && privateCall.functionData.isCallback == false)`:
        - only the first call in the kernel recursion can be a constructor or a callback.
    * Verify the `previousKernel.proof` using the `previousKernel.vk`
    * Validate that the `previousKernel.vk` is a valid _public_ kernel VK with a membership check:
        * Calculate `previousKernelVKHash := hash(previousKernel.vk)`;
        * Compute `root` using the `previousKernelVKHash`, `previousKernel.vkPath` and `previousKernel.vkIndex`.
        * Validate that `root == publicKernelVKTreeRoot`.
    * Validate consistency of 'start' and 'previous end' values:
        - verify that the `start...` values match the `previousKernel.publicInputs.end...` equivalents.
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit):
        * ensure this kernel circuit's 'constant' public inputs match the `previousKernel`'s public inputs.
        * E.g. old tree roots.

Verify the next call on the callstack:
* Verify `start.publicCallStack.length > 0` and (if not already done during the 'Base Case' logic above), pop 1 item off of `start.publicCallStack`.
* Validate that `publicCall.functionData.isPrivate == false` (otherwise this is the wrong type of kernel circuit to be using).
* Validate that this newly-popped  `publiceCallStackItemHash` corresponds to the `publicCall` data passed into this circuit:
    - Calculate the `publicInputsHashPreimage`, given the complexity that 'current' public data wasn't known to the caller at the time they made the call:
        - Include all data from `publicCall.publicInputs`, except for the data documented earlier in the Public Circuit ABI section.
    * Calculate `publicCallPublicInputsHash := hash(publicInputsHashPreimage);`
    * Verify that `publicCallStackItemHash == hash(publicCall.functionData, publicCallPublicInputsHash, publicCall.callContext, etc...)`
    * Recall, the structure of a callstack item:
    * ```js
      publicCall = {
        functionData: {
          contractAddress,
          vkIndex,
          isPrivate,
          isConstructor,
          isCallback,
        },
        publicInputsHash,
        callContext,
        isDelegateCall,
        isStaticCall,
      }
      ```
* Verify the correctness of `(proof, publicCallPublicInputsHash)` using the `vk`.
* Validate the `vk` actually represents the function that is purportedly being executed:
    * Extract the `contractAddress` and `vkIndex` from `publicCall.functionData`.
    * Compute `vkHash := hash(vk)`
    * Compute the `vkRoot` of this function's contract using the `vkIndex`, `vkPath`, and `vkHash`.
    * Compute the contract's leaf in the `contractTree`:
      * Let `leafValue := hash(contractAddress, portalContractAddress, vkRoot)`
    * Compute `contractTreeRoot` using the `contractLeafIndex`, `contractPath` and `leafValue`.
    * Validate that `contractTreeRoot == constants.oldTreeRoots.contractTree`.
* Validate consistency of the `publicCall`'s `constant` data with that of the kernel circuit's public inputs (and those of the previousKernelProof). I.e.:
    * same old tree roots
    * etc.


Update the `end` values:
- Extract the public call's `stateReads`, which will have been populated with current values by the rollup provider (i.e. each transition will be of the form `[storageSlot, current_value`]).
    - Extract the contract address of the contract being read from, based on the callContext: `storageContractAddress := publicCall.callContext.storageContractAddress`.
    - For each `stateRead` in `stateReads` (loop index `i`):
        - Calculate the `stateLeafIndex = hash(storageContractAddress, stateRead.storageSlot)` (this can be thought of as a 'key' in a key:value mapping. The `stateRead.current_value` is the 'value').
        - Check the `current_value` is correct:
            - do a membership check with: `leafValue = current_value, leafIndex = stateLeafIndex, siblingPath = stateReadPaths[i]`
            - Assert that the calculated root equals `start.publicDataTreeRoot`.
- Extract the public call's `stateTransitions`, which will have been populated with old and new values by the rollup provider (i.e. each transition will be of the form `[storageSlot, old_value, new_value`]).
    - Extract the contract address to be used for storage, based on the callContext: `storageContractAddress := publicCall.callContext.storageContractAddress`.
    - Let `tempPublicDataTreeRoot := start.publicDataTreeRoot`
    - For each `stateTransition` in `stateTransitions` (loop index `i`):
        - Calculate the `stateLeafIndex = hash(storageContractAddress, stateTransition.storageSlot)` (this can be thought of as a 'key' in a key:value mapping. The `stateTransition.old_value` is the 'value').
        - Check the old_value is correct:
            - do a membership check with: `leafValue = old_value, leafIndex = stateLeafIndex, siblingPath = stateTransitionPaths[i]`
            - Assert that the calculated root equals `tempPublicDataTreeRoot`.
        - Update the tree with the new_value:
            - Hash up the tree using `leafValue = old_value, leafIndex = stateLeafIndex, siblingPath = stateTransitionPaths[i]`.
            - Set `tempPublicDataTreeRoot = the newly computed root`.
    - Set `end.publicDataTreeRoot = tempPublicDataTreeRoot`
* Extract the public call's `outputCommitments` (recall: a public circuit can indeed output new commitments, much like the claim circuit). 
    - If `publicCall.isStaticCall == true`:
        - Assert that `outputCommitments` is empty, since no state changes are allowed for static calls.
    - Else, 'silo' the `outputCommitments` with the contract address of the callContext: `storageContractAddress := publicCall.callContext.storageContractAddress`:
        - For each `outputCommitment` in `outputCommitments`:
            - `siloedOutputCommitment := hash(storageContractAddress, outputCommitment);`
    - Push those values (`siloedOutputCommitments`) onto `end.outputCommitments`.
    * See earlier note on how we can push these values onto the end arrays without gaps (i.e. dynamically)
* Calculate new data to push to `proverRecords`:
    - Extract `publicCallProverAddress := publicCall.publicInputs.proverAddress`.
    - Extract `publicCallNumberOfGates := vk.n`
    - Extract `previousKernelProverAddress := previousKernel.publicInputs.proverAddress`
    - Extract `previousKernelNumberOfGates := previousKernel.vk.n`
    - Below is expensive looping and dynamic pushing within a circuit (O(n^2) stuff). Consider alternatives.
    - Find in the `proverRecords` array an entry matching `publicCallProverAddress`.
        - If found: increment their `totalNumberOfGates += publicCallNumberOfGates`.
        - If not found: push a new record `[publicCallProverAddress, publicCallNumberOfGates]` onto `proverRecords`.
    - Find in the `proverRecords` array an entry matching `previousKernelProverAddress`.
        - If found: increment their `totalNumberOfGates += previousKernelNumberOfGates`.
        - If not found: push a new record `[previousKernelProverAddress, previousKernelNumberOfGates]` onto `proverRecords`.
* Extract the public call's `publicCallStack`, `contractDeploymentCallStack` and `partialL1CallStack`.
    - Validate the call contexts of these calls:
        - For each `newCallStackItem` in `publicCallStack`, `contractDeploymentCallStack`:
            - If `newCallStackItem.isDelegateCall == true`:
                - Assert `newCallStackItem.callContext == publicCall.callContext`
            - Else:
                - Assert `newCallStackItem.callContext.msgSender == publicCall.functionData.contractAddress`
                - Assert `newCallStackItem.callContext.storageContractAddress == newCallStackItem.functionData.contractAddress`
    - Validate `partialL1CallStack` and `callbackStack` lengths:
      - `require(partialL1CallStack.length == callbackStack.length)` - every L1 call must have a callback entry (even if the callbacks are zeroes).
    - For each `partialL1CallStackItem` in the `partialL1CallStack` (index `i`):
      - Ensure that the call is being sent to the associated portal contract address, by adding the `portalContractAddress` here:
        - Let `l1CallStackItem := keccak(portalContractAddress, partialL1CallStackItem)`
      - Validate `publicCall.callbackStack[i]`:
        - Let `successCallback = publicCall.callbackStack[i].successCallback`
        - Let `failureCallback = publicCall.callbackStack[i].failureCallback`
        - Ensure the contract address of each of the two callbacks matches that of the public call:
          - `require(successCallback.functionData.contractAddress == publicCall.functionData.contractAddress)`;
          - `require(failureCallback.functionData.contractAddress == publicCall.functionData.contractAddress)`;
        - Calculate the callbackHashes:
          - Let `successCallbackHash = hash(successCallback.functionData, etc...)`
          - Let `failureCallbackHash = hash(failureCallback.functionData, etc...)`
    - Push the contents of these call stacks onto the kernel circuit's `end.publicCallStack`, `end.contractDeploymentCallStack` and `end.l1CallStack`.
    - Also push the each `{callbackPublicKey, successCallbackHash, failureCallbackHash}` onto `end.callbackStack`.
    - As per the commitments/nullifiers bullet above, it would be nice if these 'pushes' could result in tightly-packed stacks.
- Determine whether any values need to be optionally revealed to the Contract Deployment kernel circuit, or to L1, by referring to the `publicCall`'s booleans:
    - Let `optionallyRevealedData = {};`
    - If `publicCall.functionData.isConstructor`, set:
        - `optionallyRevealedData.callStackItemHash = publicCallStackItemHash;`
        - `optionallyRevealedData.vkHash = vkHash;`
        - `optionallyRevealedData.emittedPublicInputs = publicCall.publicInputs.emittedPublicInputs;`
    - If `publicCall.functionData.isCallback`, set:
        - `optionallyRevealedData.functionData = publicCall.functionData;`
        - `optionallyRevealedData.emittedPublicInputs = publicCall.publicInputs.emittedPublicInputs;`
          - TODO: consider whether we can get rid of the emittedPublicInputs being emitted by a callback. More [here](../contracts/l1-calls.md#more-details).
    - If `isFeePayment`, set:
        - `optionallyRevealedData.callStackItemHash = publicCallStackItemHash;`
        - `optionallyRevealedData.functionData = publicCall.functionData;`
        - `optionallyRevealedData.emittedPublicInputs = publicCall.publicInputs.emittedPublicInputs;`
    - If `payFeeFromL1`, set:
        - `optionallyRevealedData.callStackItemHash = publicCallStackItemHash;`
    - If `calledFromL1`, set:
        - `optionallyRevealedData.callStackItemHash = publicCallStackItemHash;`
        - `optionallyRevealedData.functionData = publicCall.functionData;`
        - `optionallyRevealedData.emittedPublicInputs = publicCall.publicInputs.emittedPublicInputs;`
    - Push the `optionallyRevealedData` onto the `optionallyRevealedData`.
* If `end.publicCallStack.length == 0`, set `end.publicCallCount = 0`, else `end.publicCallCount = start.publicCallCount + 1`
* Set `isPublic := true;` (and the others are false)
* Ensure all unused public inputs (which are _not_ shown in the table above, but will be included in practice so that all kernel snarks have the same public input ABI) are `0`.
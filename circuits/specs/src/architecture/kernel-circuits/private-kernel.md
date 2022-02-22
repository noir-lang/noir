# Private Kernel Circuit

## Private inputs ABI

| Data Type | Description |
| -------- | -------- |
| `signature` | ECDSA signature (where `message = publicInputsHash`) signed by the user (if `start.privateCallCount == 0`, otherwise empty) |
| `start: {` | |
| `- aggregatedProof` | The current aggregated proof state (if any) |
| `- privateCallCount` | How many calls have been recursively executed so far? |
| `- privateCallStack` | Starting state of private call stack (max depth 64) |
| `- publicCallStack` | Starting state of public call stack (max depth 64) |
| `- contractDeploymentCallStack` |  Output state of contract deployment call stack (max depth 4?) |
| `- l1CallStack` | Starting state of L1 call stack (max depth 64?) |
| `- callbackStack` | See breakdown in the public inputs ABI in the next table |
| `- optionallyRevealedData` | See breakdown in the public inputs ABI in the next table |
| `- outputCommitments` | Starting state of commitments to be added to `privateDataTree` |
| `- inputNullifiers` | Starting state of nullifiers to be added to `nullifierTree` (up to 64) |
| `},` | |
| `previousKernel: {` | |
| `- proof` | Previous Kernel snark proof (if any). |
| <pre>- publicInputs = {<br/>&nbsp;&nbsp;end: start, // copy within circuit.<br/>&nbsp;&nbsp;constants: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;oldTreeRoots: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;privateDataTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;contractTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;&nbsp;&nbsp;privateKernelVKTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isConstructorRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isCallbackRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;executedCallback: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;l1ResultHash,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;l1ResultsTreeLeafIndex,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;&nbsp;&nbsp;globals: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;minTimestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;}<br/>&nbsp;&nbsp;isPrivate,<br/>&nbsp;&nbsp;isPublic,<br/>&nbsp;&nbsp;isContractDeployment,<br/>}</pre> |  The first Kernel Circuit execution can have `previousKernel.proof = 0` (or equivalent). |
| `- vk` | The VK that should be used to verify the `previousKernel.proof`. |
| `- vkIndex` | The leaf index to accompany the `previousKernel.vkPath`. |
| `- vkPath` | We can support multiple 'sizes' of kernel circuit (i.e. kernel circuits with varying numbers of public inputs) if we store all of the VKs of such circuits as vkHashes in a little Merkle tree. This path is the sibling path from the `previousKernel.vk`'s leaf to the root of such a tree. |
| `}` | |
| `privateCall: {` | Data relating to the next private callstack item that we're going to pop off the callstack and verify within this snark. Recall we're representing items in the callstack as a `callStackItemHash`. We'll pass the unpacked data here, for validation within this kernel circuit. |
| `- functionSignature,` | 64-bits (see earlier section) |
| `- publicInputs: {...},` | Rather than pass in the `publicInputsHash`, we pass in its preimage here (we'll hash it within this kernel circuit). This is _all_ of the data listed in the Private Circuit ABI's table of public inputs (see earlier section). |
| `- callContext: {...},` |  |
| `- isDelegateCall` | |
| `- isStaticCall` | |
| `- proof` | The proof for this item of the callstack. We'll be verifying this within this kernel circuit. |
| `- vk` | The full verification key of the private call proof that will be verified within this circuit. (I.e. the proof `start.privateCallStack.pop().proof`) |
| `- vkPath` | The sibling path of the `vk`'s `vkHash` within the aztec contract's 'mini merkle tree'. Note, the `vkIndex` to use when proving membership can be grabbed from the `privateCall` being executed (which itself is popped off the callstack). |
| `- portalContractAddress` | The portal contract address that corresponds to the contract of the function being called (`privateCall`). |
| `- contractLeafIndex` | The leaf index of the contracts tree where this contract's vkTree is stored |
| `- contractPath` | The sibling path from this contract's root (a leaf of the contract tree) to the `oldContractTreeRoot` |
| `- privatelyExecutedCallback: {` | Only populated (if at all) by the first call in a tx, if that call is a callback. This gives details of the callback that's been executed, so the callback and result can be cross-checked against the l1ResultsTree within this private kernel circuit. It's checked within the private kernel circuit so that the fact that this callback has even been executed can be hidden. |
| `-  - l1SuccessResultValues`, | The results of an L1 interaction. These underlying result values will be reconciled against this `privateCall.publicInputs.executedCallback.l1ResultHash` |
| `-  - l1SuccessResultArgPositions`, | An array which details how the L1 Results array gets mapped to the argument positions of the callback function. This will be checked within this circuit whether these values 'accumulate' to give the `successResultArgMapAcc`. The format can be cumbersome, if it saves on constraints, since this is a private input. |
| `-  - l1ResultsTreeSiblingPath`, | The leafIndex is a public input of the privateCall itself. |
| <pre>-  - callbackStackItem: {<br/>        callbackPublicKey,<br/>        successCallbackHash,<br/>        failureCallbackHash,<br/>        successResultArgMapAcc<br/>-  - } | |
| `-  - callbackPrivateKey`, | |
| `- },` | |
| `}` | |

## Public inputs ABI

| Data type | Description |
| --- | --- |
| `end: {` | |
| `- aggregatedProof,` | Output aggregated proof |
| `- privateCallCount,` | How many calls have been recursively executed at end of circuit execution? (either `start.privateCallCount + 1` or `0` iff `endPrivateCallStack` is empty |
| `- privateCallStack,` | Output state of private call stack (max depth 64) |
| `- publicCallStack,` | Output state of public call stack (max depth 64) |
| `- contractDeploymentCallStack` |  Output state of contract deployment call stack (max depth 4?) |
| `- l1CallStack,` | Output state of l1 call stack (max depth 64?) |
| `- callbackStack: [{` | Data to add to the `l1ResultsTree`. See [here](../contracts/l1-calls.md) |
| `-  - callbackPublicKey` | |
| `-  - successCallbackCallHash,` | of the callback function to execute upon success of the L1 call. |
| `-  - failureCallbackCallHash` | of the callback function to execute upon failure of the L1 call. |
| `- - successResultArgMapAcc` | See [here](../contracts/l1-calls.md#successresultargmapacc). |
| `}],` | |
| `- optionallyRevealedData: [{` | Some values from a private call can be optionally revealed to the 'public world', depending on bools of the private circuit ABI. For some/every private call each 'object' in this 'array' contains the following fields (some of which might be 0, depending on the bools) (note: there might be more efficient ways to encode this data - this is just for illustration): |
| `-  - callStackItemHash,` | Serves as a 'lookup key' of sorts. |
| `-  - functionSignature,` | |
| `-  - emittedPublicInputs: [_, _, _, _],` | |
| `-  - vkHash,` | |
| `-  - portalContractAddress,` | |
| `-  - <bools>` | :question: Discussion needed. We might also need to reveal all the bools to the public kernel snark, so that it may filter out data that doesn't need to be revealed to L1. |
| `- }, ...]` | |
| `- outputCommitments,` | Output state of commitments to be added to `privateDataTree` (up to 64) |
| `- inputNullifiers,` | Output state of nullifiers to be added to `nullifierTree` (up to 64) | 
| `},` | |
| `constants: {` | |
| `- oldTreeRoots: {` | |
| `- - privateDataTree,` | must equal value used in private call proof |
| `- - contractTree,` | A recent root of the contract tree (for vk membership checks) |
| `- - l1ResultsTree,` | A recent root of the L1 results tree. This MUST always be populated (even though it's only used when `privatelyExecutedCallback` is nonempty) to hide whether or not this is a callback. To enforce this, the correctness of this value (even if it's unused within thie circui) will be checked by the base rollup circuit. |
| `- },` | |
| `- privateKernelVKTreeRoot` | The root of a little Merkle tree whose leaves are hashes of the private kernel circuit VKs, allowing support for multuple 'sizes' of kernel circuit (i.e. with varying numbers of public inputs) |
| `- isConstructorRecursion`, | Flags whether this entire recursion began with a constructor function. |
| `- isCallbackRecursion`, | Flags whether this entire recursion began with a callback function. |
| `- executedCallback: {` | Only populated (if at all) by the first call in a tx, if that call is a callback AND if the callback's execution doesn't want to be 'hidden' :question: WHY WOULD WE EVER WANT THIS CASE?. This gives details of the callback that's been executed, so the callback and result can be cross-checked against the l1ResultsTree eventually in the Base Rollup Circuit.<br/> We assume the rollup provider will have a record of the contents of each leaf of the l1ResultsTree, so we don't need to provide the data here; the rollup provider can pass it as a private input to the Base Rollup snark. |
| `-  - l1ResultHash`, | This will eventually be compared against a leaf of the L1 Callback Tree in the Base Rollup Circuit. |
| `-  - l1ResultsTreeLeafIndex`, | Communicates to the rollup provider the correct leaf to prove membership against in the l1ResultsTree. This will eventually be used to lookup a leaf of the L1 Callback Tree in the Base Rollup Circuit. |
| `- },` | |
| `- globals: {` | |
| `-  - minTimestamp` | must equal value used in private call proof |
| `- }` | |
| `}` | |
| `isPrivate = true` | Bool. Tells the next kernel circuit that this is a private kernel snark. |
| `isPublic = false` | Bool. Tells the next kernel circuit that this is not a public kernel snark. |
| `isContractDeployment = false` | Bool. Tells the next kernel circuit that this is not a contract deployment kernel snark. |

## Execution Logic

- `require(previousKernel.publicInputs.isPublic == false && previousKernel.publicInputs.isContractDeployment == false)`


TODO: less nesting! (Or, in practice, we'll modularise this all into neat functions).

Base case:
* if `start.privateCallCount == 0`:
    * Require previous kernel data to be empty.
    * Validate that `start.privateCallStack.length == 1 && start.publicCallStack.length == 0 && start.contractDeploymentCallStack.length == 0 && start.l1CallStack.length == 0`
        - TBD: to allow the option of a fee payment, we might require `start.privateCallStack.length` to be "1" or "2, where one tx has an `isFeePayment` indicator". We could even allow any number of initial private calls on the stack, but that's a pretty big deviation from the ethereum tx model.
    * Pop the only (TBD) `privateCallStackItemHash` off the `start.privateCallStack`.
        - If `privateCall.functionSignature.isConstructor == true`:
            - then we don't need a signature from the user, since this entire 'callstack' has been instantiated by a Contract Deployment kernel snark (which itself will have been signed by the user).
            - Set `constants.isConstructorRecursion := true` - This public input will percolate to, and be checked by. the Contract Deployment Kernel Circuit which calls this constructor. This check is required to prevent a person from circumventing the ECDSA signature check by simply setting `isConstructor = true` when making a private call. If this aggregated kernel snark reaches the rollup circuit without this flag being reset to `false` by the Contract Deployment Kernel Circuit (to say "yes, this kernel was indeed a constructor for a Contract Deployment Kernel Circuit"), then the entire tx will be rejected by the rollup circuit.
        - Else:
            - Set `constants.isConstructorRecursion := false`
            - Verify the ECDSA `signature`, with `message := privateCallStackItemHash` and `signer := privateCall.callContext.msgSender`.
            - Validate the `callContext`. Usually the correctness of a callContext is checked between the `privateCall` and all the new calls it makes (see later in this logic). That means for this 'Base case', those checks haven't been done for this `privateCall` (since there was no prior iteration of this kernel circuit to make those checks).
                - If `privateCall.isDelegateCall == true || privateCall.isStaticCall == true`:
                    - Revert - a user cannot make a delegateCall or staticCall.
                - Else:
                    - Assert `privateCall.callContext.storageContractAddress == privateCall.functionSignature.contractAddress`
        - If `privateCall.functionSignature.isCallback == true`:
            - Set `constants.isCallbackRecursion = true;` - this can only be set in the _first_ call of a recursion.
            - We can infer whether the user wants to keep this callback's execution private, or if they want to expose it, from the `privatelyExecutedCallback` private inputs:
            - If `privatelyExecutedCallback` is zeroes, then the user doesn't want to execute this callback privately (maybe it interacts with some public state tree data).
                - Let `exposeCallback := true;`
                - Copy over the `privateCall.publicInputs.executedCallback` data to this snark's output: `constants.executedCallback`.
            - Else:
                - Let `exposeCallback := false;`
                - In which case we'll check the validity of this callback here:
                - Extract `{ l1ResultsTreeSiblingPath, callbackStackItem: { callbackPublicKey, successCallback, failureCallback }, callbackPrivateKey } = privateCall.privatelyExecutedCallback;`
                - Extract `{ l1ResultHash, l1ResultsTreeLeafIndex } = privateCall.publicInputs.executedCallback;`
                - If `l1ResultHash != 0` (success):
                    - Check the successCallback's `functionSignature` matches the `privateCall`'s:
                    - `require(successCallback.functionSignature == privateCall.functionSignature)`
                    - In fact, check the entire callbackCallHash reconciles.
                    - Also check the `successResultArgMapAcc` reconciles.
                      - Extract array `positionData = privateCall.privatelyExecutedCallback.l1SuccessResultArgPositions;`
                      - Extract the underlying results `l1SuccessResultValues = privateCall.privatelyExecutedCallback.l1SuccessResultValues;`
                      - Reconcile the `privateCall.privatelyExecutedCallback.callbackStackItem.successResultArgMapAcc` against `positionData` (messy computation if using turboplonk).
                        - `positionData` is probably binary decompositions, so range checks aren't needed. Multiply all the stuff together with the primes (details omitted, since this solution is a stop-gap). No need to prevent duplicate positions here: the app circuit must do that when it makes the initial call.
                        - Compare the resulting value against `successResultArgMapAcc`.
                        - Then, for turbo: sum each of the `positionData` 'columns' to get the `position` as a value. For each `position` loop through the `privateCall.publicInputs.customPublicInputs` until reaching the index `i` of the `position`, and then `require(customPublicInputs[i] == l1SuccessResultValue[i]`. What a pain - Bring on ultraplonk!
                - Else:
                    - Check the failureCallback's `functionSignature` matches the `privateCall`'s:
                    - `require(failureCallback.functionSignature == privateCall.functionSignature)`
                    - In fact, check the entire callbackCallHash reconciles.
                - Let `callbackStackItemHash = hash(callbackStackItem)`
                - Let `leafValue = hash(l1ResultHash, callbackStackItemHash)`
                - Check membership of the `leafValue` in the `l1ResultsTree` using:
                    - `l1ResultsTreeLeafIndex`
                    - `l1ResultsTreeSiblingPath`
                    - `constants.oldTreeRoots.l1ResultsTree`
                - Validate the `callbackPrivateKey`:
                    - `require(callbackPrivateKey * G = callbackPublicKey)` (for some fixed point `G`).
                - Set `callbackNullifier := hash(callbackDatumHash, callbackPrivateKey)` to prevent this callback from being executed again.
                - Set `isCallbackRecursion = false;` to hide that this is a callback, and to prevent callback logic being triggered later in the Base Rollup circuit.
                - Ensure `constants.executedCallback` is zeroes.
                - We don't include the whole `leafValue` in the preimage of the nullifier, to prevent a user maliciously calling the callback twice: once for 'success' and once for 'fail' (although that shouldn't be possible if the 'L1 results copier' circuit works correctly).
        - Else:
            - Set `isCallbackRecursion = false;`
            - Assert `l1ResultHash == 0`

Recursion:
* If `previousKernel.publicInputs.isPrivate && start.privateCallCount > 0`:
    - If `privateCall.functionSignature.isConstructor == true || privateCall.functionSignature.isCallback == true`:
        - Revert - only the first call in the kernel recursion can be a constructor or a callback.
    * Verify the `previousKernelProof` using the `previousKernelVK`
    * Validate that the `previousKernelVK` is a valid private kernel VK with a membership check:
        * Calculate `previousKernelVKHash := hash(previousKernelVK);`
        * Compute `root` using the `previousKernelVKHash`, `previousKernelVKPath` and `previousKernelVKIndex`.
        * Validate that `root == privateKernelVKTreeRoot`.
    * Validate consistency of 'starting' and 'previous end' values:
        - verify that the `start...` values match the `previousKernelProofPublicInputs.end...` equivalents. (As mentioned in the table above, we might be able to just pass one set of inputs to avoid this cross-checking)
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit):
        * ensure this kernel circuit's 'constant' public inputs match the `previousKernelProof`'s public inputs.
            * E.g. old tree roots.
        - Also ensure that any 'append-only' stacks or arrays have the same entries as the previous kernel proof, before pushing more data onto them!

Verify the next call on the callstack:
* Verify `start.privateCallStack.length > 0` and (if not already done during the 'Base Case' logic above), pop 1 item off of `start.privateCallStack`.
* Validate that `privateCall.functionSignature.isPrivate == true` (otherwise this is the wrong type of kernel circuit to be using).
* Validate that this newly-popped  `privateCallStackItemHash` corresponds to the `privateCall` data passed into this circuit:
    * Calculate `privateCallPublicInputsHash := hash(privateCall.publicInputs);`
    * Verify that `privateCallStackItemHash == hash(privateCall.functionSignature, privateCallPublicInputsHash, privateCall.callContext, etc...)`
    * Recall, the structure of a callstack item:
    * ```js
      privateCall = {
        functionSignature: {
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
* Verify the correctness of `(proof, privateCallPublicInputsHash)` using the `vk`.
* Validate the `vk` actually represents the function that is purportedly being executed:
    * Extract the `contractAddress` and `vkIndex` from `privateCall.functionSignature`.
    * Compute `vkHash := hash(vk)`
    * Compute the `vkRoot` of this function's contract using the `vkIndex`, `vkPath`, and `vkHash`.
    * Compute the contract's leaf in the `contractTree`:
      * Let `leafValue := hash(contractAddress, portalContractAddress, vkRoot)`
    * Compute `contractTreeRoot` using the `contractLeafIndex`, `contractPath` and `leafValue`.
    * Validate that `contractTreeRoot == constants.oldTreeRoots.contractTree`.
* Validate consistency of the `privateCall`'s `constant` data with that of the kernel circuit's public inputs (and those of the previousKernelProof). I.e.:
    * same old tree roots
    * same timestamp values, etc.

Update the `end` values:
* Extract the private call's `outputCommitments` and `inputNullifiers`.
    - If `privateCall.isStaticCall == true`:
        - Assert that the `outputCommitments` and `inputNullifiers` are empty, since no state changes are allowed for static calls.
        - NOTE: although it's technically possible to emulate pure reads from the privateDataTree by creating an output commitment with value equal to the input commitment being nullified (i.e. being 'read') - there's no way for the kernel circuit to 'trust' that the logic of the private circuit actually did a pure read and didn't change state (since everything is private).
    - Else, 'silo' the `outputCommitments` and `inputNullifiers` with the contract address of the callContext: `storageContractAddress := privateCall.callContext.storageContractAddress`:
        - For each `outputCommitment` in `outputCommitments`:
            - `siloedOutputCommitment := hash(storageContractAddress, outputCommitment);`
        - For each `inputNullifier` in `inputNullifiers`:
            - `siloedInputNullifier := hash(storageContractAddress, inputNullifier);`
    - Push those values (`siloedOutputCommitments` and `siloedInputNullifiers`) onto `end.outputCommitments`, `end.inputNullifiers`
    - Also push `callbackNullifier` onto `end.inputNullifiers`, if it exists.
    * It would be great if these new commitments & nullifiers could be pushed onto the first nonzero element of the `end` lists, so that these `end` lists remain tightly-packed, allowing many more rounds of recursion. Within a circuit, searching for this first nonzero element's index and then pushing to that position will use approx `(3 * 16 + 1) * 256 = 12544` constraints - so `2 * 12544 = 25088` constraints to push both the nullifier and commitment lists.
        * n = 256
        * m = 16
        * `n` checks for the first nonzero entry in the `end` list.
        * For each of the m nullifiers, loop through the `end` list and conditionally add the nullifier to that position of the `end` list if it's the next nonzero index available. Assuming 3 constraints per iteraction, that's `3mn` constraints.
        * So `(3m + 1)n` altogether. (Of course it'll end up being way more in practice).
* Extract the private call's `privateCallStack`, `publicCallStack`, `contractDeploymentCallStack`.
    - Validate the call contexts of these calls:
        - For each `newCallStackItem` in `privateCallStack`, `publicCallStack`, `contractDeploymentCallStack` and `partialL1CallStack`.
            - If `newCallStackItem.functionSignature.contractAddress == 0`:
                - Then this `0` is understood to be a call to `address(this)`, which cannot be populated by the app's circuit itself.
                - We must mutate the contract address to be that of the `privateCall`.
                - Mutation: Set `newCallStackItem.functionSignature.contractAddress = privateCall.functionSignature.contractAddress`
            - If `newCallStackItem.isDelegateCall == true`:
                - Assert `newCallStackItem.callContext == publicCall.callContext`
            - Else:
                - Assert `newCallStackItem.callContext.msgSender == publicCall.functionSignature.contractAddress`
                - Assert `newCallStackItem.callContext.storageContractAddress == newCallStackItem.functionSignature.contractAddress`
    - Validate `partialL1CallStack` and `callbackStack` lengths:
      - `require(partialL1CallStack.length == callbackStack.length)` - every L1 call must have a callback entry (even if the callbacks are zeroes).
    - For each `partialL1CallStackItem` in the `partialL1CallStack` (index `i`):
      - Ensure that the call is being sent to the associated portal contract address, by adding the `portalContractAddress` here:
        - Let `l1CallStackItem := keccak(portalContractAddress, partialL1CallStackItem)`
      - Validate `publicCall.callbackStack[i]`:
        - Let `successCallback = publicCall.callbackStack[i].successCallback`
        - Let `failureCallback = publicCall.callbackStack[i].failureCallback`
        - Ensure the contract address of each of the two callbacks matches that of the public call:
          - TODO: we might actually want to _mutate_ the contractAddress from `0` to the correct address here. `0` initially, because a contract doesn't know its own address, so `0` can mean an instruction to this kernel snark to insert `address(this)`
          - `require(successCallback.functionSignature.contractAddress == publicCall.functionSignature.contractAddress)`;
          - `require(failureCallback.functionSignature.contractAddress == publicCall.functionSignature.contractAddress)`;
        - Calculate the callbackHashes:
          - Let `successCallbackHash = hash(successCallback.functionSignature, etc...)`
          - Let `failureCallbackHash = hash(failureCallback.functionSignature, etc...)`
    - Push the contents of these call stacks onto the kernel circuit's `end.privateCallStack`, `end.publicCallStack`, `end.contractDeploymentCallStack` and `end.l1CallStack`.
    - Also push the each `{callbackPublicKey, successCallbackHash, failureCallbackHash}` onto `end.callbackStack`.
    - As per the commitments/nullifiers bullet immediately above, it would be nice if these 'pushes' could result in tightly-packed stacks.
- Determine whether any values need to be optionally revealed to the 'public world', by referring to the `publicCall`'s booleans:
    - Let `optionallyRevealedData = {};`
    - If `privateCall.functionSignature.isConstructor`, set:
        - `optionallyRevealedData.callStackItemHash = privateCallStackItemHash;`
        - `optionallyRevealedData.vkHash = vkHash;`
        - `optionallyRevealedData.emittedPublicInputs = privateCall.publicInputs.emittedPublicInputs;`
    - If `privateCall.functionSignature.isCallback && exposeCallback`, set:
        - `optionallyRevealedData.functionSignature = privateCall.functionSignature;`
        - `optionallyRevealedData.emittedPublicInputs = privateCall.publicInputs.emittedPublicInputs;`
          - TODO: consider whether we can get rid of the emittedPublicInputs being emitted by a callback. More [here](../contracts/l1-calls.md#more-details).
    - If `isFeePayment`, set:
        - `optionallyRevealedData.callStackItemHash = privateCallStackItemHash;`
        - `optionallyRevealedData.functionSignature = privateCall.functionSignature;`
        - `optionallyRevealedData.emittedPublicInputs = privateCall.publicInputs.emittedPublicInputs;`
    - If `payFeeFromL1`, set:
        - `optionallyRevealedData.callStackItemHash = privateCallStackItemHash;`
    - If `calledFromL1`, set:
        - `optionallyRevealedData.callStackItemHash = privateCallStackItemHash;`
        - `optionallyRevealedData.functionSignature = privateCall.functionSignature;`
        - `optionallyRevealedData.emittedPublicInputs = privateCall.publicInputs.emittedPublicInputs;`
        - `optionallyRevealedData.portalContractAddress = portalContractAddress;` // TODO: we need to check this portal contract address is the correct one within this kernel circuit (with a membership check). We need to decide where in L2 this should be stored: either in the contract tree or in the public data tree (e.g. at storage slot 0 for each contract). Argh, it cannot be in the public data tree, because the private kernel circuit cannot access that :heavy_exclamation_mark:
    - Push the `optionallyRevealedData` onto the `optionallyRevealedData`.
- Throw an error if any of the stacks is out of room!
* If `end.privateCallStack.length == 0`, set `end.privateCallCount = 0`, else `end.privateCallCount = start.privateCallCount + 1`
* Set `isPrivate := true;` (and the others are false)
* Ensure all unused public inputs (which are _not_ shown in the table above, but will be included in practice so that all kernel snarks have the same public input ABI) are `0`.


A private transaction is executed by creating a private Kernel Snark proof over a circuit that contains 1 entry in either `start.privateCallStack` or `start.publicCallStack` or `start.contractDeploymentCallStack` and a valid ECDSA signature.

Execution finishes when `end.privateCallStack.length == 0`. If this condition is not met, the kernel snark proof is recursively fed into the kernel snark circuit and a new proof is made.
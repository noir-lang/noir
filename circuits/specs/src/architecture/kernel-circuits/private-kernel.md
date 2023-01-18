# Private Kernel Circuit

# Private inputs ABI

## `PreviousKernelData`

| Value | Description |
| -------- | -------- |
| `publicInputs`: [`PrivateKernelPublicInputs`](#privatekernelpublicinputs) | The public inputs from the previous iteration of the kernel snark, to be verified within this iteration. |
| `proof`: `Proof` | The previous kernel snark's proof. |
| `vk`: `VerificationKey` | The vk that should be used when verifying the previous kernel snark. |
| `privateKernelVKTreeRoot`: `Field` | To allow multiple sizes and permutations of kernel circuits, we can put their vks as leaves in a big tree. We can then perform a membership check to ensure the vk is from an approved kernel circuit. |
| `vkIndex`: `Field` | The index of the kernel circuit's vk within the private kernel vk tree. | 
| `vkPath`: `Array[Field]` | Part of the membership proof of the kernel circuit's vk within the private kernel vk tree. |

## `PrivateCallData`

| Value | Description |
| -------- | -------- |
| `callStackItem`: [`PrivateCallStackItem`](../contracts/transactions.md#privatecallstackitem) | Unpacked data relating to the next private callstack item that we're going to pop off the callstack and verify within this snark. |
| `proof`: `Proof` | The proof for this item of the callstack. We'll be verifying this within this kernel circuit. |
| `vk`: `VerificationKey` | The full verification key of the private call proof that will be verified within this circuit. |
| `vkPath`: `Array[Field]` | The sibling path of the `vk`'s `vkHash` within the aztec contract's 'mini merkle tree'. Note, the `vkIndex` to use when proving membership can be grabbed from the `callStackItem`. |
| `oldContractTreeRoot`: `Field` | The root of the contract tree which the kernel circuit may use to validate the existence of this contract. |
| `contractLeafIndex`: `Field` | The leaf index of the contracts tree where this contract's vkTree is stored |
| `contractPath` | The sibling path from this contract's root (a leaf of the contract tree) to the `oldContractTreeRoot` |
| `portalContractAddress` | The portal contract address that corresponds to the contract of the function being called. |

## `PrivateKernelPrivateInputs`

| Value | Description |
| -------- | -------- |
| `signedTxObject`: [`SignedTxObject`](../contracts/transactions.md#signedtxobject) | Only required to validate permissions for the first call in the callstack. TODO: generalise for account abstraction. |
| `previousKernel`: [`PreviousKernelData`](#previouskerneldata) | Previous kernel data (empty if this is the first iteration). |
| `privateCall`: [`PrivateCallData`](#privatecalldata) | Details about the next call which will be popped of the callstack during this kernel recursion. |


# Public inputs ABI

## `AggregationObject`

A barrretenberg object. Might change in future. Don't worry about this too much.

| Value | Description |
| -------- | -------- |
| `P0`: `Point` | |
| `P1`: `Point` | |
| `publicInputs` | |
| `proofWitnessIndices` | |

## `OptionallyRevealedData`

Some values from a private call can be optionally revealed to the 'public world', depending on the function and its [`TxContext`](../contracts/transactions.md#txcontext). For some/every private call we have the following fields (some of which might be 0). (Note: there might be more efficient ways to encode this data - this is just for illustration):

| Value | Description |
| -------- | -------- |
| `callStackItemHash`: `Field` | Serves as a 'lookup key' of sorts, for when an L1 function has made a call to a L2 function, and will need to validate that the correct call was made. |
| `functionSignature`: [`FunctionSignature`](../contracts/transactions.md#functionsignature) | |
| `emittedEvents` | Emitting data to another layer. |
| `vkHash` | |
| `portalContractAddress` | Needed when making a call from L2 to L1. |

## `AccumulatedData`

The end state of a kernel recursion. It can be used as the 'start' state of the next kernel recursion.

| Value | Description |
| -------- | -------- |
| `aggregationObject`: [`AggregationObject`](#aggregationobject) | A representation of the aggregation of all proofs so far in the recursion. |
| `privateCallCount`: `Field` | How many calls have been recursively executedso far? |
| `newCommitments`: `Array[Field]` | A list of all commitments created by all circuits recursed-through so far. |
| `newNullifiers`: `Array[Field]` | A list of all nullifiers created by all circuits recursed-through so far. |
| `privateCallStack`: `Array[Field]` | A call stack containing any calls made by all of the circuits previously recursed-through, which haven't-yet been popped off this stack. | 
| `publicCallStack`: `Array[Field]` | " | 
| `contractDeploymentCallStack`: `Array[Field]` | " | 
| `l1CallStack`: [`Array[L1CallstackItem]`](../contracts/transactions.md#l1callstackitem) | " |
| `optionallyRevealedData`: [`Array[OptionallyRevealedData]`](#optionallyrevealeddata) |  |


## `OldTreeRoots`

| Value | Description |
| -------- | -------- |
| `privateDataTreeRoot`: `Field` | |
| `constPublicDataTreeRoot`: `Field` | (maybe) |
| `nullifierTreeRoot`: `Field` | |
| `contractTreeRoot`: `Field` | |
| `resultsTreeRoot`: `Field` | |
| `privateKernelVKTreeRoot`: `Field` | |

## `RecursionContext`

Contains many of the same values as [`TxContext`](../contracts/transactions.md#txcontext), but with some extra info thrown in. TODO: some of this data might need to be 'swallowed' by the last kernel recursion, and only some of the data optionally revealed, via [`OptionallyRevealedData`](#optionallyrevealeddata).

| Value | Description |
| -------- | -------- |
| `calledFromL1`: `Field` | |
| `calledFromPublicL2`: `Field` | |
| `isConstructor`: `Bool` | |
| `isFeePaymentTx`: `Bool` | |
| `feeData`: [`FeeData`](../contracts/transactions.md#feedata) | |
| `referenceBlockNum`: `Field` | |
| `referenceTimestamp`: `Field` | |

## `ConstantData`

| Value | Description |
| -------- | -------- |
| `oldTreeeRoots`: [`OldTreeRoots`](#oldtreeroots) | |
| `recursionContext`: [`RecursionContext`](#recursioncontext) | |

## `PrivateKernelPublicInputs`

| Value | Description |
| --- | --- |
| `end`: `AccumulatedData` | |
| `constants`: [`ConstantData`](#constantdata) | Data which will not change between kernel recursions. |
| `kernelType`: `Enum[private, public, contract_deployment]` | |

## Execution Logic

- `require(previousKernel.publicInputs.isPublic == false && previousKernel.publicInputs.isContractDeployment == false)`


In practice, we'll modularise this all into neat functions when we actually write the code.

Base case:
* Let `start := previousKernelData.publicInputs.end`  ✅
* if `start.privateCallCount == 0`: ✅
    * Require previous kernel data to be empty. (Note: bear in mind - the `verify_proof()` function needs a valid dummy proof and vk to complete execution). ❓
    * Validate that `start.privateCallStack.length == 1 && start.publicCallStack.length == 0 && start.contractDeploymentCallStack.length == 0 && start.l1CallStack.length == 0` ✅
        - TBD: to allow the option of a fee payment, we might require `start.privateCallStack.length` to be "1" or "2, where one tx has an `isFeePayment` indicator". ✅
    * Pop the only (TBD) `privateCallHash` off the `start.privateCallStack`. ✅
        - Validate that `hash(privateCall.callStackItem) == privateCallHash` ✅
        - If `privateCall.callStackItem.functionSignature.isConstructor == true`: ❌
            - THIS SECTION IS OUT OF DATE - IGNORE!
            - then we don't need a signature from the user, since this entire 'callstack' has been instantiated by a Contract Deployment kernel snark (which itself will have been signed by the user).
            - Set `constants.recursionContext.isConstructor := true` - This public input will percolate to -- and be checked by -- the Contract Deployment Kernel Circuit which calls this constructor. This check is required to prevent a person from circumventing the ECDSA signature check by simply setting `isConstructor = true` when making a private call. If this aggregated kernel snark reaches the rollup circuit without this flag being reset to `false` by the Contract Deployment Kernel Circuit (to say "yes, this kernel was indeed a constructor for a Contract Deployment Kernel Circuit"), then the entire tx will be rejected by the rollup circuit.
        - Else:
            - Set `constants.recursionContext.isConstructor := false`
            - Verify the ECDSA signature contained in `signedTxObject`.
            - Validate the `callContext`. Usually the correctness of a callContext is checked between the `privateCall` and all the new calls it makes (see later in this logic). That means for this 'Base case', those checks haven't been done for this `privateCall` (since there was no prior iteration of this kernel circuit to make those checks).
                - If `privateCall.isDelegateCall == true || privateCall.isStaticCall == true`:
                    - Revert - a user cannot make a delegateCall or staticCall.
                - Else:
                    - Assert `privateCall.callStackItem.publicInputs.callContext.storageContractAddress == privateCall.callStackItem.contractAddress`

Recursion:
* If `previousKernel.publicInputs.isPrivate && start.privateCallCount > 0`: ✅
    - If `privateCall.callStackItem.functionSignature.isConstructor == true`:
        - Revert - only the first call in the kernel recursion can be a constructor.
    * Verify the `previousKernel.proof` using the `previousKernel.vk` ✅
    * Validate that the `previousKernel.vk` is a valid private kernel VK with a membership check:
        * Calculate `previousKernelVKHash := hash(previousKernel.vk);`
        * Compute `root` using the `previousKernelVKHash`, `previousKernel.vkPath` and `previousKernel.vkIndex`.
        * Validate that `root == privateKernelVKTreeRoot`.
    * Validate consistency of 'starting' and 'previous end' values: ✅
        - verify that the `start...` values match the `previousKernel.publicInputs.end...` equivalents. ✅
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit): ✅
        * ensure this kernel circuit's 'constant' public inputs match the `previousKernel.publicInputs.constants`. ✅
            * E.g. old tree roots. ✅
        - Also ensure that any 'append-only' stacks or arrays have the same entries as the previous kernel proof, before pushing more data onto them! ✅

Verify the next call on the callstack:
* Verify `start.privateCallStack.length > 0` and (if not already done during the 'Base Case' logic above, depending on how we do the implementation), pop 1 item off of `start.privateCallStack` (a `privateCallStackItemHash`)
* Validate that `privateCall.callStackItem.functionSignature.isPrivate == true` (otherwise this is the wrong type of kernel circuit to be using).
* Validate that this newly-popped  `privateCallStackItemHash` corresponds to the `privateCall` data passed into this circuit:
    * Calculate `privateCallPublicInputsHash := hash(privateCall.callStackItem.publicInputs);`
    * Verify that `privateCallStackItemHash == hash(privateCall.callStackItem)`
    * Recall, the structure of a [callstack item](../contracts/transactions.md#privatecallstackitem).
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
    - For each `partialL1CallStackItem` in the `partialL1CallStack` (index `i`):
      - Ensure that the call is being sent to the associated portal contract address, by adding the `portalContractAddress` here:
        - Let `l1CallStackItem := keccak(portalContractAddress, partialL1CallStackItem)`
    - Push the contents of these call stacks onto the kernel circuit's `end.privateCallStack`, `end.publicCallStack`, `end.contractDeploymentCallStack` and `end.l1CallStack`.
    - As per the commitments/nullifiers bullet immediately above, it would be nice if these 'pushes' could result in tightly-packed stacks.
- Determine whether any values need to be optionally revealed to the 'public world', by referring to the `publicCall`'s booleans:
    - Let `optionallyRevealedData = {};`
    - If `privateCall.functionSignature.isConstructor`, set:
        - `optionallyRevealedData.callStackItemHash = privateCallStackItemHash;`
        - `optionallyRevealedData.vkHash = vkHash;`
        - `optionallyRevealedData.emittedPublicInputs = privateCall.publicInputs.emittedPublicInputs;`
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
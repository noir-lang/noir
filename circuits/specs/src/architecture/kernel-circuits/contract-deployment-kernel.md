# THIS PAGE IS OUT OF DATE

# Contract Deployment Kernel Circuit


**NOTE: this section is out of date (naming and architecture), given recent conversations.**


Notice, that this is a _kernel_ circuit. That's because this circuit will itself verify a private &or public kernel snark representing the new contract's 'constructors'. Unlike the other two (private & public) kernel snarks though, Contract Deployment Kernel Circuits can be called by users and/or functions, meaning they have a callStack of their own which can be pushed-to. The Contract Deployment call stack will be processed at the very end of the kernel chain; after the private and public kernels have been executed.

You'll notice that the private/public inputs for this kernel circuit are a superset of the inputs of both the private & public kernel circuits. That's because contract deployments are processed at the very end of a tx, so all other data needs to 'pass through' this circuit.

### Private inputs ABI

| Data Type | Description |
| -------- | -------- |
| `signature` | ECDSA signature signed by user (if no private/public kernel proof has been provided && `start.contractDeploymentCallCount == 0`, otherwise empty) |
| `start: {` | |
| `- aggregatedProof` | The current aggregated proof state (if any) |
| `- contractDeploymentCallCount` | How many _contract deployment_ calls have been recursively executed so far? |
| `- contractDeploymentCallStack` |  Starting state of contract deployment call stack (max depth 4?) |
| `- l1CallStack` | Starting state of l1 call stack (max depth 64?) |
| `- callbackStack` | See breakdown in the public inputs ABI in the next table |
| `- optionallyRevealedData` | See breakdown in the public inputs ABI in the next table |
| `- deployedContractData` | A keccak hash of information about each deployed contract. |
| `- proverRecords` | |
| `- outputCommitments` | Starting state of commitments to be added to `privateDataTree`.<br>Notice that a contract deployment circuit is allowed to add commitments to the privateDataTree. |
| `- inputNullifiers` | Deployment of a new contract to an address produces a `contractAddressNullifier` which needs to be added to this list of nullifiers. |
| `- publicDataTreeRoot` | The publicDataTree's root _before_ we make any state transitions (as requested in the contractDeploymentCall being processed within this circuit). |
| `- contractTreeRoot` | The contractTree's root _before_ we deploy any new contract (as requested in the contractDeploymentCall being processed within this circuit). |
| `- nextAvailableContractTreeLeafIndex` | The index of the left-most nonempty leaf of the contract tree _after_ deploying a new contract within this circuit. |
| `}` | |
| `previousKernel: {` | |
| `- proof` | |
| <pre>- publicInputs = {<br/>&nbsp;&nbsp;end: start, // copy within circuit.<br/>&nbsp;&nbsp;constants: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;oldTreeRoots: {<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;privateDataTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;contractTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;l1ResultsTree,<br/>&nbsp;&nbsp;&nbsp;&nbsp;},<br/>&nbsp;&nbsp;&nbsp;&nbsp;privateKernelVKTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;publicKernelVKTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;initialPublicDataTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;initialContractTreeRoot,<br/>&nbsp;&nbsp;&nbsp;&nbsp;initialNextAvailableContractTreeLeafIndex,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isConstructorRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;isCallbackRecursion,<br/>&nbsp;&nbsp;&nbsp;&nbsp;executedCallback: {},<br/>&nbsp;&nbsp;&nbsp;&nbsp;globals: {,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;minTimestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;block.timestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;block.number,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;prevBlock.ethTimestamp,<br/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;prevBlock.ethNumber,<br/>&nbsp;&nbsp;&nbsp;&nbsp;}<br/>&nbsp;&nbsp;}<br/>&nbsp;&nbsp;proverAddress,<br/>&nbsp;&nbsp;isPrivate,<br/>&nbsp;&nbsp;isPublic,<br/>&nbsp;&nbsp;isContractDeployment,<br/>}</pre> |  |
| `- vk` | The VK that should be used to verify the `previousKernel.proof`. |
| `- vkIndex` | The leaf index of the `previousKernel.vk`, to accompany the `previousKernel.vkPath` in doing a membership check. |
| `- vkPath` | We can support multiple 'sizes' of kernel circuit (i.e. kernel circuits with varying numbers of public inputs) if we store all of the VKs of such circuits as vkHashes in a little Merkle tree. This path is the sibling path from the `previousKernel.vk`'s leaf to the root of such a tree. |
| `}` | |
| `privateConstructorKernel: {` | |
| `- proof` | |
| `- publicInputs` | The public inputs of a private _kernel_ snark (see the relevant section). |
| `- vk` | The VK that should be used to verify the `privateConstructorKernel.proof`. |
| `- vkIndex` | The leaf index of the `privateConstructorKernel.vk`, to accompany the `privateConstructorKernel.vkPath` in doing a membership check. |
| `- vkPath` | We can support multiple 'sizes' of kernel circuit (i.e. kernel circuits with varying numbers of public inputs) if we store all of the VKs of such circuits as vkHashes in a little Merkle tree. This path is the sibling path from the `privateConstructorKernel.vk`'s leaf to the root of such a tree. |
| `}` | |
| `publicConstructorKernel: {` | |
| `- proof` | |
| `- publicInputs` | The public inputs of a public _kernel_ snark (see the relevant section). |
| `- vk` | The VK that should be used to verify the `publicConstructorKernel.proof`. |
| `- vkIndex` | The leaf index of the `publicConstructorKernel.vk`, to accompany the `publicConstructorKernel.vkPath` in doing a membership check. |
| `- vkPath` | We can support multiple 'sizes' of kernel circuit (i.e. kernel circuits with varying numbers of public inputs) if we store all of the VKs of such circuits as vkHashes in a little Merkle tree. This path is the sibling path from the `publicConstructorKernel.vk`'s leaf to the root of such a tree. |
| `}` | |
| `contractDeploymentCall: {` | Data relating to the next contract deployment callstack item that we're going to pop off the callstack and verify within this snark. Recall we're representing items in the callstack as a `callStackItemHash`. We'll pass the unpacked data here, for validation within this kernel circuit. | 
| <pre>- publicInputs: {<br/>&nbsp;&nbsp;privateConstructorPublicInputsHash, <br/>&nbsp;&nbsp;publicConstructorPublicInputsHash, <br/>&nbsp;&nbsp;privateConstructorVKHash, <br/>&nbsp;&nbsp;publicConstructorVKHash, <br/>&nbsp;&nbsp;contractAddress, <br/>&nbsp;&nbsp;salt,<br/>&nbsp;&nbsp;vkRoot, <br/>&nbsp;&nbsp;circuitDataKeccakHash, <br/>&nbsp;&nbsp;portalContractAddress, <br/>},</pre> | Rather than pass in the `publicInputsHash`, we pass in its preimage here (we'll hash it within this kernel circuit). This is _all_ of the data listed in the Contract Deployment ABI's table of public inputs (see earlier section). |
| `- callContext: {...},` | |
| `- isDelegateCall,` | |
| `- isStaticCall = false,` | |
| `- newContractSiblingPath` | The sibling path (in the contract tree) that will allow the new contract to be inserted at the next available leaf. |
| `- newContractStorageSlotSiblingPath[254]` | We'll need to write data to the `0`th storage slot of the new contract. This sibling paths allow such updates to be made to leaves of the `publicDataTree`. |
| `}` | |


### Public inputs ABI

| Data type | Description |
| --- | --- |
| `end: {` | |
| `- aggregatedProof` | Output aggregated proof |
| `- contractDeploymentCallCount` | How many calls have been recursively executed at end of circuit execution? (either `start.publicCallCount + 1` or `0` iff `end.publicCallStack` is empty |
| `- contractDeploymentCallStack` |  Output state of contract deployment call stack (max depth 4?) |
| `- l1CallStack` | Output state of l1 call stack (max depth 64?) |
| `- callbackStack: [{` | Data to add to the `l1ResultsTree`. See [here](../contracts/l1-calls.md) |
| `-  - callbackPublicKey` | |
| `-  - successCallbackCallHash,` | of the callback function to execute upon success of the L1 call |
| `-  - failureCallbackCallHash` | of the callback function to execute upon failure of the L1 call |
| `-  - successResultArgMapAcc` | |
| `}],` | |
| `- optionallyRevealedData: [{` | Some values from a public call can be optionally revealed to the Contract Deployment kernel circuit / L1, depending on bools of the public circuit ABI. For some/every public call, each 'object' in this 'array' contains the following fields (some of which might be 0, depending on the bools) (note: there might be more efficient ways to encode this data - this is just for illustration): |
| `-  - callStackItemHash,` | Serves as a 'lookup key' of sorts. |
| `-  - functionSignature,` | |
| `-  - emittedPublicInputs: [_, _, _, _],` | |
| `-  - vkHash,` | |
| `-  - portalContractAddress,` | |
| `-  - <bools>` | :question: Discussion needed. We might also need to reveal all the bools to the public kernel snark, so that it may filter out data that doesn't need to be revealed to L1. |
| `- }, ...]` | :question: Should a callStackItemHash contain a salt, to hide the nature of the call? |
| `- deployedContractData: [...]` | Each entry (one per newly deployed contract) is a keccak hash of information about that contract. The preimage of the hash will be submitted on-chain for reconciliation, so that users may validate the contract was deployed as purported. The preimage is simply the public inputs of a contractDeploymentCallStackItem. |
| `- proverRecords` | We need to record info about who generated public proofs and who generated public kernel snarks, so that they may be rewarded for their work. It's not as simple as there being one prover. Proving might be delegated to various people by the rollup processor, so we need to track an array (obvs of some bounded size) with an element for each prover. Each prover record entry in `proverRecords` will be of the form `[proverAddress, totalNumberOfGates]`, where we track the `totalNumberOfGates` this prover has generated proofs over, as an approximation of the work they've done.  |
| `- outputCommitments` | Output state of commitments to be added to `privateDataTree`.<br/><br/>Notice that a public circuit is allowed to add commitments to the privateDataTree. Imagine, for example, a defi-bridge between private L2 and public L2. Then the rollup processor can complete partial commitments (partial commitments can be added to the _public_ data tree), and the completed commitments can be added straight to the private data tree. |
| `- inputNullifiers` | Deployment of a new contract to an address produces a `contractAddressNullifier` which needs to be added to this list of nullifiers. |
| `- publicDataTreeRoot` | The root of the public data tree _after_ performing the stateTransitions requested by the contractDeploymentCall being processed within this circuit.  |
| `- contractTreeRoot` | The root of the contract tree _after_ deploying a new contract within this circuit. |
| `- nextAvailableContractTreeLeafIndex` | The index of the left-most nonempty leaf of the contract tree _after_ deploying a new contract within this circuit. |
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
| `- initialContractTreeRoot` | The root of the contract tree before _any_ of the calls from the contractDeploymentCallStack were executed. We need this to demonstrate that we started our 'contract deployment' recursion using the correct tree. This will be kept the same throughout the recursion. |
| `- initialnextAvailableContractTreeLeafIndex` | The index of the left-most nonempty leaf of the contract tree before _any_ of the calls from the contractDeploymentCallStack were executed. We need this to demonstrate that we started our contract deployments from the correct leaf. |
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
| `- },` | |
| `}` | |
| `proverAddress` | Who is going to be generating the proof for this kernel circuit (any value can be injected here by the prover). |
| `isPrivate = false` | Bool. Tells the next kernel circuit that this is not a private kernel snark. |
| `isPublic = false` | Bool. Tells the next kernel circuit that this is not a public kernel snark. |
| `isContractDeployment = true` | Bool. Tells the next kernel circuit that this is a contract deployment kernel snark. |



### Execution Logic

:heavy_exclamation_mark: It would be ideal if the number of public inputs of the private kernel, public kernel, and contract deployment kernel circuits are the same, so that the below base case and recursion cases can use the same verification code paths (and also so that the Rollup Circuit can accept any of these kernel proofs, to avoid having to unnecessarily 'wrap' every tx in a contract deployment kernel proof).

Note: the logic would be simplified if we only allowed 1 contract deployment per tx. TBC.

Current rule: you can't deploy a contract directly as an L1 callback (because it's too complicated, and because there's no 'app-specific' contract deployment circuit that could handle an L1 response - only public and private circuits can have custom logic). You'd have to call a public/private function as a callback which would then deploy a contract for you.

- `require(previousKernel.publicInputs.isPrivate == true NAND previousKernel.publicInputs.isPublic == true NAND previousKernel.publicInputs.isContractDeployment == true)`

Base case (verifying a public or private kernel snark):
* If `start.contractDeploymentCallCount == 0`:
    *  validate that `start.contractDeploymentCallStack.length > 0`
    *  if `(previousKernelProof.proof && (previousKernelProof.isPublic || previousKernelProof.isPrivate)`
        * Verify the `previousKernelProof` using the `previousKernelVK`
        * Validate that the `previousKernelVK` is a valid public or private kernel VK with a membership check:
            * Calculate `previousKernelVKHash := hash(previousKernelVK`;
            * Compute `root` using the `previousKernelVKHash`, `previousKernelVKPath` and `previousKernelVKIndex`.
            * Validate that `root == privateKernelTreeRoot` or `root == publicKernelTreeRoot`.
    - else:
        * Pop the only `contractDeploymentCallStackItemHash` off the `start.contractDeploymentCallStack`.
        *  Verify the ECDSA `signature`, with `message := contractDeploymentCallStackItemHash` and `signer := contractDeploymentCall.callContext.msgSender`.
    * Validate consistency of 'start' and 'previous end' values:
        - verify that the `start...` values match the `previousKernelProofPublicInputs.end...` equivalents.
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit):
        * ensure this kernel circuit's 'constant' public inputs match the `previousKernelProof`'s public inputs.
            * E.g. old tree roots.
        - Also ensure that any 'append-only' stacks or arrays have the same entries as the previous kernel proof, before pushing more data onto them!
    - Set `initialContractTreeRoot := start.contractTreeRoot`.

Recursion (of contract deployment kernel snarks - if we choose to support more than one contract deployment per tx):
* If `start.contractDeploymentCallCount > 0`:
    * Verify the `previousKernelProof` using the `previousKernelVK`
    * Validate that the `previousKernelVK` is a valid _public_ kernel VK with a membership check:
        * Calculate `previousKernelVKHash := hash(previousKernelVK)`;
        * Compute `root` using the `previousKernelVKHash`, `previousKernelVKPath` and `previousKernelVKIndex`.
        * Validate that `root == publicKernelVKTreeRoot`.
    * Validate consistency of 'start' and 'previous end' values:
        - verify that the `start...` values match the `previousKernelProofPublicInputs.end...` equivalents.
    * Validate consistency of values which must remain the same throughout the recursion (when passed from kernel circuit to kernel circuit):
        * ensure this kernel circuit's 'constant' public inputs match the `previousKernelProof`'s public inputs.
        * E.g. old tree roots.

Validate the next call on the contractDeploymentCallStack:
* Verify `start.contractDeploymentCallStack.length > 0` and (if not already done during the 'Base Case' logic above), pop 1 item off of `start.contractDeploymentCallStack` - call it `contractDeploymentCall`.
* Validate that this newly-popped `contractDeploymentCallStackItemHash` corresponds to the `contractDeploymentCall` data passed into this circuit:
    * Calculate `contractDeploymentCallPublicInputsHash := hash(contractDeploymentCall.publicInputs);`
    * Verify that `contractDeploymentCallStackItemHash == hash(contractDeploymentCall.functionSignature, contractDeploymentCallPublicInputsHash, contractDeploymentCall.callContext, etc...)`

Deploy the contract:
- Let `deployerAddress := contractDeploymentCall.callContext.msgSender`
- Extract `{ privateConstructorPublicInputsHash, publicConstructorPublicInputsHash, privateConstructorVKHash, publicConstructorVKHash, contractAddress, salt } = contractDeploymentCall;`
- Let `constructorsHash := hash(privateConstructorPublicInputsHash, publicConstructorPublicInputsHash, privateConstructorVKHash, publicConstructorVKHash)`
- Let `newContractAddress := hash(deployerAddress, salt, vkRoot, constructorHash)`
- `assert(contractAddress == newContractAddress)`
- Let `newContractAddressNullifier := hash(newContractAddress)` - used to prevent someone else deploying to this same contract address.
    - Remember to add this `newContractAddressNullifier` to the `inputNullifiers` array.
- Let `newPortalContractAddressNullifier := hash(portalContractAddress)` - used to prevent someone else linking to this same portal contract address.
    - Remember to add this `newPortalContractAddressNullifier` to the `inputNullifiers` array.
- Let `newContractLeafIndex = start.nextAvailableContractTreeLeafIndex`
- Construct the leaf that will be inserted into the `contractTree` (consult the diagram):
  - `leafValue = hash(newContractAddress, portalContractAddress, contractDeploymentCall.publicInputs.vkRoot)`
- Insert the vk tree into the `contractTree` using: `leafIndex = newContractLeafIndex, leafValue, siblingPath = contractDeploymentCall.newContractSiblingPath`. Get the `newContractTreeRoot` in return
- Write new 'meta' information about this contract into the first storage slots of this contract in the `publicDataTree`:
    - Store a pointer to the `newContractLeafIndex` which can be queried using the `newContractAddress`:
        - Let `publicDataTreeLeafIndex := hash(newContractAddress, 0)` (the 0th storage slot of each contract is reserved especially for this leaf index pointer).
        - Insert the pointer into the `publicDataTree` using: `leafIndex = publicDataTreeLeafIndex, leafValue = newContractLeafIndex, siblingPath = contractDeploymentCall.newContractStorageSlotSiblingPath`. Get a `newPublicDataTreeRoot` in return.
        - Note: we don't need to check whether we're overwriting an existing state in the tree here. The `newContractAddressNullifier`, together with the infeasibility of inter-contract state collisions, is enough to prevent a collision.


Verify the private constructor's kernel snark:
* Verify the (`privateConstructorKernel.proof`, `privateConstructorKernel.publicInputs`) using the `privateConstructorKernel.vk`
    - Validate that the `privateConstructorKernel.vk` is a valid _private_ kernel VK with a membership check:
        * Calculate `privateConstructorKernelVKHash := hash(privateConstructorKernel.vk)`;
        * Compute `root` using the `privateConstructorKernelVKHash`, `privateConstructorKernel.vkPath` and `privateConstructorKernel.vkIndex`.
        * Validate that `root == constants.privateKernelVKTreeRoot`.
    * Validate consistency of values which must remain the same between the private constructor kernel's inputs and this kernel's inputs:
        * ensure this kernel circuit's 'constant' public inputs match the `privateConstructorKernel`'s public inputs.
        * E.g. old tree roots.
    - We'll push public inputs from this constructor onto stacks shortly.
    - Validate that the expected constructor function was indeed actually executed as the first call of the private kernel snark, with the expected inputs, by comparing call data:
        - We need to reconcile the exposed callStackItemHash of the actually-executed constructor of the private kernel snark, with what we'd expect:
            - Let `actualCallStackItemHash = privateConstructorKernel.publicInputs.end.optionallyRevealedData[0].callStackItemHash;` (the `0`-th value will be the constructor's callStack item, otherwise something has gone wrong).
            - Let's try to reconcile that `actualCallStackItemHash` with the hash we'd expect: `expectedCallStackItemHash`:
                - Let `expectedFunctionSignature := concat(newContractAddress, 0, false, true, false)`. Note: the `0` in the 2nd parameter might suggest we want to call the function whose vkIndex is at position `0` - but the `isConstructor = true` tells the kernel circuit that we're calling a constructor; not looking up a deployed function. Note: this line is justification for `isConstructor` being part of the functionSignature. If it were alternatively part of the `privateConstructorPublicInputsHash`, we'd have to do _much_ more hashing to extract its value here.
                - Let `expectedPublicInputsHash := contractDeploymentCall.publicInputs.privateConstructorPublicInputsHash`
                - Let `expectedCallContext := { msgSender: contractDeploymentCall.callContext.msgSender, storageContractAddress: newContractAddress }`. Notice: The calling of a constructor is as though called by the person/contract who called to deploy the contract in the first place.
                - Let `expectedIsDelegateCall = false`, `expectedIsStaticCall = false`.
                - Hash it all together:
                    - `expectedCallStackItemHash = hash(expectedFunctionSignature, expectedPublicInputsHash, expectedCallContext, false, false)`
                - Assert `actualCallStackItemHash == expectedCallStackItemHash`.
        - We need to reconcile the exposed `privateConstructorVKHash` of the actually-executed constructor of the private kernel snark, with what we'd expect:
            - Let `actualPrivateConstructorVKHash = privateConstructorKernel.publicInputs.end.optionallyRevealedData[0].vkHash` (the `0`-th value will be the constructor's callStack item, otherwise something has gone wrong).
            - Let `expectedPrivateConstructorVKHash = contractDeploymentCall.publicInputs.privateConstructorVKHash`
            - Assert `expectedPrivateConstructorVKHash == actualPrivateConstructorVKHash` 
        

Verify the public constructor's kernel snark:
* Verify the (`publicConstructorKernel.proof`, `publicConstructorKernel.publicInputs`) using the `publicConstructorKernel.vk`
    - Validate that the `publicConstructorKernel.vk` is a valid _public_ kernel VK with a membership check:
        * Calculate `publicConstructorKernelVKHash := hash(publicConstructorKernel.vk)`;
        * Compute `root` using the `publicConstructorKernelVKHash`, `publicConstructorKernel.vkPath` and `publicConstructorKernel.vkIndex`.
        * Validate that `root == constants.publicKernelVKTreeRoot`.
    * Validate consistency of values which must remain the same between the public constructor kernel's inputs and this kernel's inputs:
        * ensure this kernel circuit's 'constant' public inputs match the `publicConstructorKernel`'s public inputs.
        * E.g. old tree roots.
    - We'll push public inputs from this constructor onto stacks shortly.
    - Validate that the expected constructor function was indeed executed as the first call of the public kernel snark, with the expected inputs, by comparing call data:
        - We need to reconcile the exposed callStackItemHash of the actually-executed constructor of the public kernel snark, with what we'd expect:
            - Let `actualCallStackItemHash = publicConstructorKernel.publicInputs.end.optionallyRevealedData[0].callStackItemHash;` (the `0`-th value will be the constructor's callStack item, otherwise something has gone wrong).
            - Let's try to reconcile that `actualCallStackItemHash` with the hash we'd expect: `expectedCallStackItemHash`:
                - Let `expectedFunctionSignature := concat(newContractAddress, 0, false, true, false)`. Note: the `0` in the 2nd parameter might suggest we want to call the function whose vkIndex is at position `0` - but the `isConstructor = true` tells the kernel circuit that we're calling a constructor; not looking up a deployed function. Note: this line is justification for `isConstructor` being part of the functionSignature. If it were alternatively part of the `publicConstructorPublicInputsHash`, we'd have to do _much_ more hashing to extract its value here.
                - Let `expectedPublicInputsHash := contractDeploymentCall.publicInputs.publicConstructorPublicInputsHash`
                - Let `expectedCallContext := { msgSender: contractDeploymentCall.callContext.msgSender, storageContractAddress: newContractAddress }`. Notice: The calling of a constructor is as though called by the person/contract who called to deploy the contract in the first place.
                - Let `expectedIsDelegateCall = false`, `expectedIsStaticCall = false`.
                - Hash it all together:
                    - `expectedCallStackItemHash = hash(expectedFunctionSignature, expectedPublicInputsHash, expectedCallContext, false, false)`
                - Assert `actualCallStackItemHash == expectedCallStackItemHash`.
        - We need to reconcile the exposed `publicConstructorVKHash` of the actually-executed constructor of the public kernel snark, with what we'd expect:
            - Let `actualPublicConstructorVKHash = publicConstructorKernel.publicInputs.end.optionallyRevealedData[0].vkHash` (the `0`-th value will be the constructor's callStack item, otherwise something has gone wrong).
            - Let `expectedPublicConstructorVKHash = contractDeploymentCall.publicInputs.publicConstructorVKHash`
            - Assert `expectedPublicConstructorVKHash == actualPublicConstructorVKHash` 

Update the `end` values:
- Extract the following from the `privateConstructorKernel.publicInputs`, and push the data onto the relevant stack of _this_ circuit's public inputs:
    - `l1CallStack`, `optionallyRevealedData`, `outputCommitments`, `inputNullifiers`, `proverRecords`
    - Merge the stacks. (It'll probably require some expensive O(n^2) logic to merge these stacks).
    - Ensure the `initial` tree roots exposed by the `privateConstructorKernel` match the latest tree roots that this circuit was aware of. Then update this circuit's understanding of the latest tree roots to be the `end` tree roots exposed by the `privateConstructorKernel`.
- Extract the following from the `publicConstructorKernel.publicInputs`, and push the data onto the relevant stack of _this_ circuit's public inputs:
    - `l1CallStack`, `callbackStack`, `optionallyRevealedData`, `outputCommitments`, `inputNullifiers`
    - Merge the stacks. (It'll probably require some expensive O(n^2) logic to merge these stacks).
    - Ensure the `initial` tree roots exposed by the `publicConstructorKernel` match the latest tree roots that this circuit was aware of. Then update this circuit's understanding of the latest tree roots to be the `end` tree roots exposed by the `publicConstructorKernel`.
- Push a log of this contract deployment to the `deployedContractData`.
    - Let `log := keccak256(contractDeploymentCall.publicInputs);` NOTE: this is expensive, since it's keccak, but I think this needs to be validated on-chain as well. We've already computed a hash of this data earlier in this circuit, but that was a cheaper pedersen hash (since the app needs to be able to efficiently produce that hash in order to create a contractDeploymentCallStackHash cheaply).
    - Push `log` onto `deployedContractData`.
* Calculate new data to push to `proverRecords`:
    - Extract `previousKernelProverAddress := previousKernel.publicInputs.proverAddress`
    - Extract `previousKernelNumberOfGates := previousKernel.vk.n`
    - Extract `privateConstructorKernelProverAddress := privateConstructorKernel.publicInputs.proverAddress`
    - Extract `privateConstructorKernelNumberOfGates := privateConstructorKernel.vk.n`
    - Extract `publicConstructorKernelProverAddress := publicConstructorKernel.publicInputs.proverAddress`
    - Extract `publicConstructorKernelNumberOfGates := publicConstructorKernel.vk.n`
    - Below is expensive looping and dynamic pushing within a circuit (O(n^2) stuff). Consider alternatives.
    - Find in the `proverRecords` array an entry matching `previousKernelProverAddress`.
        - If found: increment their `totalNumberOfGates += previousKernelNumberOfGates`.
        - If not found: push a new record `[previousKernelProverAddress, previousKernelNumberOfGates]` onto `proverRecords`.
    - Find in the `proverRecords` array an entry matching `privateConstructorKernelProverAddress`.
        - If found: increment their `totalNumberOfGates += privateConstructorKernelNumberOfGates`.
        - If not found: push a new record `[privateConstructorKernelProverAddress, privateConstructorKernelNumberOfGates]` onto `proverRecords`.
    - Find in the `proverRecords` array an entry matching `publicConstructorKernelProverAddress`.
        - If found: increment their `totalNumberOfGates += publicConstructorKernelNumberOfGates`.
        - If not found: push a new record `[publicConstructorKernelProverAddress, publicConstructorKernelNumberOfGates]` onto `proverRecords`.
* Extract the contract deployment call's `contractDeploymentCallStack` and `l1CallStack`.
    - Validate the call contexts of these calls:
        - For each `newCallStackItem` in `contractDeploymentCallStack`:
            - If `newCallStackItem.isDelegateCall == true`:
                - Assert `newCallStackItem.callContext == contractDeploymentCall.callContext`
            - Else:
                - Assert `newCallStackItem.callContext.msgSender == contractDeploymentCall.functionSignature.contractAddress`
                - Assert `newCallStackItem.callContext.storageContractAddress == newCallStackItem.functionSignature.contractAddress`
    - Push the contents of these call stacks onto the kernel circuit's `end.contractDeploymentCallStack` and `end.l1CallStack`.
        - It would be nice if these 'pushes' could result in tightly-packed stacks.

* If `end.contractDeploymentCallStack.length == 0`, set `end.contractDeploymentCallCount = 0`, else `end.contractDeploymentCallCount = start.contractDeploymentCallCount + 1`

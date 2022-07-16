# Public Input ABIs

The following describes how many public inputs each public/private circuit will have and how they will be interpreted.


## Private Circuit ABI

All private contract circuits have a fixed number of public inputs. Most of these public inputs will eventually be swallowed by the [private kernel circuit](../kernel-circuits/private-kernel.md). Under some circumstances, some public inputs will be optionally revealed to the 'public world, depending on the various booleans in this table.

> Note: some of these inputs might be packed into 1 field for efficiencies in the implementation, but for ease of understanding they're kept separate here.


| Data type | Description |
| -------- | -------- |
| *`customPublicInputs` | Up to 32 circuit-specific inputs passed into this circuit. All of these inputs will be 'swallowed' by the private kernel snark. Hence any private args can safely be put here, which enables private circuits to call other private circuits, and pass private data to them. To reveal data to the 'public world', either call a public function or emit an event. |
| *`customPublicOutputs` | Up to 32 circuit-specific values _returned by_ circuit. All of these inputs will be 'swallowed' by the private kernel snark. Hence any private outputs can safely be put here, which enables private circuits to call other private circuits, and pass private data to them. |
| `emittedPublicInputs` | Public inputs which will be revealed to the 'public world' (L1 or L2). E.g. allowing certain inputs to be more-easily extracted by the RollupProcessor contract (rather than having to unpack this vast ABI object of inputs, which would cost too much hash-wise). An example of its usefulness is passing a value from L1 to L2 when doing an L1->L2 tx (see 'deposit' example in the other doc). TODO: consider whether this could be a single value, for simplicity. It could be a hash of data if more values need to be exposed. If so, we'll rename to `emittedPublicInputsHash`. |
| *`executedCallback: {` | Populated if this circuit is a callback, so that the purported L1 Result to which this circuit is responding can be validated against the l1ResultsTree. |
| `- l1ResultHash`, | If this function is a callback function, following some L1 call, then it needs to expose the `l1ResultHash` to ensure the callback is actually using data emitted by L1. The preimage of the `l1ResultHash` will need to be passed in as private inputs to the circuit. Note: the `l1ResultHash` MUST be computed with a sha256 hash, because that's what the RollupProcessor will have used. This is unfortunate, as it forces apps to adhere to a specific calculation within their circuits. DISAGREE NOW - we don't need to calcualte this within the app circuit: we can just hash the custom_inputs and compare that hash against the L1ResultHash. |
| `- l1ResultsTreeLeafIndex`, | Communicates to the rollup provider the correct leaf to prove membership against in the l1ResultsTree. Only if we don't want to hide the callback will the rollup provider perform the membership check (within the Base Rollup Circuit). Otherwise it'll be performed in the Private Kernel Circuit. |
| `}`|
| `outputCommitments` | output notes to be added into the `privateDataTree` (up to 16) | 
| `inputNullifiers` | input nullifiers to be added into the `nullifierTree` (up to 16) |
| `privateCallStack` | additional private calls created by this transaction (up to 16) |
| `publicCallStack` | additional public calls created by this transaction (up to 16) |
| `contractDeploymentCallStack` | additional calls which  _deploy contracts_, created by this transaction (up to 1?) |
| `partialL1CallStack` | additional calls to L1 created by this transaction (up to 16). "Partial" because the kernel circuit will add the correct portal contract address to 'complete' the call into a proper L1 CallStack Item. See [here](../contracts/l1-calls.md). |
| <pre>callbackStack = [{<br/>&nbsp;&nbsp;callbackPublicKey, <br/>&nbsp;&nbsp;successCallbackCallHash, <br/>&nbsp;&nbsp;failureCallbackCallHash, <br/>&nbsp;&nbsp;successResultArgMapAcc, <br/>},...]</pre> | An element in the array for each new L1 call on the `l1CallStack`. See [more](../contracts/l1-calls.md). We need to expose all of this data so that the kernel snark ensures the callback is a function of the same contract which made the call. |
| *`oldPrivateDataTreeRoot` | used to query the `privateDataTree` |
| Booleans: | |
| *`bool isFeePayment` | Notifies the kernel circuit that the following public inputs of this private circuit MUST be revealed to the public kernel circuit:<br/><ul><li>`functionSignature` - so that the rollup provider can see how they're to be paid </li><li>`emittedPublicInputs` - so that the rollup provider can validate the amount they'll be paid (they'll need to be provided with the underlying public inputs separately, to validate the hash).</li></ul> |
| *`bool payFeeFromL1`| Provides a way of paying for private L2 function execution from L1 (the RollupProcessor.sol could provide the interface for ETH or ERC20... but we could generalise this by somehow pointing to a particular 'other' payment L1 contract state?). "Submit on-chain the callHash and state a fee (in L1 currency) for the L2 tx, and then when the L2 tx is executed, the rollup provider may redeem the previously-published fee". <br/><br/>  Notifies the kernel circuit that the following public inputs of this private circuit MUST be revealed to the public kernel circuit:<br/><ul><li>`callHash` - no other data is needed (in order to allow the user to keep the function they've actually executed private).</li></ul> |
| *`bool payFeeFromPublicL2`| Provides a way of paying for private L2 function execution from a public L2 circuit. This input being `true` will cause the rollup provider to look for an `isFeePayment` tx in the _public_ callStack. |
| *`bool calledFromL1` | If the L1 contract wants to call a specific L2 circuit, then the function signature needs to be revealed on-chain so that it can be checked that the correct, intended function was executed. <br/><br/> Notifies the private kernel circuit that an L1 contract wants to call this specific private L2 circuit, and so the following MUST be revealed to the public kernel circuit: <br/><ul><li>`functionSignature` - needed so the L1 contract can confirm the intended function was executed on L2. Although a callHash contains the functionSignature, the L1 contract wouldn't (cheaply) be able to unpack the callHash. So we expose the function signature as well to keep costs down. Although this leaks the function which was called, there's no way around that; this is an L1 -> L2 call after all.</li><li>`callHash` - used as a 'lookup' key. The RollupProcessor will store this `callHash`, and await an L2 tx with this `callHash` before triggering a callback to the L1 contract which made this L1 -> L2 call in the first place.</li><li>`emittedPublicInputs` - needed so that a set of values can be emitted by an L2 function and exposed to L1. It would be too expensive to unpack the callHash and extract all of the custom public inputs of a circuit. This is much cheaper, and is very similar to how the EVM exposes only a few values (via event emissions) to JavaScript (for example).</li></ul> <br/><br/> We need this convoluted process, because the RollupProcessor has to be _sure_ an L1 fee has been set-aside for them, before they add any corresponding L2 states. |
| Global vars: | |
| *`minTimestamp` | a timestamp value that must be exceeded when the block containing this txn is mined |

Apart from the public input API, the structure of a private contract circuit is  undefined by the Aztec 3 architecture.

> In practice we will be using Noir to compile programs into circuits that conform to the above ABI. In theory we could, in the future, write Solidity or ewasm transpilers.

:heavy_exclamation_mark: I think a circuit should only be able to make an L1 call to _its_ portal contract. The portal contract can then make calls to other L1 contracts, using its address as msg.sender. We don't want to circuits to be able to make calls from the RollupProcessor.sol, as it should never 'own state' in any Eth contracts on behalf of a specific aztec app. States should be siloed by app - that's the point of the portal contract.


> **Aside:** A loose argument against the need for public -> private calls: 
>- Suppose we support public -> private calls.
>- A user will need to generate the private call's proof with some custom public inputs. This proof will be called by a public circuit, and so the private function being executed can't be hidden.
>- The public circuit calls the private circuit.
>  - I.e. the public circuit will push a private call onto its private callStack (which includes the publicInputsHash of the public inputs that will be passed to the private circuit).
>- Clearly, the inputs that are passed to the private circuits must be known in advance (since the proof has already been generated), so cannot depend on any mutable public states.
>- In which case the private circuit could have come _first_ in the chain of calls, and so public -> private functionality isn't needed.
>
>Now...
>- If a public circuit wanted to pass dynamic inputs (inputs which depend on mutable public state) to a private circuit, that's not really possible (since the rollup provider would produce the public proof and then have to wait for the user to inject secrets into a private proof and generate it).
>- What _can_ be done is a defi-bridge-like chain of events: a private circuit generates a partial commitment, and then the public circuit completes that partial commitment with some public state and adds it to the privateDataTree. But that doesn't require explicit public -> private calls.


> **Aside:** we don't think we need chained transactions for Aztec 3. Recall: chained transactions give the ability to nullify a newly-created commitment _within_ the same rollup; before it's been added to the dataTree. This isn't needed. For a user to chain state to themselves, they can just create a callstack. The notion of chaining _between_ users is really complicated, and possibly not useful in practice. In particular, chaining doesn't play well with the idea of binary rollup trees, since the number of 'chaining' comparisons would blow up exponentially with the level of the rollup tree (meaning we'd need to build differently-sized rollup circuits per level). Recall: linked transactions give the ability to 'await' the inclusion of 'other' transactions in the rollup, before processing 'this' transaction. Unsure whether that's needed.






## Public Circuit ABI

The public contract ABI is similar to the private one.

Whilst the _user_ provides all the values specified by this Public Circuit ABI, it's the _rollup provider_ who actually generates public contract proofs. This is because the rollup provider is the only party with perfect knowledge of the current Merkle roots, meaning only they can access/update the public data tree, as well as know the exact block number etc.


| Data Type | Description |
| -------- | -------- |
| `customPublicInputs` | Up to 32 circuit-specific inputs passed into this circuit. All of these inputs will be 'swallowed' by the public kernel circuit. |
| `customOutputs` | Up to 32 variables 'returned' by calling this function. All of these inputs will be 'swallowed' by the public kernel circuit. Note, unlike the private circuit ABI (which doesn't have a `customOutputs` field like this), it's possible for public circuit execution to output values which were derived from the current state of the public data tree - such values cannot have been known by the caller of the function, and hence can truly be considered 'outputs'. Note, since the caller won't necessarily known what these outputs will be when they make the call, this entire `customOutputs` field is omitted from the `callStackItemHash` when the user makes the call. |
| `emittedPublicInputs` | Public inputs which may be revealed to L1. E.g. allowing certain inputs to be more-easily extracted by the RollupProcessor contract (rather than having to unpack this vast ABI object of inputs, which would cost too much hash-wise). An example of its usefulness is passing a value from L1 to L2 when doing an L1->L2 tx (see 'deposit' example in the other doc). |
| `emittedOutputs` | As above, except these values won't necessarily be known when the call is made; only when the rollup provider is generating the witness for this function's execution. Note, since the caller won't necessarily known what these outputs will be when they make the call, this entire `customOutputs` field is omitted from the `callStackItemHash` when the user makes the call. |
| `executedCallback: {` | Populated if this circuit is a callback, so that the purported L1 Result to which this circuit is responding can be validated against the l1ResultsTree. |
| `- l1ResultHash`, | If this function is a callback function, following some L1 call, then it needs to expose the `l1ResultHash` to ensure the callback is actually using data emitted by L1. The preimage of the `l1ResultHash` will need to be passed in as private inputs to the circuit. Note: the `l1ResultHash` MUST be computed with a sha256 hash, because that's what the RollupProcessor will have used. This is unfortunate, as it forces apps to adhere to a specific calculation within their circuits. |
| `- l1ResultsTreeLeafIndex`, | Communicates to the rollup provider the correct leaf to prove membership against in the l1ResultsTree. Only if we don't want to hide the callback will the rollup provider perform the membership check (within the Base Rollup Circuit). Otherwise it'll be performed in the Private Kernel Circuit. |
| `}`|
| 256 `stateTransitions` | Describes up to 256 `publicDataTree` read+write operations. <br> A 'state transition' is expressed as: <br> `[storageSlot, old_value, new_value]`. <br/>:heavy_exclamation_mark: For state transitions, the caller might not know the `old_value` nor the `new_value` when they make the call (i.e. when they add the call to their callStack), since the value will depend on the current state of the publicDataTree. (Imagine if the storage slot represented total liquidity in some pool which changes frequently. Then the old and new values are only known at the time the rollup processor actually organises the ordering of txs in their block). Conclusion: the `publicInputsHash` which is included in a public call's `callStackItemHash` cannot include the `old_value` nor `new_value` inputs. </br/><br/> Therefore, the `stateTransitions` that are populated by a user and hashed when generating the `callStackItemHash` will have `0`-valued placeholders for `old_value` and `new_value`. <br/><br/> Pure reads can be emulated by setting a new write value to equal the read value. However, for this tree, we could halve the amount of hashing for pure reads by having a separate `reads` set of inputs. A read can be done without a write for this tree (see next field). |
| `stateReads` | Pure reads. Each 'read' request is simply the `[storageSlot, current_value]`. As explained above, the `current_value` won't necessarily be known by the user when they generate the call, so `current_value` is replaced with `0` when the `callStackItemHash` is generated for calls to a public function. |
| `publicCallStack` | additional public calls to be made by this transaction (up to 16) |
| `contractDeploymentCallStack` | additional calls which  _deploy contracts_, created by this transaction (up to 1?) |
| `partialL1CallStack` | Additional L1 calls to be made by this transaction (up to 16). "Partial" because the kernel circuit will add the correct portal contract address to 'complete' the call into a proper L1 CallStack Item. See [here](../contracts/l1-calls.md). |
| <pre>callbackStack = [{<br/>&nbsp;&nbsp;callbackPublicKey, <br/>&nbsp;&nbsp;successCallbackCallHash, <br/>&nbsp;&nbsp;failureCallbackCallHash, <br/>&nbsp;&nbsp;successResultArgMapAcc, <br/>},...]</pre> | An element in the array for each new L1 call on the `partialL1CallStack`. See [more](../contracts/l1-calls.md). We need to expose all of this data so that the kernel snark ensures the callback is a function of the same contract which made the call. |
| `oldPrivateDataTreeRoot` | used to query the `privateDataTree` |
| `proverAddress` | Who is going to be generating the proof for this circuit (any value can be injected here by the prover). :question: IS THIS ALLOWED HERE? It can be allowed here as long as Noir abstracts its existence away from the circuit developer. |
| Booleans: | |
| `bool isFeePayment` | Notifies the rollup provider that this function pays them their fee. The rollup provider will need a way to interpret the many ways it can be paid. :question: Perhaps a fee payment ABI is required for the structure of the `emittedPublicInputs`, for example? |
| `bool payFeeFromL1`| Provides a way of paying for L2 function execution from L1 (the RollupProcessor.sol could provide the interface for ETH or ERC20... but we could generalise this by somehow pointing to a particular 'other' payment L1 contract state?). "Submit on-chain the callHash and state a fee (in L1 currency) for the L2 tx, and then when the L2 tx is executed, the rollup provider may redeem the previously-published fee". <br/><br/>  Notifies the kernel circuit that the following public inputs of this public circuit MUST also be revealed by the public kernel circuit:<br/><ul><li>`callHash`</li></ul> |
| `bool calledFromL1` | If the L1 contract wants to call a specific L2 circuit, then the function signature needs to be revealed on-chain so that it can be checked that the correct, intended function was executed. <br/><br/> Notifies the public kernel circuit that an L1 contract wants to call this specific public L2 circuit, and so the following MUST be revealed by the public kernel circuit: <ul><li>`functionSignature`</li><li>`callHash`</li><li>`emittedPublicInputs` (possibly)</li></ul><br/><br/> We need this convoluted process, because the RollupProcessor has to be _sure_ an L1 fee has been set-aside for them, before they add any corresponding L2 states. |
| Global vars: | |
| `block.timestamp` | aztec block timestamp |
| `block.number` | aztec block number | 
| `prevBlock.ethTimestamp` | Ethereum timestamp of block that contained previous Aztec 3 block |
| `prevBlock.ethNumber` | block number of Ethereum block that contained previous Aztec 3 block |

### publicInputsHash

As explained briefly in the table above, for calls to a public circuit, certain inputs are _not_ known when the user makes the call (since their values might depend on the state of the public data tree at the time of proof generation (rather than at the time of making the call)). Therefore, the `publicInputsHash` for public calls is defined as the hash of the above ABI data, _except_ for the following modifications to the preimage:
- `customOutputs` fields are completely omitted from the preimage.
- `emittedOutputs` fields are completely omitted from the preimage.
- Each state transition is truncated to be `storageSlot` (instead of `[storageSlot, old_value, new_value]`).
- Each state read is truncated to be `storageSlot` (instead of `[storageSlot, current_value]`).









## Contract Deployment ABI

Recall:

Contracts can be deployed to Aztec's Layer 2. A contract comprises:
- a set of functions, each encapsulated by a `(vk, proving_key, circuit)` tuple, with the hash of the `vk` being a succinct and unique representation of each function.
- state variables, split across these trees:
    - `publicDataTree`
    - `privateDataTree`

A contract deployment ABI, therefore, needs to communicate this information to the rollup provider.

This is the ABI for the public inputs that must be specified when making a call to deploy a new contract. These public inputs will then be hashed into a `publicInputsHash` which will form part of the [`callStackItemHash`](../contracts/transactions.md#callstackitemhash) which will get added to the `contractDeploymentCallStack`.

**Important note**: Unlike for the above Public Input ABIs for private circuits and public circuits, we don't actually have a 'contract deployment' _circuit_. These public inputs are simply defined so that a callStackItem can be created. These public inputs (and others) are then fed directly into a [Contract Deployment _Kernel_ Circuit](../kernel-circuits/contract-deployment-kernel.md).


TODO: we might be able to remove some of these public inputs and provide them privately instead; but such optimisations can happen _much_ later in this project.

| Data Type | Description |
| -------- | -------- |
| `privateConstructorPublicInputsHash` | So that vested parties (with access to the underlying public inputs) can confirm the constructor was executed with the correct params. Note: a user (or contract) cannot provide a conventional callStackItem to make a call to a private constructor, because the constructor's function signature isn't known until the contract address is known, but the contract address needs to contain the constructor arguments (i.e. the privateConstructorPublicInputsHash), so we'd get a cyclic dependency. |
| `publicConstructorPublicInputsHash` | As above, but public. |
| `privateConstructorVKHash` | The hash of the vk that was used to verify the private constructor, so that vested parties can confirm the correct function was run as a constructor (the vk itself may OPTIONALLY be submitted on-chain as calldata (for data availability), but some users might want to keep that private). <br/> (Note, we can't use function signatures here, because the contract hasn't yet been deployed, so no contract address exists!) |
| `publicConstructorVKHash` | As above, but public. |
| `contractAddress` | The proposed contract address that this new contract will be deployed to. |
| `salt` | The salt used in the preimage of the contractAddress derivation - to have some control over the contract address |
| `vkRoot` | The root of the mini merkle tree of VKs. Note: the underlying data (the circuit's compressed ACIR representation) can optionally be submitted on-chain as calldata (for data availability), but some users might want to keep that info private, or might not want to pay for that. |
| `circuitDataKeccakHash` | For example, a hash of the circuit's compressed ACIR representation. This value will be read on-chain by the RollupProcessor.sol. If it's `0`, it's assumed the user didn't want data to be submitted (to keep their contract logic private). If it's nonzero, the rollup provider MUST submit the hash's preimage as calldata. To ensure the rollup provider does actually submit, the RollupProcessor.sol will try to reconcile the hash, and reject if it doesn't reconcile. Note, there's no explicit checks that can be done (within a snark or on-chain, due to expense) to ensure this `circuitDataKeccakHash` actually corresponds to the `vkRoot`. Clients (off-chain) must decompress the submitted circuit data, recreate the verification keys, then recreate the `vkRoot`, to be convinced the deployed contract actually represents the logic they expect. If circuit data isn't submitted here (to keep the logic of the contract private), then the developer must find another way (off-chain) to circulate the ACIR representations of the circuits to their users. |
| `portalContractAddress` | Gets calculated within the circuit and stored in the `contractTree` (within the same leaf as the contract's vkTree). We expose it here, because the RollupProcessor will need redo this calculation to ensure it's been done correctly. |






 ## Custom inputs ABI

Circuits have up to 32 custom 'inputs' (and 'outputs' for public circuits). Each Noir data type that is set to be a public input is represented by a single field element (we don't need to do this for private inputs). If the data type is >1 field element, the data is Pedersen hashed. We apply this heuristic recursively for structs and nested data. e.g. for

```
struct foo {
    field a;
    bytes b[256];
};
```

`b = pedersen(b.values), foo = pedersen(b, a)`

:question: Can we instead go for `foo = pedersen(a, b)`? Hmmm, actually, maybe it is easier to mirror the Solidity ABI, to avoid mistakes?

The full, uncompressed values will be attached to the transaction as 'auxiliary data', used by the proof creator to unpack the hashed values. This data will be swallowed before the transaction gets mined into a block. Literally nothing is broadcast on-chain except for non-validium state changes.
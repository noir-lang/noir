# Public Input ABIs

The following describes how many public inputs each public/private circuit will have and how they will be interpreted.


## `CallContext`

Much of the CallContext comes from the [`TxContext`](../contracts/transactions.md#txcontext) over which the user signs.

| Value | Description |
| --- | --- |
| `txOrigin`: `aztecAddress` | The user who signed over this tx. We'll need this when making nested function calls, because private keys for nested circuits might need to be grabbed from the Private Client's DB based on this address -- not from the calling contract's address (since a contract cannot have private keys). |
| `msgSender`: `aztecAddress` | - If doing a `call` or `staticCall`: Either the user address or the address of the contract that created the call. (Can be set to `0` for private -> public calls) <br/> - If doing a `delegateCall`: the address of the calling contract's own `callContext.msgSender` (since delegate calls can be chained). |
| `storageContractAddress`: [`ContractAddress`](../contracts/transactions.md#contractaddress) | - If doing a `call` or `staticCall`: the address of the contract being called. <br/> - If doing a `delegateCall`: the address of the calling contract's own `callContext.storageContractAddress` (since delegate calls can be chained).
| `isDelegateCall` : `Bool` | Used by the kernel snark to validate that the `callContext` of newly-pushed `callStackItems` is consistent with the contract making the call. |
| `isStaticCall` : `Bool` | Informs the kernel snark that it MUST fail if the function being called modifies state. <br/><br/> A state modification includes: creating a new contract; emitting events; writing to trees; making a 'call' to another contract. :question: Not sure why 'delegatecall' is not included as a potentially state-modifying tx in ethereum specs? <br/><br/> Note: static calls to private circuits might not make sense. 'Reads' from the privateDataTree require a write of equal value, but the kernel snark cannot 'see' what has been written (it's a private tx), and so cannot validate that a state change didn't take place. So there would be no output commitments or nullifiers for a private static call. But a private call's only use is (I think :question:) to read/modify private state. So I'm thinking a staticCall to a private circuit doesn't make sense. |
| `reference_block_num`: Field | The rollup number which should be used if referring to any historic tree values. Useful if the proof needs to use a particular tree state snapshot of a particular historic rollup. |

## `StateTransition`

Describes a single `publicDataTree` read+write operation.

A 'state transition' is expressed as:
- `[storageSlot, old_value, new_value]`.

:heavy_exclamation_mark: For state transitions, the caller might not know the `old_value` nor the `new_value` when they make the call (i.e. when they add the call to their callStack), since the value will depend on the current state of the publicDataTree. (Imagine if the storage slot represented total liquidity in some pool which changes frequently. Then the old and new values are only known at the time the rollup processor actually organises the ordering of txs in their block).

Conclusion: the `publicInputs` data which is included in a [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem) cannot include the `old_value` nor `new_value` inputs. Therefore, the `stateTransitions` that are populated by a user when generating the `PublicCallStackItemHash` will have `0`-valued placeholders for `old_value` and `new_value`. Pure reads can be emulated by setting a new write value to equal the read value. However, for this tree, we could halve the amount of hashing for pure reads by having a separate `reads` set of inputs. A read can be done without a write for this tree (see next field).

| Value | Description |
| --- | --- |
| `storageSlot`: `Field` | The slot that the circuit wants to modify. |
| `oldValue`: `Field` | Old value is checked by the kernel circuit to be correct. |
| `newValue`: `Field` | New value inserted by the kernel circuit. |

## `StateRead`

For efficiency (vs a StateTransition), we can read public state without writing. As explained above, the `current_value` won't necessarily be known by the user when they generate the call, so `current_value` is replaced with `0` when the `PublicCallStackItem` is generated for calls to a public function.

| Value | Description |
| --- | --- |
| `storageSlot`: `Field` | The slot that the circuit wants to modify. |
| `currentValue`: `Field` | Current value is checked by the kernel circuit to be correct. |

# Private Circuit ABI

All private contract circuits have a fixed number of public inputs. Most of these public inputs will eventually be swallowed by the [private kernel circuit](../kernel-circuits/private-kernel.md). Under some circumstances, some public inputs will be optionally revealed to the 'public world, depending on the various booleans in this table.

> Note: some of these inputs might be packed into 1 field for efficiencies in the implementation, but for ease of understanding they're kept separate here.

## `PrivateCircuitPublicInputs`

| Data type | Description |
| -------- | -------- |
| `callContext`: [`CallContext`](#callcontext) | Information about what this function used as its call context, so that the kernel circuit may validate that the correct context was indeed used. |
| `args`: `Array[Field]` | Arguments passed into this circuit. All of these inputs will be 'swallowed' by the private kernel snark. Hence any private args can safely be put here, which enables private circuits to call other private circuits, and pass private data to them. To reveal data to the 'public world', either call a public function or emit an event. |
| `returnValues`: `Array[Field]` | Values _returned by_ circuit. All of these outputs will be 'swallowed' by the private kernel snark. Hence any private outputs can safely be put here, which enables private circuits to call other private circuits, and receive return data from them. |
| `emittedEvents`: `Array[Field]` | Public inputs which will be revealed to the 'public world' (L1 or L2). E.g. allowing certain inputs to be more-easily extracted by the RollupProcessor contract (rather than having to unpack this vast ABI object of inputs, which would cost too much hash-wise). An example of its usefulness is passing a value from L1 to L2 when doing an L1->L2 tx. |
| `newCommitments`: `Array[Field]` | new commitments to be added into the `privateDataTree` | 
| `newNullifiers`: `Array[Field]` | new nullifiers to be added into the `nullifierTree` |
| `privateCallStack`: `Array[Field]` | Additional private calls made by this function. The `Field` is a hash of a [`PrivateCallStackItem`](../contracts/transactions.md#privatecallstackitem). |
| `publicCallStack`: `Array[Field]` | Additional public calls created by this transaction. The `Field` is a hash of a [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem). |
| `contractDeploymentCallStack`: `Array[Field]` | Additional calls which  _deploy contracts_, created by this transaction. The `Field` is a hash of a [`ContractDeploymentCallStackItem`](../contracts/transactions.md#contractdeploymentcallstackitem). |
| `partialL1CallStack`: [`Array[PartialL1CallStackItem]`](../contracts/transactions.md#l1callstackitem) | Additional calls to L1 created by this transaction. "Partial" because the kernel circuit will add the correct portal contract address to 'complete' the call into a proper L1 CallStack Item. See [here](../contracts/l1-calls.md). We need to expose all of this data (rather than hashing it to a `Field`), because the kernel snark needs to pass this data to L1, and unlike the earlier callstacks, we'll never have another opportunity to pass this 'unpacked' data into any other kernel snark. |
| `oldPrivateDataTreeRoot`: `Field` | The root that has been used to query the `privateDataTree` |
| `oldNullifierTreeRoot`: `Field` | The root that has been used to query the `nullifierDataTree` |
| `oldConstPublicDataTreeRoot`: `Field` | The root that has been used to query the `constPublicDataTree` (if we choose to have one). |
| `oldContractTreeRoot`: `Field` | The root that has been used to query the `contractTree` |


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




# Public Circuit ABI

The public contract ABI is similar to the private one.

Whilst the _user_ provides all the values specified by this Public Circuit ABI, it's the _rollup provider_ who actually generates public contract proofs. This is because the rollup provider is the only party with perfect knowledge of the current Merkle roots, meaning only they can access/update the public data tree, as well as know the exact block number etc.

## `PublicCircuitPublicInputs`

**NOTE** This section is out of date. Public function execution might differ drastically from this old draft. Not worth even reading this section anymore.

| Data Type | Description |
| -------- | -------- |
| *`callContext`: [`CallContext`](#callcontext) | Information about what this function used as its call context, so that the kernel circuit may validate that the correct context was indeed used. |
| *`args` | Circuit-specific inputs passed into this circuit. |
| `returnValues` | Values 'returned' by calling this function. Note: it's possible for public circuit execution to output values which were derived from the current state of the public data tree - such values cannot have been known by the caller of the function. Note, since the caller won't necessarily known what these outputs will be when they make the call, this entire `returnValues` field is omitted from the [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem) when the user makes the call. |
| `emittedEvents` | Public inputs which may be revealed to L1. E.g. allowing certain inputs to be more-easily extracted by the RollupProcessor contract (rather than having to unpack this vast ABI object of inputs, which would cost too much hash-wise). An example of its usefulness is passing a value from L1 to L2 when doing an L1->L2 tx (see 'deposit' example in the other doc). Note: some of these values won't necessarily be known when the call is made; only when the rollup provider is generating the witness for this function's execution. Note, since the caller won't necessarily know what these outputs will be when they make the call, this entire `emittedEvents` field is omitted from the [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem) when the user makes the call. |
| `stateTransitions`: [`Array[StateTransition]`](#statetransition) |  Public state changes. |
| `stateReads`: [`Array[StateRead]`](#stateread) | Public state reads. |
| `publicCallStack`: `Array[Field]` | Additional public calls created by this transaction. The `Field` is a hash of a [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem). |
| `contractDeploymentCallStack`: `Array[Field]` | Additional calls which  _deploy contracts_, created by this transaction. The `Field` is a hash of a [`ContractDeploymentCallStackItem`](../contracts/transactions.md#contractdeploymentcallstackitem). |
| `partialL1CallStack`: `Array[Field]` | Additional calls to L1 created by this transaction. "Partial" because the kernel circuit will add the correct portal contract address to 'complete' the call into a proper L1 CallStack Item. See [here](../contracts/l1-calls.md). The `Field` is a hash of a [`PartialL1CallStackItem`](../contracts/transactions.md#partiall1callstackitem). |
| *`oldPrivateDataTreeRoot`: `Field` | The root that has been used to query the `privateDataTree` |
| *`oldNullifierTreeRoot`: `Field` | The root that has been used to query the `nullifierDataTree` |
| *`oldConstPublicDataTreeRoot`: `Field` | The root that has been used to query the `constPublicDataTree` (if we choose to have one). |
| *`oldContractTreeRoot`: `Field` | The root that has been used to query the `contractTree` |
| *`proverAddress`: `aztecAddress` | Who is going to be generating the proof for this circuit (any value can be injected here by the prover). |


**Note**: For calls to a public circuit, certain inputs are _not_ known when the user makes the call (since their values might depend on the state of the public data tree at the time of proof generation (rather than at the time of making the call)). Therefore, the `publicInputs` provided when making public calls (i.e. when creating a [`PublicCallStackItem`](../contracts/transactions.md#publiccallstackitem)) sets-to-0 some of the above data.
- `returnValues`
- `emittedEvents`
- `stateTransitions`' old and new values.
- `stateReads`' current values.
- All call stacks (as the calls could be made based on some dynamic state read of some address, for example)





# Contract Deployment ABI

Recall:

Contracts can be deployed to Aztec's Layer 2. A contract comprises:
- a set of functions, each encapsulated by a `(vk, proving_key, circuit)` tuple, with the hash of the `vk` being a succinct and unique representation of each function.
- state variables, split across these trees:
    - `publicDataTree` (maybe also a const public data tree)
    - `privateDataTree`

A contract deployment ABI, therefore, needs to communicate this information to the rollup provider.

This is the ABI for the public inputs that must be specified when making a call to deploy a new contract. These public inputs will form part of the [`ContractDeploymentCallStackItem`](../contracts/transactions.md#contractdeploymentcallstackitem) which will get added to the `contractDeploymentCallStack`.

**Important note**: Unlike for the above Public Input ABIs for private circuits and public circuits, we don't actually have a 'contract deployment' _circuit_. These public inputs are simply defined so that a callStackItem can be created. These public inputs (and others) are then fed directly into a [Contract Deployment _Kernel_ Circuit](../kernel-circuits/contract-deployment-kernel.md).


TODO: we might be able to remove some of these public inputs and provide them privately instead; but such optimisations can happen _much_ later in this project.

PROBLEM: we might wish to _prove_ that a set of ACIR++ opcodes compiles to a particular vkHash. This would be very complex to do inside a circuit, but might be essential.

## `ContractDeploymentPublicInputs`

| Data Type | Description |
| -------- | -------- |
| `privateConstructorPublicInputsHash` | So that vested parties (with access to the underlying public inputs) can confirm the constructor was executed with the correct params. Note: a user (or contract) cannot provide a conventional callStackItem to make a call to a private constructor, because the constructor's function data isn't known until the contract address is known, but the contract address needs to contain the constructor arguments (i.e. the privateConstructorPublicInputsHash), so we'd get a cyclic dependency. |
| `publicConstructorPublicInputsHash` | As above, but public. |
| `privateConstructorVKHash` | The hash of the vk that was used to verify the private constructor, so that vested parties can confirm the correct function was run as a constructor (the vk itself may OPTIONALLY be submitted on-chain as calldata (for data availability), but some users might want to keep that private). <br/> (Note, we can't use function datas here, because the contract hasn't yet been deployed, so no contract address exists!) |
| `publicConstructorVKHash` | As above, but public. |
| `contractAddress` | The proposed contract address that this new contract will be deployed to. |
| `salt` | The salt used in the preimage of the contractAddress derivation - to have some control over the contract address |
| `vkRoot` | The root of the mini merkle tree of VKs. Note: the underlying data (the circuit's compressed ACIR representation) can optionally be submitted on-chain as calldata (for data availability), but some users might want to keep that info private, or might not want to pay for that. |
| `circuitDataKeccakHash` | For example, a hash of the circuit's compressed ACIR representation. This value will be read on-chain by the RollupProcessor.sol. If it's `0`, it's assumed the user didn't want data to be submitted (to keep their contract logic private). If it's nonzero, the rollup provider MUST submit the hash's preimage as calldata. To ensure the rollup provider does actually submit, the RollupProcessor.sol will try to reconcile the hash, and reject if it doesn't reconcile. Note, there's no explicit checks that can be done (within a snark or on-chain, due to expense) to ensure this `circuitDataKeccakHash` actually corresponds to the `vkRoot`. Clients (off-chain) must decompress the submitted circuit data, recreate the verification keys, then recreate the `vkRoot`, to be convinced the deployed contract actually represents the logic they expect. If circuit data isn't submitted here (to keep the logic of the contract private), then the developer must find another way (off-chain) to circulate the ACIR representations of the circuits to their users. |
| `portalContractAddress` | Gets calculated within the circuit and stored in the `contractTree` (within the same leaf as the contract's vkTree). We expose it here, because the RollupProcessor will need redo this calculation to ensure it's been done correctly. |






 ## Custom inputs ABI

Circuits have custom 'args' and 'return values'. Each Noir data type that is set to be a public input is represented by a single field element (we don't need to do this for private inputs). If the data type is >1 field element, the data is Pedersen hashed. We apply this heuristic recursively for structs and nested data. e.g. for

```
struct foo {
    field a;
    bytes b[256];
};
```

`b = pedersen(b.values), foo = pedersen(b, a)`

:question: Can we instead go for `foo = pedersen(a, b)`? Hmmm, actually, maybe it is easier to mirror the Solidity ABI, to avoid mistakes?

The full, uncompressed values will be attached to the transaction as 'auxiliary data', used by the proof creator to unpack the hashed values. This data will be swallowed before the transaction gets mined into a block. Literally nothing is broadcast on-chain except for non-validium state changes.
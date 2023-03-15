# Transactions

A transaction always comes from a user account, and must be signed by that user's aztec private key. A transaction represents a call to some function of some contract, passing some parameters, and specifying some fee.


<!-- A transaction begins as a call to a single function[^1]. A function can be uniquely identified (within a contracts) by its [`functionData`](#function-data). A 'call' to a function is expressed through a [`callStackItem`](#call-stacks). During the execution of a transaction, the initial function might make calls to other functions, so callstacks are required. Call stack items are verified recursively. The final kernel snark that is produced by the recursion represents the entire transaction. -->

Contracts can be deployed with a special type of call (a contractDeploymentCall).

[^1]: Most transactions will likely begin with _two_ callstack items: one for the actual call, and one to pay the rollup provider a fee (by invoking some payment circuit). [Fee payments](../fees/fees.md) still need to be spec'd out.

## `TxRequest`

| Value | Description |
| --- | --- |
| `from`: `aztecAddress` | The aztec address of the user signing the tx. |
| `to`: [`contractAddress`](#contractaddress) | The contract address of the contract being called. <br/>For calls which deploy a contract, this is `0` (context will be understood from the fact this will be popped of a contractDeploymentCallStack). <br/>For calls to private constructors, this is not known when generating the private kernel proof which verifies the private constructor (as the contract won't yet have been assigned an address), so is `0` (in combination with `functionData.isConstructor == true`). |
| `functionData`: [`FunctionData`](#functiondata) | An identifier for the function being called. |
| `args`: `Array[Field]` | Arguments being passed into the function by this tx. |
| `nonce`: `Field` | Useful for overriding a tx which has already been sent to the pool. Note: for private function calls, this _must not_ be incremented sequentially, but should be random-looking. <br/>`0` if this is a fee-paying tx to accompany a 'proper' tx. |
| `txContext`: [`TxContext`](#txcontext) | Miscellaneous data relating to this tx, which might be useful for all subsequent nested function calls, and kernel circuits of this tx. |
| `chainId`: `Field` | Needed to prevent replay attacks on other rollups (e.g. testnet / devnet). <br/>`0` if this is a fee-paying tx to accompany a 'proper' tx. |


## `ContractAddress`

| Value | Description |
| --- | --- |
|  `contractAddress`: `Field` | See [here](deployment.md#l2-contract-address) for contract address derivation. |

## `FunctionData`

| Value | Description |
| --- | --- |
| `vkIndex`: `Field` | The position of the vk (i.e. its leaf index) in this contract's `vkTree`. This is used to identify the function being called. It is derived as the first 4 bytes of the hash of the abi encoding of the function, similar to Solidity. The reason we do this, is so that a vkIndex can be derived from a contract interface, regardless of the ordering of that interface's functions. |
| `isPrivate`: `Bool` | A bit describing if this function is public or private. |
| `isConstructor`: `Bool` | If a function is a constructor, it's going to be called by the Contract Deployment Kernel circuit. This flag notifies the kernel circuit that this private circuit is being executed as a constructor for the deployment of a new contract, and so the following public inputs of this private circuit MUST be revealed to the public kernel circuit (so that the Contract Deployment kernel circuit may validate them): <br/><ul><li>`callStackItemHash` - so that the Contract Deployment kernel circuit (and interested parties) can be convinced that the constructor was run with the correct set of inputs (this might need to exposed all the way to L1)</li><li>`vkHash` - so that the Contract Deplyment constructor can validate that the correct function was executed. (Notice, we don't use a conventional `functionData` because we won't be adding constructor vkHashes to the vkTree). Notice, by design, this doesn't reveal any details to observers about the nature of the executed function. We might want to optionally allow the underlying vk to be broadcast (and hence we might need to expose this vkHash all the way to L1).</li><li>`emittedPublicInputs` - to pass particular inputs to L1.</li></ul> |


_Note_: the `contractTree` is append-only, so individual verification keys can't be 'replaced' for bug-fixes etc.; the entire contract (a `vkTree`) would need to be re-deployed to the next available slot in the `contractTree`. Such re-deployments, of course, would change the contract address, unless using a proxy pattern.


## `FeeData`

Currently, the fee model is inspired by EIP-1559.

| Value | Description |
| --- | --- |
| `maxPriorityFeePerAztecGas`: `Field` | |
| `maxFeePerAztecGas`: `Field` | |
| `aztecGasLimit`: `Field` | |
| `maxPriorityFeePerEthGas`: `Field` | |
| `maxFeePerEthGas`: `Field` | |
| `ethGasLimit`: `Field` | |
| `payFeePrivately`: `Bool` | |
| `payFeeFromL1`: `Bool` | Provides a way of paying for private L2 function execution from L1 (the RollupProcessor.sol could provide the interface for ETH or ERC20... but we could generalise this by somehow pointing to a particular 'other' payment L1 contract state?). "Submit on-chain the callHash and state a fee (in L1 currency) for the L2 tx, and then when the L2 tx is executed, the rollup provider may redeem the previously-published fee". <br/><br/>  Notifies the kernel circuit that the following public inputs of this private circuit MUST be revealed to the public kernel circuit:<br/><ul><li>`callHash` - no other data is needed (in order to allow the user to keep the function they've actually executed private).</li></ul> |
| `payFeeFromPublicL2`: `Bool` | Provides a way of paying for private L2 function execution from a public L2 circuit. This input being `true` will cause the rollup provider to look for an `isFeePayment` tx in the _public_ callStack. |
| `feeAmount`: `Field` | |
| `feeStandardId`: `Field` | |
| `signedFeePaymentTxHash`: `Field` | Hash of a [SignedTxRequest](#signedTxRequest) |


## `TxContext`

| Value | Description |
| --- | --- |
| `calledFromL1`: `Bool` | Is this tx being sent as a continuation of some L1 tx which made a call to L2? If the L1 contract wants to call a specific L2 circuit, then the function data needs to be revealed on-chain so that it can be checked that the correct, intended function was executed. <br/><br/> Notifies the private kernel circuit that an L1 contract wants to call this specific private L2 circuit, and so the following MUST be revealed to the public kernel circuit: <br/><ul><li>`functionData` - needed so the L1 contract can confirm the intended function was executed on L2. Although a callHash contains the functionData, the L1 contract wouldn't (cheaply) be able to unpack the callHash. So we expose the function data as well to keep costs down. Although this leaks the function which was called, there's no way around that; this is an L1 -> L2 call after all.</li><li>`callHash` - used as a 'lookup' key. The RollupProcessor will store this `callHash`, and await an L2 tx with this `callHash` before triggering a callback to the L1 contract which made this L1 -> L2 call in the first place.</li><li>`emittedPublicInputs` - needed so that a set of values can be emitted by an L2 function and exposed to L1. It would be too expensive to unpack the callHash and extract all of the custom public inputs of a circuit. This is much cheaper, and is very similar to how the EVM exposes only a few values (via event emissions) to JavaScript (for example).</li></ul> <br/><br/> We need this convoluted process, because the RollupProcessor has to be _sure_ an L1 fee has been set-aside for them, before they add any corresponding L2 states. |
| `calledFromPublicL2`: `Bool` | Is this tx being sent as a continuation of some public L2 tx which made a call to Private L2? |
| `isCallback`: `Bool` | Is this tx being sent as a continuation of some L2 --> L1 call, which now needs to continue L2 execution? |
| `resultsTreeLeafIndex`: `Field` | Not sure if we want this or some kind of tx hash. If executing a callback, the user/dapp needs to feed-in info about the original call they made, so that the Private Client can find the right results tree leaf to use. |
| `isFeePaymentTx`: `Bool` | Is this tx the 'fee payment' component of some other tx? Notifies the kernel circuit that the following public inputs of this private circuit MUST be revealed to the public kernel circuit:<br/><ul><li>`functionData` - so that the rollup provider can see how they're to be paid </li><li>`emittedPublicInputs` - so that the rollup provider can validate the amount they'll be paid (they'll need to be provided with the underlying public inputs separately, to validate the hash).</li></ul> |
| `feeData`: [`FeeData`](#feedata) | Empty if `isFeePaymentTx == false`. Conveys information about the fee being paid for this tx. |
| `referenceRollupNum`: `Field` | The rollup number which should be used if referring to any historic tree values. Useful if the proof needs to use a particular tree state snapshot of a particular historic rollup. |


## `SignedTxRequest`

| Value | Description |
| --- | --- |
| `TxRequest`: [`TxRequest`](#TxRequest) | The TxRequest being signed over. |
| r, s, v | The usual ecdsa signature values. |

---

# Call Stacks

To fully define how Aztec 3 implements call semantics, we start by defining 4 call stack data structures.

A call stack is a vector where each entry describes a transaction that is yet to be processed.

There are 4 call stack types in the Aztec 3 architecture: public calls, private calls, L1 calls (see [earlier](./function-types.md) for details of those 3), and [contract deployment](./deployment.md) calls.


## Structure of a call stack item

A call stack item represents a call which has been made by a function during execution of a tx. It is formatted this way so as to be read by a kernel circuit.

The format varies, depending on the type of call being made.


### `PrivateCallStackItem`

| Data  | Description |
| -------- | -------- |
| `contractAddress`: [`ContractAddress`](#contractaddress) | The address of the contract being called. |
| `functionData`: [`FunctionData`](#functiondata) | The 'function data' of the circuit being called. |
| `publicInputs`: [`PrivateCircuitPublicInputs`](../app-circuits/public-input-abis.md#privatecircuitpublicinputs) | The public inputs of the call, which can be calculated in advanced through simulation. |

### `PublicCallStackItem`

| Data  | Description |
| -------- | -------- |
| `contractAddress`: [`ContractAddress`](#contractaddress) | The address of the contract being called. |
| `functionData`: [`FunctionData`](#functiondata) | The 'function data' of the circuit being called. |
| `publicInputs`: [`PublicCircuitPublicInputs`](../app-circuits/public-input-abis.md#publiccircuitpublicinputs) | The public inputs of the call, which can be calculated in advanced through simulation. <br/>**Note:** for the public circuit ABI, _not all_ public inputs can be included in `publicInputs`, since variables read/written from/to the publicDataTree are not known at the time the call is made by the user (or by another circuit). See the Public Circuit ABI section. |

> Note: A `proof` is not included in the call stack item. For public calls, the proof is not known at the time the call is made, because public proofs are generated by the rollup provider. For private calls, the proof isn't needed for a call stack item to be unique; the input nullifiers and output commitments ensure uniqueness of calls. :question: Q: is a private call which doesn't read/modify state nonsensical (if so, that's good - we can reject such calls (they won't have any nullifiers or commitments), thereby ensuring uniqueness of private callStack items)?

> Note: Certain public inputs are omitted when calculating the publicInputsHash for a public call stack item, because such inputs will depend on the 'current' state of trees, which won't be known by the caller; only by the party executing the call.

> Note: Certain public inputs are omitted when calculating the publicInputsHash for a callback's call stack item, because such inputs will depend on the L1 Result.

> Note: The `contractAddress` is sometimes set to `0` in the `functionData` if a function is calling another function of the same contract. (Functions cannot know their own contract address, since it is set after the function's circuit has been compiled). The kernel circuits are capable of interpreting this. Examples of a function calling another function of the same contract are: a private-to-public call; and a call to a callback function.

### `ContractDeploymentCallStackItem`

An instruction to deploy a contract is also expressed through a call, but the call stack item's data is slightly abbreviated (no function data):

| Data  | Description |
| -------- | -------- |
| `publicInputs`: [`ContractDeploymentPublicInputs`](../app-circuits/public-input-abis.md#contractdeploymentpublicinputs) | The public inputs of this call. |


### `L1CallstackItem`

An L1 callstack item is a call to L1. So it's the Ethereum-defined tuple (functionSelector, argument encoding). See [here](https://docs.soliditylang.org/en/v0.8.11/abi-spec.html#function-selector-and-argument-encoding).

Given that the argument encoding is of a variable size, we can't handle such a thing in our kernel snarks. The easiest thing to do is probably to hash the L1 call's `functionSelector, argumentEncoding` into a single value, and then unpack that value when we come to use it on-chain in the RollupProcessor.sol contract. We should probably use a **keccak256** hash for this, which is cheap on-chain (but unfortunately expensive within a snark).

To ensure the `l1CallStackItem` is a call to the correct portal contract (that is associated with the contract making the call), the kernel circuit adds the `portalContractAddress` to the preimage of an `l1CallStackItem`.

We disambiguate the two calls (with and without a portal contract address) with their name.

So:
- In an app-specific circuit:
  - `partialL1CallStackItem = keccak(functionSelector, argumentEncoding)`
- In the kernel circuit, it gets modified to include the portal contract address of the calling contract:
  - `l1CallStackItem = keccak(portalContractAddress,l1FunctionCall)`
  - We don't do this within the app-specific circuit to save on expensive keccak hashing (since the kernel has to do a portal contract address check, so needs to do this hashing already).


### `CallbackStackItem`

When a function makes a call to another 'layer', it can specify a success or failure callback. By the end of recursing through many calls, a kernel circuit might need to keep track of multiple callbacks that have been requested by many circuits. See [more here](./l1-calls.md), for info on the contents of this table.

| Data  | Description |
| -------- | -------- |
| `callbackPublicKey`: `Point` |  The public inputs of this call. |
| `successCallbackCallHash`: `Field` | |
| `failureCallbackCallHash`: `Field` | |
| `successResultArgMapAcc`: `Field` | |


#### `callStackItemHash`
`callStackItemHash := hash(functionData, publicInputsHash, callContext, etc.)`
a.k.a. `callHash` - similar to Ethereum's notion of a txHash.

We also use the term `callbackCallHash` to refer to the `callHash` of a callback function (which has certain public inputs omitted when calculating the call stack item).

Having a single value for each item on the stack (a hash) makes thinking about the callstack easier.

### Passing inputs between functions

Brief explanation of passing inputs between functions:

Suppose function F0 calls F1 and F2, passing parameter set P1 to F1 receiving response R1 (similar for F2).

Firstly, note that R1 & R2 don't really exist (for private circuits, at least) - there's no return data, it's all input params P1 & P2, because it's a circuit.

So to call F1, F0 adds a callstack item to its public inputs representing a call to F1:

```
callStackItem = {
    functionData,
    publicInputsHash,
    callContext,
    etc.
}
```

Notice the parameters P1 are squished into the `publicInputsHash`. It's up to the logic of function F0 to interpret the underlying public inputs of this callstack item as input params or return params of F1.

Similarly, it's up to the logic of function F0 to interpret the underlying public inputs of the F2 callstack item as input params or return params of F2.



## Function visibility

TODO: maybe move this section?

Aztec contracts will support `internal`, `public` and maybe also `external` function visibility. But this logic will happen at the Noir level. We don't need the core architecture (i.e. the kernel circuits) to perform checks on the visibility of functions, as explained below:

Suppose a contract `A` is calling a contract `B`:

- If `B` is `internal` it won’t exist as a distinct verification key (it'll just be part of A's circuit).
- If `B` is `public` it will exist as a distinct verification key but there is nothing stopping a compiler from rolling an implementation of `B` into other functions in the same contract that call `B` (in fact this is probably exactly what will happen).
- If `B` is `external` there is nothing the kernel snark can do to validate the compiler has chosen or not chosen to roll an implementation of an external function into a different function in the same contract.

It's just language semantics. It’s important but not something that is part of the core architecture spec.



## Aztec 3 transaction flow 

TODO: diagram in mdbook friendly way.

(apologies for badly drawn diagram!)
Edit: see [here](https://drive.google.com/file/d/1gCFhE78QhfEboF0hq3scb4vAU1pE0emH/view?usp=sharing) for more diagrams (and a suggestion to recurse in a merkle tree shape instead of linearly).

![](https://hackmd.io/_uploads/S1WCbhxHK.jpg =600x)


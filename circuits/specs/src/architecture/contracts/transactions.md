# Transactions (Calls)

A transaction begins as a call to a single function[^1]. A function can be uniquely identified (across all contracts) by its [`functionSignature`](#function-signature). A 'call' to a function is expressed through a [`callStackItem`](#call-stacks). During the execution of a transaction, the initial function might make calls to other functions, so callstacks are required. Call stack items are verified recursively. The final kernel snark that is produced by the recursion represents the entire transaction.

Contracts can be deployed with a special type of call (a contractDeploymentCall).

[^1]: Most transactions will likely begin with _two_ callstack items: one for the actual call, and one to pay the rollup provider a fee (by invoking some payment circuit). [Fee payments](../fees/fees.md) still need to be spec'd out.
## Function signature

A private function call is uniquely defined by a 64-bit integer, which we'll call a `functionSignature`:


| Num Bits | Description |
| --- | --- |
| 0-31 (32-bits) | `contractAddress` | The contract address being called. <br/> For calls which deploy a contract, this is `0` (context will be understood from the fact this will be popped of a contractDeploymentCallStack). <br/>For calls to private constructors, this is not known when generating the private kernel proof which verifies the private constructor, so is `0` (in combination with `isConstructor`). |
| 32-60 (29-bits) | `vkIndex` - the position of the vk (i.e. its leaf index) in this contract's `vkTree` (this defines the function being called) |
| 61 (1-bit) | `isPrivate` - A bit describing if this function is public or private. <br/><br/>:question: Is this needed? Perhaps the differences in private/public ABIs is sufficient to cause a circuit to fail if passed the wrong proof type? Or, maybe the vk tree could be arranged in a way which clearly separates public/private vks. |
| 62 (1-bit) | `isConstructor` - If a function is a constructor, it's going to be called by the Contract Deployment Kernel circuit. This flag notifies the kernel circuit that this private circuit is being executed as a constructor for the deployment of a new contract, and so the following public inputs of this private circuit MUST be revealed to the public kernel circuit (so that the Contract Deployment kernel circuit may validate them): <br/><ul><li>`callStackItemHash` - so that the Contract Deployment kernel circuit (and interested parties) can be convinced that the constructor was run with the correct set of inputs (this might need to exposed all the way to L1)</li><li>`vkHash` - so that the Contract Deplyment constructor can validate that the correct function was executed. (Notice, we don't use a conventional `functionSignature` because we won't be adding constructor vkHashes to the vkTree). Notice, by design, this doesn't reveal any details to observers about the nature of the executed function. We might want to optionally allow the underlying vk to be broadcast (and hence we might need to expose this vkHash all the way to L1).</li><li>`emittedPublicInputs` - to pass particular inputs to L1.</li></ul><br/>Special note: this `isConstructor` flag is not part of the private circuit's ABI (as it potentially could have been), because the checks in the contract deployment kernel circuit are much cheaper if it's accessible here. |
| 63 (1-bit) | `isCallback` - Tells the kernel circuit that this function is being executed as a callback from an earlier L2 --> L1 call. |


_Note_: the `contractTree` is append-only, so individual verification keys can't be 'replaced' for bug-fixes etc.; the entire contract (a `vkTree`) would need to be re-deployed to the next available slot in the `contractTree`. Such re-deployments, of course, would change the contract address.

---

## Call Stacks

To fully define how Aztec 3 implements call semantics, we start by defining 4 call stack data structures.

A call stack is a vector where each entry describes a transaction that is yet to be processed.

There are 4 call stack types in the Aztec 3 architecture: public calls, private calls, L1 calls (see [earlier](./function-types.md) for details of those 3), and [contract deployment](./deployment.md) calls.


### Structure of a call stack item

A call stack item contains the following witnesses:

| Data  | Description |
| -------- | -------- |
| `functionSignature` | The 'function signature' of the circuit being called. I.e. `concat(contractAddress, vkIndex, isPrivate)` (see earlier) |
| `publicInputsHash` | The public inputs of the call (represented as a pedersen hash. Preimage provided as auxiliary data. When a call stack item is processed, the preimage is unpacked).<br/><br/>Note: for the public circuit ABI, _not all_ public inputs are hashed to form this `publicInputsHash`, since variables read/written from/to the publicDataTree are not known at the time the call is made by the user (or by another circuit). See the Public Circuit ABI section. |
| `callContext: {` | 'Object' for distinguishing between `call` and `delegateCall` |
| ` - msgSender,` | - If doing a `call` or `staticCall`: Either the user address or the address of the contract that created the call. (Can be set to `0` for private -> public calls) <br/> - If doing a `delegateCall`: the address of the calling contract's own `callContext.msgSender` (since delegate calls can be chained). |
| ` - storageContractAddress,` | - If doing a `call` or `staticCall`: the address of the contract being called. <br/> - If doing a `delegateCall`: the address of the calling contract's own `callContext.storageContractAddress` (since delegate calls can be chained). |
| `}` | |
| `bool isDelegateCall` | Used by the kernel snark to validate that the `callContext` of newly-pushed `callStackItems` is consistent with the contract making the call. |
| `bool isStaticCall` | Informs the kernel snark that it MUST fail if the function being called modifies state. <br/><br/> A state modification includes: creating a new contract; emitting events; writing to trees; making a 'call' to another contract. :question: Not sure why 'delegatecall' is not included as a potentially state-modifying tx in ethereum specs? <br/><br/> Note: static calls to private circuits might not make sense. 'Reads' from the privateDataTree require a write of equal value, but the kernel snark cannot 'see' what has been written (it's a private tx), and so cannot validate that a state change didn't take place. So there would be no output commitments or nullifiers for a private static call. But a private call's only use is (I think :question:) to read/modify private state. So I'm thinking a staticCall to a private circuit doesn't make sense. |

> Note: A `proof` is not included in the call stack item. For public calls, the proof is not known at the time the call is made, because public proofs are generated by the rollup provider. For private calls, the proof isn't needed for a call stack item to be unique; the input nullifiers and output commitments ensure uniqueness of calls. :question: Q: is a private call which doesn't read/modify state nonsensical (if so, that's good - we can reject such calls (they won't have any nullifiers or commitments), thereby ensuring uniqueness of private callStack items)?

> Note: Certain public inputs are omitted when calculating the publicInputsHash for a public call stack item, because such inputs will depend on the 'current' state of trees, which won't be known by the caller; only by the party executing the call.

> Note: Certain public inputs are omitted when calculating the publicInputsHash for a callback's call stack item, because such inputs will depend on the L1 Result.

> Note: The `contractAddress` is sometimes set to `0` in the `functionSignature` if a function is calling another function of the same contract. (Functions cannot know their own contract address, since it is set after the function's circuit has been compiled). The kernel circuits are capable of interpreting this. Examples of a function calling another function of the same contract are: a private-to-public call; and a call to a callback function.


#### `callStackItemHash`
`callStackItemHash := hash(functionSignature, publicInputsHash, callContext, etc.)`
a.k.a. `callHash` - similar to Ethereum's notion of a txHash.

We also use the term `callbackCallHash` to refer to the `callHash` of a callback function (which has certain public inputs omitted when calculating the call stack item).

Having a single value for each item on the stack (a hash) makes thinking about the callstack easier.

#### Contract Deployment call stack item

An instruction to deploy a contract is also expressed through a call, but the call stack item's data is slightly abbreviated (no function signature):

| Data  | Description |
| -------- | -------- |
| `publicInputsHash` | The public inputs of the call (represented as a pedersen hash. Preimage provided as auxiliary data. When a call stack item is processed, the preimage is unpacked). |
| `callContext: {` | 'Object' for distinguishing between `call` and `delegateCall`. :question: Not sure if we need this for a contract deployment? |
| ` - msgSender,` |  |
| ` - storageContractAddress,` |  |
| `}` |  |
| `bool isDelegateCall` |  |
| `bool isStaticCall` |  |

### L1 callstack items

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


### Passing inputs between functions

Brief explanation of passing inputs between functions:

Suppose function F0 calls F1 and F2, passing parameter set P1 to F1 receiving response R1 (similar for F2).

Firstly, note that R1 & R2 don't really exist (for private circuits, at least) - there's no return data, it's all input params P1 & P2, because it's a circuit.

So to call F1, F0 adds a callstack item to its public inputs representing a call to F1:

```
callStackItem = {
    functionSignature,
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


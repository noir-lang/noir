# Local Execution

Transactions are initiated via a _transaction execution request_ sent from the user to their local _private execution environment_ (PXE). The PXE first executes the transaction locally in a _simulation_ step, and then generates a _zero-knowledge proof_ of correct execution. The PXE is then responsible for converting a _transaction execution request_ into a [_transaction_](./tx-object.md) ready to be broadcasted to the network.

<!--
Mike review:
- Perhaps rename all subheadings to be the name of the struct, e.g. `TransactionExecutionRequest` (in backticks), for easier searching and referencing.
    - (We should probably adopt this approach throughout the protocol specs)
- Link to any types / fields which are defined on some other page of the protocol specs (e.g. `AuthWitness`).
- Is the hash used to compute `argsHash` protocol-defined or app-defined? If the former, we should define it (in a way which is consistent with all other hash definitions).
- How are the packed arguments packed? What's the encoding? Or is it app-specific and hence out-of-protocol?
- "Entrypoint" is such an important term, perhaps it needs to be a subheading (with the text rearranged to accommodate such a subheading), for easier referencing and searching?
- Do we need to describe how public functions will be simulated? (I'm not sure the sandbox does such simulation yet, but it ought to, eventually).
- Where we link to definitions (such as "transaction"), if that definition is actually a specific struct, we should use the exact name of the struct, wrapped in backticks, to a subheading whose name exactly matches the name of the struct.
-->

## Execution request

A transaction execution request has the following structure. Note that, since Aztec uses full native account abstraction where every account is backed by a contract, a transaction execution request only needs to provide the contract address, function, and arguments of the initial call; nonces and signatures are arguments to the call, and thus opaque to the protocol.

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `origin`        | `AztecAddress`    | Address of the contract where the transaction is initiated.  |
| `functionSelector`  | u32 | Selector (identifier) of the function to be called as entrypoint in the origin contract.  |
| `argsHash`      | `Field`    | Hash of the arguments to be used for calling the entrypoint function.  |
| `txContext`     | `TxContext`    | Includes chain id, protocol version, and gas settings.  |
| `packedArguments` | `PackedValues[]`    | Preimages for argument hashes. When executing a function call with the hash of the arguments, the PXE will look for the preimage of that hash in this list, and expand the arguments to execute the call. |
| `authWitnesses`   | `AuthWitness[]`    | Authorization witnesses. When authorizing an action identified by a hash, the PXE will look for the authorization witness identified by that hash and provide that value to the account contract. |

## Simulation step

Upon receiving a transaction execution request to _simulate_, the PXE will locally execute the function identified by the given `functionSelector` in the given `origin` contract with the arguments committed to by `argsHash`. We refer to this function as the _entrypoint_. During execution, contracts may request authorization witnesses or expanded arguments from the _execution oracle_ <!-- n/d -->, which are answered with the `packedArguments` and `authWitnesses` from the request.

The _entrypoint_ may enqueue additional function calls, either private or public. The simulation step will always execute all private functions in the call stack until emptied. The result of the simulation is a [_transaction_](./tx-object.md) object without an associated _proof_ which is returned to the application that requested the simulation.

In terms of circuitry, the simulation step must execute all application circuits that correspond to private function calls, and then execute the private kernel circuit until the private call stack is empty. Note that circuits are only executed, there is no witness generation or proving involved.

## Proving step

The proving step is similar to the simulation step, though witnesses are generated for all circuits and proven. Note that it is not necessary to execute the simulation step before the proving step, though it is desirable in order to provide the user with info on their transaction and catch any failed assertions early.

The output of the proving step is a [_transaction_](./tx-object.md) object with a valid _proof_ associated, ready to be broadcasted to the network.

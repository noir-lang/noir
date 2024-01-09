# Public execution

Transactions have a _public execution_ component. Once a transaction is picked up by a sequencer to be included in a block, the sequencer is responsible for executing all enqueued public function calls in the transaction. These are defined by the `data.accumulatedData.publicCallStack` field of the [transaction object](./tx-object.md), which are commitments to the preimages of the `enqueuedPublicFunctionCalls` in the transaction. The sequencer pops function calls from the stack, and pushes new ones as needed, until the public call stack is empty.

## Bytecode

Unlike private functions, which are native circuits, public functions in the Aztec Network are specified in Brillig, a zkVM-friendly bytecode. This bytecode is executed and proven in the Brillig public virtual machine. Each function call is a run of the virtual machine, and a _public kernel circuit_ aggregates these calls and produces a final proof for the transaction, which also includes the _private kernel circuit_ proof of the transaction generated during [local execution](./local-execution.md).

## State

Since public execution is run by the sequencer, it is run on the state of the chain as it is when the transaction is included in the block. Public functions operate on _public state_, an updateable key-value mapping, instead of notes.

## Reverts

Note that, unlike local private execution, public execution can _revert_ due to a failed assertion, running out of gas, trying to call a non-existing function, or other failures. If this happens, the sequencer halts execution and discards all side effects from the [transaction payload phase](../gas-and-fees/gas-and-fees.md#transaction-payload). The transaction is still included in the block and pays fees, but is flagged as reverted.

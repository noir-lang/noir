# Enqueued calls

<!-- TODO: some kind of diagram showing this -->

Calls from private functions to public functions are asynchronous. Since private and public functions are executed in different domains at different times and in different contexts -- the former are run locally by the user in a PXE and the latter by the sequencer -- it is not possible for a private function to call a public one and await its result. Instead, private functions can _enqueue_ public function calls.

The process is analogous to [synchronous calls](./sync-calls.md), but relies on an `enqueuePublicFunctionCall` oracle call that accepts the same arguments. The object returned by the oracle is a `PublicCallStackItem` with a flag `is_execution_request` set, and empty side effects to reflect that the stack item has not been executed yet. As with synchronous calls, the caller is responsible for validating the function and arguments in the call stack item, and to push its hash to its public call stack, which represents the list of enqueued public function calls.

Once the transaction is received by the sequencer, the public kernel circuit can begin processing the enqueued public function calls from the transaction's public call stack, pushing new recursive calls to the stack as needed, and popping-off one call stack item at a time, until the public call stack is empty, as described in the [synchronous calls](./sync-calls.md) section.

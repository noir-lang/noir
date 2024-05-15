# Batched calls

:::warning
The low-level specifics of how batched calls will work is still being discussed.
:::

Calls to private functions can be _batched_ instead of executed [synchronously](./sync-calls.md). When executing a batched call to a private function, the function is not executed on the spot, but enqueued for execution at the end of local execution. Once the private call stack has been emptied, all batched execution requests are grouped by target (contract and function selector), and executed via a single call to each target.

<!-- TODO (possibly in Q2): work with the circuits team to decide on how exactly the kernel circuit(s) will do batched calls -->

Batched calls are implemented by pushing a [`PrivateCallStackItem`](../circuits/private-kernel-initial#privatecallstackitem) with the flag `is_execution_request` into a `private_batched_queue` in the execution context, and require an oracle call to a `batchPrivateFunctionCall` function with the same argument types as for other oracle function calls.

Batched calls are processed by the private kernel circuit. On each kernel circuit iteration, if the private call stack is not empty, the kernel circuit pops and processes the topmost entry. Otherwise, if the batched queue is not empty, the kernel pops the first item, collects and deletes all other items with the same target, and calls into the target. Note that this allows batched calls to trigger further synchronous calls.

<!-- Mike review: In addition to (or perhaps instead of) describing these structures in prose, it would be helpful to write the arrays/structs themselves. My brain is struggling to parse the below paragraph.  -->

The arguments for the batched call are arranged in an array with one position for each individual call. Each position within the array is a nested array where the first element is the call context for that individual call, followed by the actual arguments of the call. A batched call is expected to return an array of `PrivateCircuitPublicInputs`, where each public input's call context matches the call context from the corresponding individual call. This allows batched delegate calls, where each individual call processed has a context of its own. This can be used to emit logs on behalf of multiple contracts within a single batched call.

<!-- TODO: The above seems to make the kernel circuit unnecessarily more complex, since we now need dedicated kernels that handle arrays of app circuit outputs instead of a single one. However, it is needed for precompiles that need to emit tagged notes on behalf of multiple calling contracts. The other option here is to grant precompiles special privileges to emit an event on behalf of any address, so they just use the call_context.msg_sender from each individual call. But the phrase "special privileges" makes me wary. -->

In pseudocode, the kernel circuit executes the following logic:

```
loop:
  if next_call_stack_item = context.private_call_stack.pop():
    execute(next_call_stack_item.address, next_call_stack_item.function_selector, next_call_stack_item.arguments)
  else if next_batched_call = context.private_batched_queue.pop():
    let calls = context.private_batched_queue.filter(call => call.target == target)
    context.private_batched_queue.delete_many(calls)
    execute(target.address, target.function_selector, calls.map(call => [call.call_context, ...call.arguments]))
  else:
    break
```

The rationale for batched calls is to minimize the number of function calls in private execution, in order to reduce total proving times. Batched calls are mostly intended for usage with note delivery precompiles, since these do not require synchronous execution, and allows for processing all notes that are to be encrypted and tagged with the same mechanism using a single call. Batched calls can also be used for other common functions which do not require synchronous execution and which are likely to be invoked multiple times.

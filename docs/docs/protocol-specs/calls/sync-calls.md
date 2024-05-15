# Synchronous calls

<!-- MIKE: come back to this -->

Calls from a private function to another private function, as well as calls from a public function to another public function, are _synchronous_. When a synchronous function call is found during execution, execution jumps to the target of the call, and returns to the caller with a return value from the function called. This allows easy composability across contracts.

At the protocol level, each call is represented as a [`CallStackItem`](../circuits/private-kernel-initial#privatecallstackitem), which includes the contract address and function being called, as well as the public inputs [`PrivateCircuitPublicInputs`](../circuits/private-function.md#public-inputs) or `PublicCircuitPublicInputs` <!-- TODO: the term "public inputs" might be outdated for the AVM. --> that are outputted by the execution of the called function. These public inputs include information on the call context, the side effects of the execution, and the block header.

At the contract level, a call is executed via an oracle call `callPrivateFunction` or `callPublicFunction`, both of which accept the contract address to call, the function selector, and a hash of the arguments. The oracle call prompts the executor to pause the current frame, jump to the target of the call, and return its result. The result is a `CallStackItem` that represents the nested execution.

The calling function is responsible for asserting that the function and arguments in the returned `CallStackItem` match the requested ones, otherwise a malicious oracle could return a `CallStackItem` for a different execution. The calling function must also push the hash of the returned `CallStackItem` into the private or public call stack of the current execution context, which is returned as part of the circuit's [PublicInputs](../circuits/private-function.md#public-inputs) output. The end result is a top-level entrypoint `CallStackItem`, which itself contains (nested within) a stack of call stack items to process.

The kernel circuit is then responsible for iteratively processing each `CallStackItem`, pushing new items into the stack as it encounters nested calls, and popping one item off the stack with each kernel iteration until the stack is empty. The private kernel circuit processes private function calls locally in the PXE, whereas the public kernel circuit processes public function calls on the sequencer's machine.

The private kernel circuit iterations begin with the entrypoint <!-- n/d --> execution, empty output and proof. The public kernel circuit starts with the public call stack in the transaction object <!-- n/d -->, and builds on top of the output and proof of the private kernel circuit.

<!-- what are `top_level_execution` and `tx`? -->

```
let call_stack, kernel_public_inputs, proof
if is_private():
  call_stack = [top_level_execution]
  kernel_public_inputs = empty_inputs
  proof = empty_proof
else:
  call_stack = tx.public_call_stack
  kernel_public_inputs = tx.kernel_public_inputs
  proof = tx.proof

while call_stack is not empty:
  let call_stack_item = call_stack.pop()
  call_stack.push(...call_stack_item.call_stack)
  kernel_public_inputs, proof = kernel_circuit(call_stack_item, kernel_public_inputs, proof)
```

The kernel circuit asserts that nested functions and their side effects are processed in order, and that the hash of each nested execution matches the corresponding hash outputted in the call stack by each `CircuitPublicInputs`.

For more information about how the private kernel circuit works, see [here](../circuits/private-kernel-initial.mdx).

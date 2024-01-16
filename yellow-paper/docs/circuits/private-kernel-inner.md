# Private Kernel Circuit - Inner

## Requirements

Each **inner** kernel iteration processes a private function call and the results of a previous kernel iteration.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the provided proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.md).
- Inner private kernel proof.
- [Reset private kernel proof](./private-kernel-reset.md).

The previous proof and the proof for the current function call are verified using recursion.

### Processing Private Function Call

#### Ensuring the function being called exists in the contract.

This section follows the same [process](./private-kernel-initial.md#ensuring-the-function-being-called-exists-in-the-contract) as outlined in the initial private kernel circuit.

#### Ensuring the function is legitimate:

For the _[function_data](./private-kernel-initial.md#functiondata)_ in _[private_call](#privatecall).[call_stack_item](./private-kernel-initial.md#privatecallstackitem)_, this circuit verifies that:

- It must be a private function:
  - _`function_data.function_type == private`_

#### Ensuring the current call matches the call request.

The top item in the _private_call_requests_ of the _[previous_kernel](#previouskernel)_ must pertain to the current function call.

This circuit will:

1. Pop the request from the stack:

   - _`call_request = previous_kernel.public_inputs.transient_accumulated_data.private_call_requests.pop()`_

2. Compare the hash with that of the current function call:

   - _`call_request.hash == private_call.call_stack_item.hash()`_
   - The hash of the _call_stack_item_ is computed as:
     - _`hash(contract_address, function_data.hash(), public_inputs.hash(), counter_start, counter_end)`_
     - Where _function_data.hash()_ and _public_inputs.hash()_ are the hashes of the serialized field elements.

#### Ensuring this function is called with the correct context.

For the _call_context_ in the [public_inputs](./private-function.md#public-inputs) of the _[private_call](#privatecall).[call_stack_item](./private-kernel-initial.md#privatecallstackitem)_ and the _call_request_ popped in the [previous step](#ensuring-the-current-call-matches-the-call-request), this circuit checks that:

1. If it is a standard call (`call_context.is_delegate_call == false`):

   - The _msg_sender_ of the current iteration must be the same as the caller's _contract_address_:
     - _`call_context.msg_sender == call_request.caller_contract_address`_
   - The _storage_contract_address_ of the current iteration must be the same as its _contract_address_:
     - _`call_context.storage_contract_address == call_stack_item.contract_address`_

2. If it is a delegate call (`call_context.is_delegate_call == true`):

   - The _caller_context_ in the _call_request_ must not be empty. Specifically, the following values of the caller must not be zeros:
     - _msg_sender_
     - _storage_contract_address_
   - The _msg_sender_ of the current iteration must equal the caller's _msg_sender_:
     - _`call_context.msg_sender == caller_context.msg_sender`_
   - The _storage_contract_address_ of the current iteration must equal the caller's _storage_contract_address_:
     - _`call_context.storage_contract_address == caller_context.storage_contract_address`_
   - The _storage_contract_address_ of the current iteration must not equal the _contract_address_:
     - _`call_context.storage_contract_address != call_stack_item.contract_address`_

3. If it is an internal call (`call_stack_item.function_data.is_internal == true`):

   - The _msg_sender_ of the current iteration must equal the _storage_contract_address_:
     - _`call_context.msg_sender == call_context.storage_contract_address`_

#### Verifying the private function proof.

It verifies that the private function was executed successfully with the provided proof data, verification key, and the public inputs, sourced from _[private_inputs](#private-inputs).[private_call](#privatecall)_.

This circuit verifies this proof and [the proof of the previous kernel iteration](#verifying-the-previous-kernel-proof) using recursion, and generates a single proof. This consolidation of multiple proofs into one is what allows the private kernel circuits to gradually merge private function proofs into a single proof of execution that represents the entire private section of a transaction.

#### Verifying the public inputs of the private function circuit.

It ensures the private function circuit's intention by checking the following in _[private_call](#privatecall).[call_stack_item](#privatecallstackitem).[public_inputs](./private-function.md#public-inputs)_:

- The _block_header_ must match the one in the _[constant_data](./private-kernel-initial.md#constantdata)_.
- If it is a static call (_`public_inputs.call_context.is_static_call == true`_), it ensures that the function does not induce any state changes by verifying that the following arrays are empty:
  - _note_hashes_
  - _nullifiers_
  - _l2_to_l1_messages_
  - _unencrypted_log_hashes_
  - _encrypted_log_hashes_
  - _encrypted_note_preimage_hashes_

#### Verifying the counters.

This section follows the same [process](./private-kernel-initial.md#verifying-the-counters) as outlined in the initial private kernel circuit.

Additionally, it verifies that for the _[call_stack_item](#privatecallstackitem)_, the _counter_start_ and _counter_end_ must match those in the _call_request_ [popped](#ensuring-the-current-call-matches-the-call-request) from the _private_call_requests_ in a previous step.

### Validating Public Inputs

#### Verifying the transient accumulated data.

The _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ in this circuit's _[public_inputs](#public-inputs)_ includes values from both the previous iterations and the _[private_call](#privatecall)_.

For each array in the _transient_accumulated_data_, this circuit verifies that:

1. It is populated with the values from the previous iterations, specifically:

   - _`public_inputs.transient_accumulated_data.ARRAY[0..N] == private_inputs.previous_kernel.public_inputs.transient_accumulated_data.ARRAY[0..N]`_

   > It's important to note that the top item in the _private_call_requests_ from the _previous_kernel_ won't be included, as it has been removed in a [previous step](#ensuring-the-current-call-matches-the-call-request).

2. As for the subsequent items appended after the values from the previous iterations, they constitute the values from the _private_call_, and each must undergo the same [verification](./private-kernel-initial.md#verifying-the-transient-accumulated-data) as outlined in the initial private kernel circuit.

#### Verifying the constant data.

It verifies that the _[constant_data](./private-kernel-initial.md#constantdata)_ in the _[public_inputs](#public-inputs)_ matches the _constant_data_ in _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs)_.

## Private Inputs

### _PreviousKernel_

Data of the previous kernel iteration.

| Field                | Type                                                                            | Description                                  |
| -------------------- | ------------------------------------------------------------------------------- | -------------------------------------------- |
| _public_inputs_      | _[InitialPrivateKernelPublicInputs](./private-kernel-initial.md#public-inputs)_ | Public inputs of the proof.                  |
| _proof_              | _Proof_                                                                         | Proof of the kernel circuit.                 |
| _vk_                 | _VerificationKey_                                                               | Verification key of the kernel circuit.      |
| _membership_witness_ | _[MembershipWitness](./private-kernel-initial.md#membershipwitness)_            | Membership witness for the verification key. |

### _PrivateCall_

The format aligns with the _[PrivateCall](./private-kernel-initial.md#privatecall)_ of the initial private kernel circuit.

## Public Inputs

The format aligns with the _[Public Inputs](./private-kernel-initial.md#public-inputs)_ of the initial private kernel circuit.

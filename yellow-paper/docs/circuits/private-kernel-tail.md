# Private Kernel Circuit - Tail

## Requirements

The **tail** circuit abstains from processing individual private function calls. Instead, it incorporates the outcomes of a private kernel circuit and conducts additional processing essential for generating the final public inputs suitable for submission to the transaction pool, subsequently undergoing processing by Sequencers and Provers. The final public inputs must safeguard against revealing any private information unnecessary for the execution of public kernel circuits and rollup circuits.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.md).
- [Inner private kernel proof](./private-kernel-inner.md).
- [Reset private kernel proof](./private-kernel-reset.md).

An inner iteration may be omitted when there's only a single private function call for the transaction. And a reset iteration can be skipped if there are no read requests and transient notes in the public inputs from the last iteration.

#### Ensuring the previous iteration is the last.

It checks the data within _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs).[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ to ensure that no further private kernel iteration is needed.

1. The following must be empty to ensure all the private function calls are processed:

   - _private_call_requests_

2. The following must be empty to ensure a comprehensive final reset:

   - _read_requests_
   - The _nullifier_counter_ associated with each note hash in _note_hash_contexts_.
   - The _nullified_note_hash_ associated with each nullifier in _nullifier_contexts_.

   > A [reset iteration](./private-kernel-reset.md) should ideally precede this step. Although it doesn't have to be executed immediately before the tail circuit, as long as it effectively clears the specified values.

### Processing Final Outputs

#### Siloing values.

Siloing a value with the address of the contract generating the value ensures that data produced by a contract is accurately attributed to the correct contract and cannot be misconstrued as data created in a different contract. This circuit guarantees the following siloed values:

1. Silo _nullifiers_:

   For each _nullifier_ at index _i_ _> 0_ in the _nullifier_contexts_ within _private_inputs_, if _`nullifier.value != 0`_:

   _`nullifier_contexts[i].value = hash(nullifier.contract_address, nullifier.value)`_

   > This process does not apply to _nullifier_contexts[0]_, which is the [hash of the transaction request](./private-kernel-initial.md#ensuring-transaction-uniqueness) created by the initial private kernel circuit.

2. Silo _note_hashes_:

   For each _note_hash_ at index _i_ in the _note_hash_contexts_ within _private_inputs_, if _`note_hash.value != 0`_:

   _`note_hash_contexts[i].value = hash(nonce, siloed_hash)`_

   Where:

   - _`nonce = hash(first_nullifier, index)`_
     - _`first_nullifier = nullifier_contexts[0].value`_.
     - _`index = note_hash_indices[i]`_, which is the index of the same note hash within _public_inputs.note_hashes_. Where _note_hash_indices_ is provided as [hints](#hints) via _private_inputs_ and are verified in the [following step](#verifying-ordered-arrays).
   - _`siloed_hash = hash(note_hash.contract_address, note_hash.value)`_

   > Siloing with a nonce guarantees that each final note hash is a unique value in the note hash tree.

3. Verify the _l2_to_l1_messages_ within _[public_inputs](#public-inputs).[accumulated_data](./public-kernel-tail.md#accumulateddata)_:

   For each _l2_to_l1_message_ at index _i_ in _l2_to_l1_message_contexts_ within _[private_inputs](#private-inputs).[previous_kernel](./private-kernel-inner.md#previouskernel).[public_inputs](./private-kernel-initial.md#private-inputs).[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_:

   - If _l2_to_l1_message.value == 0_:
     - Verify that _`l2_to_l1_messages[i] == 0`_
   - Else:
     - Verify that _`l2_to_l1_messages[i] == hash(l2_to_l1_message.contract_address, version_id, l2_to_l1_message.portal_contract_address, chain_id, l2_to_l1_message.value)`_
     - Where _version_id_ and _chain_id_ are defined in _[public_inputs](#public-inputs).[constant_data](./private-kernel-initial.md#constantdata).[tx_context](./private-kernel-initial.md#transactioncontext)_.

#### Verifying ordered arrays.

The initial and inner kernel iterations may produce values in an unordered state due to the serial nature of the kernel, contrasting with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following arrays within _[public_inputs](#public-inputs).[accumulated_data](./public-kernel-tail.md#accumulateddata)_:

- _note_hashes_
- _nullifiers_
- _public_call_requests_

The corresponding unordered array for each of the above is sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs).[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_.

1. Verify ordered _note_hashes_:

   For each _note_hash_ at index _i_ in _note_hashes_, the associated _note_hash_context_ is at _`note_hash_contexts[note_hash_hints[i]]`_.

   - If _`note_hash != 0`_, verify that:
     - _`note_hash == note_hash_context.value`_
     - _`note_hash_indices[note_hash_hints[i]] == i`_
       - The values in _note_hash_indices_ are provided as [hints](#hints) to [silo note hashes](#siloing-values) and must be verified.
     - If _i > 0_, verify that:
       - _`note_hash_context.counter > note_hash_contexts[note_hash_hints[i - 1]].counter`_
   - Else:
     - All the subsequent values in _note_hashes_ must be 0.
     - All the subsequent contexts in _note_hash_contexts_ must have 0 values.

2. Verify ordered _nullifiers_:

   For each _nullifier_ at index _i_ in _nullifiers_, the associated _nullifier_context_ is at _`nullifier_contexts[nullifier_hints[i]]`_.

   - If _`nullifier != 0`_, verify that:
     - _`nullifier == nullifier_context.value`_
     - If _i > 0_, verify that:
       - _`nullifier_context.counter > nullifier_contexts[nullifier_hints[i - 1]].counter`_
   - Else:
     - All the subsequent values in _nullifiers_ must be 0.
     - All the subsequent contexts in _nullifier_contexts_ must have 0 values.

3. Verify ordered _public_call_requests_:

   For each _request_ at index _i_ in _public_call_requests_ within _public_inputs_, the associated _mapped_request_ is at _`unordered_requests[public_call_request_hints[i]]`_, where _unordered_requests_ refers to the _public_call_requests_ within _private_inputs_.

   - If _`request.hash != 0`_, verify that:
     - _`request.hash == mapped_request.hash`_
     - _`request.caller_contract == mapped_request.caller_contract`_
     - _`request.caller_context == mapped_request.caller_context`_
     - If _i > 0_, verify that:
       - _`mapped_request.counter < unordered_requests[public_call_request_hints[i - 1]].counter`_
   - Else:
     - All the subsequent requests in both _public_call_requests_ and _unordered_requests_ must have 0 hashes.

   > Note that _public_call_requests_ must be arranged in descending order to ensure the calls are executed in chronological order.

> While ordering could occur gradually in each kernel iteration, the implementation is much simpler and **typically** more efficient to be done once in the tail circuit.

#### Recalibrating counters.

While the _counter_start_ of a _public_call_request_ is initially assigned in the private function circuit to ensure proper ordering within the transaction, it should be modified in this step. As using _counter_start_ values obtained from private function circuits may leak information.

The _counter_start_ in the _public_call_requests_ within _public_inputs_ should have been recalibrated. This circuit validates the values through the following checks:

- The _counter_start_ of the non-empty requests are continuous values in descending order:
  - _`public_call_requests[i].counter_start == public_call_requests[i + 1].counter_start + 1`_
- The _counter_start_ of the last non-empty request must be _1_.

> It's crucial for the _counter_start_ of the last request to be _1_, as it's assumed in the [tail public kernel circuit](./public-kernel-tail.md#grouping-storage-writes) that no storage writes have a counter _1_.

> The _counter_end_ for a public call request is determined by the overall count of call requests, reads and writes, note hashes and nullifiers within its scope, including those nested within its child function executions. This calculation will be performed by the sequencer for the executions of public function calls.

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after siloing, as verified in a [previous step](#siloing-values):

   - _l2_to_l1_messages_

2. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - _note_hashes_
   - _nullifiers_

3. The following must match the respective values within _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs).[accumulated_data](./private-kernel-initial.md#accumulateddata)_:

   - _encrypted_logs_hash_
   - _unencrypted_logs_hash_
   - _encrypted_log_preimages_length_
   - _unencrypted_log_preimages_length_

4. The following must be empty:

   - _old_public_data_tree_snapshot_
   - _new_public_data_tree_snapshot_

#### Verifying the transient accumulated data.

It ensures that all data in the _[transient_accumulated_data](./public-kernel-tail.md#transientaccumulateddata)_ within _[public_inputs](#public-inputs)_ is empty, with the exception of the _public_call_requests_.

The _public_call_requests_ must [adhere to a specific order](#verifying-ordered-arrays) with [recalibrated counters](#recalibrating-counters), as verified in the previous steps.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.md#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## Private Inputs

### _PreviousKernel_

The format aligns with the _[PreviousKernel](./private-kernel-inner.md#previouskernel)_ of the inner private kernel circuit.

### _Hints_

Data that aids in the verifications carried out in this circuit:

| Field                       | Type           | Description                                                                                                            |
| --------------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------- |
| _note_hash_indices_         | [_field_; _C_] | Indices of _note_hashes_ for _note_hash_contexts_. _C_ equals the length of _note_hashes_.                             |
| _note_hash_hints_           | [_field_; _C_] | Indices of _note_hash_contexts_ for ordered _note_hashes_. _C_ equals the length of _note_hash_contexts_.              |
| _nullifier_hints_           | [_field_; _C_] | Indices of _nullifier_contexts_ for ordered _nullifiers_. _C_ equals the length of _nullifier_contexts_.               |
| _public_call_request_hints_ | [_field_; _C_] | Indices of _public_call_requests_ for ordered _public_call_requests_. _C_ equals the length of _public_call_requests_. |

## Public Inputs

The format aligns with the _[Public Inputs](./public-kernel-tail.md#public-inputs)_ of the tail public kernel circuit.

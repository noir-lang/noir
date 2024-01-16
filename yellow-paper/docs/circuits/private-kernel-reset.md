# Private Kernel Circuit - Reset

## Requirements

A **reset** circuit is designed to abstain from processing individual private function calls. Instead, it injects the outcomes of an initial, inner, or another reset private kernel circuit, scrutinizes the public inputs, and clears the verifiable data within its scope. A reset circuit can be executed either preceding the tail private kernel circuit, or as a means to "reset" public inputs at any point between two private kernels, allowing data to accumulate seamlessly in subsequent iterations.

There are 2 variations of reset circuits:

- [Read Request Reset Private Kernel Circuit](#read-request-reset-private-kernel-circuit).
- [Transient Note Reset Private Kernel Circuit](#transient-note-reset-private-kernel-circuit).

The incorporation of these circuits not only enhances the modularity and repeatability of the "reset" process but also diminishes the overall workload. Rather than conducting resource-intensive computations such as membership checks in each iteration, these tasks are only performed as necessary within the reset circuits.

### Read Request Reset Private Kernel Circuit.

This reset circuit conducts verification on some or all accumulated read requests and subsequently removes them from the _read_request_contexts_ in the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ within the _public_inputs_ of the [_previous_kernel_](#previouskernel).

A read request can pertain to one of two note types:

- A settled note: generated in a prior successful transaction and included in the note hash tree.
- A pending note: created in the current transaction, not yet part of the note hash tree.

1. To clear read requests for settled notes, the circuit performs membership checks for the targeted read requests using the [hints](#hints-for-read-request-reset-private-kernel-circuit) provided via _private_inputs_.

   For each _persistent_read_index_ at index _i_ in _persistent_read_indices_:

   1. If the _persistent_read_index_ equals the length of the _read_request_contexts_ array, there is no read request to be verified. Skip the rest.
   2. Locate the _read_request_: _`read_request_contexts[persistent_read_index]`_
   3. Perform a membership check on the note being read. Where:
      - The leaf corresponds to the hash of the note: _`read_request.note_hash`_
      - The index and sibling path are in: _`read_request_membership_witnesses[i]`_.
      - The root is the _note_hash_tree_root_ in the _[block_header](./private-function.md#blockheader)_ within _[public_inputs](#public-inputs).[constant_data](./private-kernel-initial.md#constantdata)_.

   > Following the above process, at most _N_ read requests will be cleared, where _N_ is the length of the _persistent_read_indices_ array. It's worth noting that there can be multiple versions of this reset circuit, each with a different value of _N_.

2. To clear read requests for pending notes, the circuit ensures that the notes were created before the corresponding read operation, utilizing the [hints](#hints-for-read-request-reset-private-kernel-circuit) provided via _private_inputs_

   For each _transient_read_index_ at index _i_ in _transient_read_indices_:

   1. If the _transient_read_index_ equals the length of the _read_request_contexts_ array, there is no read request to be verified. Skip the rest.
   2. Locate the _read_request_: _`read_request_contexts[transient_read_index]`_
   3. Locate the _note_hash_ being read: _`note_hash_contexts[pending_note_indices[i]]`_
   4. Verify the following:
      - _`read_request.note_hash == note_hash.value`_
      - _`read_request.contract_address == note_hash.contract_address`_
      - _`read_request.counter > note_hash.counter`_
      - _`(read_request.counter < note_hash.nullifier_counter) | (note_hash.nullifier_counter == 0)`_

   > Given that a reset circuit can execute between two private kernel circuits, there's a possibility that a note is created in a nested execution and hasn't been added to the _public_inputs_. In such cases, the read request cannot be verified in the current reset circuit and must be processed in another reset circuit after the note has been included in the _public_inputs_.

3. This circuit then ensures that the read requests that haven't been verified should remain in the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ within its _public_inputs_.

   For each _read_request_ at index _i_ in the _read_request_contexts_ within the _private_inputs_, find its _status_ at _`read_request_statuses[i]`_:

   - If _status.state == persistent_, _`i == persistent_read_indices[status.index]`_.
   - If _status.state == transient_, _`i == transient_read_indices[status.index]`_.
   - If _status.state == nada_, _`read_request == public_inputs.transient_accumulated_data.read_request_contexts[status.index]`_.

### Transient Note Reset Private Kernel Circuit.

In the event that a pending note is nullified within the same transaction, its note hash, nullifier, and all encrypted note preimage hashes can be removed from the public inputs. This not only avoids redundant data being broadcasted, but also frees up space for additional note hashes and nullifiers in the subsequent iterations.

1. Ensure that each note hash is either propagated to the _public_inputs_ or nullified in the same transaction.

   Initialize both _notes_kept_ and _notes_removed_ to _0_.

   For each _note_hash_ at index _i_ in _note_hash_contexts_ within the _private_inputs_, find the index of its nullifer at _`transient_nullifier_indices[i]`_, provided as [hints](#hints-for-transient-note-reset-private-kernel-circuit):

   - If _`transient_nullifier_indices[i] == nullifier_contexts.len()`_:
     - Verify that the _note_hash_ remains within the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ in the _public_inputs_:
       _`note_hash == public_inputs.transient_accumulated_data.note_hash_contexts[notes_kept]`_
     - Increment _notes_kept_ by 1: _`notes_kept += 1`_
   - Else, locate the _nullifier_ at _`nullifier_contexts[transient_nullifier_indices[i]]`_:

     - Verify that the nullifier is associated with the note:
       - _`nullifier.contract_address == note_hash.contract_address`_
       - _`nullifier.note_hash_counter == note_hash.counter`_
       - _`nullifier.counter == note_hash.nullifier_counter`_
     - Increment _notes_removed_ by 1: _`notes_removed += 1`_
     - Ensure that an empty _note_hash_ is appended to the end of _note_hash_contexts_ in the _public_inputs_:
       - _`public_inputs.transient_accumulated_data.note_hash_contexts[N - notes_removed].is_empty() == true`_
       - Where _N_ is the length of _note_hash_contexts_.

     > Note that the check `nullifier.counter > note_hash.counter` is not necessary as the _nullifier_counter_ is assured to be greater than the counter of the note hash when [propagated](./private-kernel-initial.md#verifying-the-transient-accumulated-data) from either the initial or inner private kernel circuits.

2. Ensure that nullifiers not associated with note hashes removed in the previous step are retained within the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ in the _public_inputs_.

   Initialize both _nullifiers_kept_ and _nullifiers_removed_ to _0_.

   For each _nullifier_ at index _i_ in the _nullifier_contexts_ within the _private_inputs_, find the index of its corresponding transient nullifier at _`nullifier_index_hints[i]`_, provided as [hints](#hints-for-transient-note-reset-private-kernel-circuit):

   - If _`nullifier_index_hints[i] == transient_nullifier_indices.len()`_:
     - Verify that the _nullifier_ remains within the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ in the _public_inputs_:
       _`nullifier == public_inputs.transient_accumulated_data.nullifier_contexts[nullifiers_kept]`_
     - Increment _nullifiers_kept_ by 1: _`nullifiers_kept += 1`_
   - Else, compute _transient_nullifier_index_ as _`transient_nullifier_indices[nullifier_index_hints[i]]`_:
     - Verify that: _`transient_nullifier_index == i`_
     - Increment _nullifiers_removed_ by 1: _`nullifiers_removed += 1`_
     - Ensure that an empty _nullifier_ is appended to the end of _nullifier_contexts_ in the _public_inputs_:
       - _`public_inputs.transient_accumulated_data.nullifier_contexts[N - nullifiers_removed].is_empty() == true`_
       - Where _N_ is the length of _nullifer_contexts_.

   After these steps, ensure that all nullifiers associated with transient note hashes have been identified and removed:

   _`nullifiers_removed == notes_removed`_

3. Ensure that _encrypted_note_preimage_hashes_ not associated with note hashes removed in the previous step are retained within the _[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_ in the _public_inputs_.

   Initialize both _hashes_kept_ and _hashes_removed_ to _0_.

   For each _preimage_hash_ at index _i_ in the _encrypted_note_preimage_hash_contexts_ within the _private_inputs_, find the _index_hint_ of its corresponding hash within _public_inputs_ at _`encrypted_note_preimage_hash_index_hints[i]`_, provided as [hints](#hints-for-transient-note-reset-private-kernel-circuit):

   - If _`index_hint == encrypted_note_preimage_hash_contexts.len()`_:
     - Ensure that the associated note hash is removed:
       - Locate the _note_hash_ at _`private_inputs.transient_accumulated_data.note_hash_contexts[log_note_hash_hints[i]]`_.
       - Verify that the _preimage_hash_ is associated with the _note_hash_:
         - _`preimage_hash.note_hash_counter == note_hash.counter`_
         - _`preimage_hash.contract_address == note_hash.contract_address`_
       - Confirm that the _note_hash_ has a corresponding nullifier and has been removed in the first step of this section:
         - _`transient_nullifier_indices[log_note_hash_hints[i]] != nullifier_contexts.len()`_
     - Increment _hashes_removed_ by 1: _`hashes_removed += 1`_
     - Ensure that an empty item is appended to the end of _encrypted_note_preimage_hash_contexts_ in the _public_inputs_:
       - _`encrypted_note_preimage_hash_contexts[N - hashes_removed].is_empty() == true`_
       - Where _N_ is the length of _encrypted_note_preimage_hash_contexts_.
   - Else, find the _mapped_preimage_hash_ at _`encrypted_note_preimage_hash_contexts[index_hint]`_ within _public_inputs_:
     - Verify that the context is aggregated to the _public_inputs_ correctly:
       - _`index_hint == hashes_kept`_
       - _`mapped_preimage_hash == preimage_hash`_
     - Ensure that the associated note hash is retained in the _public_inputs_:
       - Locate the _note_hash_ at _`public_inputs.transient_accumulated_data.note_hash_contexts[log_note_hash_hints[i]]`_.
       - Verify that the _preimage_hash_ is associated with the _note_hash_:
         - _`preimage_hash.note_hash_counter == note_hash.counter`_
         - _`preimage_hash.contract_address == note_hash.contract_address`_
     - Increment _hashes_kept_ by 1: _`hashes_kept += 1`_

> Note that this reset process may not necessarily be applied to all transient notes at a time. In cases where a note will be read in a yet-to-be-processed nested execution, the transient note hash and its nullifier must be retained in the _public_inputs_. The reset can only occur in a later reset circuit after all associated read requests have been verified and cleared.

### Common Verifications

Below are the verifications applicable to all reset circuits:

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.md).
- [Inner private kernel proof](./private-kernel-inner.md).
- Reset private kernel proof.

#### Verifying the accumulated data.

It ensures that the _accumulated_data_ in the _[public_inputs](#public-inputs)_ matches the _accumulated_data_ in _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs)_.

#### Verifying the transient accumulated data.

The following must equal the corresponding arrays in _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./private-kernel-initial.md#public-inputs).[transient_accumulated_data](./private-kernel-initial.md#transientaccumulateddata)_:

- _l2_to_l1_message_contexts_
- _private_call_requests_
- _public_call_requests_

The following must remain the same for [read request reset private kernel circuit](#read-request-reset-private-kernel-circuit):

- _note_hash_contexts_
- _nullifier_contexts_

The following must remain the same for [transient note reset private kernel circuit](#transient-note-reset-private-kernel-circuit):

- _read_request_contexts_

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.md#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## Private Inputs

### _PreviousKernel_

The format aligns with the _[PreviousKernel](./private-kernel-inner.md#previouskernel)_ of the inner private kernel circuit.

### _Hints_ for [Read Request Reset Private Kernel Circuit](#read-request-reset-private-kernel-circuit)

| Field                               | Type                                                                        | Description                                                                              |
| ----------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| _transient_read_indices_            | [_field_; _N_]                                                              | Indices of the read requests for transient notes.                                        |
| _pending_note_indices_              | [_field_; _N_]                                                              | Indices of the note hash contexts for transient reads.                                   |
| _persistent_read_indices_           | [_field_; _M_]                                                              | Indices of the read requests for settled notes.                                          |
| _read_request_membership_witnesses_ | [_[MembershipWitness](./private-kernel-initial.md#membershipwitness)_; _M_] | Membership witnesses for the persistent reads.                                           |
| _read_request_statuses_             | [_[ReadRequestStatus](#readrequeststatus)_; _C_]                            | Statuses of the read request contexts. _C_ equals the length of _read_request_contexts_. |

> There can be multiple versions of the read request reset private kernel circuit, each with a different values of _N_ and _M_.

#### _ReadRequestStatus_

| Field   | Type                            | Description                             |
| ------- | ------------------------------- | --------------------------------------- |
| _state_ | persistent \| transient \| nada | State of the read request.              |
| _index_ | _field_                         | Index of the hint for the read request. |

### _Hints_ for [Transient Note Reset Private Kernel Circuit](#transient-note-reset-private-kernel-circuit)

| Field                                      | Type           | Description                                                                                                                                             |
| ------------------------------------------ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| _transient_nullifier_indices_              | [_field_; _C_] | Indices of the nullifiers for transient notes. _C_ equals the length of _note_hash_contexts_.                                                           |
| _nullifier_index_hints_                    | [_field_; _C_] | Indices of the _transient_nullifier_indices_ for transient nullifiers. _C_ equals the length of _nullifier_contexts_.                                   |
| _encrypted_note_preimage_hash_index_hints_ | [_field_; _C_] | Indices of the _encrypted_note_preimage_hash_contexts_ for transient preimage hashes. _C_ equals the length of _encrypted_note_preimage_hash_contexts_. |
| _log_note_hash_hints_                      | [_field_; _C_] | Indices of the _note_hash_contexts_ for transient preimage hashes. _C_ equals the length of _note_hash_contexts_.                                       |

## Public Inputs

The format aligns with the _[Public Inputs](./private-kernel-initial.md#public-inputs)_ of the initial private kernel circuit.

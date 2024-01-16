# Public Kernel Circuit - Tail

## Requirements

The **tail** circuit refrains from processing individual public function calls. Instead, it integrates the results of inner public kernel circuit and performs additional verification and processing necessary for generating the final public inputs.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can only be:

- [Inner public kernel proof](./public-kernel-inner.md).

#### Ensuring the previous iteration is the last.

The following must be empty to ensure all the public function calls are processed:

- _public_call_requests_ within _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./public-kernel-tail.md#public-inputs).[transient_accumulated_data](./public-kernel-tail.md#transientaccumulateddata)_.

### Processing Final Outputs

#### Siloing values.

This section follows the same [process](./private-kernel-tail.md#siloing-values) as outlined in the tail private kernel circuit.

Additionally, it silos the _storage_slot_ of each non-empty item in the following arrays:

- _storage_reads_
- _storage_writes_

The siloed storage slot is computed as: `hash(contract_address, storage_slot)`.

#### Verifying ordered arrays.

The iterations of the public kernel may yield values in an unordered state due to the serial nature of the kernel, which contrasts with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following:

- _note_hashes_
- _nullifiers_
- _storage_reads_
- _storage_writes_
- _ordered_unencrypted_log_hashes_

1. For _note_hashes_, _nullifiers_, and _ordered_unencrypted_log_hashes_, they undergo the same [process](./private-kernel-tail.md#verifying-ordered-arrays) as outlined in the tail private kernel circuit. With the exception that the loop starts from index _offset + i_, where _offset_ is the number of non-zero values in the _note_hashes_ and _nullifiers_ arrays within _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs](./public-kernel-tail.md#public-inputs).[accumulated_data](./public-kernel-tail.md#accumulateddata)_.

2. For _storage_reads_, an _ordered_storage_reads_ and _storage_read_hints_ are provided as [hints](#hints) through _private_inputs_. This circuit checks that:

   For each _read_ at index _i_ in _`storage_reads[i]`_, the associated _mapped_read_ is at _`ordered_storage_reads[storage_read_hints[i]]`_.

   - If _`read.is_empty() == false`_, verify that:
     - All values in _read_ align with those in _mapped_read_:
       - _`read.contract_address == mapped_read.contract_address`_
       - _`read.storage_slot == mapped_read.storage_slot`_
       - _`read.value == mapped_read.value`_
       - _`read.counter == mapped_read.counter`_
     - If _i > 0_, verify that:
       - _`mapped_read[i].counter > mapped_read[i - 1].counter`_
   - Else:
     - All the subsequent reads (_index >= i_) in both _storage_reads_ and _ordered_storage_reads_ must be empty.

3. For _storage_writes_, an _ordered_storage_writes_ and _storage_write_hints_ are provided as [hints](#hints) through _private_inputs_. The verification is the same as the process for _storage_reads_.

#### Verifying public data snaps.

The _public_data_snaps_ is provided through _private_inputs_, serving as hints for _storage_reads_ to prove that the value in the tree aligns with the read operation. For _storage_writes_, it substantiates the presence or absence of the storage slot in the public data tree.

A _[public_data_snap](#publicdatasnap)_ contains:

- A _storage_slot_ and its _value_.
- An _override_counter_, indicating the counter of the first _storage_write_ that writes to the storage slot. Zero if the storage slot is not written in this transaction.
- A flag _exists_ indicating its presence or absence in the public data tree.

This circuit ensures the uniqueness of each snap in _public_data_snaps_. It verifies that:

For each snap at index _i_, where _i_ > 0:

- If _snap.is_empty() == false_
  - _`snap.storage_slot > public_data_snaps[i - 1].storage_slot`_

> It is crucial for each snap to be unique, as duplicated snaps would disrupt a group of writes for the same storage slot. This could enable the unauthorized act of reading the old value after it has been updated.

#### Grouping storage writes.

To facilitate the verification of _storage_reads_ and streamline _storage_writes_, it is imperative to establish connections between writes targeting the same storage slot. Furthermore, the first write in a group must be linked to a _public_data_snap_, ensuring the dataset has progressed from the right initial state.

A new field, _prev_counter_, is incorporated to the _ordered_storage_writes_ to indicate whether each write has a preceding snap or write. Another field, _exists_, is also added to signify the presence or absence of the storage slot in the tree.

1. For each _snap_ at index _i_ in _public_data_snaps_:

   - Skip the remaining steps if it is empty or if its _override_counter_ is _0_.
   - Locate the _write_ at _`ordered_storage_writes[storage_write_indices[i]]`_.
   - Verify the following:
     - _`write.storage_slot == snap.storage_slot`_
     - _`write.counter == snap.override_counter`_
     - _`write.prev_counter == 0`_
   - Update the hints in _write_:
     - _`write.prev_counter = 1`_
     - _`write.exists = snap.exists`_

   > The value _1_ can be utilized to signify a preceding _snap_, as this value can never serve as the counter of a _storage_write_. Because the _counter_start_ for the first public function call must be 1, the counters for all subsequent side effects should exceed this initial value.

2. For each _write_ at index _i_ in _ordered_storage_writes_:

   - Skip the remaining steps if its _next_counter_ is _0_.
   - Locate the _next_write_ at _`ordered_storage_writes[next_storage_write_indices[i]]`_.
   - Verify the following:
     - _`write.storage_slot == next_write.storage_slot`_
     - _`write.next_counter == next_write.counter`_
     - _`write.prev_counter == 0`_
   - Update the hints in _next_write_:
     - _`next_write.prev_counter = write.counter`_
     - _`next_write.exists = write.exists`_

3. Following the previous two steps, verify that all non-empty writes in _ordered_storage_writes_ have a non-zero _prev_counter_.

#### Verifying storage reads.

A storage read can be reading:

- An uninitialized storage slot: the value is zero. There isn't a leaf in the public data tree representing its storage slot, nor in the _storage_writes_.
- An existing storage slot: written in a prior successful transaction. The value being read is the value in the public data tree.
- An updated storage slot: initialized or updated in the current transaction. The value being read is in a _storage_write_.

For each non-empty _read_ at index _i_ in _ordered_storage_reads_, it must satisfy one of the following conditions:

1. If reading an uninitialized or an existing storage slot, the value is in a _snap_:

   - Locate the _snap_ at _`public_data_snaps[persistent_read_hints[i]]`_.
   - Verify the following:
     - _`read.storage_slot == snap.storage_slot`_
     - _`read.value == snap.value`_
     - _`(read.counter < snap.override_counter) | (snap.override_counter == 0)`_
   - If _`snap.exists == false`_:
     - _`read.value == 0`_

   Depending on the value of the _exists_ flag in the snap, verify its presence or absence in the public data tree:

   - If _exists_ is true:
     - It must pass a membership check on the leaf.
   - If _exists_ is false:
     - It must pass a non-membership check on the low leaf. The preimage of the low leaf is at _`storage_read_low_leaf_preimages[i]`_.

   > The (non-)membership checks are executed against the root in _old_public_data_tree_snapshot_. The membership witnesses for the leaves are in _storage_read_membership_witnesses_, provided as [hints](#hints) through _private_inputs_.

2. If reading an updated storage slot, the value is in a _storage_write_:

   - Locates the _storage_write_ at _`ordered_storage_writes[transient_read_hints[i]]`_.
   - Verify the following:
     - _`read.storage_slot == storage_write.storage_slot`_
     - _`read.value == storage_write.value`_
     - _`read.counter > storage_write.counter`_
     - _`(read.counter < storage_write.next_counter) | (storage_write.next_counter == 0)`_

   > A zero _next_counter_ indicates that the value is not written again in the transaction.

#### Updating the public data tree.

It updates the public data tree with the values in _storage_writes_. The _latest_root_ of the tree is _old_public_data_tree_snapshot.root_.

For each non-empty _write_ at index _i_ in _ordered_storage_writes_, the circuit processes it base on its type:

1. Transient write.

   If _`write.next_counter != 0`_, the same storage slot is written again by another storage write that occurs later in the same transaction. This transient _write_ can be ignored as the final state of the tree won't be affected by it.

2. Updating an existing storage slot.

   For a non-transient _write_ (_write.next_counter == 0_), if _`write.exists == true`_, it is updating an existing storage slot. The circuit does the following for such a write:

   - Performs a membership check, where:
     - The leaf if for the existing storage slot.
       - _`leaf.storage_slot = write.storage_slot`_
     - The old value is the value in a _snap_:
       - _`leaf.value = public_data_snaps[public_data_snap_indices[i]].value`_
     - The index and the sibling path are in _storage_write_membership_witnesses_, provided as [hints](#hints) through _private_inputs_.
     - The root is the _latest_root_ after processing the previous write.
   - Derives the _latest_root_ for the _latest_public_data_tree_ with the updated leaf, where _`leaf.value = write.value`_.

3. Creating a new storage slot.

   For a non-transient _write_ (_write.next_counter == 0_), if _`write.exists == false`_, it is initializing a storage slot. The circuit adds it to a subtree:

   - Perform a membership check on the low leaf in the _latest_public_data_tree_ and in the subtree. One check must succeed.
     - The low leaf preimage is at _storage_write_low_leaf_preimages[i]_.
     - The membership witness for the public data tree is at _storage_write_membership_witnesses[i]_.
     - The membership witness for the subtree is at _subtree_membership_witnesses[i]_.
     - The above are provided as [hints](#hints) through _private_inputs_.
   - Update the low leaf to point to the new leaf:
     - _`low_leaf.next_slot = write.storage_slot`_
     - _`low_leaf.next_index = old_public_data_tree_snapshot.next_available_leaf_index + number_of_new_leaves`_
   - If the low leaf is in the _latest_public_data_tree_, derive the _latest_root_ from the updated low leaf.
   - If the low leaf is in the subtree, derive the _subtree_root_ from the updated low leaf.
   - Append the new leaf to the subtree. Derive the _subtree_root_.
   - Increment _number_of_new_leaves_ by 1.

> The subtree and _number_of_new_leaves_ are initialized to empty and 0 at the beginning of the process.

After all the storage writes are processed:

- Batch insert the subtree to the public data tree.
  - The insertion index is _`old_public_data_tree_snapshot.next_available_leaf_index`_.
- Verify the following:
  - _`latest_root == new_public_data_tree_snapshot.root`_
  - _`new_public_data_tree_snapshot.next_available_leaf_index == old_public_data_tree_snapshot.next_available_leaf_index + number_of_new_leaves`_

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after siloing, as verified in a [previous step](#siloing-values):

   - _l2_to_l1_messages_

2. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - _note_hashes_
   - _nullifiers_

3. The hashes and lengths for unencrypted logs are accumulated as follows:

   Initialize _accumulated_logs_hash_ to be the _unencrypted_logs_hash_ within _[private_inputs](#private-inputs).[previous_kernel](#previouskernel).[public_inputs].[accumulated_data](#accumulateddata)_.

   For each non-empty _log_hash_ at index _i_ in _ordered_unencrypted_log_hashes_, which is provided as [hints](#hints), and the [ordering](#verifying-ordered-arrays) was verified against the [siloed hashes](#siloing-values) in previous steps:

   - _`accumulated_logs_hash = hash(accumulated_logs_hash, log_hash.hash)`_
   - _`accumulated_logs_length += log_hash.length`_

   Check the values in the _public_inputs_ are correct:

   - _`unencrypted_logs_hash == accumulated_logs_hash`_
   - _`unencrypted_log_preimages_length == accumulated_logs_length`_

4. The following is referenced and verified in a [previous step](#updating-the-public-data-tree):

   - _old_public_data_tree_snapshot_
   - _new_public_data_tree_snapshot_

#### Verifying the transient accumulated data.

It ensures that the transient accumulated data is empty.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.md#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## Private Inputs

### _PreviousKernel_

| Field                | Type                                                                 | Description                                  |
| -------------------- | -------------------------------------------------------------------- | -------------------------------------------- |
| _public_inputs_      | _[PublicKernelPublicInputs](#public-inputs)_                         | Public inputs of the proof.                  |
| _proof_              | _Proof_                                                              | Proof of the kernel circuit.                 |
| _vk_                 | _VerificationKey_                                                    | Verification key of the kernel circuit.      |
| _membership_witness_ | _[MembershipWitness](./private-kernel-initial.md#membershipwitness)_ | Membership witness for the verification key. |

### _Hints_

Data that aids in the verifications carried out in this circuit:

| Field                                | Type                                                                        | Description                                                                                                                                |
| ------------------------------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| _note_hash_indices_                  | [_field_; _C_]                                                              | Indices of _note_hashes_ for _note_hash_contexts_. _C_ equals the length of _note_hashes_.                                                 |
| _note_hash_hints_                    | [_field_; _C_]                                                              | Indices of _note_hash_contexts_ for ordered _note_hashes_. _C_ equals the length of _note_hash_contexts_.                                  |
| _nullifier_hints_                    | [_field_; _C_]                                                              | Indices of _nullifier_contexts_ for ordered _nullifiers_. _C_ equals the length of _nullifier_contexts_.                                   |
| _ordered_unencrypted_log_hashes_     | [_field_; _C_]                                                              | Ordered _unencrypted_log_hashes_. _C_ equals the length of _unencrypted_log_hashes_.                                                       |
| _unencrypted_log_hash_hints_         | [_field_; _C_]                                                              | Indices of _ordered_unencrypted_log_hashes_ for _unencrypted_log_hash_contexts_. _C_ equals the length of _unencrypted_log_hash_contexts_. |
| _ordered_storage_reads_              | [_[StorageReadContext](#storagereadcontext)_; _C_]                          | Ordered _storage_reads_. _C_ equals the length of _storage_reads_.                                                                         |
| _storage_read_hints_                 | [_field_; _C_]                                                              | Indices of reads for _ordered_storage_reads_. _C_ equals the length of _storage_reads_.                                                    |
| _ordered_storage_writes_             | [_[StorageWriteContext](#storagewritecontext)_; _C_]                        | Ordered _storage_writes_. _C_ equals the length of _storage_writes_.                                                                       |
| _storage_write_hints_                | [_field_; _C_]                                                              | Indices of writes for _ordered_storage_writes_. _C_ equals the length of _storage_writes_.                                                 |
| _public_data_snaps_                  | [_[PublicDataSnap](#publicdatasnap)_; _C_]                                  | Data that aids verification of storage reads and writes. _C_ equals the length of _ordered_storage_writes_ + _ordered_storage_reads_.      |
| _storage_write_indices_              | [_field_; _C_]                                                              | Indices of _ordered_storage_writes_ for _public_data_snaps_. _C_ equals the length of _public_data_snaps_.                                 |
| _transient_read_hints_               | [_field_; _C_]                                                              | Indices of _ordered_storage_writes_ for transient reads. _C_ equals the length of _ordered_storage_reads_.                                 |
| _persistent_read_hints_              | [_field_; _C_]                                                              | Indices of _ordered_storage_writes_ for persistent reads. _C_ equals the length of _ordered_storage_reads_.                                |
| _public_data_snap_indices_           | [_field_; _C_]                                                              | Indices of _public_data_snaps_ for persistent write. _C_ equals the length of _ordered_storage_writes_.                                    |
| _storage_read_low_leaf_preimages_    | [_[PublicDataLeafPreimage](#publicdataleafpreimage)_; _C_]                  | Preimages for public data leaf. _C_ equals the length of _ordered_storage_writes_.                                                         |
| _storage_read_membership_witnesses_  | [_[MembershipWitness](./private-kernel-initial.md#membershipwitness)_; _C_] | Membership witnesses for persistent reads. _C_ equals the length of _ordered_storage_writes_.                                              |
| _storage_write_low_leaf_preimages_   | [_[PublicDataLeafPreimage](#publicdataleafpreimage)_; _C_]                  | Preimages for public data. _C_ equals the length of _ordered_storage_writes_.                                                              |
| _storage_write_membership_witnesses_ | [_[MembershipWitness](./private-kernel-initial.md#membershipwitness)_; _C_] | Membership witnesses for public data tree. _C_ equals the length of _ordered_storage_writes_.                                              |
| _subtree_membership_witnesses_       | [_[MembershipWitness](./private-kernel-initial.md#membershipwitness)_; _C_] | Membership witnesses for the public data subtree. _C_ equals the length of _ordered_storage_writes_.                                       |

## Public Inputs

### _ConstantData_

These are constants that remain the same throughout the entire transaction. Its format aligns with the _[ConstantData](./private-kernel-initial.md#constantdata)_ of the initial private kernel circuit.

### _AccumulatedData_

Data accumulated during the execution of the transaction.

| Field                              | Type                            | Description                                                 |
| ---------------------------------- | ------------------------------- | ----------------------------------------------------------- |
| _note_hashes_                      | [_field_; _C_]                  | Note hashes created in the transaction.                     |
| _nullifiers_                       | [_field_; _C_]                  | Nullifiers created in the transaction.                      |
| _l2_to_l1_messages_                | [_field_; _C_]                  | L2-to-L1 messages created in the transaction.               |
| _unencrypted_logs_hash_            | _field_                         | Hash of the accumulated unencrypted logs.                   |
| _unencrypted_log_preimages_length_ | _field_                         | Length of all unencrypted log preimages.                    |
| _encrypted_logs_hash_              | _field_                         | Hash of the accumulated encrypted logs.                     |
| _encrypted_log_preimages_length_   | _field_                         | Length of all encrypted log preimages.                      |
| _encrypted_note_preimages_hash_    | _field_                         | Hash of the accumulated encrypted note preimages.           |
| _encrypted_note_preimages_length_  | _field_                         | Length of all encrypted note preimages.                     |
| _old_public_data_tree_snapshot_    | _[TreeSnapshot](#treesnapshot)_ | Snapshot of the public data tree prior to this transaction. |
| _new_public_data_tree_snapshot_    | _[TreeSnapshot](#treesnapshot)_ | Snapshot of the public data tree after this transaction.    |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

### _TransientAccumulatedData_

| Field                       | Type                                                                              | Description                                            |
| --------------------------- | --------------------------------------------------------------------------------- | ------------------------------------------------------ |
| _note_hash_contexts_        | [_[NoteHashContext](./private-kernel-initial.md#notehashcontext)_; _C_]           | Note hashes with extra data aiding verification.       |
| _nullifier_contexts_        | [_[NullifierContext](./private-kernel-initial.md#nullifiercontext)_; _C_]         | Nullifiers with extra data aiding verification.        |
| _l2_to_l1_message_contexts_ | [_[L2toL1MessageContext](./private-kernel-initial.md#l2tol1messagecontext)_; _C_] | L2-to-l1 messages with extra data aiding verification. |
| _storage_reads_             | [_[StorageRead](#storageread)_; _C_]                                              | Reads of the public data.                              |
| _storage_writes_            | [_[StorageWrite](#storagewrite)_; _C_]                                            | Writes of the public data.                             |
| _public_call_requests_      | [_[CallRequest](./private-kernel-initial.md#callrequest)_; _C_]                   | Requests to call publics functions.                    |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

## Types

### _TreeSnapshot_

| Field                       | Type  | Description                       |
| --------------------------- | ----- | --------------------------------- |
| _root_                      | field | Root of the tree.                 |
| _next_available_leaf_index_ | field | The index to insert new value to. |

### _StorageRead_

| Field              | Type           | Description                         |
| ------------------ | -------------- | ----------------------------------- |
| _contract_address_ | _AztecAddress_ | Address of the contract.            |
| _storage_slot_     | field          | Storage slot.                       |
| _value_            | field          | Value read from the storage slot.   |
| _counter_          | _field_        | Counter at which the read happened. |

### _StorageWrite_

| Field              | Type           | Description                            |
| ------------------ | -------------- | -------------------------------------- |
| _contract_address_ | _AztecAddress_ | Address of the contract.               |
| _storage_slot_     | field          | Storage slot.                          |
| _value_            | field          | New value written to the storage slot. |
| _counter_          | _field_        | Counter at which the write happened.   |

### _StorageReadContext_

| Field              | Type           | Description                         |
| ------------------ | -------------- | ----------------------------------- |
| _contract_address_ | _AztecAddress_ | Address of the contract.            |
| _storage_slot_     | field          | Storage slot.                       |
| _value_            | field          | Value read from the storage slot.   |
| _counter_          | _field_        | Counter at which the read happened. |

### _StorageWriteContext_

| Field              | Type           | Description                                                            |
| ------------------ | -------------- | ---------------------------------------------------------------------- |
| _contract_address_ | _AztecAddress_ | Address of the contract.                                               |
| _storage_slot_     | field          | Storage slot.                                                          |
| _value_            | field          | New value written to the storage slot.                                 |
| _counter_          | _field_        | Counter at which the write happened.                                   |
| _prev_counter_     | _field_        | Counter of the previous write to the storage slot.                     |
| _next_counter_     | _field_        | Counter of the next write to the storage slot.                         |
| _exists_           | _bool_         | A flag indicating whether the storage slot is in the public data tree. |

### _PublicDataSnap_

| Field              | Type    | Description                                                              |
| ------------------ | ------- | ------------------------------------------------------------------------ |
| _storage_slot_     | field   | Storage slot.                                                            |
| _value_            | field   | Value of the storage slot.                                               |
| _override_counter_ | _field_ | Counter at which the _storage_slot_ is first written in the transaction. |
| _exists_           | _bool_  | A flag indicating whether the storage slot is in the public data tree.   |

### _PublicDataLeafPreimage_

| Field          | Type    | Description                    |
| -------------- | ------- | ------------------------------ |
| _storage_slot_ | field   | Storage slot.                  |
| _value_        | field   | Value of the storage slot.     |
| _next_slot_    | _field_ | Storage slot of the next leaf. |
| _next_index_   | _field_ | Index of the next leaf.        |

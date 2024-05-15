# Public Kernel Circuit - Tail

:::danger
The public kernel circuits are being redesigned to accommodate the latest AVM designs. This page is therefore highly likely to change significantly.
:::

## Requirements

The **tail** circuit refrains from processing individual public function calls. Instead, it integrates the results of inner public kernel circuit and performs additional verification and processing necessary for generating the final public inputs.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel).

The preceding proof can only be:

- [Inner public kernel proof](./public-kernel-inner.md).

#### Ensuring the previous iteration is the last.

The following must be empty to ensure all the public function calls are processed:

- `public_call_requests` in both `revertible_accumulated_data` and `non_revertible_accumulated_data` within [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./public-kernel-tail#public-inputs).

### Processing Final Outputs

#### Siloing values.

This section follows the same [process](./private-kernel-tail.md#siloing-values) as outlined in the tail private kernel circuit.

Additionally, it silos the `storage_slot` of each non-empty item in the following arrays:

- `storage_reads`
- `storage_writes`

The siloed storage slot is computed as: `hash(contract_address, storage_slot)`.

#### Verifying ordered arrays.

The iterations of the public kernel may yield values in an unordered state due to the serial nature of the kernel, which contrasts with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following:

- `note_hashes`
- `nullifiers`
- `storage_reads`
- `storage_writes`
- `ordered_unencrypted_log_hashes`

1. For `note_hashes`, `nullifiers`, and `ordered_unencrypted_log_hashes`, they undergo the same [process](./private-kernel-tail.md#verifying-ordered-arrays) as outlined in the tail private kernel circuit. With the exception that the loop starts from index `offset + i`, where `offset` is the number of non-zero values in the `note_hashes` and `nullifiers` arrays within [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./public-kernel-tail#public-inputs)[`.accumulated_data`](./public-kernel-tail#accumulateddata).

2. For `storage_reads`, an `ordered_storage_reads` and `storage_read_hints` are provided as [hints](#hints) through `private_inputs`. This circuit checks that:

   For each `read` at index `i` in `storage_reads[i]`, the associated `mapped_read` is at `ordered_storage_reads[storage_read_hints[i]]`.

   - If `read.is_empty() == false`, verify that:
     - All values in `read` align with those in `mapped_read`:
       - `read.contract_address == mapped_read.contract_address`
       - `read.storage_slot == mapped_read.storage_slot`
       - `read.value == mapped_read.value`
       - `read.counter == mapped_read.counter`
     - If `i > 0`, verify that:
       - `mapped_read[i].counter > mapped_read[i - 1].counter`
   - Else:
     - All the subsequent reads (index `>= i`) in both `storage_reads` and `ordered_storage_reads` must be empty.

3. For `storage_writes`, an `ordered_storage_writes` and `storage_write_hints` are provided as [hints](#hints) through `private_inputs`. The verification is the same as the process for `storage_reads`.

#### Verifying public data snaps.

The `public_data_snaps` is provided through `private_inputs`, serving as hints for `storage_reads` to prove that the value in the tree aligns with the read operation. For `storage_writes`, it substantiates the presence or absence of the storage slot in the public data tree.

A [public_data_snap](#publicdatasnap) contains:

- A `storage_slot` and its `value`.
- An `override_counter`, indicating the counter of the first `storage_write` that writes to the storage slot. Zero if the storage slot is not written in this transaction.
- A flag `exists` indicating its presence or absence in the public data tree.

This circuit ensures the uniqueness of each snap in `public_data_snaps`. It verifies that:

For each snap at index `i`, where `i` > 0:

- If `snap.is_empty() == false`
  - `snap.storage_slot > public_data_snaps[i - 1].storage_slot`

> It is crucial for each snap to be unique, as duplicated snaps would disrupt a group of writes for the same storage slot. This could enable the unauthorized act of reading the old value after it has been updated.

#### Grouping storage writes.

To facilitate the verification of `storage_reads` and streamline `storage_writes`, it is imperative to establish connections between writes targeting the same storage slot. Furthermore, the first write in a group must be linked to a `public_data_snap`, ensuring the dataset has progressed from the right initial state.

A new field, `prev_counter`, is incorporated to the `ordered_storage_writes` to indicate whether each write has a preceding snap or write. Another field, `exists`, is also added to signify the presence or absence of the storage slot in the tree.

1. For each `snap` at index `i` in `public_data_snaps`:

   - Skip the remaining steps if it is empty or if its `override_counter` is `0`.
   - Locate the `write` at `ordered_storage_writes[storage_write_indices[i]]`.
   - Verify the following:
     - `write.storage_slot == snap.storage_slot`
     - `write.counter == snap.override_counter`
     - `write.prev_counter == 0`
   - Update the hints in `write`:
     - `write.prev_counter = 1`
     - `write.exists = snap.exists`

   > The value _1_ can be utilized to signify a preceding `snap`, as this value can never serve as the counter of a `storage_write`. Because the _counter_start_ for the first public function call must be 1, the counters for all subsequent side effects should exceed this initial value.

2. For each `write` at index `i` in `ordered_storage_writes`:

   - Skip the remaining steps if its `next_counter` is `0`.
   - Locate the `next_write` at `ordered_storage_writes[next_storage_write_indices[i]]`.
   - Verify the following:
     - `write.storage_slot == next_write.storage_slot`
     - `write.next_counter == next_write.counter`
     - `write.prev_counter == 0`
   - Update the hints in `next_write`:
     - `next_write.prev_counter = write.counter`
     - `next_write.exists = write.exists`

3. Following the previous two steps, verify that all non-empty writes in `ordered_storage_writes` have a non-zero `prev_counter`.

#### Verifying storage reads.

A storage read can be reading:

- An uninitialized storage slot: the value is zero. There isn't a leaf in the public data tree representing its storage slot, nor in the `storage_writes`.
- An existing storage slot: written in a prior successful transaction. The value being read is the value in the public data tree.
- An updated storage slot: initialized or updated in the current transaction. The value being read is in a `storage_write`.

For each non-empty `read` at index `i` in `ordered_storage_reads`, it must satisfy one of the following conditions:

1. If reading an uninitialized or an existing storage slot, the value is in a `snap`:

   - Locate the `snap` at `public_data_snaps[persistent_read_hints[i]]`.
   - Verify the following:
     - `read.storage_slot == snap.storage_slot`
     - `read.value == snap.value`
     - `(read.counter < snap.override_counter) | (snap.override_counter == 0)`
   - If `snap.exists == false`:
     - `read.value == 0`

   Depending on the value of the `exists` flag in the snap, verify its presence or absence in the public data tree:

   - If `exists` is true:
     - It must pass a membership check on the leaf.
   - If `exists` is false:
     - It must pass a non-membership check on the low leaf. The preimage of the low leaf is at `storage_read_low_leaf_preimages[i]`.

   > The (non-)membership checks are executed against the root in `old_public_data_tree_snapshot`. The membership witnesses for the leaves are in `storage_read_membership_witnesses`, provided as [hints](#hints) through `private_inputs`.

2. If reading an updated storage slot, the value is in a `storage_write`:

   - Locates the `storage_write` at `ordered_storage_writes[transient_read_hints[i]]`.
   - Verify the following:
     - `read.storage_slot == storage_write.storage_slot`
     - `read.value == storage_write.value`
     - `read.counter > storage_write.counter`
     - `(read.counter < storage_write.next_counter) | (storage_write.next_counter == 0)`

   > A zero `next_counter` indicates that the value is not written again in the transaction.

#### Updating the public data tree.

It updates the public data tree with the values in `storage_writes`. The `latest_root` of the tree is _old_public_data_tree_snapshot.root_.

For each non-empty `write` at index `i` in `ordered_storage_writes`, the circuit processes it base on its type:

1. Transient write.

   If `write.next_counter != 0`, the same storage slot is written again by another storage write that occurs later in the same transaction. This transient `write` can be ignored as the final state of the tree won't be affected by it.

2. Updating an existing storage slot.

   For a non-transient `write` (`write.next_counter == 0`), if `write.exists == true`, it is updating an existing storage slot. The circuit does the following for such a write:

   - Performs a membership check, where:
     - The leaf if for the existing storage slot.
       - `leaf.storage_slot = write.storage_slot`
     - The old value is the value in a `snap`:
       - `leaf.value = public_data_snaps[public_data_snap_indices[i]].value`
     - The index and the sibling path are in `storage_write_membership_witnesses`, provided as [hints](#hints) through `private_inputs`.
     - The root is the `latest_root` after processing the previous write.
   - Derives the `latest_root` for the `latest_public_data_tree` with the updated leaf, where `leaf.value = write.value`.

3. Creating a new storage slot.

   For a non-transient `write` (`write.next_counter == 0`), if `write.exists == false`, it is initializing a storage slot. The circuit adds it to a subtree:

   - Perform a membership check on the low leaf in the _latest_public_data_tree_ and in the subtree. One check must succeed.
     - The low leaf preimage is at `storage_write_low_leaf_preimages[i]`.
     - The membership witness for the public data tree is at `storage_write_membership_witnesses[i]`.
     - The membership witness for the subtree is at `subtree_membership_witnesses[i]`.
     - The above are provided as [hints](#hints) through `private_inputs`.
   - Update the low leaf to point to the new leaf:
     - `low_leaf.next_slot = write.storage_slot`
     - `low_leaf.next_index = old_public_data_tree_snapshot.next_available_leaf_index + number_of_new_leaves`
   - If the low leaf is in the `latest_public_data_tree`, derive the `latest_root` from the updated low leaf.
   - If the low leaf is in the subtree, derive the `subtree_root` from the updated low leaf.
   - Append the new leaf to the subtree. Derive the `subtree_root`.
   - Increment `number_of_new_leaves` by `1`.

> The subtree and _number_of_new_leaves_ are initialized to empty and 0 at the beginning of the process.

After all the storage writes are processed:

- Batch insert the subtree to the public data tree.
  - The insertion index is `old_public_data_tree_snapshot.next_available_leaf_index`.
- Verify the following:
  - `latest_root == new_public_data_tree_snapshot.root`
  - `new_public_data_tree_snapshot.next_available_leaf_index == old_public_data_tree_snapshot.next_available_leaf_index + number_of_new_leaves`

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after siloing, as verified in a [previous step](#siloing-values):

   - `l2_to_l1_messages`

2. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - `note_hashes`
   - `nullifiers`

3. The hashes and lengths for unencrypted logs are accumulated as follows:

   Initialize `accumulated_logs_hash` to be the `unencrypted_logs_hash` within [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel).[public_inputs].[accumulated_data](#accumulateddata).

   For each non-empty _log_hash_ at index `i` in `ordered_unencrypted_log_hashes`, which is provided as [hints](#hints), and the [ordering](#verifying-ordered-arrays) was verified against the [siloed hashes](#siloing-values) in previous steps:

   - `accumulated_logs_hash = hash(accumulated_logs_hash, log_hash.hash)`
   - `accumulated_logs_length += log_hash.length`

   Check the values in the `public_inputs` are correct:

   - `unencrypted_logs_hash == accumulated_logs_hash`
   - `unencrypted_log_preimages_length == accumulated_logs_length`

4. The following is referenced and verified in a [previous step](#updating-the-public-data-tree):

   - `old_public_data_tree_snapshot`
   - `new_public_data_tree_snapshot`

#### Verifying the transient accumulated data.

It ensures that the transient accumulated data is empty.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.mdx#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## `PrivateInputs`

### `PreviousKernel`

| Field                | Type                                                                  | Description                                  |
| -------------------- | --------------------------------------------------------------------- | -------------------------------------------- |
| `public_inputs`      | [`PublicKernelPublicInputs`](#public-inputs)                          | Public inputs of the proof.                  |
| `proof`              | `Proof`                                                               | Proof of the kernel circuit.                 |
| `vk`                 | `VerificationKey`                                                     | Verification key of the kernel circuit.      |
| `membership_witness` | [`MembershipWitness`](./private-kernel-initial#membershipwitness) | Membership witness for the verification key. |

### _Hints_

Data that aids in the verifications carried out in this circuit:

| Field                                | Type                                                                       | Description                                                                                                                                |
| ------------------------------------ | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `note_hash_indices`                  | `[field; C]`                                                               | Indices of `note_hashes` for `note_hash_contexts`. `C` equals the length of `note_hashes`.                                                 |
| `note_hash_hints`                    | `[field; C]`                                                               | Indices of `note_hash_contexts` for ordered `note_hashes`. `C` equals the length of `note_hash_contexts`.                                  |
| `nullifier_hints`                    | `[field; C]`                                                               | Indices of _nullifier_contexts_ for ordered `nullifiers`. `C` equals the length of _nullifier_contexts_.                                   |
| `ordered_unencrypted_log_hashes`     | `[field; C]`                                                               | Ordered _unencrypted_log_hashes_. `C` equals the length of _unencrypted_log_hashes_.                                                       |
| `unencrypted_log_hash_hints`         | `[field; C]`                                                               | Indices of `ordered_unencrypted_log_hashes` for _unencrypted_log_hash_contexts_. `C` equals the length of _unencrypted_log_hash_contexts_. |
| `ordered_storage_reads`              | [`[StorageReadContext; C]`](#storagereadcontext)                           | Ordered `storage_reads`. `C` equals the length of `storage_reads`.                                                                         |
| `storage_read_hints`                 | `[field; C]`                                                               | Indices of reads for `ordered_storage_reads`. `C` equals the length of `storage_reads`.                                                    |
| `ordered_storage_writes`             | [`[StorageWriteContext; C]`](#storagewritecontext)                         | Ordered `storage_writes`. `C` equals the length of `storage_writes`.                                                                       |
| `storage_write_hints`                | `[field; C]`                                                               | Indices of writes for `ordered_storage_writes`. `C` equals the length of `storage_writes`.                                                 |
| `public_data_snaps`                  | [`[PublicDataSnap; C]`](#publicdatasnap)                                   | Data that aids verification of storage reads and writes. `C` equals the length of `ordered_storage_writes` + `ordered_storage_reads`.      |
| `storage_write_indices`              | `[field; C]`                                                               | Indices of `ordered_storage_writes` for `public_data_snaps`. `C` equals the length of `public_data_snaps`.                                 |
| `transient_read_hints`               | `[field; C]`                                                               | Indices of `ordered_storage_writes` for transient reads. `C` equals the length of `ordered_storage_reads`.                                 |
| `persistent_read_hints`              | `[field; C]`                                                               | Indices of `ordered_storage_writes` for persistent reads. `C` equals the length of `ordered_storage_reads`.                                |
| `public_data_snap_indices`           | `[field; C]`                                                               | Indices of `public_data_snaps` for persistent write. `C` equals the length of `ordered_storage_writes`.                                    |
| `storage_read_low_leaf_preimages`    | [`[PublicDataLeafPreimage; C]`](#publicdataleafpreimage)                   | Preimages for public data leaf. `C` equals the length of `ordered_storage_writes`.                                                         |
| `storage_read_membership_witnesses`  | [`[MembershipWitness; C]`](./private-kernel-initial#membershipwitness) | Membership witnesses for persistent reads. `C` equals the length of `ordered_storage_writes`.                                              |
| `storage_write_low_leaf_preimages`   | [`[PublicDataLeafPreimage; C]`](#publicdataleafpreimage)                   | Preimages for public data. `C` equals the length of `ordered_storage_writes`.                                                              |
| `storage_write_membership_witnesses` | [`[MembershipWitness; C]`](./private-kernel-initial#membershipwitness) | Membership witnesses for public data tree. `C` equals the length of `ordered_storage_writes`.                                              |
| `subtree_membership_witnesses`       | [`[MembershipWitness; C]`](./private-kernel-initial#membershipwitness) | Membership witnesses for the public data subtree. `C` equals the length of `ordered_storage_writes`.                                       |

## Public Inputs

| Field                             | Type                                                            | Description                                                 |
| --------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------------- |
| `constant_data`                   | [`ConstantData`](#constantdata)                                 |                                                             |
| `revertible_accumulated_data`     | [`RevertibleAccumulatedData`](#revertibleaccumulateddata)       |                                                             |
| `non_revertible_accumulated_data` | [`NonRevertibleAccumulatedData`](#nonrevertibleaccumulateddata) |                                                             |
| `transient_accumulated_data`      | [`TransientAccumulatedData`](#transientaccumulateddata)         |                                                             |
| `old_public_data_tree_snapshot`   | [`[TreeSnapshot]`](#treesnapshot)                               | Snapshot of the public data tree prior to this transaction. |
| `new_public_data_tree_snapshot`   | [`[TreeSnapshot]`](#treesnapshot)                               | Snapshot of the public data tree after this transaction.    |

### `ConstantData`

These are constants that remain the same throughout the entire transaction. Its format aligns with the [ConstantData](./private-kernel-initial#constantdata) of the initial private kernel circuit.

### `RevertibleAccumulatedData`

Data accumulated during the execution of the transaction.

| Field                              | Type                                                         | Description                                       |
| ---------------------------------- | ------------------------------------------------------------ | ------------------------------------------------- |
| `note_hashes`                      | `[field; C]`                                                 | Note hashes created in the transaction.           |
| `nullifiers`                       | `[field; C]`                                                 | Nullifiers created in the transaction.            |
| `l2_to_l1_messages`                | `[field; C]`                                                 | L2-to-L1 messages created in the transaction.     |
| `unencrypted_logs_hash`            | `field`                                                      | Hash of the accumulated unencrypted logs.         |
| `unencrypted_log_preimages_length` | `field`                                                      | Length of all unencrypted log preimages.          |
| `encrypted_logs_hash`              | `field`                                                      | Hash of the accumulated encrypted logs.           |
| `encrypted_log_preimages_length`   | `field`                                                      | Length of all encrypted log preimages.            |
| `encrypted_note_preimages_hash`    | `field`                                                      | Hash of the accumulated encrypted note preimages. |
| `encrypted_note_preimages_length`  | `field`                                                      | Length of all encrypted note preimages.           |
| `public_call_requests`             | [`[PublicCallRequestContext; C]`](#publiccallrequestcontext) | Requests to call public functions.                |

> The above `C`s represent constants defined by the protocol. Each `C` might have a different value from the others.

### `NonRevertibleAccumulatedData`

Data accumulated during the execution of the transaction.

| Field                              | Type                                                         | Description                                       |
| ---------------------------------- | ------------------------------------------------------------ | ------------------------------------------------- |
| `note_hashes`                      | `[field; C]`                                                 | Note hashes created in the transaction.           |
| `nullifiers`                       | `[field; C]`                                                 | Nullifiers created in the transaction.            |
| `l2_to_l1_messages`                | `[field; C]`                                                 | L2-to-L1 messages created in the transaction.     |
| `unencrypted_logs_hash`            | `field`                                                      | Hash of the accumulated unencrypted logs.         |
| `unencrypted_log_preimages_length` | `field`                                                      | Length of all unencrypted log preimages.          |
| `encrypted_logs_hash`              | `field`                                                      | Hash of the accumulated encrypted logs.           |
| `encrypted_log_preimages_length`   | `field`                                                      | Length of all encrypted log preimages.            |
| `encrypted_note_preimages_hash`    | `field`                                                      | Hash of the accumulated encrypted note preimages. |
| `encrypted_note_preimages_length`  | `field`                                                      | Length of all encrypted note preimages.           |
| `public_call_requests`             | [`[PublicCallRequestContext; C]`](#publiccallrequestcontext) | Requests to call public functions.                |

> The above `C`s represent constants defined by the protocol. Each `C` might have a different value from the others.

### `TransientAccumulatedData`

| Field                       | Type                                                                             | Description                                            |
| --------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `note_hash_contexts`        | [`[NoteHashContext; C]`](./private-kernel-initial#notehashcontext)           | Note hashes with extra data aiding verification.       |
| `nullifier_contexts`        | [`[NullifierContext; C]`](./private-kernel-initial#nullifiercontext)         | Nullifiers with extra data aiding verification.        |
| `l2_to_l1_message_contexts` | [`[L2toL1MessageContext; C]`](./private-kernel-initial#l2tol1messagecontext) | L2-to-l1 messages with extra data aiding verification. |
| `storage_reads`             | [`[StorageRead; C]`](#storageread)                                               | Reads of the public data.                              |
| `storage_writes`            | [`[StorageWrite; C]`](#storagewrite)                                             | Writes of the public data.                             |

> The above `C`s represent constants defined by the protocol. Each `C` might have a different value from the others.

## Types

### `TreeSnapshot`

| Field                       | Type    | Description                       |
| --------------------------- | ------- | --------------------------------- |
| `root`                      | `field` | Root of the tree.                 |
| `next_available_leaf_index` | `field` | The index to insert new value to. |

### `StorageRead`

| Field              | Type           | Description                         |
| ------------------ | -------------- | ----------------------------------- |
| `contract_address` | `AztecAddress` | Address of the contract.            |
| `storage_slot`     | `field`        | Storage slot.                       |
| `value`            | `field`        | Value read from the storage slot.   |
| `counter`          | `u32`          | Counter at which the read happened. |

### `StorageWrite`

| Field              | Type           | Description                            |
| ------------------ | -------------- | -------------------------------------- |
| `contract_address` | `AztecAddress` | Address of the contract.               |
| `storage_slot`     | `field`        | Storage slot.                          |
| `value`            | `field`        | New value written to the storage slot. |
| `counter`          | `u32`          | Counter at which the write happened.   |

### `StorageReadContext`

| Field              | Type           | Description                         |
| ------------------ | -------------- | ----------------------------------- |
| `contract_address` | `AztecAddress` | Address of the contract.            |
| `storage_slot`     | `field`        | Storage slot.                       |
| `value`            | `field`        | Value read from the storage slot.   |
| `counter`          | `u32`          | Counter at which the read happened. |

### `StorageWriteContext`

| Field              | Type           | Description                                                            |
| ------------------ | -------------- | ---------------------------------------------------------------------- |
| `contract_address` | `AztecAddress` | Address of the contract.                                               |
| `storage_slot`     | `field`        | Storage slot.                                                          |
| `value`            | `field`        | New value written to the storage slot.                                 |
| `counter`          | `u32`          | Counter at which the write happened.                                   |
| `prev_counter`     | `field`        | Counter of the previous write to the storage slot.                     |
| `next_counter`     | `field`        | Counter of the next write to the storage slot.                         |
| `exists`           | `bool`         | A flag indicating whether the storage slot is in the public data tree. |

### `PublicDataSnap`

| Field              | Type    | Description                                                              |
| ------------------ | ------- | ------------------------------------------------------------------------ |
| `storage_slot`     | `field` | Storage slot.                                                            |
| `value`            | `field` | Value of the storage slot.                                               |
| `override_counter` | `field` | Counter at which the `storage_slot` is first written in the transaction. |
| `exists`           | `bool`  | A flag indicating whether the storage slot is in the public data tree.   |

### `PublicDataLeafPreimage`

| Field          | Type    | Description                    |
| -------------- | ------- | ------------------------------ |
| `storage_slot` | `field` | Storage slot.                  |
| `value`        | `field` | Value of the storage slot.     |
| `next_slot`    | `field` | Storage slot of the next leaf. |
| `next_index`   | `field` | Index of the next leaf.        |

### `PublicCallRequestContext`

| Field                     | Type                                                          | Description                                   |
| ------------------------- | ------------------------------------------------------------- | --------------------------------------------- |
| `call_stack_item_hash`    | `field`                                                       | Hash of the call stack item.                  |
| `counter`                 | `u32`                                                         | Counter at which the request was made.        |
| `caller_contract_address` | `AztecAddress`                                                | Address of the contract calling the function. |
| `caller_context`          | [`CallerContext`](./private-kernel-initial#callercontext) | Context of the contract calling the function. |

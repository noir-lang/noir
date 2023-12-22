# Public Kernel Circuit - Tail

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

The **tail** circuit refrains from processing individual public function calls. Instead, it integrates the results of iterative public kernel circuit and performs additional verification and processing necessary for generating the final public inputs.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs.

The preceding proof can only be:

- [Iterative public kernel proof](./public-kernel-iterative.md).

#### Ensuring the previous iteration is the last.

The following must be empty to ensure all the public function calls are processed:

- Public call requests.

### Processing Final Outputs

#### Siloing values.

1. It silos the following in the transient accumulated data with each item's contract address:

   - Note hash contexts.
   - Nullifier contexts.

   The siloed value is computed as: `hash(contract_address, value)`.

   Siloing with a contract address ensures that data produced by a contract is accurately attributed to the correct contract and cannot be misconstrued as data created in a different contract.

2. It then applies nonces to the note hashes:

   - The nonce for a note hash is computed as: `hash(first_nullifier, index)`, where:
     - `first_nullifier` is the hash of the transaction request.
     - `index` is the position of the note hash in the note hashes array in the public inputs.

   Siloing with a nonce guarantees that each final note hash is a unique value in the note hash tree.

3. It generates the final hashes for L2-L1 messages, calculated as:

   `hash(contract_address, version_id, portal_contract_address, chain_id, message)`

   Where _version_id_ and _portal_contract_address_ equal the values defined in the constant data.

4. It silos the storage slot of each item in the following array with the item's contract address:

   - Read requests.
   - Update requests.

   The siloed storage slot is computed as: `hash(contract_address, storage_slot)`.

> While siloing could occur in each kernel iteration, it is _typically_ more efficient to be done once in the tail circuit.

#### Verifying ordered arrays.

The iterations of the public kernel may yield values in an unordered state due to the serial nature of the kernel, which contrasts with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following:

- Note hashes.
- Nullifiers.
- L2-to-L1 messages.
- New contracts.
- Read requests.
- Update requests.

The corresponding _unordered_arrays_ for the above are sourced from the [siloed results](#siloing-values).

An _ordered_requests_ array and a _hints_ array are provided for every _unordered_array_ via private inputs.

For each hint _hints[i]_ at index _i_, locate the item at index _i_ in _ordered_array_:

- If the item is not empty:
  - It must correspond to the item at index _hints[i]_ in _unordered_array_.
  - For _i_ != 0, the counter must be greater than the counter of the item at index _hints[i - 1]_ in _unordered_array_.
- If the item is empty:
  - All the subsequent items (index >= _i_) must be empty in both _ordered_array_ and _unordered_array_.

#### Verifying public data snaps.

The public data snaps array is provided through private inputs, serving as hints for read requests to prove that the value in the tree aligns with the read operation. For update requests, it substantiates the presence or absence of the storage slot in the public data tree.

A public data snap contains:

- A leaf in the public data tree, containing the storage slot and its value.
- An override counter, indicating the counter of an update request that overrides the value of the storage slot. Zero if the value is not overridden in this transaction.
- A flag _exists_ indicating its presence or absence in the public data tree.

This circuit ensures the uniqueness of each snap within the provided public data snaps array. It verifies that the storage slot of each item (except for the one at index 0) must be greater than the storage slot of the previous item in the array.

> It is crucial for each snap to be unique, as duplicated snaps would disrupt a group of update requests for the same storage slot. This could facilitate the unauthorized act of reading the old value after it has been updated.

#### Grouping update requests.

To facilitate the verification of read requests and streamline update requests, it is imperative to establish connections between update requests targeting the same storage slot. Furthermore, the first update request in a group must be linked to a public data snap, ensuring the dataset has progressed from the right initial state.

A new field, _prev_counter_, is introduced to the ordered update requests to indicate whether each request possesses a previous snap or update request. Another field, _exists_, is also added to signify the presence or absence of the storage slot in the tree.

1. For each non-empty public data snap:

   - Skip the remaining steps if its override counter is _0_.
   - Locate the request within the update requests using an index provided as a hint through private inputs.
   - Verify that the storage slot of the request matches the storage slot of the snap.
   - Verify that the counter of the request matches the override counter of the snap.
   - Ensure that the _prev_counter_ of the request is _0_.
   - Set the _prev_counter_ of the request to _1_.
   - Set the _exists_ flag of the request to be the same as the snap.

   > The value _1_ can be utilized to signify a public data snap, as this value can never serve as the counter of an update request. The _counter_start_ for the first public function call must be greater than or equal to 1. Subsequently, the counters for all subsequent function calls and requests should exceed this initial value.

2. For each non-empty update request,

   - Skip the remaining steps if its override counter is _0_.
   - Locate the request within the update requests using an index provided as a hint through private inputs.
   - Verify that the storage slot of the request matches the storage slot of the current request.
   - Verify that the counter of the request matches the override counter of the current request.
   - Ensure that the _prev_counter_ of the request is _0_.
   - Set the _prev_counter_ of the request to the counter of the current request.
   - Set the _exists_ flag of the request to be the same as the current request.

3. Following the previous two steps, verify that all non-empty update requests have a non-zero _prev_counter_.

#### Verifying read requests.

A read request can be reading:

- An updated value: initialized or updated in the current transaction. The value being read is in an update request.
- An existing value: initialized or updated in a prior successful transaction. The value being read is the value in the public data tree.
- An uninitialized value: not initialized yet. The read request is reading the value zero. There isn't a leaf in the public data tree representing its storage slot, nor in the update requests.

For each non-empty read request, it must satisfy one of the following conditions:

1. If reading an updated value, the value is in an update request:

   - Locates the update request within the update requests.
     - Its index in the update requests array is provided as a hint through private inputs.
   - The storage slot and value of the read request must match those of the update request.
   - The counter of the update request must be less than the counter of the read request.
   - The override counter of the update request must be zero or greater than the counter of the read request.

   > A zero override counter indicates that the value is not overridden in the transaction.

2. If reading an existing or an uninitialized value, the value is in a public data snap:

   - Locate the snap within the public data snaps.
     - Its index in the public data snaps array is provided as a hint through private inputs.
   - The storage slot and value of the read request must match those of the snap.
   - The override counter of the snap must be zero or greater than the counter of the read request.

   Depending on the value of the _exists_ flag in the snap, verify its presence or absence in the public data tree:

   - If _exists_ is true:
     - It must pass a membership check on the leaf.
   - If _exists_ is false:
     - The value must be zero.
     - It must pass a non-membership check on the low leaf.

   > The membership checks are executed against the root in **old** public data tree snapshot, as defined in the public inputs. The membership witnesses for the leaves and the low leaves are provided as hints through private inputs.

#### Updating the public data tree.

It updates the current public data tree with the update requests. For each non-empty request in the **ordered** and **siloed** update requests array, the circuit processes it base on its type:

1. Transient update.

   If the override counter of a request is not zero, the value is overridden by another update request that occurs later in the same transaction. This transient value can be ignored as the final state of the tree won't be affected by it.

2. Updating an existing storage slot.

   For a non-transient update request, if the _exists_ flag is true, it is updating an existing storage slot. The circuit does the following for such an update:

   - Performs a membership check, where:
     - The leaf contains the existing storage slot.
     - The leaf's old value and the sibling path are provided as hints through private inputs.
     - The root is the latest root after processing the previous request.
   - Derives the new latest root with the new value in the leaf.

3. Creating a new storage slot.

   For a non-transient update request, if the _exists_ flag is false, it is inserting to a new storage slot. The circuit adds it to a subtree:

   - Perform a membership check on the low leaf in the latest public data tree or the subtree.
     - The leaf preimage and its membership witness are provided as hints through private inputs.
   - Update the low leaf to point to the new leaf containing the new storage slot.
     - The low leaf could be in the public data tree or the subtree.
   - Append the new leaf to the subtree.

After all the update requests are processed:

- Batch insert the subtree to the public data tree.
  - The insertion index is the index in the **old** public data tree snapshot.
- Verify that the latest root matches the root in the **new** public data tree snapshot in the public inputs.
- Verify that the index in the **new** public data tree snapshot equals the index in the **old** public data tree snapshot plus the number of the new leaves appended to the subtree.

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - Note hashes.
   - Nullifiers.
   - L2-to-L1 messages.
   - New contracts.

   > Note that these are arrays of siloed values or relevant data. Attributes aiding verification and siloing only exist in the corresponding types in the transient accumulated data.

2. The following must match the respective values in the previous kernel's public inputs:

   - Log hashes.
   - Log lengths.

3. The following is referenced and verified in a [previous step](#updating-the-public-data-tree):

   - Old public data tree snapshot.
   - New public data tree snapshot.

#### Verifying the transient accumulated data.

It ensures that the transient accumulated data is empty.

#### Verifying the constant data.

It verifies that the constant data matches the one in the previous iteration's public inputs.

## Private Inputs

### Previous Kernel

The data of the previous kernel iteration:

- Proof of the kernel circuit. It must be:
  - [Iterative public kernel circuit](./public-kernel-iterative.md).
- Public inputs of the proof.
- Verification key of the circuit.
- Membership witness for the verification key.

### Hints

Data that aids in the verifications carried out in this circuit:

- Sorted indices of read requests.
- Ordered read requests.
- Sorted indices of update requests.
- Ordered update requests.
- Public data snaps.
- Indices of update requests for public data snaps.
- Indices of update requests for transient update requests.
- Hints for read requests, including:
  - A flag indicating whether it's reading an update request or a leaf in the public data tree.
  - Index of the update request or a public data snap.
  - Membership witness.
- Indices of update requests for transient updates.
- Membership witnesses for update requests.
- Membership witnesses of low leaves in public data tree for update requests.
- Membership witnesses of low leaves in subtree for update requests.

## Public Inputs

The structure of this public inputs aligns with that of the [tail private kernel circuit](./private-kernel-tail.md) and the [iterative public kernel circuit](./public-kernel-iterative.md).

### Accumulated Data

It contains data accumulated during the execution of the entire transaction:

- Note hashes.
- Nullifiers.
- L2-to-L1 messages.
- New contracts.
- Log hashes.
- Log lengths.
- Old public data tree snapshot.
- New public data tree snapshot.

### Constant Data

These are constants that remain the same throughout the entire transaction:

- Historical data - representing the states of the block at which the transaction is constructed, including:
  - Hash of the global variables.
  - Roots of the trees:
    - Note hash tree.
    - Nullifier tree.
    - Contract tree.
    - L1-to-l2 message tree.
    - Public data tree.
- Transaction context
  - A flag indicating whether it is a fee paying transaction.
  - A flag indicating whether it is a fee rebate transaction.
  - Chain ID.
  - Version of the transaction.

### Transient Accumulated Data

It includes data that aids in processing each kernel iteration. They must be empty for this circuit.

- Note hash contexts.
- Nullifier contexts.
- L2-to-L1 message contexts.
- New contract contexts.
- Read requests.
- Update requests.
- Public call requests.

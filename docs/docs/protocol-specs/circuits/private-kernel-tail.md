# Private Kernel Circuit - Tail

## Requirements

The **tail** circuit abstains from processing individual private function calls. Instead, it incorporates the outcomes of a private kernel circuit and conducts additional processing essential for generating the final public inputs suitable for submission to the transaction pool, subsequently undergoing processing by Sequencers and Provers. The final public inputs must safeguard against revealing any private information unnecessary for the execution of public kernel circuits and rollup circuits.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel).

The preceding proof can be:

- [Initial private kernel proof](./private-kernel-initial.mdx).
- [Inner private kernel proof](./private-kernel-inner.mdx).
- [Reset private kernel proof](./private-kernel-reset.md).

An inner iteration may be omitted when there's only a single private function call for the transaction. And a reset iteration can be skipped if there are no read requests and transient notes in the public inputs from the last iteration.

#### Ensuring the previous iteration is the last.

It checks the data within [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./private-kernel-initial#public-inputs)[`.transient_accumulated_data`](./private-kernel-initial#transientaccumulateddata) to ensure that no further private kernel iteration is needed.

1. The following must be empty to ensure all the private function calls are processed:

   - `private_call_request_stack`

2. The following must be empty to ensure a comprehensive final reset:

   - `note_hash_read_requests`
   - `nullifier_read_requests`
   - `key_validation_request_contexts`
   - The `nullifier_counter` associated with each note hash in `note_hash_contexts`.
   - The `note_hash_counter` associated with each nullifier in `nullifier_contexts`.

   > A [reset iteration](./private-kernel-reset.md) should ideally precede this step. Although it doesn't have to be executed immediately before the tail circuit, as long as it effectively clears the specified values.

### Processing Final Outputs

#### Siloing values.

Siloing a value with the address of the contract generating the value ensures that data produced by a contract is accurately attributed to the correct contract and cannot be misconstrued as data created in a different contract. This circuit guarantees the following siloed values:

1. Silo `nullifiers`:

   For each `nullifier` at index `i > 0` in the `nullifier_contexts` within `private_inputs`, if `nullifier.value != 0`:

   `nullifier_contexts[i].value = hash(nullifier.contract_address, nullifier.value)`

   > This process does not apply to `nullifier_contexts[0]`, which is the [hash of the transaction request](./private-kernel-initial#ensuring-transaction-uniqueness) created by the initial private kernel circuit.

   <!-- TODO / DANGER :A thought: we might need to include an optional `randomness` field in the TransactionRequest, to prevent observers from attempting to do dictionary attacks on the tx_hash (the nullifier), in an attempt to learn its preimage, and thereby learn everything about the nature of the transaction -->

2. Silo `note_hashes`:

   For each `note_hash` at index `i` in the `note_hash_contexts` within `private_inputs`, if `note_hash.value != 0`:

   `note_hash_contexts[i].value = hash(note_nonce, siloed_hash)`

   Where:

   - `note_nonce = hash(first_nullifier, index)`
     - `first_nullifier = nullifier_contexts[0].value`.
     - `index = note_hash_hints[i]`, which is the index of the same note hash within `public_inputs.note_hashes`. Where `note_hash_hints` is provided as [hints](#hints) via `private_inputs`.
   - `siloed_hash = hash(note_hash.contract_address, note_hash.value)`

   > Siloing with a `note_nonce` guarantees that each final note hash is a unique value in the note hash tree.

3. Silo `l2_to_l1_messages`:

   For each `l2_to_l1_message` at index `i` in `l2_to_l1_message_contexts` within [`private_inputs`], if `l2_to_l1_message.value != 0`:

   `l2_to_l1_message_contexts[i].value = hash(l2_to_l1_message.contract_address, version_id, l2_to_l1_message.portal_contract_address, chain_id, l2_to_l1_message.value)`

   Where `version_id` and `chain_id` are defined in [`public_inputs`](#public-inputs)[`.constant_data`](./private-kernel-initial#constantdata)[`.tx_context`](./private-kernel-initial#transactioncontext).

4. Silo `unencrypted_log_hashes`:

   For each `log_hash` at index `i` in the `unencrypted_log_hash_contexts` within `private_inputs`, if `log_hash.hash != 0`:

   `unencrypted_log_hash_contexts[i].value = hash(log_hash.hash, log_hash.contract_address)`

5. Silo `encrypted_log_hashes`:

   For each `log_hash` at index `i` in the `encrypted_log_hash_contexts` within `private_inputs`, if `log_hash.hash != 0`:

   `encrypted_log_hash_contexts[i].value = hash(log_hash.hash, contract_address_tag)`

   Where `contract_address_tag = hash(log_hash.contract_address, log_hash.randomness)`

<!-- Should there also be some kind of siloing for encrypted note preimage hashes? -->

#### Verifying and splitting ordered data.

The initial and inner kernel iterations may produce values in an unordered state due to the serial nature of the kernel, contrasting with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following:

- `note_hashes`
- `nullifiers`
- `l2_to_l1_messages`
- `unencrypted_log_hashes`
- `encrypted_log_hashes`
- `encrypted_note_preimage_hashes`
- `public_call_requests`

In addition, the circuit split the ordered data into `non_revertible_accumulated_data` and `revertible_accumulated_data` using `min_revertible_side_effect_counter`.

1. Verify ordered `public_call_requests`:

   Initialize `num_non_revertible` and `num_revertible` to `0`.

   For each `request` at index `i` in the **unordered** `public_call_request_contexts` within `private_inputs.previous_kernel.public_inputs.transient_accumulated_data`:

   - Find its associated `mapped_request` in `public_call_requests[public_call_request_hints[i]]` within `public_inputs`.
     - If `request.counter < min_revertible_side_effect_counter`:
       - The `public_call_requests` is in `non_revertible_accumulated_data`.
       - `num_added = num_non_revertible`.
     - If `request.counter >= min_revertible_side_effect_counter`:
       - The `public_call_requests` is in `revertible_accumulated_data`.
       - `num_added = num_revertible`.
   - If `request.call_stack_item_hash != 0`, verify that:
     - `request == mapped_request`
     - If `num_added > 0`, verify that:
       - `public_call_requests[num_added].counter < public_call_requests[num_added - 1].counter`
     - Increment `num_added` by `1`: `num_(non_)revertible += 1`
   - Else:
     - All the subsequent requests (`index >= i`) in `public_call_request_contexts` must be empty.
     - All the subsequent requests (`index >= num_non_revertible`) in `non_revertible_accumulated_data.public_call_requests` must be empty.
     - All the subsequent requests (`index >= num_revertible`) in `revertible_accumulated_data.public_call_requests` must be empty.

   > Note that requests in `public_call_requests` must be arranged in descending order to ensure the calls are executed in chronological order.

2. Verify the rest of the ordered arrays:

   Initialize `num_non_revertible` and `num_revertible` to `0`.

   For each `note_hash_context` at index `i` in the **unordered** `note_hash_contexts` within `private_inputs.previous_kernel.public_inputs.transient_accumulated_data`:

   - Find its associated `note_hash` in `note_hashes[note_hash_hints[i].index]` within `public_inputs`.
     - If `note_hash_context.counter < min_revertible_side_effect_counter`:
       - The `note_hashes` is in `non_revertible_accumulated_data`.
       - `num_added = num_non_revertible`.
     - If `note_hash_context.counter >= min_revertible_side_effect_counter`:
       - The `note_hashes` is in `revertible_accumulated_data`.
       - `num_added = num_revertible`.
   - If `note_hash_context.value != 0`, verify that:
     - `note_hash == note_hash_context.value`
     - `note_hash_hints[note_hash_hints[i].index].counter_(non_)revertible == note_hash_context.counter`
     - If `num_added > 0`, verify that:
       - `note_hash_hints[num_added].counter_(non_)revertible > note_hash_hints[num_added - 1].counter_(non_)revertible`
     - Increment `num_added` by `1`: `num_(non_)revertible += 1`
   - Else:
     - All the subsequent elements (index `>= i`) in `note_hash_contexts` must be empty.
     - All the subsequent elements (index `>= num_non_revertible`) in `non_revertible_accumulated_data.note_hashes` must be empty.
     - All the subsequent elements (index `>= num_revertible`) in `revertible_accumulated_data.note_hashes` must be empty.

   Repeat the same process for `nullifiers`, `l2_to_l1_messages`, `unencrypted_log_hashes`, `encrypted_log_hashes`, and `encrypted_note_preimage_hashes`, where:

   - Ordered `nullifiers` and `l2_to_l1_messages` are within [`public_inputs`](#public-inputs).
   - `ordered_unencrypted_log_hashes_(non_)revertible`, `ordered_encrypted_log_hashes_(non_)revertible`, and `ordered_encrypted_note_preimage_hashes_(non_)revertible` are provided as [`hints`](#hints) through `private_inputs`.

> While ordering could occur gradually in each kernel iteration, the implementation is much simpler and **typically** more efficient to be done once in the tail circuit.

#### Recalibrating counters.

While the `counter` of a `public_call_request` is initially assigned in the private function circuit to ensure proper ordering within the transaction, it should be modified in this step. As using `counter` values obtained from private function circuits may leak information.

The requests in the `public_call_requests` within `public_inputs` have been [sorted in descending order](#verifying-and-splitting-ordered-data) in the previous step. This circuit recalibrates their counters through the following steps:

- The `counter` of the last non-empty request is set to `1`.
- The `counter`s of the other non-empty requests are continuous values in descending order:
  - `public_call_requests[i].counter = public_call_requests[i + 1].counter + 1`

> It's crucial for the `counter` of the last request to be `1`, as it's assumed in the [tail public kernel circuit](./public-kernel-tail#grouping-storage-writes) that no storage writes have a counter `1`.

### Validating Public Inputs

#### Verifying the (non-)revertible accumulated data.

1. The following must align with the results after ordering, as verified in a [previous step](#verifying-and-splitting-ordered-data):

   - `note_hashes`
   - `nullifiers`
   - `l2_to_l1_messages`

2. The `public_call_requests` must [adhere to a specific order](#verifying-ordered-arrays) with [recalibrated counters](#recalibrating-counters), as verified in the previous steps.

3. The hashes and lengths for all logs are accumulated as follows:

   For each non-empty `log_hash` at index `i` in `ordered_unencrypted_log_hashes_(non_)revertible`, which is provided as [hints](#hints), and the [ordering](#verifying-and-splitting-ordered-data) was verified against the [siloed hashes](#siloing-values) in previous steps:

   - `accumulated_logs_hash = hash(accumulated_logs_hash, log_hash.hash)`
     - If `i == 0`: `accumulated_logs_hash = log_hash.hash`
   - `accumulated_logs_length += log_hash.length`

   Check the values in the `public_inputs` are correct:

   - `unencrypted_logs_hash == accumulated_logs_hash`
   - `unencrypted_log_preimages_length == accumulated_logs_length`

   Repeat the same process for `encrypted_logs_hashes` and `encrypted_note_preimages_hashes`.

#### Verifying the transient accumulated data.

It ensures that all data in the [`transient_accumulated_data`](./public-kernel-tail#transientaccumulateddata) within [`public_inputs`](#public-inputs) is empty.

#### Verifying other data.

This section follows the same [process](./private-kernel-inner.mdx#verifying-other-data) as outlined in the inner private kernel circuit.

In addition, it checks that the following are empty:

- `old_public_data_tree_snapshot`
- `new_public_data_tree_snapshot`

## `PrivateInputs`

### `PreviousKernel`

The format aligns with the [PreviousKernel](./private-kernel-inner.mdx#previouskernel) of the inner private kernel circuit.

### `Hints`

Data that aids in the verifications carried out in this circuit:

| Field                                                   | Type                                                                                                          | Description                                                 |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `note_hash_hints`                                       | [[`OrderHint`](#orderhint); [`MAX_NOTE_HASHES_PER_TX`](../constants.md#circuit-constants)]                | Hints for ordering `note_hash_contexts`.                    |
| `nullifier_hints`                                       | [[`OrderHint`](#orderhint); [`MAX_NULLIFIERS_PER_TX`](../constants.md#circuit-constants)]                 | Hints for ordering `nullifier_contexts`.                    |
| `public_call_request_hints`                             | [`field`; [`MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX`](../constants.md#circuit-constants)]                         | Indices of ordered `public_call_request_contexts`.          |
| `unencrypted_log_hash_hints`                            | [[`OrderHint`](#orderhint); [`MAX_UNENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]         | Hints for ordering `unencrypted_log_hash_contexts`.         |
| `ordered_unencrypted_log_hashes_revertible`             | [`field`; [`MAX_UNENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]                           | Ordered revertible `unencrypted_log_hashes`.                |
| `ordered_unencrypted_log_hashes_non_revertible`         | [`field`; [`MAX_UNENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]                           | Ordered non-revertible `unencrypted_log_hashes`.            |
| `encrypted_log_hash_hints`                              | [[`OrderHint`](#orderhint); [`MAX_ENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]           | Hints for ordering `encrypted_log_hash_contexts`.           |
| `ordered_encrypted_log_hashes_revertible`               | [`field`; [`MAX_ENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]                             | Ordered revertible `encrypted_log_hashes`.                  |
| `ordered_encrypted_log_hashes_non_revertible`           | [`field`; [`MAX_ENCRYPTED_LOG_HASHES_PER_TX`](../constants.md#circuit-constants)]                             | Ordered non-revertible `encrypted_log_hashes`.              |
| `encrypted_note_preimage_hints`                         | [[`OrderHint`](#orderhint); [`MAX_ENCRYPTED_NOTE_PREIMAGE_HASHES_PER_TX`](../constants.md#circuit-constants)] | Hints for ordering `encrypted_note_preimage_hash_contexts`. |
| `ordered_encrypted_note_preimage_hashes_revertible`     | [`field`; [`MAX_ENCRYPTED_NOTE_PREIMAGE_HASHES_PER_TX`](../constants.md#circuit-constants)]                   | Ordered revertible `encrypted_note_preimage_hashes`.        |
| `ordered_encrypted_note_preimage_hashes_non_revertible` | [`field`; [`MAX_ENCRYPTED_NOTE_PREIMAGE_HASHES_PER_TX`](../constants.md#circuit-constants)]                   | Ordered non-revertible `encrypted_note_preimage_hashes`.    |

#### `OrderHint`

| Field                    | Type    | Description                                                            |
| ------------------------ | ------- | ---------------------------------------------------------------------- |
| `index`                  | `field` | Index of the mapped element in the ordered array.                      |
| `counter_revertible`     | `u32`   | Counter of the element at index i in the revertible ordered array.     |
| `counter_non_revertible` | `u32`   | Counter of the element at index i in the non-revertible ordered array. |

## `PublicInputs`

The format aligns with the [Public Inputs](./public-kernel-tail#public-inputs) of the tail public kernel circuit.

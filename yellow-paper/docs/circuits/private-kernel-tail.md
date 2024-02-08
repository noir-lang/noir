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

It checks the data within [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./private-kernel-initial.mdx#public-inputs)[`.transient_accumulated_data`](./private-kernel-initial.mdx#transientaccumulateddata) to ensure that no further private kernel iteration is needed.

1. The following must be empty to ensure all the private function calls are processed:

   - `private_call_requests`

2. The following must be empty to ensure a comprehensive final reset:

   - `read_requests`
   - `nullifier_key_validation_request_contexts`
   - The `nullifier_counter` associated with each note hash in `note_hash_contexts`.
   - The `note_hash_counter` associated with each nullifier in `nullifier_contexts`.

   > A [reset iteration](./private-kernel-reset.md) should ideally precede this step. Although it doesn't have to be executed immediately before the tail circuit, as long as it effectively clears the specified values.

### Processing Final Outputs

#### Siloing values.

Siloing a value with the address of the contract generating the value ensures that data produced by a contract is accurately attributed to the correct contract and cannot be misconstrued as data created in a different contract. This circuit guarantees the following siloed values:

1. Silo `nullifiers`:

   For each `nullifier` at index `i > 0` in the `nullifier_contexts` within `private_inputs`, if `nullifier.value != 0`:

   `nullifier_contexts[i].value = hash(nullifier.contract_address, nullifier.value)`

   > This process does not apply to `nullifier_contexts[0]`, which is the [hash of the transaction request](./private-kernel-initial.mdx#ensuring-transaction-uniqueness) created by the initial private kernel circuit.

   <!-- TODO / DANGER :A thought: we might need to include an optional `randomness` field in the TransactionRequest, to prevent observers from attempting to do dictionary attacks on the tx_hash (the nullifier), in an attempt to learn its preimage, and thereby learn everything about the nature of the transaction -->

2. Silo `note_hashes`:

   For each `note_hash` at index `i` in the `note_hash_contexts` within `private_inputs`, if `note_hash.value != 0`:

   `note_hash_contexts[i].value = hash(nonce, siloed_hash)`

   Where:

   - `nonce = hash(first_nullifier, index)`
     - `first_nullifier = nullifier_contexts[0].value`.
     - `index = note_hash_hints[i]`, which is the index of the same note hash within `public_inputs.note_hashes`. Where `note_hash_hints` is provided as [hints](#hints) via `private_inputs`.
   - `siloed_hash = hash(note_hash.contract_address, note_hash.value)`

   > Siloing with a nonce guarantees that each final note hash is a unique value in the note hash tree.

3. Verify the `l2_to_l1_messages` within [`public_inputs`](#public-inputs)[`.accumulated_data`](./public-kernel-tail.md#accumulateddata):

   For each `l2_to_l1_message` at index `i` in `l2_to_l1_message_contexts` within [`private_inputs`](#private-inputs)[`.previous_kernel`](./private-kernel-inner.mdx#previouskernel)[`.public_inputs`](./private-kernel-initial.mdx#private-inputs)[`.transient_accumulated_data`](./private-kernel-initial.mdx#transientaccumulateddata):

   - If `l2_to_l1_message.value == 0`:
     - Verify that `l2_to_l1_messages[i] == 0`
   - Else:
     - Verify that `l2_to_l1_messages[i] == hash(l2_to_l1_message.contract_address, version_id, l2_to_l1_message.portal_contract_address, chain_id, l2_to_l1_message.value)`
     - Where `version_id` and `chain_id` are defined in [`public_inputs`](#public-inputs)[`.constant_data`](./private-kernel-initial.mdx#constantdata)[`.tx_context`](./private-kernel-initial.mdx#transactioncontext).

4. Silo `unencrypted_log_hashes`:

   For each `log_hash` at index `i` in the `unencrypted_log_hash_contexts` within `private_inputs`, if `log_hash.hash != 0`:

   `unencrypted_log_hash_contexts[i].value = hash(log_hash.hash, log_hash.contract_address)`

5. Silo `encrypted_log_hashes`:

   For each `log_hash` at index `i` in the `encrypted_log_hash_contexts` within `private_inputs`, if `log_hash.hash != 0`:

   `encrypted_log_hash_contexts[i].value = hash(log_hash.hash, contract_address_tag)`

   Where `contract_address_tag = hash(log_hash.contract_address, log_hash.randomness)`

<!-- Should there also be some kind of siloing for encrypted note preimage hashes? -->

#### Verifying ordered arrays.

The initial and inner kernel iterations may produce values in an unordered state due to the serial nature of the kernel, contrasting with the stack-based nature of code execution.

This circuit ensures the correct ordering of the following arrays:

- `note_hashes`
- `nullifiers`
- `public_call_requests`
- `ordered_unencrypted_log_hashes`
- `ordered_encrypted_log_hashes`
- `ordered_encrypted_note_preimage_hashes`

Where:

- `note_hashes`, `nullifiers`, and `public_call_requests` are within [`public_inputs`](#public-inputs)[`.accumulated_data`](./public-kernel-tail.md#accumulateddata).
- `ordered_unencrypted_log_hashes`, `ordered_encrypted_log_hashes`, and `ordered_encrypted_note_preimage_hashes` are provided as hints through `private_inputs`.
- Every corresponding unordered array for each of the ordered array is sourced from [`private_inputs`](#private-inputs)[`.previous_kernel`](#previouskernel)[`.public_inputs`](./private-kernel-initial.mdx#public-inputs)[`.transient_accumulated_data`](./private-kernel-initial.mdx#transientaccumulateddata).

1. Verify ordered `public_call_requests`:

   For each `request` at index `i` in `private_inputs.previous_kernel.public_inputs.transient_accumulated_data.public_call_requests[i]`, the associated `mapped_request` is at `public_call_requests[public_call_request_hints[i]]` within `public_inputs`.

   - If `request.hash != 0`, verify that:
     - `request.hash == mapped_request.hash`
     - `request.caller_contract == mapped_request.caller_contract`
     - `request.caller_context == mapped_request.caller_context`
     - If `i > 0`, verify that:
       - `mapped_request[i].counter < mapped_request[i - 1].counter`
   - Else:
     - All the subsequent requests (_index >= i_) in both `public_call_requests` and `unordered_requests` must be empty.

   > Note that `public_call_requests` must be arranged in descending order to ensure the calls are executed in chronological order.

2. Verify the rest of the ordered arrays:

   For each `note_hash_context` at index `i` in the **unordered** `note_hash_contexts` within `private_inputs`, the associated `note_hash` is at `note_hashes[note_hash_hints[i]]`.

   - If `note_hash != 0`, verify that:
     - `note_hash == note_hash_context.value`
     - If `i > 0`, verify that:
       - `note_hashes[i].counter > note_hashes[i - 1].counter`
   - Else:
     - All the subsequent items (index `>= i`) in both `note_hashes` and `note_hash_contexts` must be empty.

   Repeat the same process for `nullifiers`, `ordered_unencrypted_log_hashes`, `ordered_encrypted_log_hashes`, and `ordered_encrypted_note_preimage_hashes`.

> While ordering could occur gradually in each kernel iteration, the implementation is much simpler and **typically** more efficient to be done once in the tail circuit.

#### Recalibrating counters.

While the `counter_start` of a `public_call_request` is initially assigned in the private function circuit to ensure proper ordering within the transaction, it should be modified in this step. As using `counter_start` values obtained from private function circuits may leak information.

The `counter_start` in the `public_call_requests` within `public_inputs` should have been recalibrated. This circuit validates the values through the following checks:

- The `counter_start` of the non-empty requests are continuous values in descending order:
  - `public_call_requests[i].counter_start == public_call_requests[i + 1].counter_start + 1`
- The `counter_start` of the last non-empty request must be `1`.

> It's crucial for the `counter_start` of the last request to be `1`, as it's assumed in the [tail public kernel circuit](./public-kernel-tail.md#grouping-storage-writes) that no storage writes have a counter `1`.

> The `counter_end` for a public call request is determined by the overall count of call requests, reads and writes, note hashes and nullifiers within its scope, including those nested within its child function executions. This calculation will be performed by the sequencer for the executions of public function calls.

### Validating Public Inputs

#### Verifying the accumulated data.

1. The following must align with the results after siloing, as verified in a [previous step](#siloing-values):

   - `l2_to_l1_messages`

2. The following must align with the results after ordering, as verified in a [previous step](#verifying-ordered-arrays):

   - `note_hashes`
   - `nullifiers`

3. The hashes and lengths for all logs are accumulated as follows:

   For each non-empty `log_hash` at index `i` in `ordered_unencrypted_log_hashes`, which is provided as [hints](#hints), and the [ordering](#verifying-ordered-arrays) was verified against the [siloed hashes](#siloing-values) in previous steps:

   - `accumulated_logs_hash = hash(accumulated_logs_hash, log_hash.hash)`
     - If `i == 0`: `accumulated_logs_hash = log_hash.hash`
   - `accumulated_logs_length += log_hash.length`

   Check the values in the `public_inputs` are correct:

   - `unencrypted_logs_hash == accumulated_logs_hash`
   - `unencrypted_log_preimages_length == accumulated_logs_length`

   Repeat the same process for `encrypted_logs_hash`, `encrypted_log_preimages_length`, `encrypted_note_preimages_hash` and `encrypted_note_preimages_length`.

4. The following must be empty:

   - `old_public_data_tree_snapshot`
   - `new_public_data_tree_snapshot`

#### Verifying the transient accumulated data.

It ensures that all data in the [`transient_accumulated_data`](./public-kernel-tail.md#transientaccumulateddata) within [`public_inputs`](#public-inputs) is empty, with the exception of the `public_call_requests`.

The `public_call_requests` must [adhere to a specific order](#verifying-ordered-arrays) with [recalibrated counters](#recalibrating-counters), as verified in the previous steps.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.mdx#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## `PrivateInputs`

### `PreviousKernel`

The format aligns with the [PreviousKernel](./private-kernel-inner.mdx#previouskernel) of the inner private kernel circuit.

### _Hints_

Data that aids in the verifications carried out in this circuit:

| Field                                    | Type         | Description                                                                                                                                                        |
| ---------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `note_hash_hints`                        | `[field; C]` | Indices of ordered `note_hashes` for `note_hash_contexts`. `C` equals the length of `note_hash_contexts`.                                                          |
| `nullifier_hints`                        | `[field; C]` | Indices of ordered `nullifiers` for `nullifier_contexts`. `C` equals the length of `nullifier_contexts`.                                                           |
| `public_call_request_hints`              | `[field; C]` | Indices of ordered `public_call_requests` for `public_call_requests`. `C` equals the length of `public_call_requests`.                                             |
| `ordered_unencrypted_log_hashes`         | `[field; C]` | Ordered `unencrypted_log_hashes`. `C` equals the length of `unencrypted_log_hash_contexts`.                                                                        |
| `unencrypted_log_hash_hints`             | `[field; C]` | Indices of `ordered_unencrypted_log_hashes` for `unencrypted_log_hash_contexts`. `C` equals the length of `unencrypted_log_hash_contexts`.                         |
| `ordered_encrypted_log_hashes`           | `[field; C]` | Ordered `encrypted_log_hashes`. `C` equals the length of `encrypted_log_hash_contexts`.                                                                            |
| `encrypted_log_hash_hints`               | `[field; C]` | Indices of `ordered_encrypted_log_hashes` for `encrypted_log_hash_contexts`. `C` equals the length of `encrypted_log_hash_contexts`.                               |
| `ordered_encrypted_note_preimage_hashes` | `[field; C]` | Ordered `encrypted_note_preimage_hashes`. `C` equals the length of `encrypted_note_preimage_hash_contexts`.                                                        |
| `encrypted_note_preimage_hints`          | `[field; C]` | Indices of `ordered_encrypted_note_preimage_hashes` for `encrypted_note_preimage_hash_contexts`. `C` equals the length of `encrypted_note_preimage_hash_contexts`. |

## `PublicInputs`

The format aligns with the [Public Inputs](./public-kernel-tail.md#public-inputs) of the tail public kernel circuit.

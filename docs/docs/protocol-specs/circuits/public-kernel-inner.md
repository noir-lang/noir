# Public Kernel Circuit - Inner

:::danger
The public kernel circuits are being redesigned to accommodate the latest AVM designs. This page is therefore highly likely to change significantly.
:::

## Requirements

In the public kernel iteration, the process involves taking a previous iteration and public call data, verifying their integrity, and preparing the necessary data for subsequent circuits to operate.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from [private_inputs](#private-inputs).[previous_kernel](#previouskernel).

The preceding proof can be:

- [Initial public kernel proof](./public-kernel-initial.md).
- Inner public kernel proof.

### Processing Public Function Call

#### Ensuring the function being called exists in the contract.

This section follows the same [process](./private-kernel-initial#ensuring-the-function-being-called-exists-in-the-contract) as outlined in the initial private kernel circuit.

#### Ensuring the contract instance being called is deployed.

It verifies the public deployment of the contract instance by conducting a membership proof, where:

- The leaf is a nullifier emitting from the deployer contract, computed as `hash(deployer_address, contract_address)`, where:
  - `deployer_address` is defined in [`private_inputs`](#private-inputs)[`.public_call`](#publiccall)[`.contract_data`](../contract-deployment/instances.md#structure).
  - `contract_data` is defined in [`private_inputs`](#private-inputs)[`.public_call`](#publiccall)[`.call_stack_item`](#publiccallstackitem).
- The index and sibling path are provided in `contract_deployment_membership_witness` through [`private_inputs`](#private-inputs)[`.public_call`](#publiccall)\_.
- The root is the `nullifier_tree_root` in the [`header`](./private-function.md#header) within [`public_inputs`](#public-inputs)[`.constant_data`](./private-kernel-initial#constantdata).

#### Ensuring the current call matches the call request.

The top item in the `public_call_requests` of the [`previous_kernel`](#previouskernel) must pertain to the current function call.

This circuit will:

1. Pop the request from the stack:

   - `call_request = previous_kernel.public_inputs.transient_accumulated_data.public_call_requests.pop()`

2. Compare the hash with that of the current function call:

   - `call_request.hash == public_call.call_stack_item.hash()`
   - The hash of the `call_stack_item` is computed as:
     - `hash(contract_address, function_data.hash(), public_inputs.hash(), counter_start, counter_end)`
     - Where `function_data.hash()` and `public_inputs.hash()` are the hashes of the serialized field elements.

#### Ensuring this function is called with the correct context.

This section follows the same [process](./private-kernel-inner.mdx#ensuring-this-function-is-called-with-the-correct-context) as outlined in the inner private kernel circuit.

#### Verifying the public function proof.

It verifies that the public function was executed with the provided proof data, verification key, and the public inputs of the VM circuit. The result of the execution is specified in the public inputs, which will be used in subsequent steps to enforce the conditions they must satisfy.

#### Verifying the public inputs of the public function circuit.

It ensures the public function's intention by checking the following in [`public_call`](#publiccall)[`.call_stack_item`](#publiccallstackitem)[`.public_inputs`](#publicfunctionpublicinputs):

- The `header` must match the one in the [`constant_data`](./private-kernel-initial#constantdata).
- If it is a static call (`public_inputs.call_context.is_static_call == true`), it ensures that the function does not induce any state changes by verifying that the following arrays are empty:
  - `note_hashes`
  - `nullifiers`
  - `l2_to_l1_messages`
  - `storage_writes`
  - `unencrypted_log_hashes`

#### Verifying the counters.

It verifies that each value listed below is associated with a legitimate counter.

1. For the [`call_stack_item`](#privatecallstackitem):

   - The `counter_start` and `counter_end` must match those in the `call_request` [popped](#ensuring-the-current-call-matches-the-call-request) from the `public_call_requests` in a previous step.

2. For items in each ordered array in [`call_stack_item`](#publiccallstackitem)[`.public_inputs`](#publicfunctionpublicinputs):

   - The counter of the first item must be greater than the `counter_start` of the current call.
   - The counter of each subsequent item must be greater than the counter of the previous item.
   - The counter of the last item must be less than the `counter_end` of the current call.

   The ordered arrays include:

   - `storage_reads`
   - `storage_writes`

3. For the last `N` non-empty requests in `public_call_requests` within [`public_inputs`](#public-inputs)[`.transient_accumulated_data`](#transientaccumulateddata):

   - The `counter_end` of each request must be greater than its `counter_start`.
   - The `counter_start` of the first request must be greater than the `counter_start` of the `call_stack_item`.
   - The `counter_start` of the second and subsequent requests must be greater than the `counter_end` of the previous request.
   - The `counter_end` of the last request must be less than the `counter_end` of the `call_stack_item`.

   > `N` is the number of non-zero hashes in the `public_call_stack_item_hashes` in [`private_inputs`](#private-inputs)[`.public_call`](#publiccall)[`.public_inputs`](#publicfunctionpublicinputs).

### Validating Public Inputs

#### Verifying the accumulated data.

1. It verifies that the following in the [`accumulated_data`](#accumulateddata) align with their corresponding values in [`public_call`](#publiccall)[`.call_stack_item`](#publiccallstackitem)[`.public_inputs`](#publicfunctionpublicinputs).

   - `note_hashes`
   - `nullifiers`
   - `l2_to_l1_messages`
   - `encrypted_logs_hash`
   - `encrypted_log_preimages_length`
   - `encrypted_note_preimages_hash`
   - `encrypted_note_preimages_length`
   - `old_public_data_tree_snapshot`
   - `new_public_data_tree_snapshot`

#### Verifying the transient accumulated data.

The [`transient_accumulated_data`](./public-kernel-tail#transientaccumulateddata) in this circuit's [`public_inputs`](#public-inputs)\_ includes values from both the previous iterations and the [`public_call`](#publiccall).

For each array in the `transient_accumulated_data`, this circuit verifies that it is populated with the values from the previous iterations, specifically:

- `public_inputs.transient_accumulated_data.ARRAY[0..N] == private_inputs.previous_kernel.public_inputs.transient_accumulated_data.ARRAY[0..N]`

> It's important to note that the top item in the `public_call_requests` from the _previous_kernel_ won't be included, as it has been removed in a [previous step](#ensuring-the-current-call-matches-the-call-request).

For the subsequent items appended after the values from the previous iterations, they constitute the values from [`private_inputs`](#private-inputs).[public_call](#publiccall).[call_stack_item](#publiccallstackitem).[public_inputs](#publicfunctionpublicinputs) (`public_function_public_inputs`), and must undergo the following verifications:

1. Ensure that the specified values in the following arrays match those in the corresponding arrays in the `public_function_public_inputs`:

   - `note_hash_contexts`
     - `value`, `counter`
   - `nullifier_contexts`
     - `value`, `counter`
   - `l2_to_l1_message_contexts`
     - `value`
   - `storage_reads`
     - `value`, `counter`
   - `storage_writes`
     - `value`, `counter`
   - `unencrypted_log_hash_contexts`
     - `hash`, `length`, `counter`

2. For `public_call_requests`:

   - The hashes align with the values in the `public_call_stack_item_hashes` within `public_function_public_inputs`, but in **reverse** order.
   - The `caller_contract_address` equals the `contract_address` in [`public_call`](#publiccall)[`.call_stack_item`](#publiccallstackitem).
   - The `caller_context` aligns with the values in the `call_context` within `public_function_public_inputs`.

   > It's important that the call requests are arranged in reverse order to ensure they are executed in chronological order.

3. The `contract_address` for each non-empty item in the following arrays must equal the `storage_contract_address` defined in `public_function_public_inputs.call_context`:

   - `note_hash_contexts`
   - `nullifier_contexts`
   - `l2_to_l1_message_contexts`
   - `storage_reads`
   - `storage_writes`
   - `unencrypted_log_hash_contexts`

   > Ensuring the alignment of the contract addresses is crucial, as it is later used to [silo the values](./public-kernel-tail#siloing-values) and to establish associations with values within the same contract.

4. The _portal_contract_address_ for each non-empty item in `l2_to_l1_message_contexts` must equal the _portal_contract_address_ defined in _public_function_public_inputs.call_context_.

5. For each `storage_write` in `storage_writes`, verify that it is associated with an _override_counter_. The value of the _override_counter_ can be:

   - Zero: if the `storage_slot` does not change later in the same transaction.
   - Greater than `storage_write.counter`: if the `storage_slot` is written again later in the same transaction.

   > Override counters are used in the [tail public kernel circuit](./public-kernel-tail.md) to ensure a read happens **before** the value is changed in a subsequent write.

   > Zero serves as an indicator for an unchanged update, as this value can never act as the counter of a write.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.mdx#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## `PrivateInputs`

### `PreviousKernel`

The format aligns with the [`PreviousKernel`](./private-kernel-tail.md#previouskernel) of the tail public kernel circuit.

### `PublicCall`

Data that holds details about the current public function call.

| Field                                    | Type                                                                | Description                                                         |
| ---------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `call_stack_item`                        | [`PublicCallStackItem`](#publiccallstackitem)                       | Information about the current public function call.                 |
| `proof`                                  | `Proof`                                                             | Proof of the public function circuit.                               |
| `vk`                                     | `VerificationKey`                                                   | Verification key of the public function circuit.                    |
| `bytecode_hash`                          | `field`                                                             | Hash of the function bytecode.                                      |
| `contract_data`                          | [`ContractInstance`](../contract-deployment/instances.md#structure) | Data of the contract instance being called.                         |
| `contract_class_data`                    | [`ContractClass`](./private-kernel-initial#contractclassdata)   | Data of the contract class.                                         |
| `function_leaf_membership_witness`       | [`MembershipWitness`](./private-kernel-inner.mdx#membershipwitness) | Membership witness for the function being called.                   |
| `contract_deployment_membership_witness` | [`MembershipWitness`](./private-kernel-inner.mdx#membershipwitness) | Membership witness for the deployment of the contract being called. |

## `PublicInputs`

The format aligns with the [`PublicInputs`](./public-kernel-tail#public-inputs) of the tail public kernel circuit.

## Types

### `PublicCallStackItem`

| Field              | Type                                                        | Description                                               |
| ------------------ | ----------------------------------------------------------- | --------------------------------------------------------- |
| `contract_address` | `AztecAddress`                                              | Address of the contract on which the function is invoked. |
| `function_data`    | [`FunctionData`](./private-kernel-initial#functiondata) | Data of the function being called.                        |
| `public_inputs`    | [`PublicFunctionPublicInputs`](#publicfunctionpublicinputs) | Public inputs of the public vm circuit.                   |
| `counter_start`    | `field`                                                     | Counter at which the function call was initiated.         |
| `counter_end`      | `field`                                                     | Counter at which the function call ended.                 |

### `PublicFunctionPublicInputs`

| Field                           | Type                                                                  | Description                                                     |
| ------------------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------- |
| `call_context`                  | [`CallContext`](./private-function.md#callcontext)                    | Context of the call corresponding to this function execution.   |
| `args_hash`                     | `field`                                                               | Hash of the function arguments.                                 |
| `return_values`                 | `[field; C]`                                                          | Return values of this function call.                            |
| `note_hashes`                   | `[`[`NoteHash`](./private-function.md#notehash)`; C]`                 | New note hashes created in this function call.                  |
| `nullifiers`                    | [`[Nullifier; C]`](./private-function.md#nullifier)                   | New nullifiers created in this function call.                   |
| `l2_to_l1_messages`             | `[field; C]`                                                          | New L2 to L1 messages created in this function call.            |
| `storage_reads`                 | [`[StorageRead_; C]`](./public-kernel-tail#storageread)            | Data read from the public data tree.                            |
| `storage_writes`                | [`[StorageWrite; C]`](./public-kernel-tail#storagewrite)           | Data written to the public data tree.                           |
| `unencrypted_log_hashes`        | [`[UnencryptedLogHash; C]`](./private-function.md#unencryptedloghash) | Hashes of the unencrypted logs emitted in this function call.   |
| `public_call_stack_item_hashes` | `[field; C]`                                                          | Hashes of the public function calls initiated by this function. |
| `header`                        | [`Header`](./private-function.md#header)                              | Information about the trees used for the transaction.           |
| `chain_id`                      | `field`                                                               | Chain ID of the transaction.                                    |
| `version`                       | `field`                                                               | Version of the transaction.                                     |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

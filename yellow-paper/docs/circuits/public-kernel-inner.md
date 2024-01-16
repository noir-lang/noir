# Public Kernel Circuit - Inner

## Requirements

In the public kernel iteration, the process involves taking a previous iteration and public call data, verifying their integrity, and preparing the necessary data for subsequent circuits to operate.

### Verification of the Previous Iteration

#### Verifying the previous kernel proof.

It verifies that the previous iteration was executed successfully with the given proof data, verification key, and public inputs, sourced from _[private_inputs](#private-inputs).[previous_kernel](#previouskernel)_.

The preceding proof can be:

- [Initial public kernel proof](./public-kernel-initial.md).
- Inner public kernel proof.

### Processing Public Function Call

#### Ensuring the function being called exists in the contract.

This section follows the same [process](./private-kernel-initial.md#ensuring-the-function-being-called-exists-in-the-contract) as outlined in the initial private kernel circuit.

#### Ensuring the contract instance being called is deployed.

It verifies the public deployment of the contract instance by conducting a membership proof, where:

- The leaf is a nullifier emitting from the deployer contract, computed as _`hash(deployer_address, contract_address)`_, where:
  - _deployer_address_ is defined in _[private_inputs](#private-inputs).[public_call](#publiccall).[contract_data](../contract-deployment/instances.md#structure)_.
  - _contract_data_ is defined in _[private_inputs](#private-inputs).[public_call](#publiccall).[call_stack_item](#publiccallstackitem)_.
- The index and sibling path are provided in _contract_deployment_membership_witness_ through _[private_inputs](#private-inputs).[public_call](#publiccall)_.
- The root is the _nullifier_tree_root_ in the _[block_header](./private-function.md#blockheader)_ within _[public_inputs](#public-inputs).[constant_data](./private-kernel-initial.md#constantdata)_.

#### Ensuring the function is legitimate:

For the _[function_data](./private-kernel-initial.md#functiondata)_ in _[public_call](#publiccall).[call_stack_item](#publiccallstackitem)_, this circuit verifies that:

- It must be a public function:
  - _`function_data.function_type == public`_

#### Ensuring the current call matches the call request.

The top item in the _public_call_requests_ of the _[previous_kernel](#previouskernel)_ must pertain to the current function call.

This circuit will:

1. Pop the request from the stack:

   - _`call_request = previous_kernel.public_inputs.transient_accumulated_data.public_call_requests.pop()`_

2. Compare the hash with that of the current function call:

   - _`call_request.hash == public_call.call_stack_item.hash()`_
   - The hash of the _call_stack_item_ is computed as:
     - _`hash(contract_address, function_data.hash(), public_inputs.hash(), counter_start, counter_end)`_
     - Where _function_data.hash()_ and _public_inputs.hash()_ are the hashes of the serialized field elements.

#### Ensuring this function is called with the correct context.

This section follows the same [process](./private-kernel-inner.md#ensuring-this-function-is-called-with-the-correct-context) as outlined in the inner private kernel circuit.

#### Verifying the public function proof.

It verifies that the public function was executed with the provided proof data, verification key, and the public inputs of the VM circuit. The result of the execution is specified in the public inputs, which will be used in subsequent steps to enforce the conditions they must satisfy.

#### Verifying the public inputs of the public function circuit.

It ensures the public function's intention by checking the following in _[public_call](#publiccall).[call_stack_item](#publiccallstackitem).[public_inputs](#publicfunctionpublicinputs)_:

- The _block_header_ must match the one in the _[constant_data](./private-kernel-initial.md#constantdata)_.
- If it is a static call (_`public_inputs.call_context.is_static_call == true`_), it ensures that the function does not induce any state changes by verifying that the following arrays are empty:
  - _note_hashes_
  - _nullifiers_
  - _l2_to_l1_messages_
  - _storage_writes_
  - _unencrypted_log_hashes_

#### Verifying the counters.

It verifies that each value listed below is associated with a legitimate counter.

1. For the _[call_stack_item](#privatecallstackitem)_:

   - The _counter_start_ and _counter_end_ must match those in the _call_request_ [popped](#ensuring-the-current-call-matches-the-call-request) from the _public_call_requests_ in a previous step.

2. For items in each ordered array in _[call_stack_item](#publiccallstackitem).[public_inputs](#publicfunctionpublicinputs)_:

   - The counter of the first item must be greater than the _counter_start_ of the current call.
   - The counter of each subsequent item must be greater than the counter of the previous item.
   - The counter of the last item must be less than the _counter_end_ of the current call.

   The ordered arrays include:

   - _storage_reads_
   - _storage_writes_

3. For the last _N_ non-empty requests in _public_call_requests_ within _[public_inputs](#public-inputs).[transient_accumulated_data](#transientaccumulateddata)_:

   - The _counter_end_ of each request must be greater than its _counter_start_.
   - The _counter_start_ of the first request must be greater than the _counter_start_ of the _call_stack_item_.
   - The _counter_start_ of the second and subsequent requests must be greater than the _counter_end_ of the previous request.
   - The _counter_end_ of the last request must be less than the _counter_end_ of the _call_stack_item_.

   > _N_ is the number of non-zero hashes in the _public_call_stack_item_hashes_ in _[private_inputs](#private-inputs).[public_call](#publiccall).[public_inputs](#publicfunctionpublicinputs)_.

### Validating Public Inputs

#### Verifying the accumulated data.

1. It verifies that the following in the _[accumulated_data](#accumulateddata)_ align with their corresponding values in _[public_call](#publiccall).[call_stack_item](#publiccallstackitem).[public_inputs](#publicfunctionpublicinputs)_.

   - _note_hashes_
   - _nullifiers_
   - _l2_to_l1_messages_
   - _encrypted_logs_hash_
   - _encrypted_log_preimages_length_
   - _encrypted_note_preimages_hash_
   - _encrypted_note_preimages_length_
   - _old_public_data_tree_snapshot_
   - _new_public_data_tree_snapshot_

#### Verifying the transient accumulated data.

The _[transient_accumulated_data](./public-kernel-tail.md#transientaccumulateddata)_ in this circuit's _[public_inputs](#public-inputs)_ includes values from both the previous iterations and the _[public_call](#publiccall)_.

For each array in the _transient_accumulated_data_, this circuit verifies that it is populated with the values from the previous iterations, specifically:

- _`public_inputs.transient_accumulated_data.ARRAY[0..N] == private_inputs.previous_kernel.public_inputs.transient_accumulated_data.ARRAY[0..N]`_

> It's important to note that the top item in the _public_call_requests_ from the _previous_kernel_ won't be included, as it has been removed in a [previous step](#ensuring-the-current-call-matches-the-call-request).

For the subsequent items appended after the values from the previous iterations, they constitute the values from _[private_inputs](#private-inputs).[public_call](#publiccall).[call_stack_item](#publiccallstackitem).[public_inputs](#publicfunctionpublicinputs)_ (_public_function_public_inputs_), and must undergo the following verifications:

1. Ensure that the specified values in the following arrays match those in the corresponding arrays in the _public_function_public_inputs_:

   - _note_hash_contexts_
     - _value_, _counter_
   - _nullifier_contexts_
     - _value_, _counter_
   - _l2_to_l1_message_contexts_
     - _value_
   - _storage_reads_
     - _value_, _counter_
   - _storage_writes_
     - _value_, _counter_
   - _unencrypted_log_hash_contexts_
     - _hash_, _length_, _counter_

2. For _public_call_requests_:

   - The hashes align with the values in the _public_call_stack_item_hashes_ within _public_function_public_inputs_, but in **reverse** order.
   - The _caller_contract_address_ equals the _contract_address_ in _[public_call](#publiccall).[call_stack_item](#publiccallstackitem)_.
   - The _caller_context_ aligns with the values in the _call_context_ within _public_function_public_inputs_.

   > It's important that the call requests are arranged in reverse order to ensure they are executed in chronological order.

3. The _contract_address_ for each non-empty item in the following arrays must equal the _storage_contract_address_ defined in _public_function_public_inputs.call_context_:

   - _note_hash_contexts_
   - _nullifier_contexts_
   - _l2_to_l1_message_contexts_
   - _storage_reads_
   - _storage_writes_
   - _unencrypted_log_hash_contexts_

   > Ensuring the alignment of the contract addresses is crucial, as it is later used to [silo the values](./public-kernel-tail.md#siloing-values) and to establish associations with values within the same contract.

4. The _portal_contract_address_ for each non-empty item in _l2_to_l1_message_contexts_ must equal the _portal_contract_address_ defined in _public_function_public_inputs.call_context_.

5. For each _storage_write_ in _storage_writes_, verify that it is associated with an _override_counter_. The value of the _override_counter_ can be:

   - Zero: if the _storage_slot_ does not change later in the same transaction.
   - Greater than _storage_write.counter_: if the _storage_slot_ is written again later in the same transaction.

   > Override counters are used in the [tail public kernel circuit](./public-kernel-tail.md) to ensure a read happens **before** the value is changed in a subsequent write.

   > Zero serves as an indicator for an unchanged update, as this value can never act as the counter of a write.

#### Verifying the constant data.

This section follows the same [process](./private-kernel-inner.md#verifying-the-constant-data) as outlined in the inner private kernel circuit.

## Private Inputs

### _PreviousKernel_

The format aligns with the _[PreviousKernel](./private-kernel-tail.md#previouskernel)_ of the tail public kernel circuit.

### _PublicCall_

Data that holds details about the current public function call.

| Field                                    | Type                                                                 | Description                                                         |
| ---------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------- |
| _call_stack_item_                        | _[PublicCallStackItem](#publiccallstackitem)_                        | Information about the current public function call.                 |
| _proof_                                  | _Proof_                                                              | Proof of the public function circuit.                               |
| _vk_                                     | _VerificationKey_                                                    | Verification key of the public function circuit.                    |
| _bytecode_hash_                          | _field_                                                              | Hash of the function bytecode.                                      |
| _contract_data_                          | _[ContractInstance](../contract-deployment/instances.md#structure)_  | Data of the contract instance being called.                         |
| _contract_class_data_                    | _[ContractClassData](./private-kernel-initial.md#contractclassdata)_ | Data of the contract class.                                         |
| _function_leaf_membership_witness_       | _[MembershipWitness](./private-kernel-inner.md#membershipwitness)_   | Membership witness for the function being called.                   |
| _contract_deployment_membership_witness_ | _[MembershipWitness](./private-kernel-inner.md#membershipwitness)_   | Membership witness for the deployment of the contract being called. |

## Public Inputs

The format aligns with the _[Public Inputs](./public-kernel-tail.md#public-inputs)_ of the tail public kernel circuit.

## Types

### _PublicCallStackItem_

| Field              | Type                                                        | Description                                               |
| ------------------ | ----------------------------------------------------------- | --------------------------------------------------------- |
| _contract_address_ | _AztecAddress_                                              | Address of the contract on which the function is invoked. |
| _function_data_    | _[FunctionData](#functiondata)_                             | Data of the function being called.                        |
| _public_inputs_    | _[PublicFunctionPublicInputs](#publicfunctionpublicinputs)_ | Public inputs of the public vm circuit.                   |
| _counter_start_    | _field_                                                     | Counter at which the function call was initiated.         |
| _counter_end_      | _field_                                                     | Counter at which the function call ended.                 |

### _PublicFunctionPublicInputs_

| Field                           | Type                                                                    | Description                                                     |
| ------------------------------- | ----------------------------------------------------------------------- | --------------------------------------------------------------- |
| _call_context_                  | _[CallContext](./private-function.md#callcontext)_                      | Context of the call corresponding to this function execution.   |
| _args_hash_                     | _field_                                                                 | Hash of the function arguments.                                 |
| _return_values_                 | [_field_; _C_]                                                          | Return values of this function call.                            |
| _note_hashes_                   | [_[NoteHash](./private-function.md#notehash)_; _C_]                     | New note hashes created in this function call.                  |
| _nullifiers_                    | [_[Nullifier](./private-function.md#nullifier)_; _C_]                   | New nullifiers created in this function call.                   |
| _l2_to_l1_messages_             | [_field_; _C_]                                                          | New L2 to L1 messages created in this function call.            |
| _storage_reads_                 | [_[StorageRead](./public-kernel-tail.md#storageread)_; _C_]             | Data read from the public data tree.                            |
| _storage_writes_                | [_[StorageWrite](./public-kernel-tail.md#storagewrite)_; _C_]           | Data written to the public data tree.                           |
| _unencrypted_log_hashes_        | [_[UnencryptedLogHash](./private-function.md#unencryptedloghash)_; _C_] | Hashes of the unencrypted logs emitted in this function call.   |
| _public_call_stack_item_hashes_ | [_field_; _C_]                                                          | Hashes of the public function calls initiated by this function. |
| _block_header_                  | _[BlockHeader](#blockheader)_                                           | Information about the trees used for the transaction.           |
| _chain_id_                      | _field_                                                                 | Chain ID of the transaction.                                    |
| _version_                       | _field_                                                                 | Version of the transaction.                                     |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

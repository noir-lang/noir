# Private Kernel Circuit - Initial

## Requirements

In the **initial** kernel iteration, the process involves taking a transaction request and private call data, verifying their integrity, and preparing the necessary data for subsequent circuits to operate. This step is particularly beneficial due to its separation from the [inner private kernel circuit](./private-kernel-inner.md), as the first call lacks a "previous kernel" to process. Additionally, it executes tasks that are pertinent to a transaction and need only occur once.

### Key Responsibilities Specific to this Circuit

#### Validating the correspondence of function call with caller's intent.

This entails ensuring that the following from the _[private_call](#privatecall)_ aligns with the specifications in the _[transaction_request](#transactionrequest)_:

- _contract_address_
- _function_data_
- _args_hash_: Hash of the function arguments.

> Although it's not enforced in the protocol, it is customary to provide a signature signed over the transaction request and verify it in the first function call. This practice guarantees that only the party possessing the key(s) can authorize a transaction with the exact transaction request on behalf of an account.

#### Verifying the legitimacy of the function as the entrypoint.

For the _[function_data](#functiondata)_ in _[private_call](#privatecall).[call_stack_item](#privatecallstackitem)_, the circuit verifies that:

- It must be a private function:
  - _`function_data.function_type == private`_
- It must not be an internal function:
  - _`function_data.is_internal == false`_

#### Ensuring the function call is the first call.

For the _[call_context](./private-function.md#callcontext)_ in _[private_call](#privatecall).[call_stack_item](#privatecallstackitem).[public_inputs](./private-function.md#public-inputs)_, the circuit checks that:

- It must not be a delegate call:
  - _`call_context.is_delegate_call == false`_
- It must not be a static call:
  - _`call_context.is_static_call == false`_

#### Ensuring transaction uniqueness.

It must emit the hash of the _[transaction_request](#transactionrequest)_ as the **first** nullifier.

The hash is computed as:

_`hash(origin, function_data.hash(), args_hash, tx_context.hash())`_

Where _function_data.hash()_ and _tx_context.hash()_ are the hashes of the serialized field elements.

This nullifier serves multiple purposes:

- Identifying a transaction.
- Non-malleability. Preventing the signature of a transaction request from being reused in another transaction.
- Generating values that should be maintained within the transaction's scope. For example, it is utilized to [compute the nonces](./private-kernel-tail.md#siloing-values) for all the note hashes in a transaction.

> Note that the final transaction data is not deterministic for a given transaction request. The production of new notes, the destruction of notes, and various other values are likely to change based on the time and conditions when a transaction is being composed. However, the intricacies of implementation should not be a concern for the entity initiating the transaction.

### Processing Private Function Call

#### Ensuring the function being called exists in the contract.

With the following data provided from _[private_inputs](#private-inputs).[private_call](#privatecall)_:

- _contract_address_ in _private_call.[call_stack_item](#privatecallstackitem)_.
- _contract_data_
- _contract_class_data_
- _function_data_ in _private_call.[call_stack_item](#privatecallstackitem)_.

This circuit validates the existence of the function in the contract through the following checks:

1. Verify that the _contract_address_ can be derived from the _contract_data_:

   Refer to the details [here](../contract-deployment/instances.md#address) for the process of computing the address for a contract instance.

2. Verify that the _contract_data.contract_class_id_ can be derived from the given _contract_class_data_:

   Refer to the details [here](../contract-deployment/classes.md#class-identifier) for the process of computing the _contract_class_id_.

3. Verify that _contract_class_data.private_functions_ includes the function being called:

   1. Compute the hash of the verification key:
      - _`vk_hash = hash(private_call.vk)`_
   2. Compute the function leaf:
      - _`hash(function_data.selector, vk_hash, private_call.bytecode_hash)`_
   3. Perform a membership check on the function leaf, where:
      - The index and sibling path are provided through _function_leaf_membership_witness_ within _[private_call](#privatecall)_.
      - The root is _contract_class_data.private_functions_.

#### Verifying the private function proof.

It verifies that the private function was executed successfully with the provided proof data, verification key, and the public inputs of the [private function circuit](./private-function.md).

#### Verifying the public inputs of the private function circuit.

It ensures the private function circuit's intention by checking the following in _[private_call](#privatecall).[call_stack_item](#privatecallstackitem).[public_inputs](./private-function.md#public-inputs)_:

- The _block_header_ must match the one in the _[constant_data](#constantdata)_.

#### Verifying the counters.

It verifies that each value listed below is associated with a legitimate counter.

1. For the _[call_stack_item](#privatecallstackitem)_:

   - The _counter_start_ must be 0.
     - This check can be skipped for [inner private kernel circuit](./private-kernel-inner.md#verifying-the-counters).
   - The _counter_end_ must be greater than the _counter_start_.

2. For items in each ordered array in _[call_stack_item](#privatecallstackitem).[public_inputs](./private-function.md#public-inputs)_:

   - The _counter_ of the first item must be greater than the _counter_start_ of the _call_stack_item_.
   - The _counter_ of each subsequent item must be greater than the _counter_ of the previous item.
   - The _counter_ of the last item must be less than the _counter_end_ of the _call_stack_item_.

   The ordered arrays include:

   - _note_hashes_
   - _nullifiers_
   - _read_requests_
   - _unencrypted_log_hashes_
   - _encrypted_log_hashes_
   - _encrypted_note_preimage_hashes_

3. For the last _N_ non-empty items in the _private_call_requests_ in the _[transient_accumulated_data](#transientaccumulateddata)_:

   - The _counter_end_ of each request must be greater than its _counter_start_.
   - The _counter_end_ of the first request must be less than the _counter_end_ of the _call_stack_item_.
   - The _counter_end_ of the second and subsequent requests must be less than the _counter_start_ of the previous request.
   - The _counter_start_ of the last request must be greater than the _counter_start_ of the _call_stack_item_.

   > _N_ is the number of non-zero hashes in the _private_call_stack_item_hashes_ in _[private_inputs](#private-inputs).[private_call](#privatecall).[public_inputs](./private-function.md#public-inputs)_.

4. For the last _N_ non-empty items in the _public_call_requests_ in the _[transient_accumulated_data](#transientaccumulateddata)_:

   - The _counter_start_ of the first request must be greater than the _counter_start_ of the _call_stack_item_.
   - The _counter_start_ of each subsequent request must be greater than the _counter_start_ of the previous item.
   - The _counter_start_ of the last item must be less than the _counter_end_ of the _call_stack_item_.

   > _N_ is the number of non-zero hashes in the _public_call_stack_item_hashes_ in _[private_inputs](#private-inputs).[private_call](#privatecall).[public_inputs](./private-function.md#public-inputs)_.

   > Note that the _counter_end_ of public call request is unknown at this point. Both counters will be [recalibrated](./public-kernel-initial.md#recalibrating-counters) in the initial public kernel circuit following the simulation of all public function calls.

> Note that, for the initial private kernel circuit, all values within the _[transient_accumulated_data](#transientaccumulateddata)_ originate from the _[private_call](#privatecall)_. However, this process of counter verification is also applicable to the [inner private kernel circuit](./private-kernel-inner.md#verifying-the-counters), where the _transient_accumulated_data_ comprises values from previous iterations and the current function call. Therefor, only the last _N_ items need to be checked in the above operations.

### Validating Public Inputs

#### Verifying the transient accumulated data.

Within the _[public_inputs](#public-inputs)_, the _[transient_accumulated_data](#transientaccumulateddata)_ encapsulates values reflecting the operations conducted by the _private_call_.

This circuit verifies that the values in _[private_inputs](#private-inputs).[private_call](#privatecall).[call_stack_item](#privatecallstackitem).[public_inputs](./private-function.md#public-inputs)_ (_private_function_public_inputs_) are aggregated into the _public_inputs_ correctly:

1. Ensure that the specified values in the following arrays match those in the corresponding arrays in the _private_function_public_inputs_:

   - _note_hash_contexts_
     - _value_, _counter_
   - _nullifier_contexts_
     - _value_, _counter_
   - _l2_to_l1_message_contexts_
     - _value_
   - _read_request_contexts_
     - _note_hash_, _counter_
   - _public_call_requests_
     - _hash_, _counter_
   - _unencrypted_log_hash_contexts_
     - _hash_, _length_, _counter_
   - _encrypted_log_hash_contexts_
     - _hash_, _length_, _randomness_, _counter_
   - _encrypted_note_preimage_hash_contexts_
     - _hash_, _length_, _counter_, _note_hash_counter_

2. Check that the hashes in the _private_call_requests_ align with the values in the _private_call_stack_item_hashes_ in the _private_function_public_inputs_, but in **reverse** order.

   > It's important that the call requests are arranged in reverse order to ensure they are executed in chronological order.

3. For each non-empty call request in both _private_call_requests_ and _public_call_requests_:

   - The _caller_contract_address_ equals the _contract_address_ in _[private_call](#privatecall).[call_stack_item](#privatecallstackitem)_.
   - The _caller_context_ is either empty or aligns with the values in the _call_context_ within _private_function_public_inputs_.

   > The caller context in a call request may be empty for standard calls. This precaution is crucial to prevent information leakage, particularly as revealing the _msg_sender_ of this private function when calling a public function could pose security risks.

4. For each non-empty item in the following arrays, its _contract_address_ must equal the _storage_contract_address_ defined in _private_function_public_inputs.call_context_:

   - _note_hash_contexts_
   - _nullifier_contexts_
   - _l2_to_l1_message_contexts_
   - _read_request_contexts_
   - _unencrypted_log_hash_contexts_
   - _encrypted_log_hash_contexts_
   - _encrypted_note_preimage_hash_contexts_

   > Ensuring the alignment of the contract addresses is crucial, as it is later used to [silo the values](./private-kernel-tail.md#siloing-values) and to establish associations with values within the same contract.

5. For each non-empty item in _l2_to_l1_message_contexts_, its _portal_contract_address_ must equal the _portal_contract_address_ defined in _private_function_public_inputs.call_context_.

6. For each _note_hash_ in the _note_hash_contexts_, verify that it is associated with a _nullifier_counter_. The value of the _nullifier_counter_ can be:

   - Zero: if the note is not nullified in the same transaction.
   - Greater than _note_hash.counter_: if the note is nullified in the same transaction.

   > Nullifier counters are used in the [reset private kernel circuit](./private-kernel-reset.md#read-request-reset-private-kernel-circuit) to ensure a read happens **before** a transient note is nullified.

   > Zero can be used to indicate a non-existing transient nullifier, as this value can never serve as the counter of a nullifier. It corresponds to the _counter_start_ of the first function call.

> Note that the verification process outlined above is also applicable to the inner private kernel circuit. However, given that the _transient_accumulated_data_ for the inner private kernel circuit comprises both values from previous iterations and the _private_call_, the above process specifically targets the values stemming from the _private_call_. The inner kernel circuit performs an [extra check](./private-kernel-inner.md#verifying-the-transient-accumulated-data) to ensure that the _transient_accumulated_data_ also contains values from the previous iterations.

#### Verifying the constant data.

It verifies that:

- The _tx_context_ in the _[constant_data](#constantdata)_ matches the _tx_context_ in the _[transaction_request](#transactionrequest)_.
- The _block_header_ must align with the one used in the private function circuit, as verified [earlier](#verifying-the-public-inputs-of-the-private-function-circuit).

## Private Inputs

### _TransactionRequest_

Data that represents the caller's intent.

| Field           | Type                                        | Description                                  |
| --------------- | ------------------------------------------- | -------------------------------------------- |
| _origin_        | _AztecAddress_                              | The Aztec address of the transaction sender. |
| _function_data_ | _[FunctionData](#functiondata)_             | Data of the function being called.           |
| _args_hash_     | _field_                                     | Hash of the function arguments.              |
| _tx_context_    | _[TransactionContext](#transactioncontext)_ | Information about the transaction.           |

### _PrivateCall_

Data that holds details about the current private function call.

| Field                              | Type                                                                | Description                                          |
| ---------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------- |
| _call_stack_item_                  | _[PrivateCallStackItem](#privatecallstackitem)_                     | Information about the current private function call. |
| _proof_                            | _Proof_                                                             | Proof of the private function circuit.               |
| _vk_                               | _VerificationKey_                                                   | Verification key of the private function circuit.    |
| _bytecode_hash_                    | _field_                                                             | Hash of the function bytecode.                       |
| _contract_data_                    | _[ContractInstance](../contract-deployment/instances.md#structure)_ | Data of the contract instance being called.          |
| _contract_class_data_              | _[ContractClassData](#contractclassdata)_                           | Data of the contract class.                          |
| _function_leaf_membership_witness_ | _[MembershipWitness](#membershipwitness)_                           | Membership witness for the function being called.    |

## Public Inputs

### _ConstantData_

Data that remains the same throughout the entire transaction.

| Field          | Type                                               | Description                                                   |
| -------------- | -------------------------------------------------- | ------------------------------------------------------------- |
| _block_header_ | _[BlockHeader](./private-function.md#blockheader)_ | Roots of the trees at the time the transaction was assembled. |
| _tx_context_   | _[TransactionContext](#transactioncontext)_        | Context of the transaction.                                   |

### _TransientAccumulatedData_

| Field                                   | Type                                                                    | Description                                                                 |
| --------------------------------------- | ----------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| _note_hash_contexts_                    | [_[NoteHashContext](#notehashcontext)_; _C_]                            | Note hashes with extra data aiding verification.                            |
| _nullifier_contexts_                    | [_[NullifierContext](#nullifiercontext)_; _C_]                          | Nullifiers with extra data aiding verification.                             |
| _l2_to_l1_message_contexts_             | [_[L2toL1MessageContext](#l2tol1messagecontext)_; _C_]                  | L2-to-l1 messages with extra data aiding verification.                      |
| _read_request_contexts_                 | [_[ReadRequestContext](#readrequestcontext)_; _C_]                      | Requests to read notes in the note hash tree.                               |
| _unencrypted_log_hash_contexts_         | [_[EncryptedLogHashContext](#encryptedloghashcontext)_; _C_]            | Hashes of the unencrypted logs with extra data aiding verification.         |
| _encrypted_log_hash_contexts_           | [_[UnencryptedLogHashContext](#unencryptedloghashcontext)_; _C_]        | Hashes of the encrypted logs with extra data aiding verification.           |
| _encrypted_note_preimage_hash_contexts_ | [_[EncryptedNotePreimageHashContext](#encryptednotepreimagehash)_; _C_] | Hashes of the encrypted note preimages with extra data aiding verification. |
| _private_call_requests_                 | [_[CallRequest](#callrequest)_; _C_]                                    | Requests to call private functions.                                         |
| _public_call_requests_                  | [_[CallRequest](#callrequest)_; _C_]                                    | Requests to call publics functions.                                         |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

## Types

#### _FunctionData_

| Field               | Type                               | Description                                         |
| ------------------- | ---------------------------------- | --------------------------------------------------- |
| _function_selector_ | _u32_                              | Selector of the function being called.              |
| _function_type_     | private \| public \| unconstrained | Type of the function being called.                  |
| _is_internal_       | _bool_                             | A flag indicating whether the function is internal. |

#### _ContractClassData_

| Field                     | Type           | Description                                                        |
| ------------------------- | -------------- | ------------------------------------------------------------------ |
| _version_                 | _u8_           | Version identifier.                                                |
| _registerer_address_      | _AztecAddress_ | Address of the canonical contract used for registering this class. |
| _artifact_hash_           | _field_        | Hash of the contract artifact.                                     |
| _private_functions_       | _field_        | Merkle root of the private function tree.                          |
| _public_functions_        | _field_        | Merkle root of the public function tree.                           |
| _unconstrained_functions_ | _field_        | Merkle root of the unconstrained function tree.                    |

#### _TransactionContext_

| Field      | Type                                 | Description                  |
| ---------- | ------------------------------------ | ---------------------------- |
| _tx_type_  | standard \| fee_paying \| fee_rebate | Type of the transaction.     |
| _chain_id_ | _field_                              | Chain ID of the transaction. |
| _version_  | _field_                              | Version of the transaction.  |

#### _PrivateCallStackItem_

| Field              | Type                                                                 | Description                                               |
| ------------------ | -------------------------------------------------------------------- | --------------------------------------------------------- |
| _contract_address_ | _AztecAddress_                                                       | Address of the contract on which the function is invoked. |
| _function_data_    | _[FunctionData](#functiondata)_                                      | Data of the function being called.                        |
| _public_inputs_    | _[PrivateFunctionPublicInputs](./private-function.md#public-inputs)_ | Public inputs of the private function circuit.            |
| _counter_start_    | _field_                                                              | Counter at which the function call was initiated.         |
| _counter_end_      | _field_                                                              | Counter at which the function call ended.                 |

#### _CallRequest_

| Field             | Type                              | Description                                   |
| ----------------- | --------------------------------- | --------------------------------------------- |
| _hash_            | _field_                           | Hash of the call stack item.                  |
| _caller_contract_ | _AztecAddress_                    | Address of the contract calling the function. |
| _caller_context_  | _[CallerContext](#callercontext)_ | Context of the contract calling the function. |
| _counter_start_   | _field_                           | Counter at which the call was initiated.      |
| _counter_end_     | _field_                           | Counter at which the call ended.              |

#### _CallerContext_

| Field              | Type           | Description                                      |
| ------------------ | -------------- | ------------------------------------------------ |
| _msg_sender_       | _AztecAddress_ | Address of the caller contract.                  |
| _storage_contract_ | _AztecAddress_ | Storage contract address of the caller contract. |

#### _NoteHashContext_

| Field               | Type           | Description                                              |
| ------------------- | -------------- | -------------------------------------------------------- |
| _value_             | _field_        | Hash of the note.                                        |
| _counter_           | _field_        | Counter at which the note hash was created.              |
| _nullifier_counter_ | _field_        | Counter at which the nullifier for the note was created. |
| _contract_address_  | _AztecAddress_ | Address of the contract the note was created.            |

#### _NullifierContext_

| Field               | Type           | Description                                                                                                              |
| ------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------ |
| _value_             | _field_        | Value of the nullifier.                                                                                                  |
| _counter_           | _field_        | Counter at which the nullifier was created.                                                                              |
| _note_hash_counter_ | _field_        | Counter of the transient note the nullifier is created for. 0 if the nullifier does not associate with a transient note. |
| _contract_address_  | _AztecAddress_ | Address of the contract the nullifier was created.                                                                       |

#### _L2toL1MessageContext_

| Field                     | Type           | Description                                      |
| ------------------------- | -------------- | ------------------------------------------------ |
| _value_                   | _field_        | L2-to-l2 message.                                |
| _portal_contract_address_ | _AztecAddress_ | Address of the portal contract to the contract.  |
| _contract_address_        | _AztecAddress_ | Address of the contract the message was created. |

#### _ReadRequestContext_

| Field              | Type           | Description                                   |
| ------------------ | -------------- | --------------------------------------------- |
| _note_hash_        | _field_        | Hash of the note to be read.                  |
| _counter_          | _field_        | Counter at which the request was made.        |
| _contract_address_ | _AztecAddress_ | Address of the contract the request was made. |

#### _UnencryptedLogHashContext_

| Field              | Type           | Description                                  |
| ------------------ | -------------- | -------------------------------------------- |
| _hash_             | _field_        | Hash of the unencrypted log.                 |
| _length_           | _field_        | Number of fields of the log preimage.        |
| _counter_          | _field_        | Counter at which the hash was emitted.       |
| _contract_address_ | _AztecAddress_ | Address of the contract the log was emitted. |

#### _EncryptedLogHashContext_

| Field              | Type           | Description                                  |
| ------------------ | -------------- | -------------------------------------------- |
| _hash_             | _field_        | Hash of the encrypted log.                   |
| _length_           | _field_        | Number of fields of the log preimage.        |
| _randomness_       | _field_        | A random value to hide the contract address. |
| _counter_          | _field_        | Counter at which the hash was emitted.       |
| _contract_address_ | _AztecAddress_ | Address of the contract the log was emitted. |

#### _EncryptedNotePreimageHashContext_

| Field               | Type           | Description                                  |
| ------------------- | -------------- | -------------------------------------------- |
| _hash_              | _field_        | Hash of the encrypted note preimage.         |
| _length_            | _field_        | Number of fields of the note preimage.       |
| _note_hash_counter_ | _field_        | Counter of the corresponding note hash.      |
| _counter_           | _field_        | Counter at which the hash was emitted.       |
| _contract_address_  | _AztecAddress_ | Address of the contract the log was emitted. |

#### _MembershipWitness_

| Field          | Type         | Description                           |
| -------------- | ------------ | ------------------------------------- |
| _leaf_index_   | _field_      | Index of the leaf in the tree.        |
| _sibling_path_ | [_field_; H] | Sibling path to the leaf in the tree. |

> **H** represents the height of the tree.

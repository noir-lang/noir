# Private Function Circuit

## Requirements

Private function circuits represent smart contract functions that modify the Aztec private state trees. They serve as untrusted, third-party code that is executed as part of evaluating an Aztec transaction.

The logic of each private function circuit is tailored to the needs of a particular application or scenario, yet its public inputs must adhere to a specific format. This circuit should be designed to handle private data processing while generating public inputs that safeguard the application and account's intentions without compromising sensitive information.

## Private Inputs

The private inputs of a private function circuit are customizable.

## Public Inputs

The public inputs of a private function circuit will be incorporated into the private inputs of a private kernel circuit. Private kernel circuits leverage these public inputs, coupled with proof data and verification key from a private function circuit, to prove the correct execution of a private function.

The following format defines the ABI that is used by the private kernel circuit when processing private function public inputs:

| Field                            | Type                                                             | Description                                                           |
| -------------------------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------- |
| _call_context_                   | _[CallContext](#callcontext)_                                    | Context of the call corresponding to this function execution.         |
| _args_hash_                      | _field_                                                          | Hash of the function arguments.                                       |
| _return_values_                  | [_field_; _C_]                                                   | Return values of this function call.                                  |
| _read_requests_                  | [_[ReadRequest](#readrequest)_; _C_]                             | Requests to read notes in the note hash tree.                         |
| _note_hashes_                    | [_[NoteHash](#notehash)_; _C_]                                   | New note hashes created in this function call.                        |
| _nullifiers_                     | [_[Nullifier](#nullifier)_; _C_]                                 | New nullifiers created in this function call.                         |
| _l2_to_l1_messages_              | [_field_; _C_]                                                   | New L2 to L1 messages created in this function call.                  |
| _unencrypted_log_hashes_         | [_[UnencryptedLogHash](#unencryptedloghash)_; _C_]               | Hashes of the unencrypted logs emitted in this function call.         |
| _encrypted_log_hashes_           | [_[EncryptedLogHash](#encryptedloghash)_; _C_]                   | Hashes of the encrypted logs emitted in this function call.           |
| _encrypted_note_preimage_hashes_ | [_[EncryptedNotePreimageHash](#encryptednotepreimagehash)_; _C_] | Hashes of the encrypted note preimages emitted in this function call. |
| _private_call_stack_item_hashes_ | [_field_; _C_]                                                   | Hashes of the private function calls initiated by this function.      |
| _public_call_stack_item_hashes_  | [_field_; _C_]                                                   | Hashes of the public function calls initiated by this function.       |
| _block_header_                   | _[BlockHeader](#blockheader)_                                    | Information about the trees used for the transaction.                 |
| _chain_id_                       | _field_                                                          | Chain ID of the transaction.                                          |
| _version_                        | _field_                                                          | Version of the transaction.                                           |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

## Types

#### _CallContext_

| Field                      | Type           | Description                                                             |
| -------------------------- | -------------- | ----------------------------------------------------------------------- |
| _msg_sender_               | _AztecAddress_ | Address of the caller contract.                                         |
| _storage_contract_address_ | _AztecAddress_ | Address of the contract against which all state changes will be stored. |
| _portal_contract_address_  | _AztecAddress_ | Address of the portal contract to the storage contract.                 |
| _is_delegate_call_         | _bool_         | A flag indicating whether the call is a delegate call.                  |
| _is_static_call_           | _bool_         | A flag indicating whether the call is a static call.                    |

#### _ReadRequest_

| Field       | Type    | Description                            |
| ----------- | ------- | -------------------------------------- |
| _note_hash_ | _field_ | Hash of the note to be read.           |
| _counter_   | _field_ | Counter at which the request was made. |

#### _NoteHash_

| Field     | Type    | Description                                 |
| --------- | ------- | ------------------------------------------- |
| _value_   | _field_ | Hash of the note.                           |
| _counter_ | _field_ | Counter at which the note hash was created. |

#### _Nullifier_

| Field               | Type    | Description                                                                                                              |
| ------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------ |
| _value_             | _field_ | Value of the nullifier.                                                                                                  |
| _counter_           | _field_ | Counter at which the nullifier was created.                                                                              |
| _note_hash_counter_ | _field_ | Counter of the transient note the nullifier is created for. 0 if the nullifier does not associate with a transient note. |

#### _UnencryptedLogHash_

| Field     | Type    | Description                            |
| --------- | ------- | -------------------------------------- |
| _hash_    | _field_ | Hash of the unencrypted log.           |
| _length_  | _field_ | Number of fields of the log preimage.  |
| _counter_ | _field_ | Counter at which the hash was emitted. |

#### _EncryptedLogHash_

| Field        | Type    | Description                                  |
| ------------ | ------- | -------------------------------------------- |
| _hash_       | _field_ | Hash of the encrypted log.                   |
| _length_     | _field_ | Number of fields of the log preimage.        |
| _randomness_ | _field_ | A random value to hide the contract address. |
| _counter_    | _field_ | Counter at which the hash was emitted.       |

#### _EncryptedNotePreimageHash_

| Field               | Type    | Description                             |
| ------------------- | ------- | --------------------------------------- |
| _hash_              | _field_ | Hash of the encrypted note preimage.    |
| _length_            | _field_ | Number of fields of the note preimage.  |
| _counter_           | _field_ | Counter at which the hash was emitted.  |
| _note_hash_counter_ | _field_ | Counter of the corresponding note hash. |

#### _BlockHeader_

| Field                         | Type    | Description                                                                                     |
| ----------------------------- | ------- | ----------------------------------------------------------------------------------------------- |
| _note_hash_tree_root_         | _field_ | Root of the note hash tree.                                                                     |
| _nullifier_tree_root_         | _field_ | Root of the nullifier tree.                                                                     |
| _l1_to_l2_messages_tree_root_ | _field_ | Root of the l1-to-l2 messages tree.                                                             |
| _public_data_tree_root_       | _field_ | Root of the public data tree.                                                                   |
| _archive_tree_root_           | _field_ | Root of the state roots tree archived at the block prior to when the transaction was assembled. |
| _global_variables_hash_       | _field_ | Hash of the previous global variables.                                                          |

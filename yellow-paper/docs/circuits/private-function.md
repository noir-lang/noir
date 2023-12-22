# Private Function Circuit

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

## Requirements

A private function circuit is a custom circuit tailored to the needs of a specific application. This circuit should be designed to handle private data processing while generating public inputs that safeguard the application and account's intentions without compromising sensitive information.

The logic of this circuit is flexible, yet its public inputs must adhere to a specific format.

## Private Inputs

The private inputs of a private function circuit are customizable.

## Public Inputs

The public inputs of a private function circuit will be incorporated into the private inputs of a private kernel circuit. Private kernel circuits leverage these public inputs, coupled with proof data and verification key from a private function circuit, to prove the correct execution of a private function.

It must adhere to the following format:

| Field                              | Type                       | Description                                                            |
| ---------------------------------- | -------------------------- | ---------------------------------------------------------------------- |
| _call_context_                     | _CallContext_              | Context of the call corresponding to this function execution.          |
| _args_hash_                        | _field_                    | Hash of the function arguments.                                        |
| _return_values_                    | [_field_; C]               | Return values of this function call.                                   |
| _read_requests_                    | [_ReadRequest_; C]         | Requests to read a note in the note hash tree.                         |
| _note_hash_contexts_               | [_NoteHashContext_; C]     | New note hashes created in this function call.                         |
| _nullifier_contexts_               | [_NullifierContext_; C]    | New nullifiers created in this function call.                          |
| _l2_to_l1_msg_contexts_            | [_L2L1MessageContext; C]   | New L2 to L1 messages created in this function call.                   |
| _new_contract_contexts_            | [_ContractDataContext_; C] | Data of contracts deployed in this function call.                      |
| _encrypted_logs_hash_              | [_field_; N]               | Hash of the encrypted logs emitted in this function call.              |
| _unencrypted_logs_hash_            | [_field_; N]               | Hash of the unencrypted logs emitted in this function call.            |
| _encrypted_log_preimages_length_   | [_field_; N]               | Length of the encrypted log preimages emitted in this function call.   |
| _unencrypted_log_preimages_length_ | [_field_; N]               | Length of the unencrypted log preimages emitted in this function call. |
| _private_call_stack_hashes_        | [_field_; C]               | Hashes of the private function calls initiated by this function.       |
| _public_call_stack_hashes_         | [_field_; C]               | Hashes of the public function calls initiated by this function.        |
| _block_header_                     | _BlockHeader_              | Information about the trees used for the transaction.                  |
| _chain_id_                         | _field_                    | Chain ID of the transaction.                                           |
| _version_                          | _field_                    | Version of the transaction.                                            |

> The above **C**s represent constants defined by the protocol. Each **C** might have a different value from the others.

> The above **N**s represent the number of _field_ of a hash. Its value depends on the hash function chosen by the protocol.

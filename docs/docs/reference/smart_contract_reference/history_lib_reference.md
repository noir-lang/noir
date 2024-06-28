---
title: History Reference
sidebar_position: 3
---

<!-- Note: This will soon be moved into an Aztec.nr reference category under Aztec.nr smart contracts -->

## Note inclusion

Note inclusion proves that a note existed (its hash was included in a note hash tree) in a block header.

### prove_note_inclusion

`prove_note_inclusion` takes 1 parameter:

| Name | Type | Description                            |
| ---- | ---- | -------------------------------------- |
| note | Note | The note you are proving inclusion for |

#### Example

#include_code prove_note_inclusion noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Note validity

This proves that a note exists and has not been nullified in a specific block header.

### prove_note_validity

`prove_note_validity` takes 2 parameters:

| Name             | Type           | Description                            |
| ---------------- | -------------- | -------------------------------------- |
| note_with_header | Note           | The note you are proving inclusion for |
| context          | PrivateContext | Private context                        |

#### Example

#include_code prove_note_validity noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Nullifier inclusion

This proves that a nullifier exists in a given block header (can be used to prove that a note had been nullified).

### prove_nullifier_inclusion

`prove_nullifier_inclusion` takes 1 parameter:

| Name      | Type  | Description                                 |
| --------- | ----- | ------------------------------------------- |
| nullifier | Field | The nullifier you are proving inclusion for |

#### Example

#include_code prove_nullifier_inclusion noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

### prove_note_is_nullified

Instead of passing the nullifier, you can check that a note has been nullified by passing the note.

#### Implementation

#include_code prove_note_is_nullified noir-projects/aztec-nr/aztec/src/history/nullifier_inclusion.nr rust

## Nullifier non inclusion

This proves that a nullifier was not included in a certain block, given the block header (can be used to prove that a note had not yet been nullified in a given block).

### prove_nullifier_non_inclusion

`prove_nullifier_non_inclusion` takes 1 parameters:

| Name      | Type  | Description                                 |
| --------- | ----- | ------------------------------------------- |
| nullifier | Field | The nullifier you are proving inclusion for |

#### Example

#include_code prove_nullifier_non_inclusion noir-projects/aztec-nr/aztec/src/history/contract_inclusion.nr rust

### prove_note_not_nullified

Instead of passing the nullifier, you can check that a note has not been nullified by passing the note.

#### Implementation

#include_code prove_note_not_nullified noir-projects/aztec-nr/aztec/src/history/nullifier_non_inclusion.nr rust

## Public storage historical reads

These return the value stored in a public storage slot of a given contract at the end of the execution of a certain block (the latest one if using `public_storage_historical_read`).

Note that it is never possible to read the _current_ value in a public storage slot in private since private execution is local and by definition always works on _historical_ state.

### public_storage_historical_read

`public_storage_historical_read` takes 2 parameters:

| Name             | Type         | Description                             |
| ---------------- | ------------ | --------------------------------------- |
| storage_slot     | Field        | Storage slot                            |
| contract_address | AztecAddress | The contract that owns the storage slot |

#### Example

#include_code public_storage_historical_read noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Contract inclusion

This proves that a contract exists in, ie had been deployed before or in, a certain block.

### prove_contract_deployment

`prove_contract_deployment` takes 1 parameter:

| Name             | Type         | Description                                 |
| ---------------- | ------------ | ------------------------------------------- |
| contract_address | AztecAddress | The contract address to prove deployment of |

#### Example

#include_code prove_contract_deployment noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

### prove_contract_non_deployment

`prove_contract_non_deployment` takes 1 parameter:

| Name             | Type         | Description                                     |
| ---------------- | ------------ | ----------------------------------------------- |
| contract_address | AztecAddress | The contract address to prove non-deployment of |

#### Example

#include_code prove_contract_non_deployment noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

### prove_contract_initialization

`prove_contract_initialization` takes 1 parameter:

| Name             | Type         | Description                                     |
| ---------------- | ------------ | ----------------------------------------------- |
| contract_address | AztecAddress | The contract address to prove initialization of |

#### Example

#include_code prove_contract_initialization noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

### prove_contract_non_initialization

`prove_contract_non_initialization` takes 1 parameter:

| Name             | Type         | Description                                         |
| ---------------- | ------------ | --------------------------------------------------- |
| contract_address | AztecAddress | The contract address to prove non-initialization of |

#### Example

#include_code prove_contract_non_initialization noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

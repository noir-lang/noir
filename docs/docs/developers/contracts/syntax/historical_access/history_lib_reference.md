---
title: History Reference
---

<!-- Note: This will soon be moved into an Aztec.nr reference category under Aztec.nr smart contracts -->

## Note inclusion 

Note inclusion proves that a note existed (its hash was included in a note hash tree) at a specific block number.

## prove_note_inclusion

`prove_note_inclusion` takes 4 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_interface  | NoteInterface<Note, N> | Interface for the note with necessary functionality|
| note_with_header| Note                   | The note you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

## prove_note_commitment_inclusion

A **commitment**, also referred to as a **note hash** is a public acknowledgment of the existence of a note without revealing the content of the note. You can learn more about how to compress a note to a note hash [here](../../../../learn/concepts/storage/trees/main.md#example-note).

`prove_note_commitment_inclusion` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| commitment  | Field | Note commitment we are checking inclusion of |   
| block_number    | u32                    | Block number for proving note's existence           |
| context| PrivateContext                   | Private Context |    

## Note validity

This proves that a note exists and has not been nullified at a specified block.

### prove_note_validity

`prove_note_validity` takes 4 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_interface  | NoteInterface<Note, N> | Interface for the note with necessary functionality|
| note_with_header| Note                   | The note you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

## Nullifier inclusion

This proves that a nullifier was included in a certain block (can be used to prove that a note had been nullified). 

### prove_nullifier_inclusion

`prove_nullifier_inclusion` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

## Nullifier non inclusion

This proves that a nullifier was not included in a certain block (can be used to prove that a note had not yet been nullified in a given block).

### prove_nullifier_non_inclusion

`prove_nullifier_non_inclusion` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |
                           
 
### note_not_nullified

Instead of passing the nullifier, you can check that a note has not been nullified by passing the note.

## Public value inclusion

This proves that a public value exists at a certain block.

### prove_public_value_inclusion

`prove_public_value_inclusion` takes 4 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| value | Field                   | The public value you are proving inclusion for             |
| storage_slot    | Field                    | Storage slot the value exists in          |
| block_number         | u32         | Block number for proving value's existence     |
| context         | PrivateContext         | Private context     |

## Contract inclusion

This proves that a contract exists in, ie had been deployed before or in, a certain block.

### prove_contract_inclusion

`prove_contract_inclusion` takes 7 parameters:

| Name                      | Type            | Description                                           |
|---------------------------|-----------------|-------------------------------------------------------|
| deployer_public_key       | GrumpkinPoint   | Public key of the contract deployer                   |
| contract_address_salt     | Field           | Unique identifier for the contract's address          |
| function_tree_root        | Field           | Root of the contract's function tree                  |
| constructor_hash          | Field           | Hash of the contract's constructor                    |
| portal_contract_address   | EthAddress      | Ethereum address of the associated portal contract             |
| block_number              | u32             | Block number for proof verification                   |
| context                   | PrivateContext  | Private context                    |

If there is no associated portal contract, you can use a zero Ethereum address:

```ts
new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES));
```

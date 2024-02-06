---
title: History Reference
---

<!-- Note: This will soon be moved into an Aztec.nr reference category under Aztec.nr smart contracts -->

## Note inclusion 

Note inclusion proves that a note existed (its hash was included in a note hash tree) at a specific block number. There exists a version that tests for note inclusion at current block number. It is recommended to use this version whenever possible to reduce cost.

### prove_note_inclusion

`prove_note_inclusion_at` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_with_header| Note                   | The note you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

`prove_note_inclusion` takes 2 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_with_header| Note                   | The note you are proving inclusion for             |
| context         | PrivateContext         | Private context     |

## Note validity

This proves that a note exists and has not been nullified at a specified block. Again as above, there exists a version that tests for validity at current block. It is recommended to use this version whenever possible to reduce cost.

### prove_note_validity

`prove_note_validity_at` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_with_header| Note                   | The note you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

`prove_note_validity` takes 2 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| note_with_header| Note                   | The note you are proving inclusion for             |
| context         | PrivateContext         | Private context     |

## Nullifier inclusion

This proves that a nullifier was included in a certain block (can be used to prove that a note had been nullified). The same disclaimer above holds true for this, and subsequent functions that specify another version without a block_number argument.

### prove_nullifier_inclusion

`prove_nullifier_inclusion_at` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

`prove_nullifier_inclusion` takes 2 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| context         | PrivateContext         | Private context     |

### prove_note_is_nullified_at / prove_note_is_nullified

Instead of passing the nullifier, you can check that a note has been nullified by passing the note.

## Nullifier non inclusion

This proves that a nullifier was not included in a certain block (can be used to prove that a note had not yet been nullified in a given block).

### prove_nullifier_not_included

`prove_nullifier_not_included_at` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| block_number    | u32                    | Block number for proving note's existence           |
| context         | PrivateContext         | Private context     |

`prove_nullifier_not_included` takes 2 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| nullifier | Field                   | The nullifier you are proving inclusion for             |
| context         | PrivateContext         | Private context     |

### prove_note_not_nullified_at / prove_note_not_nullified

Instead of passing the nullifier, you can check that a note has not been nullified by passing the note.

## Public value inclusion

This proves that a public value exists at a certain block.

### prove_public_value_inclusion

`prove_public_value_inclusion_at` takes 4 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| value | Field                   | The public value you are proving inclusion for             |
| storage_slot    | Field                    | Storage slot the value exists in          |
| block_number         | u32         | Block number for proving value's existence     |
| context         | PrivateContext         | Private context     |

`prove_public_value_inclusion` takes 3 parameters:

| Name            | Type                   | Description                                         |
|-----------------|------------------------|-----------------------------------------------------|
| value | Field                   | The public value you are proving inclusion for             |
| storage_slot    | Field                    | Storage slot the value exists in          |
| context         | PrivateContext         | Private context     |

## Contract inclusion

This proves that a contract exists in, ie had been deployed before or in, a certain block.

### prove_contract_inclusion

`prove_contract_inclusion_at` takes 7 parameters:

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

---
title: Using the Archive Tree
sidebar_position: 4
tags: [contracts]
---

The Aztec Protocol uses an append-only Merkle tree to store hashes of the headers of all previous blocks in the chain as its leaves. This is known as the Archive tree. You can learn more about how it works in the [concepts section](../../../../aztec/concepts/storage/trees/index.md).

This page is a quick introductory guide to creating historical proofs proofs from the archive tree.

For a reference, go [here](../../../../reference/developer_references/smart_contract_reference/aztec-nr/aztec/history/contract_inclusion.md).

## Inclusion and non-inclusion proofs

Inclusion and non-inclusion proofs refer to proving the inclusion (or absence) of a specific piece of information within a specific Aztec block with a block header. You can prove any of the following at a given block height before the current height:

- Note inclusion
- Nullifier inclusion
- Note validity
- Existence of public value
- Contract inclusion

Using this library, you can check that specific notes or nullifiers were part of Aztec network state at specific blocks. This can be useful for things such as:

- Verifying a minimum timestamp from a private context
- Checking eligibility based on historical events (e.g. for an airdrop by proving that you knew the nullifier key for a note)
- Verifying historic ownership / relinquishing of assets
- Proving existence of a value in public data tree at a given contract slot
- Proving that a contract was deployed in a given block with some parameters

**In this guide you will learn how to**

- Prove a note was included in a specified block
- Create a nullifier and prove it was not included in a specified block

## Create a note to prove inclusion of

In general you will likely have the note you want to prove inclusion of. But if you are just experimenting you can create a note with a function like below:

#include_code create_note noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Get the note from the PXE

Retrieve the note from the user's PXE.

#include_code get_note_from_pxe noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

In this example, the user's notes are stored in a map called `private_values`. We retrieve this map, then select 1 note from it with the value of `1`.

## Prove that a note was included in a specified block

To prove that a note existed in a specified block, call `prove_note_inclusion` on the `header` as shown in this example:

#include_code prove_note_inclusion noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

Here, if `block_number` exists as an argument, it will prove inclusion in that block. Else, it will use the current block.

This will only prove the note existed at the specific block number, not whether or not the note has been nullified. You can prove that a note existed and had not been nullified in a specified block by using `prove_note_validity` on the block header which takes the following arguments:

#include_code prove_note_validity noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Create a nullifier to prove inclusion of

You can easily nullify a note like so:

#include_code nullify_note noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

This function gets a note from the PXE and nullifies it with `remove()`.

You can then compute this nullifier with `note.compute_nullifier(&mut context)`.

## Prove that a nullifier was included in a specified block

Call `prove_nullifier_inclusion` on a block header like so:

#include_code prove_nullifier_inclusion noir-projects/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

It takes the nullifier as an argument.

You can also prove that a note was not nullified in a specified block by using `prove_note_not_nullified` which takes the note and a reference to the private context.

## Prove contract inclusion, public value inclusion, and use current state in lookups

To see what else you can do with historical proofs, check out the [reference](../../../../reference/developer_references/smart_contract_reference/aztec-nr/aztec/history/contract_inclusion.md)

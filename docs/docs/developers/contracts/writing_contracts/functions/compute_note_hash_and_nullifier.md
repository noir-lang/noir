---
title: How to define compute_note_hash_and_nullifier
---

Aztec will automatically compute and manage notes and nullifiers that are created in smart contracts. However, in some cases, it might make sense to write custom logic for how these are computed. This is achieved through the `compute_note_hash_and_nullifier()` function, which tells the [PXE](../../../../learn/concepts/pxe/main.md) how to handle notes in your smart contract.

## Params and returns

The function should take 5 parameters:

* Contract address
* Nonce
* Storage slot
* Note type ID
* Serialiazed note

It should return `pub [Field; 4]` which is an array of 4 elements that tells the PXE how to handle the notes and nullifiers:

#include_code compute_note_hash_and_nullifier_returns noir-projects/aztec-nr/aztec/src/note/utils.nr rust

## Placeholder

If you don't have any private state variables defined, you can use this placeholder function:

#include_code compute_note_hash_and_nullifier_placeholder /noir-projects/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

## When using notes
 
If you are using custom notes or note types that come with Aztec.nr, you can call the util function `compute_note_hash_and_nulilfier` from the `aztec::utils` library in Aztec.nr. This will return the array needed.

This function takes:

#include_code compute_note_hash_and_nullifier_args /noir-projects/aztec-nr/aztec/src/note/utils.nr rust

Here is an example from the [token contract](../../../tutorials/writing_token_contract.md):

#include_code compute_note_hash_and_nullifier /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust


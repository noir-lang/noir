---
title: How to prove existence of historical notes and nullifiers
---

The Aztec Protocol uses an append-only Merkle tree to store hashes of the headers of all previous blocks in the chain as its leaves. This is known as an archive tree. You can learn more about how it works in the [concepts section](../../../../learn/concepts/storage/trees/main.md#archive-tree).

# History library

The history library allows you to prove any of the following at a given block height before the current height:
* [Note inclusion](./history_lib_reference.md#note-inclusion)
* [Nullifier inclusion](./history_lib_reference.md#nullifier-inclusion)
* [Note validity](./history_lib_reference.md#note-validity)
* [Existence of public value](./history_lib_reference.md#public-value-inclusion)
* [Contract inclusion](./history_lib_reference.md#contract-inclusion)

Using this library, you can check that specific notes or nullifiers were part of Aztec network state at specific blocks. This can be useful for things such as:

* Verifying a minimum timestamp from a private context
* Checking eligibility based on historical events (e.g. for an airdrop by proving that you owned a note) 
* Verifying historic ownership / relinquishing of assets
* Proving existence of a value in public data tree at a given contract slot
* Proving that a contract was deployed in a given block with some parameters

**In this guide you will learn how to**
* Prove a note was included in a specified block
* Create a nullifier and prove it was not included in a specified block

For a more extensive reference, go to [the reference page](./history_lib_reference.md).

## 1. Import the `history` library from `aztec`

```rust
aztec::{
    #include_code imports yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr raw
}
```

This imports all functions from the `history` library. You should only import the functions you will use in your contract.

## 2. Create a note to prove inclusion of

In general you will likely have the note you want to prove inclusion of. But if you are just experimenting you can create a note with a function like below:

#include_code create_note yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## 3. Get the note from the PXE

Retrieve the note from the user's PXE.

#include_code get_note_from_pxe yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

In this example, the user's notes are stored in a map called `private_values`. We retrieve this map, then select 1 note from it with the value of `1`.

## 4. Prove that a note was included in a specified block

To prove that a note existed in a specified block, call `prove_note_inclusion` as shown in this example: 

#include_code prove_note_inclusion yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

This function takes in 4 arguments:

1. The note interface (`ValueNoteMethods`)
2. The note (`maybe_note.unwrap_unchecked()`). Here, `unwrap_unchecked()` returns the inner value without asserting `self.is_some()`
3. The block number
4. Private context

Note: for this to work, you will need to import `ValueNoteMethods` at the beginning of the contract:

#include_code value_note_imports yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

This will only prove the note existed, not whether or not the note has been nullified. You can prove that a note existed and had not been nullified in a specified block by using `prove_note_validity` which takes the same arguments:

#include_code prove_note_validity yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## 5. Create a nullifier to prove inclusion of

You can easily nullify a note like so:

#include_code nullify_note yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

This function gets a note from the PXE like we did in [step 3](#3-get-the-note-from-the-pxe) and nullifies it with `remove()`.

You can then compute this nullifier with `note.compute_nullifier(&mut context)`.

## 6. Prove that a nullifier was included in a specified block

Call `prove_nullifier_inclusion` like so:

#include_code prove_nullifier_inclusion yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

This takes three arguments: 
1. The nullifier
2. Block number
3. Private context

You can also prove that a nullifier was not included in a specified block by using `prove_nullifier_non_inclusion` which takes the same arguments:

#include_code prove_nullifier_non_inclusion yarn-project/noir-contracts/contracts/inclusion_proofs_contract/src/main.nr rust

## Prove contract inclusion, public value inclusion, and note commitment inclusion

To see what else you can do with the `history` library, check out the [reference](./history_lib_reference.md).

---
title: Trees
---

import Disclaimer from "../../../misc/common/\_disclaimer.mdx";
import Image from "@theme/IdealImage";

<Disclaimer/>

## Private State Tree

The private state tree is an append-only tree, primarily designed for storing private state variables.

Each leaf value in the tree is a 254-bit altBN-254 scalar field element. Since this tree is primarily intended to keep data private, this leaf value will often be a cryptographic commitment to some larger blob of data. In Aztec, we call any such blob of data a "Note".

Any function of any Aztec contract may insert new leaves into the this tree.

Once inserted into this tree, a leaf's value can never be modified. We enforce this, to prevent linkability of transactions. If an observer sees that 'tx A' inserted a leaf and 'tx B' modified that leaf, then the observer knows these two transactions are related in some way. This is a big 'no no' if we want to ensure privacy.

So, if an app needs to edit a private state variable (which will be represented by one or more leaves in the tree), it may do so in a manner inspired by [zerocash](http://zerocash-project.org/media/pdf/zerocash-extended-20140518.pdf). (See Nullifier Tree, further down this page). This allows the leaf to be 'nullified' and a new leaf value inserted into the next empty position in the tree, in a way which prevents observers from linking the old and new leaves.

<Image img={require("/img/private-data-tree.png")} />

<!-- TODO: consider separating Note and Nullifier examples into their own doc, because it's actually a very large topic in itself, and can be expanded upon in much more detail than shown here. -->

### Example Note

An example blob of data might be defined in an Aztec.nr Contract as:

```rust
struct MyNote {
    storage_slot: Field, // determined by the Aztec.nr Contract
    value: Field,
    owner_public_key: Point, // The owner of this private state
                             // (and the person who may edit it).
    randomness: Field,
}
```

The note might be committed-to, within a function of the Aztec.nr Contract as:

```rust
note_hash: Field = pedersen::compress(
    storage_slot,
    value,
    owner_public_key.x,
    owner_public_key.y,
    randomness
);
```

The Private Kernel circuit will modify this `note_hash` further, before it is inserted into the tree. It will:

- Silo the commitment, to prevent cross-contamination of this contract's state variables with other contracts' state variables:
  `siloed_note_hash: Field = hash(note_hash, contract_address);`
- Ensure uniqueness of the commitment, by hashing it with a nonce
  `unique_siloed_note_hash: Field = hash(siloed_note_hash, nonce);`, where `nonce: Field = hash(new_nullifiers[0], index)`, where `index` is the position of the new note hash in all new note hashes.

> Note, all hashes will be appropriately domain-separated.

The tree is append-only for a few of reasons:

- It saves on the number of hashes required to perform membership proofs and insertions. As long as we know an upper bound on the number of leaves we'll ever need, we can shrink the tree down to that size (instead of using a gigantic sparse tree with 2^254 leaves).
- It allows us to insert leaves in batches with fewer hashes than a sparse tree.
- It allows syncing to be performed much quicker than a sparse tree, as a node can sync from left to right, and can adopt some efficient syncing algorithms.

Cryptographic commitments have the nice property that they can _hide_ the contents of the underlying blob of data, whilst ensuring the
'preimage' of that data cannot be modified (the latter property is called 'binding'). A simple commitment can be achieved by choosing your favourite hash function, and including a large random number in with the data you wish to commit to. (The randomness must be newly-generated for each commitment).

Instead of the term 'Note', other protocols might refer to a blob of data representing private state as a 'record'. This is terminology used by zexe-like protocols.

## Nullifier Tree

A nullifier tree is basically a tree whose leaf values must be unique. An attempt to insert a duplicate leaf value will be rejected. (See [indexed merkle tree](./indexed_merkle_tree) for technical implementation details).

<Image img={require("/img/nullifier-tree.png")} />

### Example uses of nullifiers

#### Signalling an action which cannot be repeated

Most generally, a nullifier can be emitted by a private function after it has performed some action, as a way of preventing that action from ever being taken again, all whilst preventing observers from knowing what action just took place.

#### Deleting Notes (privately)

A common use case is to signal that a Note has been 'deleted', without revealing to the network which Note has been deleted. In such cases, a Nullifier for a Note might be derived as:

```rust
nullifier = hash(note_hash, owner_secret_key);
```

This has the property that it's inextricably linked to the Note it is nullifying, and it can only be derived by the owner of the `owner_public_key` contained within the Note (a smart contract's logic would ensure that the secret key corresponds to the public key).

If a function of a smart contract generates this Nullifier and submits it to the network, it will only be allowed to submit it once; a second submission will be rejected by the Base Rollup Circuit (which performs Merkle non-membership checks against the Nullifier Tree). This prevents a Note from being 'deleted' twice.

> Note: a Note cannot actually be "deleted" from the Private Data Tree, because it is an append-only tree. This is why we produce nullifiers; as a way of emulating deletion in a way where observers won't know which Note has been deleted.
> Note: this nullifier derivation example is an oversimplification for the purposes of illustration.

#### Initialising Singleton Notes

'Singleton Note' is a term we've been using to mean: "A single Note which contains the whole of a private state's current value, and must be deleted and replaced with another single Note, if one ever wishes to edit that state". It's in contrast to a Note which only contains a small fragment of a Private State's current value. <!-- TODO: write about fragmented private state, somewhere. -->

We've found that such notes require an 'Initialisation Nullifier'; a nullifier which, when emitted, signals the initialisation of this state variable. I.e. the very first time the state variable has been written-to.

> There's more on this topic in [the Aztec forum](https://discourse.aztec.network/t/utxo-syntax-2-initialising-singleton-utxos/47).

## Public State Tree

A sparse Merkle tree, intended for storing public state variables of Aztec contracts. Each non-zero leaf contains the current value of some public state variable of some Aztec contract. Leaves of this tree may be updated in-place (as opposed to convoluted nullification of private states).

<Image img={require("/img/public-data-tree.png")} />

Each leaf is a key:value mapping, which maps a `(contract_address, storage_slot)` tuple to the current value of a state variable.

E.g. for a state variable at some `storage_slot` of some `contract_address`, its current `value` is stored at the leaf with:

- `leaf_index = pedersen(contract_address, storage_slot);`
- `leaf_value = value;`

> Note: pedersen hashes are domain-separated in the codebase; this is not shown here.

> Note: we might modify the type of hash used, before mainnet.

Note this tree's data can only be read/written by the Sequencer, since only they can know the very-latest state of the tree when processing a tx.

## Contract Tree

The contract tree contains information about every function of every contract deployed to the Aztec network. This allows the Kernel Circuits to validate that for every function execution request, the requested function actually belongs to a deployed contract.

<Image img={require("/img/contract-tree.png")} />

> Note: Aztec supports the ability to keep the logic of private functions of a smart contract private. In such cases, no information about the logic of that private function will be broadcast; only a randomised merkle root of that contract's data.

## Trees of historic trees' roots

- `treeOfHistoricPrivateDataTreeRoots`: for membership checks against historic roots of the `privateDataTree`
- `treeOfHistoricContractTreeRoots`: for membership checks against historic roots of the `contractTree`

## Trees of valid Kernel/Rollup circuit VKs

Eventually, we'll have trees of VKs for various permutations of kernel/rollup circuits. Such permutations might be the number of public inputs, or the logic contained within the circuits.

## Participate

Keep up with the latest discussion and join the conversation in the [Aztec forum](https://discourse.aztec.network).

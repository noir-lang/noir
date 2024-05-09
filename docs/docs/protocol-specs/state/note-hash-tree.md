# Note Hash Tree

The Note Hash tree is an [append-only Merkle tree](./tree-implementations.md#append-only-merkle-trees) that stores [siloed](./tree-implementations.md#siloing-leaves) note hashes as its elements. Each element in the tree is a 254-bit altBN-254 scalar field element. This tree is part of the global state, and is used to prove existence of private notes via Merkle membership proofs.

Note commitments <!-- A "note commitment" is not defined. Suggest using "note hash" throughout, with a short comment on why we call them note hashes and not commitments (because not all entries in the tree are "hiding", so are not technically "commitments"). --> are immutable once created. Still, notes can be consumed ("read") by functions. To preserve privacy, a consumed note is not removed from the tree, otherwise it would be possible to link the transaction that created a note with the one that consumed it. Instead, a note is consumed by emitting a deterministic [nullifier](./nullifier-tree.md).

Contracts emit new note commitments via the `new_note_hashes` in the `CircuitPublicInputs` <!-- n/d. Please link to where this is defined -->, which are subsequently [siloed](./tree-implementations.md#siloing-leaves) by contract address by the Kernel circuit. Siloing the commitment ensures that a malicious contract cannot create notes for (that is, modify the state of) another contract.

The Kernel circuit also guarantees uniqueness of commitments by hashing them with a nonce, derived from the transaction identifier <!-- n/d --> and the index of the commitment within the transaction's array of newly-created note hashes. Uniqueness means that a note with the same contents can be emitted more than once, and each instance can be independently nullified. Without uniqueness, two notes with the same content would yield the same commitment and nullifier, so nullifying one of them would render the second one as nullified as well.

The pseudocode for siloing and making a commitment unique is the following, where each `hash` operation is a Pedersen hash with a unique generator index, indicated by the constant in all caps.

```
fn compute_siloed_note_hash(commitment, contract, transaction):
  let index = index_of(commitment, transaction.commitments)
  let nonce = hash([transaction.tx_hash, index], NOTE_HASH_NONCE)
  let unique_note_hash = hash([nonce, commitment], UNIQUE_NOTE_HASH);
  return hash([contract, unique_note_hash], SILOED_NOTE_HASH)
```

The unique siloed commitment of a note is included in the [transaction `data`](../transactions/tx-object.md), and then inserted into the Note Hash tree by the sequencer as the transaction is included in a block.

The protocol does not enforce any constraints on any note hashes emitted by an application. This means that applications are responsible for including a `randomness` field in the note hash to make the commitment _hiding_ in addition to _binding_. If an application does not include randomness, and the note preimage can be guessed by an attacker, it makes the note vulnerable to preimage attacks, since the siloing and uniqueness steps do not provide hiding.

Furthermore, since there are no constraints to the commitment emitted by an application, an application can emit any value whatsoever as a `new_note_hash`, including values that do not map to a note hash.

# Note Hash Tree

The Note Hash tree is an [append-only Merkle tree](./tree-implementations.md#append-only-merkle-trees) that stores siloed note hashes as its elements. Each element in the tree is a 254-bit altBN-254 scalar field element. This tree is part of the global state, and allows to prove existence of private notes via Merkle membership proofs.

Note commitments are immutable once created, since notes cannot be modified. Still, notes can be consumed, which means they can no longer be used. To preserve privacy, a consumed note is not removed from the tree, otherwise it would be possible to link the transaction that created a note with the one that consumed it. Instead, a note is consumed by emitting a deterministic [nullifier](./nullifier-tree.md).

Contracts emit new note commitments via the `new_commitments` in the `CircuitPublicInputs`, which are subsequently [siloed](./tree-implementations.md#siloing-leaves) per contract by the Kernel circuit. Siloing the commitment ensures that a contract cannot emit a commitment for a note that could be used for a different contract.

The Kernel circuit also guarantees uniqueness of commitments by further hashing them with a nonce, derived from the transaction identifier and the index of the commitment within the transaction. Uniqueness means that a note with the same contents can be emitted more than once, and each instance can be independently nullified. Without uniqueness, two notes with the same content would yield the same commitment and nullifier, so nullifying one of them would flag the second one as nullified as well.

The pseudocode for siloing and making a commitment unique is the following, where each `hash` operation is a Pedersen hash with a unique generator index, indicated by the constant in all caps.

```
fn compute_unique_siloed_commitment(commitment, contract, transaction):
  let siloed_commitment = hash([contract, commitment], SILOED_COMMITMENT)
  let index = index_of(commitment, transaction.commitments)
  let nonce = hash([transaction.tx_hash, index], COMMITMENT_NONCE)
  return hash([nonce, siloed_commitment], UNIQUE_COMMITMENT)
```

The unique siloed commitment of a note is included in the [transaction `data`](../transactions/tx-object.md), and then included into the Note Hash tree by the sequencer as the transaction is included in a block.

The protocol does not enforce any constraints to the commitment emitted by an application. This means that applications are responsible for including a `randomness` field in the note hash to make the commitment _hiding_ in addition to _binding_. If an application does not include randomness, and the note preimage can be guessed by an attacker, it makes the note vulnerable to preimage attacks, since the siloing and uniqueness steps do not provide hiding.

Furthermore, since there are no constraints to the commitment emitted by an application, an application can emit any value whatsoever as a `new_commitment`, including values that do not map to a note hash.

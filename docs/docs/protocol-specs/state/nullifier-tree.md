# Nullifier Tree

The Nullifier tree is an [indexed Merkle tree](./tree-implementations.md#indexed-merkle-trees) that stores nullifier values. Each value stored in the tree is a 254-bit altBN-254 scalar field element. This tree is part of the global state, and is primarily used to prove non-existence of a nullifier when a note is consumed (as a way of preventing double-spend).

In addition to storing nullifiers of notes, the nullifier tree is more generally useful to prevent any action from being repeated twice. This includes preventing re-initialization of state variables, and [re-deployment of contracts](../contract-deployment/instances.md).

Nullifiers are asserted to be unique during insertion, by checking that the inserted value is not equal to the value and next-value stored in the prior leaf in the indexed tree. Any attempt to insert a duplicated value is rejected.

Contracts emit new nullifiers via the `new_nullifiers` field of the `CircuitPublicInputs` ABI <!-- TODO: link -->. Similarly to elements in the [Note Hash tree](./note-hash-tree.md), nullifiers are [siloed](./tree-implementations.md#siloing-leaves) by contract address, by the Kernel circuit, before being inserted into the tree. This ensures that a contract cannot emit the nullifiers of other contracts' state variables!

<!--
We should seek some consistency to how we document hashes. Copying some boilerplate commentary from my review of another section:

We should specify exactly how this hash is computed.
- Details of the hash to use, and a domain separator for the hash. We might not know the final hash that we'll use, but we should propose one, and we should probably also give each hash a name.
    - E.g. `compute_siloed_nullifier("silo nullifier".to_field(), contract_address, nullifier)` where `compute_siloed_nullifier = pedersen_hash` (for now).
 -->

<!-- TODO: consider whether a 'bit twiddly' hash should be used instead of pedersen, here. -->

```
fn compute_siloed_nullifier(nullifier, contract):
  return hash([contract, nullifier], OUTER_NULLIFIER)
```

Nullifiers are primarily used for privately marking notes as consumed. When a note is consumed in an application, the application computes and emits a deterministic nullifier associated to the note. If a user attempts to consume the same note more than once, the same nullifier will be generated, and will be rejected on insertion by the nullifier tree.

Nullifiers provide privacy by being computed using a deterministic secret value, such as the owner siloed nullifier secret key, or a random value stored in an encrypted note. This ensures that, without knowledge of the secret value, it is not possible to calculate the associated nullifier, and thus it is not possible to link a nullifier to its associated note commitment.

Applications are not constrained by the protocol on how the nullifier for a note is computed. It is responsibility of the application to guarantee determinism in calculating a nullifier, otherwise the same note could be spent multiple times.

Furthermore, nullifiers can be emitted by an application just to ensure that an action can be executed only once, such as initializing a value, and are not required to be linked to a note commitment.

<!--
Mike review:

Missing info:
- We're missing how the nodes of the tree are computed. Similar boilerplate to my other hash comments:
    - We should specify exactly how this hash is computed.
        - Details of the hash to use, and a domain separator for the hash. We might not know the final hash that we'll use, but we should propose one, and we should probably also give each hash a name.
        - E.g. `compute_parent_node("nullifier parent node".to_field(), left_child, right_child)` where `compute_siloed_nullifier = pedersen_hash` (for now).

Pseudocode/algorithms for insertion, batch insertion, membership proofs, non-membership proofs, so that the security of our approach can be validated. We should discuss the best way to consistently present such information, for all sections of the protocol specs.

EDIT: maybe all these comments should actually go in cryptography/merkle-tree.md
 -->

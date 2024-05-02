# Public Data Tree

The Public Data tree is an [indexed Merkle tree](./tree-implementations.md#indexed-merkle-trees) that stores public-state. Each item stored in the tree is a key-value pair, where both key and value are 254-bit altBN-254 scalar field elements. Items are sorted based on their key, so each indexed tree leaf contains a tuple with the key, the value, the next-highest key, and the index in the tree for the next-highest key. This tree is part of the global state, and is updated by the sequencer during the execution of public functions.

An indexed Merkle tree is used instead of a sparse Merkle tree in order to reduce the tree height. A lower height means shorter membership proofs.

Keys in the Public Data tree are [siloed](./tree-implementations.md#siloing-leaves) using the contract address, to prevent a contract from overwriting the public state of another contract.

<!--
Consider renaming. `data_item` isn't descriptive enough.
`value` isn't used (not really).
-->

```
fn compute_siloed_public_data_item(key, value, contract):
  let siloed_key = hash([contract, key], PUBLIC_DATA_LEAF)
  return [siloed_key, value]
```

When attempting to read a key from the Public Data tree, the key may or may not be present. If the key is not present, then a non-membership proof can be produced. When a key is written to, either a new node is appended to the tree if the key was not present, or its value is overwritten if it was.

Public functions can read from or write to the Public Data tree by emitting `contract_storage_read` and `contract_storage_update_requests` in the `PublicCircuitPublicInputs`. The Kernel circuit then siloes these requests per contract.

Contracts can store arbitrary data at a given key, which is always stored as a single field element. Applications are responsible for interpreting this data. Should an application need to store data larger than a single field element, they are responsible for partitioning it across multiple keys.

<!--
Mike review:

Missing info:
(EDIT: consider putting info that relates generally to the indexed merkle tree inside cryptography/merkle-trees.md)
- A clear struct specifying the contents (name, type, comment) of a leaf preimage.
- Explanation that there's a convention that the key is derived from a "storage slot", but that this is not enforced, and contracts are free to devise keys however they want.
- The tree needs to be pre-populated with the leaf `{ 0, 0, 0}`, right? We should explain how that works.
    - Is it pre-populated with any leaves?
        - Perhaps contracts which must exist at genesis (e.g. the contract class registry and instance deployment contracts) might need to contain state?

- We're missing how the nodes of the tree are computed. Similar boilerplate to my other hash comments (EDIT: put this in cryptography/hashing/):
    - We should specify exactly how this hash is computed.
        - Details of the hash to use, and a domain separator for the hash. We might not know the final hash that we'll use, but we should propose one, and we should probably also give each hash a name.
        - E.g. `compute_parent_node("nullifier parent node".to_field(), left_child, right_child)` where `compute_siloed_nullifier = pedersen_hash` (for now. Poseidon eventually, iiuc. Perhaps we should write this spec to state Poseidon).

Pseudocode/algorithms for insertion, batch insertion, membership proofs, non-membership proofs, so that the security of our approach can be validated. We should discuss the best way to consistently present such information, for all sections of the protocol specs. (EDIT: put this in cryptography/merkle-trees)
 -->

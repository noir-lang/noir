# Tree implementations

Aztec relies on two Merkle tree implementations in the protocol: append-only and indexed Merkle trees.

## Append-only Merkle trees

In an append-only Merkle tree, new leaves are inserted in order from left to right. Existing leaf values are immutable and cannot be modified. These trees are useful to represent historical data, as historical data is not altered, and new entries can be added as new transactions and blocks are processed.

Append-only trees allow for more efficient syncing than sparse trees, since clients can sync from left to right starting with their last known value. Updates to the tree root, when inserting new leaves, can be computed from the rightmost "frontier" of the tree (i.e., from the sibling path of the rightmost nonzero leaf). Batch insertions can be computed with fewer hashes than in a sparse tree. The historical snapshots of append-only trees also enable efficient membership proofs; as older roots can be computed by completing the merkle path from a past left subtree with an empty right subtree.

### Wonky Merkle Trees

We also use a special type of append-only tree to structure the rollup circuits. Given `n` leaves, we fill from left to right and attempt to pair them to produce the next layer. If `n` is a power of 2, this tree looks exactly like a standard append-only merkle tree. Otherwise, once we reach an odd-sized row we shift the final node up until we reach another odd row to combine them.

This results in an unbalanced tree where there are no empty leaves. For rollups, this means we don't have to pad empty transactions and process them through the rollup circuits. A full explanation is given [here](./wonky-tree.md).

## Indexed Merkle trees

Indexed Merkle trees, introduced [here](https://eprint.iacr.org/2021/1263.pdf), allow for proofs of non-inclusion more efficiently than sparse Merkle trees. Each leaf in the tree is a tuple of: the leaf value, the next-highest value in the tree, and the index of the leaf where that next-highest value is stored. New leaves are inserted from left to right, as in the append-only tree, but existing leaves can be _modified_ to update the next-highest value and next-highest index (a.k.a. the "pointer") if a new leaf with a "closer value" is added to the tree. An Indexed Merkle trees behaves as a Merkle tree over a sorted linked list.

With an Indexed Merkle tree, proving non-membership of a value `x` then requires a membership proof of the node with value lower than `x` and a next-highest value greater than `x`. The cost of this proof is proportional to the height of the tree, which can be set according to the expected number of elements to be stored in the tree. For comparison, a non-membership proof in a sparse tree requires a tree with height proportional to the size of the elements, so when working with 256-bit elements, 256 hashes are required for a proof.

Refer to [this page](../../aztec/concepts/storage/trees/indexed_merkle_tree.mdx) for more details on how insertions, updates, and membership proofs are executed on an Indexed Merkle tree.

<!-- Q: should we embed the diagrams and pseudocode here, instead of linking? -->

## Siloing leaves

In several trees in the protocol we indicate that its leaves are "siloed". This refers to hashing the leaf value with some other "siloing" value before inserting it into the tree. The siloing value is typically the contract address of the contract that produced the value. This allows us to store disjoint "domains" within the same tree, ensuring that a value emitted from one domain cannot affect others.

To guarantee the siloing of leaf values, siloing is performed by a trusted protocol circuit, such as a kernel or rollup circuit, and not by an application circuit.

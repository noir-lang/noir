# Tree implementations

Aztec relies on two Merkle tree implementations in the protocol: append-only and indexed Merkle trees.

## Append-only Merkle trees

In an append-only Merkle tree new leaves are inserted in order from left to right. Existing leaf values are immutable and cannot be modified. These tree are useful to represent historic data, as new entries are added as new transactions and blocks are processed, and historic data is not altered.

Append-only trees allow for more efficient syncing than sparse trees, since clients can sync from left to right starting with their last known value. Updates to the tree root from new leaves can be computed just by keeping the rightmost boundary of the tree, and batch insertions can be computed with fewer hashes than in a sparse tree. Append-only trees also provide cheap historic snapshots, as older roots can be computed by completing the merkle path from a past left subtree with an empty right subtree.

## Indexed Merkle trees

Indexed Merkle trees, introduced [here](https://eprint.iacr.org/2021/1263.pdf), allow for proofs of non-inclusion more efficiently than sparse Merkle trees. Each leaf in the tree is a tuple with the leaf value, the next higher value in the tree, and the index of the leaf where that value is stored. New nodes are inserted left to right, as in the append-only tree, but existing nodes can be modified to update the next value and its pointer. Indexed Merkle trees behave as a Merkle tree over a sorted linked list.

Assuming the indexed Merkle tree invariants hold, proving non-membership of a value `x` then requires a membership proof of the node with value lower than `x` and a next higher value greater than `x`. The cost of this proof is proportional to the height of the tree, which can be set according to the expected number of elements to be stored in the tree. For comparison, a non-membership proof in a sparse tree requires a tree with height proportional to the size of the elements, so when working with 256-bit elements, 256 hashes are required for a proof.

Refer to [this page](https://docs.aztec.network/concepts/advanced/data_structures/indexed_merkle_tree) for more details on how insertions, updates, and membership proofs are executed on an Indexed Merkle tree.

<!-- Q: should we embed the diagrams and pseudocode here, instead of linking? -->

## Siloing leaves

In several trees in the protocol we indicate that its leaves are "siloed". This refers to hashing the leaf value with a siloing value before inserting it in the tree. The siloing value is typically an identifier of the contract that produced the value. This allows us to store disjoint "domains" within the same tree, ensuring that a value emitted from one domain cannot affect others.

To guarantee the siloing of leaf values, siloing is performed by a trusted protocol circuit, such as the kernel or rollup circuits, and not by an application circuit. Siloing is performed by Pedersen hashing the contract address and the value.

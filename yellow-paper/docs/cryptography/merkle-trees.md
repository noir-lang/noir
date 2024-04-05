# Merkle Trees

<!-- TODO: make this more mathematically precise?-->

## Notation

### Orientation

A tree is always visualised as a "triangle" with its point at the top (the root) and its base at the bottom (the leaves). Like this: $\Delta$.

Hopefully this gives a clear intuition whenever the terms "left", "right", "up", "down", "above", "below" are used when describing trees.

### Arity

Trees in Aztec are currently all binary Merkle trees (2-ary).

<!-- This section will need to be updated if the arity ever changes -->

### Height

The `height` of a tree with $l$ leaves is $\lceil \log_2(l) \rceil$.

### Layers

The `layers` of a tree are an enumerated from `0`. The leaves are at layer `0`; the root is at layer [`height`](#height).

### Levels

Synonymous with [layers](#layers).

### Rows

Synonymous with [layers](#layers) and [levels](#levels).

### Leaf Index

The leaves of a tree are indexed from `0`. The first, [left-most](#orientation) leaf is at `leaf_index = 0`.

### Node Index

All nodes of the tree (including the leaves) can be indexed. The method of indexing might depend on the algorithm being applied to the tree.

### Path

The path from (or "of") a particular node is a vector of that node's ancestors. That is, the node's parent, then its parent's parent, and so on, all the way up to and including the root.

### Sibling Path

The sibling path of a particular node is, loosely, a vector of the siblings of the nodes in its [path](#path), except it also includes the node's sibling, and excludes the root (which has no sibling).
The first element in the sibling path is the node's sibling. Then, the node's parent's sibling, then its parent's parent's sibling, and so on.

### Membership Witness

The membership witness for a particular leaf, is the minimum data needed to prove that leaf value's existence in the tree. That is:

- The leaf's [leaf index](#leaf-index)
- The leaf's [sibling path](#sibling-path)

(and the leaf value itself, of course, but we don't include that in this "membership witness" definition).

## Hashing

Used for computing the parent nodes of all merkle trees.

<!-- HASH DEFINITION -->

```rust
enum TreeId {
    Archive,
    NoteHash,
    Nullifier,
    PrivateFunction,
    L1ToL2Msgs,
    PublicData
}

fn merkle_crh(
    tree_id: TreeId,
    layer: u64,
    left: Field,
    right: Field
) -> Field {
    let tree_id_domain_separator: string = match tree_id {
        TreeId::Archive => "archive",
        TreeId::NoteHash => "note_hash",
        TreeId::Nullifier => "nullifier",
        TreeId::PrivateFunction => "private_function",
        TreeId::L2ToL2Msgs => "l1_to_l2_msgs",
        TreeId::PublicData => "public_data",
    };

    let merkle_domain_separator: string = "az_merkle" + tree_id_domain_separator;

    let parent = poseidon2(
        be_string_to_field(merkle_domain_separator),
        int_to_field(layer),

        left,
        right
    );

    parent
}
```

> `tree_id` reflects the various [trees in the protocol](../state/index.md). The `PrivateFunction` tree is discussed in the [contract classes](../contract-deployment/classes.md) section.
> `layer` is the [layer](#layers) of the `left` and `right` children being hashed. For example, when hashing two leaves, `layer = 0`.

:::danger

- Q: Do we need the domain separator "az_merkle" + tree_id, for each of the trees?
- Q: do we need domain separation between different layers of the tree?
- Q: Can we optimise the two domain separators to take up 1 Field, instead of 2, or does squashing them together add too many constraints?
- Note: if it helps with optimisation, we can reduce the bit-length of the domain separator strings.
- Q: Can we specify the arguments to Poseidon as Fields, or do we need to specify them as bit-sequences?

:::

## Append-only Merkle Tree

TODO

## Indexed Merkle Tree

TODO

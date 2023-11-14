---
title: Merkle Trees
description: Learn about Merkle Trees in Noir with this tutorial. Explore the basics of computing a merkle root using a proof, with examples.
keywords:
  [
    Merkle trees in Noir,
    Noir programming language,
    check membership,
    computing root from leaf,
    Noir Merkle tree implementation,
    Merkle tree tutorial,
    Merkle tree code examples,
    Noir libraries,
    pedersen hash.,
  ]
---

## compute_merkle_root

Returns the root of the tree from the provided leaf and its hash path, using a [Pedersen hash](cryptographic_primitives/00_hashes.mdx#pedersen).

```rust
fn compute_merkle_root(leaf : Field, index : Field, hash_path: [Field]) -> Field
```

example:

```rust
/**
    // these values are for this example only
    index = "0"
    priv_key = "0x000000000000000000000000000000000000000000000000000000616c696365"
    secret = "0x1929ea3ab8d9106a899386883d9428f8256cfedb3c4f6b66bf4aa4d28a79988f"
    note_hash_path = [
    "0x1e61bdae0f027b1b2159e1f9d3f8d00fa668a952dddd822fda80dc745d6f65cc",
    "0x0e4223f3925f98934393c74975142bd73079ab0621f4ee133cee050a3c194f1a",
    "0x2fd7bb412155bf8693a3bd2a3e7581a679c95c68a052f835dddca85fa1569a40"
    ]
 */
fn main(index: Field, priv_key: Field, secret: Field, note_hash_path: [Field; 3]) {

    let pubkey = std::scalar_mul::fixed_base_embedded_curve(priv_key);
    let pubkey_x = pubkey[0];
    let pubkey_y = pubkey[1];
    let note_commitment = std::hash::pedersen([pubkey_x, pubkey_y, secret]);

    let root = std::merkle::compute_merkle_root(note_commitment[0], index, note_hash_path);
    std::println(root);
}
```

To check merkle tree membership:

1. Include a merkle root as a program input.
2. Compute the merkle root of a given leaf, index and hash path.
3. Assert the merkle roots are equal.

For more info about merkle trees, see the Wikipedia [page](https://en.wikipedia.org/wiki/Merkle_tree).

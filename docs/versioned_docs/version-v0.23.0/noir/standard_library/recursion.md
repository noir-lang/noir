---
title: Recursive Proofs
description: Learn about how to write recursive proofs in Noir.
keywords: [recursion, recursive proofs, verification_key, verify_proof]
---

Noir supports recursively verifying proofs, meaning you verify the proof of a Noir program in another Noir program. This enables creating proofs of arbitrary size by doing step-wise verification of smaller components of a large proof.

Read [the explainer on recursion](../../explainers/explainer-recursion.md) to know more about this function and the [guide on how to use it.](../../how_to/how-to-recursion.md)

```rust
#[foreign(verify_proof)]
fn verify_proof(_verification_key : [Field], _proof : [Field], _public_input : Field, _key_hash : Field) {}
```

:::info

This is a black box function. Read [this section](./black_box_fns) to learn more about black box functions in Noir.

:::

## Example usage

```rust
use dep::std;

fn main(
    verification_key : [Field; 114],
    proof : [Field; 93],
    public_inputs : [Field; 1],
    key_hash : Field,
    proof_b : [Field; 93],
) {
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );

    std::verify_proof(
        verification_key.as_slice(),
        proof_b.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );
}
```

You can see a full example of recursive proofs in [this example recursion demo repo](https://github.com/noir-lang/noir-examples/tree/master/recursion).

## Parameters

### `verification_key`

The verification key for the zk program that is being verified.

### `proof`

The proof for the zk program that is being verified.

### `public_inputs`

These represent the public inputs of the proof we are verifying.

### `key_hash`

A key hash is used to check the validity of the verification key. The circuit implementing this opcode can use this hash to ensure that the key provided to the circuit matches the key produced by the circuit creator.

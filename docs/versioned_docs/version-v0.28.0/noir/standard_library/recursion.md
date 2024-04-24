---
title: Recursive Proofs
description: Learn about how to write recursive proofs in Noir.
keywords: [recursion, recursive proofs, verification_key, verify_proof]
---

Noir supports recursively verifying proofs, meaning you verify the proof of a Noir program in another Noir program. This enables creating proofs of arbitrary size by doing step-wise verification of smaller components of a large proof.

Read [the explainer on recursion](../../explainers/explainer-recursion.md) to know more about this function and the [guide on how to use it.](../../how_to/how-to-recursion.md)

## The `#[recursive]` Attribute

In Noir, the `#[recursive]` attribute is used to indicate that a circuit is designed for recursive proof generation. When applied, it informs the compiler and the tooling that the circuit should be compiled in a way that makes its proofs suitable for recursive verification. This attribute eliminates the need for manual flagging of recursion at the tooling level, streamlining the proof generation process for recursive circuits.

### Example usage with `#[recursive]`

```rust
#[recursive]
fn main(x: Field, y: pub Field) {
    assert(x == y, "x and y are not equal");
}

// This marks the circuit as recursion-friendly and indicates that proofs generated from this circuit
// are intended for recursive verification.
```

By incorporating this attribute directly in the circuit's definition, tooling like Nargo and NoirJS can automatically execute recursive-specific duties for Noir programs (e.g. recursive-friendly proof artifact generation) without additional flags or configurations.

## Verifying Recursive Proofs

```rust
#[foreign(recursive_aggregation)]
pub fn verify_proof(verification_key: [Field], proof: [Field], public_inputs: [Field], key_hash: Field) {}
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

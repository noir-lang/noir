---
title: Recursive Proofs
description: Learn about how to write recursive proofs in Noir.
keywords: [recursion, recursive proofs, verification_key, aggregation object, verify_proof]
---

Noir supports recursively verifying proofs, meaning you verify the proof of a Noir program in another Noir program. This enables creating proofs of arbitrary size by doing step-wise verification of smaller components of a large proof.

The `verify_proof` function takes a verification key, proof and public inputs for a zk program, as well as a key hash and an input aggregation object. The key hash is used to check the validity of the verification key and the input aggregation object is required by some proving systems. The `verify_proof` function returns an output aggregation object that can then be fed into future iterations of the proof verification if required.

```rust
#[foreign(verify_proof)]
fn verify_proof(_verification_key : [Field], _proof : [Field], _public_input : Field, _key_hash : Field, _input_aggregation_object : [Field]) -> [Field] {}
```

:::info

This is a black box function. Read [this section](./black_box_fns) to learn more about black box functions in Noir.

:::

## Aggregation Object

The purpose of the input aggregation object is a little less clear though (and the output aggregation object that is returned from the `std::verify_proof` method). Recursive zkSNARK schemes do not necessarily "verify a proof" in the sense that you expect a true or false to be spit out by the verifier. Rather an aggregation object is built over the public inputs. In the case of PLONK the recursive aggregation object is two G1 points (expressed as 16 witness values). The final verifier (in our case this is most often the smart contract verifier) has to be aware of this aggregation object to execute a pairing and check the validity of these points (thus completing the recursive verification).

So for example in this circuit:

```rust
use dep::std;

fn main(
    verification_key : [Field; 114],
    proof : [Field; 94],
    public_inputs : [Field; 1],
    key_hash : Field,
    input_aggregation_object : [Field; 16],
    proof_b : [Field; 94],
) -> pub [Field; 16] {
    let output_aggregation_object_a = std::verify_proof(
        verification_key,
        proof,
        public_inputs,
        key_hash,
        input_aggregation_object
    );

    let output_aggregation_object = std::verify_proof(
        verification_key,
        proof_b,
        public_inputs,
        key_hash,
        output_aggregation_object_a
    );

    let mut output = [0; 16];
    for i in 0..16 {
        output[i] = output_aggregation_object[i];
    }
    output
}
```

In this example we have a circuit, that generates proofs A and B, that is being verified in circuit C. Assuming that the proof being passed in is not already a recursive proof, the `input_aggregation_object` will be all zeros. It will then generate an `output_aggregation_object`. This blob of data then becomes the `input_aggregation_object` of the next recursive aggregation we wish to compute. We can see here as the same public inputs, verification key, and key hash are used that we are verifying two proofs generated from the same circuit in this single circuit. `std::verify_proof` returns a `[Field]` because the size of an aggregation object is proof system dependent--in barretenberg, aggregation objects are two G1 points, while in Halo2, the aggregation object is a list of G1 points that is log the circuit size. So for the final step we convert the slice into an array of size 16 because we are generating proofs using UltraPlonk.

## Parameters

### `verification_key`

The verification key for the zk program that is being verified.

### `proof`

The proof for the zk program that is being verified.

### `public_inputs`

These represent the public inputs of the proof we are verifying. They should be checked against in the circuit after construction of a new aggregation state.

### `key_hash`

A key hash is used to check the validity of the verification key. The circuit implementing this opcode can use this hash to ensure that the key provided to the circuit matches the key produced by the circuit creator.

### `input_aggregation_object`

An aggregation object is blob of data that the top-level verifier must run some proof system specific algorithm on to complete verification. The size is proof system specific and will be set by the backend integrating this opcode. The input aggregation object is only not `None` when we are verifying a previous recursive aggregation in the current circuit. If this is the first recursive aggregation there is no input aggregation object. It is left to the backend to determine how to handle when there is no input aggregation object.

## Return value

### `output_aggregation_object`

This is the result of a recursive aggregation and is what will be fed into the next verifier.
The next verifier can either perform a final verification (returning true or false) or perform another recursive aggregation where this output aggregation object will be the input aggregation object of the next recursive aggregation.

## Example

You can see an example of how to do recursive proofs in [this example recursion demo repo](https://github.com/Savio-Sou/recursion-demo/tree/main).

---
title: Black Box Functions
description: Black box functions are functions in Noir that rely on backends implementing support for specialized constraints.
keywords: [noir, black box functions]
---

Black box functions are functions in Noir that rely on backends implementing support for specialized constraints. This makes certain zk-snark unfriendly computations cheaper than if they were implemented in Noir.

:::warning

It is likely that not all backends will support a particular black box function.

:::

Because it is not guaranteed that all backends will support black box functions, it is possible that certain Noir programs won't compile against a particular backend if they use an unsupported black box function. It is possible to fallback to less efficient implementations written in Noir/ACIR in some cases.

Black box functions are specified with the `#[foreign(black_box_fn)]` attribute. For example, the SHA256 function in the Noir [source code](https://github.com/noir-lang/noir/blob/v0.5.1/noir_stdlib/src/hash.nr) looks like:

```rust
#[foreign(sha256)]
fn sha256<N>(_input : [u8; N]) -> [u8; 32] {}
```

## Function list

Here is a list of the current black box functions that are supported by UltraPlonk:

- AES
- [SHA256](./cryptographic_primitives/hashes#sha256)
- [Schnorr signature verification](./cryptographic_primitives/schnorr)
- [Blake2s](./cryptographic_primitives/hashes#blake2s)
- [Pedersen](./cryptographic_primitives/hashes#pedersen)
- [HashToField128Security](./cryptographic_primitives/hashes#hash_to_field)
- [ECDSA signature verification](./cryptographic_primitives/ecdsa_sig_verification)
- [Fixed base scalar multiplication](./cryptographic_primitives/scalar)
- [Compute merkle root](./merkle_trees#compute_merkle_root)
- AND
- XOR
- RANGE
- [Keccak256](./cryptographic_primitives/hashes#keccak256)
- [Recursive proof verification](./recursion)

Most black box functions are included as part of the Noir standard library, however `AND`, `XOR` and `RANGE` are used as part of the Noir language syntax. For instance, using the bitwise operator `&` will invoke the `AND` black box function. To ensure compatibility across backends, the ACVM has fallback implementations of `AND`, `XOR` and `RANGE` defined in its standard library which it can seamlessly fallback to if the backend doesn't support them.

You can view the black box functions defined in the ACVM code [here](https://github.com/noir-lang/acvm/blob/acir-v0.12.0/acir/src/circuit/black_box_functions.rs).

---
title: Black Box Functions
description: Black box functions are functions in Noir that rely on backends implementing support for specialized constraints.
keywords: [noir, black box functions]
---

Black box functions are functions in Noir that rely on backends implementing support for specialized constraints. This makes certain zk-snark unfriendly computations cheaper than if they were implemented in Noir.

The ACVM spec defines a set of blackbox functions which backends will be expected to implement. This allows backends to use optimized implementations of these constraints if they have them, however they may also fallback to less efficient naive implementations if not.

## Function list

Here is a list of the current black box functions:

- [AES128](./cryptographic_primitives/ciphers.mdx#aes128)
- [SHA256](./cryptographic_primitives/hashes.mdx#sha256)
- [Schnorr signature verification](./cryptographic_primitives/schnorr.mdx)
- [Blake2s](./cryptographic_primitives/hashes.mdx#blake2s)
- [Blake3](./cryptographic_primitives/hashes.mdx#blake3)
- [Pedersen Hash](./cryptographic_primitives/hashes.mdx#pedersen_hash)
- [Pedersen Commitment](./cryptographic_primitives/hashes.mdx#pedersen_commitment)
- [ECDSA signature verification](./cryptographic_primitives/ecdsa_sig_verification.mdx)
- [Embedded curve operations (MSM, addition, ...)](./cryptographic_primitives/embedded_curve_ops.mdx)
- AND
- XOR
- RANGE
- [Keccak256](./cryptographic_primitives/hashes.mdx#keccak256)
- [Recursive proof verification](./recursion.mdx)

Most black box functions are included as part of the Noir standard library, however `AND`, `XOR` and `RANGE` are used as part of the Noir language syntax. For instance, using the bitwise operator `&` will invoke the `AND` black box function.

You can view the black box functions defined in the ACVM code [here](https://github.com/noir-lang/noir/blob/master/acvm-repo/acir/src/circuit/black_box_functions.rs).

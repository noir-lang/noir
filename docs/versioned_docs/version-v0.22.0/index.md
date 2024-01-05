---
title: Noir
description:
  Learn about the public alpha release of Noir, a domain specific language heavily influenced by Rust that compiles to
  an intermediate language which can be compiled to an arithmetic circuit or a rank-1 constraint system.
keywords:
  [
    Noir,
    Domain Specific Language,
    Rust,
    Intermediate Language,
    Arithmetic Circuit,
    Rank-1 Constraint System,
    Ethereum Developers,
    Protocol Developers,
    Blockchain Developers,
    Proving System,
    Smart Contract Language,
  ]
sidebar_position: 0
---

## What's new about Noir?

Noir, a domain-specific language crafted for SNARK proving systems, stands out with its simplicity, flexibility,
and robust capabilities. Unlike conventional approaches that compile directly to a fixed NP-complete language,
Noir takes a two-pronged path. It first compiles to an adaptable intermediate language known as ACIR. From there,
depending on the project's needs, ACIR can be further compiled into an arithmetic circuit for integration with Aztec's
barretenberg backend or transformed into a rank-1 constraint system suitable for R1CS backends like Arkworks' Marlin
backend, among others.

This innovative design introduces unique challenges, yet it strategically separates the programming language from the
backend. Noir's approach echoes the modular philosophy of LLVM, offering developers a versatile toolkit for cryptographic
programming.

## Who is Noir for?

### Solidity Developers

Noir streamlines the creation of Solidity contracts that interface with SNARK systems.
[`Utilize the nargo codegen-verifier`](./reference/nargo_commands.md#nargo-codegen-verifier) command to construct verifier
contracts efficiently. While the current alpha version offers this as a direct feature, future updates aim
to modularize this process for even greater ease of use.

Noir currently includes a command to create a Solidity contract which verifies your Noir program. This will be
modularized in the future; however, as of the alpha, you can use the `nargo codegen-verifier` command to create a verifier contract.

### Protocol Developers

Should the Aztec backend not align with your existing tech stack, or if you're inclined to integrate alternative
proving systems, Noir's agnostic compilation to a proof-agnostic intermediate language offers unmatched flexibility.
This allows protocol engineers the freedom to substitute the default PLONK-based system with an alternative of their
choice, tailoring the proving system to their specific needs.

### Blockchain developers

Blockchain developers often face environmental constraints, such as predetermined proving systems and smart contract
languages. Noir addresses this by enabling the implementation of custom proving system backends and smart contract
interfaces, ensuring seamless integration with your blockchain's architecture, and expanding the horizons for innovation
within your projects.

## Libraries

Noir does not currently have an official package manager. You can find a list of available Noir libraries in the
[awesome-noir repo here](https://github.com/noir-lang/awesome-noir#libraries).

Some libraries that are available today include:

- [Standard Library](https://github.com/noir-lang/noir/tree/master/noir_stdlib) - the Noir Standard Library
- [Ethereum Storage Proof Verification](https://github.com/aragonzkresearch/noir-trie-proofs) - a library that contains
  the primitives necessary for RLP decoding (in the form of look-up table construction) and Ethereum state and storage
  proof verification (or verification of any trie proof involving 32-byte long keys)
- [BigInt](https://github.com/shuklaayush/noir-bigint) - a library that provides a custom BigUint56 data type, allowing
  for computations on large unsigned integers
- [ECrecover](https://github.com/colinnielsen/ecrecover-noir/tree/main) - a library to verify an ECDSA signature and
  return the source Ethereum address
- [Sparse Merkle Tree Verifier](https://github.com/vocdoni/smtverifier-noir/tree/main) - a library for verification of
  sparse Merkle trees
- [Signed Int](https://github.com/resurgencelabs/signed_int) - a library for accessing a custom Signed Integer data
  type, allowing access to negative numbers on Noir
- [Fraction](https://github.com/resurgencelabs/fraction) - a library for accessing fractional number data type in Noir,
  allowing results that aren't whole numbers

See the section on [dependencies](noir/modules_packages_crates/dependencies.md) for more information.

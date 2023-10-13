---
title: Introducing Noir
description:
  Learn about the public alpha release of Noir, a domain specific language heavily influenced by
  Rust that compiles to an intermediate language which can be compiled to an arithmetic circuit or a
  rank-1 constraint system.
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
slug: /
---

This version of the book is being released with the public alpha. There will be a lot of features
that are missing in this version, however the syntax and the feel of the language will mostly be
completed.

## What is Noir?

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

It's design choices are influenced heavily by Rust and focuses on a simple, familiar syntax.

## Who is Noir for?

Noir can be used for a variety of purposes.

### Solidity Developers

Noir currently includes a command to create a Solidity contract which verifies your Noir program. This will
be modularised in the future; however, as of the alpha, you can use the [`nargo codegen-verifier`](./nargo/commands#nargo-codegen-verifier) command to create
a verifier contract.

### Protocol Developers

As a protocol developer, you may not want to use the Aztec backend due to it not being a fit for
your stack, or maybe you simply want to use a different proving system. Since Noir does not compile
to a specific proof system, it is possible for protocol developers to replace the PLONK-based
proving system with a different proving system altogether.

### Blockchain developers

As a blockchain developer, you will be constrained by parameters set by your blockchain (for example, the
proving system and smart contract language has been pre-defined). In order for you to use Noir in
your blockchain, a proving system backend and a smart contract interface
must be implemented for it.

## What's new about Noir?

Noir is simple and flexible in its design, as it does not compile immediately to a fixed
NP-complete language. Instead, Noir compiles to an intermediate language (ACIR), which itself can be compiled
to an arithmetic circuit (if choosing to target Aztec's barretenberg backend) or a rank-1 constraint system (if choosing to target an R1CS backend like Arkwork's Marlin backend, or others).

This in itself brings up a few challenges within the design process, but allows one to decouple the programming language completely from the backend. This is similar in theory to LLVM.

## Current Features

Compiler:

- Module System
- For expressions
- Arrays
- Bit Operations
- Binary operations (<, <=, >, >=, +, -, \*, /, %) [See documentation for an extensive list]
- Unsigned integers
- If statements
- Structures and Tuples
- Generics

ACIR Supported OPCODES:

- Sha256
- Blake2s
- Schnorr signature verification
- MerkleMembership
- Pedersen
- HashToField

## Libraries

Noir does not currently have an official package manager. You can find a list of available Noir libraries in the [awesome-noir repo here](https://github.com/noir-lang/awesome-noir#libraries).

Some libraries that are available today include:

- [Standard Library](https://github.com/noir-lang/noir/tree/master/noir_stdlib) - the Noir Standard Library
- [Ethereum Storage Proof Verification](https://github.com/aragonzkresearch/noir-trie-proofs) - a library that contains the primitives necessary for RLP decoding (in the form of look-up table construction) and Ethereum state and storage proof verification (or verification of any trie proof involving 32-byte long keys)
- [BigInt](https://github.com/shuklaayush/noir-bigint) - a library that provides a custom BigUint56 data type, allowing for computations on large unsigned integers
- [ECrecover](https://github.com/colinnielsen/ecrecover-noir/tree/main) - a library to verify an ECDSA signature and return the source Ethereum address
- [Sparse Merkle Tree Verifier](https://github.com/vocdoni/smtverifier-noir/tree/main) - a library for verification of sparse Merkle trees
- [Signed Int](https://github.com/resurgencelabs/signed_int) - a library for accessing a custom Signed Integer data type, allowing access to negative numbers on Noir
- [Fraction](https://github.com/resurgencelabs/fraction) - a library for accessing fractional number data type in Noir, allowing results that aren't whole numbers

See the section on [dependencies](./modules_packages_crates/dependencies) for more information.

# The Noir Programming Language

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable to be used in production. Expect bugs!**

## Quick Start

Read the installation section [here](https://noir-lang.github.io/book/getting_started/nargo/installation.html).

Once you have read through the documentation, you can also run the examples located in the `examples` folder.

## Current Features

Backends:

- Barretenberg via FFI
- Marlin via arkworks

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

## Future Work

The current focus is to gather as much feedback as possible while in the alpha phase. The main focuses of Noir are _safety_ and _developer experience_. If you find a feature that does not seem to be in line with these goals, please open an issue!

Concretely the following items are on the road map:

- General code sanitization and documentation (ongoing effort)
- Prover and Verifier Key logic. (Prover and Verifier pre-process per compile)
- Fallback mechanism for backend unsupported opcodes
- Visibility modifiers
- Signed integers
- Backend integration: (Bulletproofs)
- Recursion
- Big integers

## Nargo CLI - pre-built

`nargo` - command line interface tool for interacting with Noir programs - allows compiling, proving, verifying, and more. Nightly binary builds can be found [here](https://github.com/noir-lang/noir/releases/tag/nightly). Please refer [noir-lang/build-nargo](https://github.com/noir-lang/build-nargo) to inspect how these are built for various platforms.

## Nargo CLI - install scripts

[noir-lang/noirup](https://github.com/noir-lang/noirup) repository contains install scripts for Linux, macOS, and Windows systems to allow easy installation.

## Minimum Rust version

This crate's minimum supported rustc version is 1.64.0.

## License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Barretenberg License

Barretenberg is licensed under GPL V2.0.

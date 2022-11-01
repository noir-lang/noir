# The Noir Programming Language

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable to be used in production. Expect bugs!**

## Quick Start

Read the installation section [here](https://noir-lang.github.io/book/getting_started/nargo/installation.html).

Once you have read through the documentation, you can also run the examples located in the `examples` folder.

## Current Features

Backends:

 - Barretenberg via FFI

Compiler:

 - Module System
 - For expressions
 - Arrays
 - Bit Operations, except for OR
 - Binary operations (<, <=, >, >=, +, -, *, /) [See documentation for an extensive list]
 - Unsigned integers

ACIR Supported OPCODES:

 - Sha256
 - Blake2s
 - Schnorr signature verification
 - MerkleMembership
 - Pedersen
 - HashToField

## Future Work

The current focus is to gather as much feedback as possible while in the alpha phase. The main focusses of Noir are _safety_ and _developer experience_. If you find a feature that does not seem to be inline with these goals, please open an issue!

Concretely the following items are on the road map:

- If statements
- OR operator
- General code sanitisation and documentation
- Prover and Verifier Key logic. (Prover and Verifier preprocess per compile)
- Structures
- Visibility modifiers
- Signed integers
- Backend integration: (Marlin, Bulletproofs)
- Recursion

## License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Barretenberg License

Barretenberg is currently the only backend that Noir has integrated. It is licensed under GPL V2.0.

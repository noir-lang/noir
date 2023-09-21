# The Noir Programming Language

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable to be used in production. Expect bugs!**

## Quick Start

Read the installation section [here](https://noir-lang.org/getting_started/nargo_installation).

Once you have read through the documentation, you can visit [Awesome Noir](https://github.com/noir-lang/awesome-noir) to run some of the examples that others have created.

## Current Features

Backends:

- Barretenberg via FFI
- Marlin via arkworks (Note -- latest interfaces may not be updated to support Marlin backend. Please open an issue if this is relevant to your project and requires attention.)

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

## Minimum Rust version

This crate's minimum supported rustc version is 1.67.0.

## Working on this project

This project uses [Nix](https://nixos.org/) and [direnv](https://direnv.net/) to streamline the development experience. Please follow [our guidelines](https://noir-lang.org/getting_started/nargo_installation/#option-4-compile-from-source) to setup your environment for working on the project.

### Building against a different local/remote version of Barretenberg

If you are working on this project and want a different version of Barretenberg (instead of the version this project is pinned against), you'll want to replace the lockfile version with your version. This can be done by running:

```sh
nix flake lock --override-input barretenberg /absolute/path/to/your/barretenberg
```

You can also point at a fork and/or branch on GitHub using:

```sh
nix flake lock --override-input barretenberg github:username/barretenberg/branch_name
```

__Note:__ You don't want to commit the updated lockfile, as it will fail in CI!

## License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

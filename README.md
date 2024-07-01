# blocksense.network

## Overview

The [blocksense.network](https://blocksense.network) team is working (as of May
2024) on a PLONKY2 backend to Noir. This backend can be used for proving and
verifying circuits. We have reached the milestone where a fairly non-trivial
program can be compiled and proved with the new PLONKY2 backend.

In order to check this for yourself follow these steps. If you have NixOS, setup is slightly easier:

1. Checkout this repo and the `plonky2` branch
2. Issue `direnv allow` to get the context set up
3. Run `cargo test zk_dungeon`

If you have another Linux, you need to make sure you have the right version of
Rust:

1. Checkout this repo and the `plonky2` branch
2. Make sure you have `rustup` installed
3. Select the `nightly` version of Rust as the default one
  - `rustup default nightly`
4. Run `cargo test zk_dungeon`

If the test passes then you just confirmed that the PLONKY2 backend for Noir
works for you too!

## Run manually

To run the PLONKY2 backend manually, call `nargo prove` and construct proofs for
ZK circuits written in Noir. Once you have a proof, `nargo verify` can be used
to verify that it is correct.

## More details

To have a look at the ZK program that is the subject of the test look at the
directory `test_programs/plonky2_prove_success/zk_dungeon` and in particular at
`src/dungeon.nr`. This program is a solution to the second part of the
"Discovering Noir" campaign at https://nodeguardians.io/. The task is to verify
that the prover knows an eight-step path of a knight on a chess board that
starts from a given location, reaches another location, and avoids being
attacked by a set of opposing bishops.

Another proof-of-concept feature is the fact that the sha256 hashing algorithm
is implemented by the PLONKY2 backend as an intrinsic function, as demonstrated
by the `test_programs/plonky2_prove_success/sha256` test (as well as the
`test_programs/plonky2_prove_failure/sha256` test).

The next steps for this project are to add more intrinsics, better debugging
capabilities and more. Investigating the potential support for recursion is
particularly interesting.

## Why PLONKY2 backend does not adhere to Noir backend API

The purpose of this section is to outline the reason why this new backend does
not follow the Backend API as anticipated by the Noir team, but instead,
translates the earlier SSA form of the intermediate representation into PLONKY2
primitives which are then used to carry out a ZK proof.

### Backend API

Historically (until 23 May 2024), Noir provided the `nargo prove` and `nargo
verify` commands, which internally called the Barretenberg backend as a proving
system. At the end of May 2024, the Noir team removed that feature, decoupling
their compiler from the way the proof is performed.

To use PLONKY2 as a proving system, it is natural to keep the `nargo prove` and
`nargo verify` commands. The way they work now (after they have been removed
upstream) is to keep the same compiler pipeline as upstream Noir until the final
SSA form is generated and optimized. After the optimization phases, we fork the
pipeline and instead of generating ACIR code, we generate PLONKY2 operations.
 * When ACIR is generated, the ZK program can be executed or debugged.
 * When PLONKY2 is generated, a proof can be generated or a proof can be verified.

The main reason for producing an alternative intermediate representation for the
program, is that PLONKY2 has direct implementation for some of the
intermediate-level operations, which the ACIR backend translates to combinations
of several other operations. If we didn't do that, but tried to convert ACIR to
PLONKY2 instead, we would have to pattern match combinations of instructions to
single or combinations of PLONKY2 operations. Compared to our approach that
would be harder and less likely to produce as few operations.

the blocksense.network team

| Original README follows. |
|--------------------------|

<div align="center">
  <picture>
    <img src="./noir-logo.png" alt="The Noir Programming Language" width="35%">
  </picture>

[Website][Noir] | [Getting started] | [Documentation] | [Contributing]
</div>



# The Noir Programming Language

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable to be used in production. Expect bugs!**

## Quick Start

Read the [installation section][Getting started] from the [Noir docs][Documentation].

Once you have read through the documentation, you can visit [Awesome Noir](https://github.com/noir-lang/awesome-noir) to run some of the examples that others have created.

## Getting Help

Join the Noir [forum][Forum] or [Discord][Discord]

## Contributing

See [CONTRIBUTING.md][CONTRIBUTING].

## Future Work

The current focus is to gather as much feedback as possible while in the alpha phase. The main focuses of Noir are _safety_ and _developer experience_. If you find a feature that does not seem to be in line with these goals, please open an issue!

## Minimum Rust version

This workspace's minimum supported rustc version is 1.74.1.

## License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[Noir]: https://www.noir-lang.org/
[Getting Started]: https://noir-lang.org/docs/getting_started/installation/
[Forum]: https://forum.aztec.network/c/noir
[Discord]: https://discord.gg/JtqzkdeQ6G
[Documentation]: https://noir-lang.org/docs
[Contributing]: CONTRIBUTING.md

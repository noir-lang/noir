# blocksense.network

## Overview

The [blocksense.network](https://blocksense.network) team is working (as of May
2024) on a PLONKY2 backend to Noir. This backend can be used for proving and
verifying circuits. We have reached the milestone where a fairly non-trivial
program can be compiled and proved with the new PLONKY2 backend.

In order to check this for yourself follow these steps:

1. Checkout this repo and the `plonky2` branch
2. Make sure you have `rustup` installed
3. Select the `nightly` version of Rust as the default one
  - `rustup default nightly`
4. Run `cargo test zk_dungeon`

If the test passes then you just confirmed that the PLONKY2 backend for Noir
works for you too!

## Run manually

To run the PLONKY2 backend manually, call `nargo prove` and construct proofs for
ZK circuits written in Noir.

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

The next steps for this project are to add verification, more intrinsics,
better debugging capabilities and more. Investigating the potential support for
recursion is particularly interesting.

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

This is an improvement for the mainline Noir workflow. At the same time, the
PLONKY2 backend that we're developing fits more naturally with the Noir
frontend, since it is written in Rust and called as a library rather than an
external binary. For this reason, while Noir have removed the aforementioned
commands, we have kept them for PLONKY2 proofs.

In addition to the more natural fit, the user experience for the PLONKY2 backend
is also simpler this way, when compared to requiring users to execute an
additional step for proving or verifying their circuits. We might reconsider
this decision in the future, but for the time being this is the path we're
taking.

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

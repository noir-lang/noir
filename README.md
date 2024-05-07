# blocksense.network

## Overview

The [blocksense.network](https://blocksense.network) team is working (as of
April 2024) towards contributing a PLONKY2 backend to Noir. This backend can be
used in place of brillig for proving and verifying circuits. We have reached
the milestone where a fairly non-trivial program can be compiled and proved
with the new PLONKY2 backend.

In order to check this for yourself follow these steps:

1. Checkout this repo and the `plonky2` branch
2. Make sure you have `rustup` installed
3. Select the `nightly` version of Rust as the default one
  - `rustup default nightly`
4. Run `cargo test zk_dungeon`

If the test passes then you just confirmed that the PLONKY2 backend for Noir
works for you too!

## Run manually

To run the PLONKY2 backend manually, pass the
`--use-plonky2-backend-experimental` flag to `nargo prove` when using it to
construct proofs for ZK circuits written in Noir.

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

`tooling/nargo_cli/src/cli/prove_cmd.rs` demonstrates how the Noir compiler is
expected to be used programmatically. There are a few steps involved:

1. A FileManager (compiler/fm/src/lib.rs) is constructed from the project dir
   using file_manager_with_stdlib (compiler/noirc_driver/src/lib.rs);
2. parse_all (tooling/nargo/src/lib.rs) turns it into a ParsedFiles
   (compiler/noirc_frontend/src/hir/mod.rs) object;
3. compile_program (tooling/nargo/src/ops/compile.rs) produces a
   CompiledProgram (compiler/noirc_driver/src/lib.rs) from it;
4. prove_package (nargo_cli/src/cli/prove_cmd.rs) uses the CompiledProgram
   object and returns a CliError or nothing on success; it takes a Backend
   (tooling/backend_interface/src/lib.rs) object as input in order to carry out
   the proof (calls the prove method on it and then saves the returned binary
   to disk).

From this structure it is obvious that the most natural way to implement a new
backend is to implement the Backend interface and pass an instance of this new
implementation to prove_package. This would require no changes to the general
structure and would be the cleanest and most maintainable approach.

### What we did instead

The approach taken by the PLONKY2 backend implementation that we produced is the following:

1. we append the data flow at step 3 from the previous section: compile_program
   is modified to not only produce the previous program artifacts (most
   notably, an AcirProgram [type Program defined in
   acvm-repo/acir/src/circuit/mod.rs; renamed as AcirProgram in
   compiler/noirc_evaluator/src/ssa.rs] object, which is the core
   representation of the program), but also a Plonky2Circuit [new type defined
   by our prototype], which is a duplicate representation of the program, using
   the plonky2 primitives;
2. we modify the control flow at step 4 from the previous section:
   prove_package is modified to switch the control flow, depending on a new CLI
   flag (--use-plonky2-backend-experimental). If the flag is enabled, the input
   Backend object is ignored. Instead, the prove method on the Plonky2Circuit
   object is called and the result from that is used when writing out the proof
   to disk in step 4.

### Why we did it this way

The main reason for producing an alternative intermediate representation for
the program (which is a bit of a hack), is that PLONKY2 has direct
implementation for some of the intermediate-level operations, which the ACIR
backend translates to combinations of several other operations.

We wanted to enable the compilation of programs that do not depend on Brillig
in order to avoid bloating the circuits with the additional functionality. By
doing this, we could also avoid trying to figure out a way of implementing
these functions (e.g. Jump, JumpIf, JumpIfNot, Call, ForeignCall, Mov,
ConditionalMov, etc.) in PLONKY2.

The following are some examples of "too much work done" by the conversion from
SSA into ACIR which we want to avoid for our backend:

1. add_var, sub_var, mul_var, and many others
   [compiler/noirc_evaluator/src/ssa/acir_gen/acir_ir/acir_variable.rs] convert
   the respective operations into a combination of addition and multiplication
   via the add_mul_var method; in PLONKY2, the primitives allow us to directly
   specify many arithmetic operations, **so this conversion is something that at
   best we could undo, but most likely will make it impossible to figure out
   what was the original operation that lead to the produced IR**; we would need
   this original operation to produce more efficient PLONKY2 code (with less
   operations);
2. div_var and mod_var represent division in target-specific ways, introducing
   calls to Brillig functions; again, it would be infeasible to try to analyze
   this generated code to figure out the intended operation is division, so
   that we can generate a PLONKY2 implementation of this operation;
3. convert_ssa_binary [compiler/noirc_evaluator/src/ssa/acir_gen/mod.rs] throws
   an ICE for Shl and Shr (bit-shift operators); we might want to handle these
   in our backend, but if we use this representation we won't be able to.

In addition to these concerns, another consideration is that the Barretenberg
backend is invoked as a separate binary, rather than via function calls (either
linking the code statically or dynamically as a standalone library). While this
is not a deal-breaker, we found it easier to deploy Noir locally if this was
not the case. The Backend interface is designed with this implementation detail
in place (e.g. see the assert_binary_exists
[tooling/backend_interface/src/lib.rs] method on the interface), which
distracts from the main purpose of the Backend interface: to carry out proofs
and to verify them.

### Recommendations

We believe that it would be cleaner if what is now called Backend is renamed to
ExternalProver for clarity. A new interface boundary called "Backend" could be
introduced between compile_program and this new ExternalProver that lowers the
SSA IR to an appropriate lower-level representation (either ACIR or PLONKY2 or
the next big proving system target). In this way, there would be no need to
duplicate the program representation as ACIR and PLONKY2 as it is currently
done in our prototype, but the correct one could be generated and then later
used by the ExternalProver (or a more abstract Prover, that is not necessarily
invoked as a binary) to carry out the proof.

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

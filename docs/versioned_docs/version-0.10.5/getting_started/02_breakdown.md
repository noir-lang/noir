---
title: Project Breakdown
description:
  Learn about the anatomy of a Nargo project, including the purpose of the Prover and Verifier TOML
  files, and how to prove and verify your program.
keywords:
  [Nargo, Nargo project, Prover.toml, Verifier.toml, proof verification, private asset transfer]
---

This section breaks down our hello world program in section _1.2_. We elaborate on the project
structure and what the `prove` and `verify` commands did in the previous section.

## Anatomy of a Nargo Project

Upon creating a new project with `nargo new` and building the in/output files with `nargo check`
commands, you would get a minimal Nargo project of the following structure:

    - src
    - Prover.toml
    - Verifier.toml
    - Nargo.toml

The source directory _src_ holds the source code for your Noir program. By default only a _main.nr_
file will be generated within it.

### Prover.toml

_Prover.toml_ is used for specifying the input values for executing and proving the program. You can specify `toml` files with different names by using the `--prover-name` or `-p` flags, see the [Prover](#provertoml) section below. Optionally you may specify expected output values for prove-time checking as well.

### Verifier.toml

_Verifier.toml_ contains public in/output values computed when executing the Noir program.

### Nargo.toml

_Nargo.toml_ contains the environmental options of your project. It contains a "package" section and a "dependencies" section.

Example Nargo.toml:

```toml
[package]
name = "noirstarter"
type = "bin"
authors = ["Alice"]
compiler_version = "0.9.0"
description = "Getting started with Noir"
entry = "circuit/main.nr"
license = "MIT"

[dependencies]
ecrecover = {tag = "v0.9.0", git = "https://github.com/colinnielsen/ecrecover-noir.git"}
```

Nargo.toml for a [workspace](../modules_packages_crates/workspaces) will look a bit different. For example:

```toml
[workspace]
members = ["crates/a", "crates/b"]
default-member = "crates/a"
```

#### Package section

The package section requires a number of fields including:

- `name` (**required**) - the name of the package
- `type` (**required**) - can be "bin", "lib", or "contract" to specify whether its a binary, library or Aztec contract
- `authors` (optional) - authors of the project
- `compiler_version` (optional) - specifies the version of the compiler to use. This is not currently enforced by the compiler, but will be in future versions.
- `description` (optional)
- `entry` (optional) - a relative filepath to use as the entry point into your package (overrides the default of `src/lib.nr` or `src/main.nr`)
- `backend` (optional)
- `license` (optional)

#### Dependencies section

This is where you will specify any dependencies for your project. See the [Dependencies page](../modules_packages_crates/dependencies) for more info.

`./proofs/` and `./contract/` directories will not be immediately visible until you create a proof or
verifier contract respectively.

### main.nr

The _main.nr_ file contains a `main` method, this method is the entry point into your Noir program.

In our sample program, _main.nr_ looks like this:

```rust
fn main(x : Field, y : Field) {
    assert(x != y);
}
```

The parameters `x` and `y` can be seen as the API for the program and must be supplied by the
prover. Since neither `x` nor `y` is marked as public, the verifier does not supply any inputs, when
verifying the proof.

The prover supplies the values for `x` and `y` in the _Prover.toml_ file.

As for the program body, `assert` ensures the satisfaction of the condition (e.g. `x != y`) is
constrained by the proof of the execution of said program (i.e. if the condition was not met, the
verifier would reject the proof as an invalid proof).

### Prover.toml

The _Prover.toml_ file is a file which the prover uses to supply his witness values(both private and
public).

In our hello world program the _Prover.toml_ file looks like this:

```toml
x = "1"
y = "2"
```

When the command `nargo prove` is executed, two processes happen:

1. Noir creates a proof that `x` which holds the value of `1` and `y` which holds the value of `2`
   is not equal. This not equal constraint is due to the line `assert(x != y)`.

2. Noir creates and stores the proof of this statement in the _proofs_ directory in a file called your-project.proof. So if your project is named "private_voting" (defined in the project Nargo.toml), the proof will be saved at `./proofs/private_voting.proof`. Opening this file will display the proof in hex format.

#### Arrays of Structs

The following code shows how to pass an array of structs to a Noir program to generate a proof.

```rust
// main.nr
struct Foo {
    bar: Field,
    baz: Field,
}

fn main(foos: [Foo; 3]) -> pub Field {
    foos[2].bar + foos[2].baz
}
```

Prover.toml:

```toml
[[foos]] # foos[0]
bar = 0
baz = 0

[[foos]] # foos[1]
bar = 0
baz = 0

[[foos]] # foos[2]
bar = 1
baz = 2
```

#### Custom toml files

You can specify a `toml` file with a different name to use for proving by using the `--prover-name` or `-p` flags.

This command looks for proof inputs in the default **Prover.toml** and generates the proof and saves it at `./proofs/<project-name>.proof`:

```bash
nargo prove
```

This command looks for proof inputs in the custom **OtherProver.toml** and generates proof and saves it at `./proofs/<project-name>.proof`:

```bash
nargo prove -p OtherProver
```

## Verifying a Proof

When the command `nargo verify` is executed, two processes happen:

1. Noir checks in the _proofs_ directory for a proof file with the project name (eg. test_project.proof)

2. If that file is found, the proof's validity is checked

> **Note:** The validity of the proof is linked to the current Noir program; if the program is
> changed and the verifier verifies the proof, it will fail because the proof is not valid for the
> _modified_ Noir program.

In production, the prover and the verifier are usually two separate entities. A prover would
retrieve the necessary inputs, execute the Noir program, generate a proof and pass it to the
verifier. The verifier would then retrieve the public inputs from usually external sources and
verifies the validity of the proof against it.

Take a private asset transfer as an example:

A user on browser as the prover would retrieve private inputs (e.g. the user's private key) and
public inputs (e.g. the user's encrypted balance on-chain), compute the transfer, generate a proof
and submit it to the verifier smart contract.

The verifier contract would then draw the user's encrypted balance directly from the blockchain and
verify the proof submitted against it. If the verification passes, additional functions in the
verifier contract could trigger (e.g. approve the asset transfer).

Now that you understand the concepts, you'll probably want some editor feedback while you are writing more complex code.

In the [next section](language_server), we will explain how to utilize the Noir Language Server.

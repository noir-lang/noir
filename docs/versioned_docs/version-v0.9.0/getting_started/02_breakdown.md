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

_Prover.toml_ is used for specifying the input values for executing and proving the program. You can specify `toml` files with different names by using the `--prover-name` or `-p` flags, see the [Prover](#provertoml) section below. Optionally you may specify expected output values for prove-time checking as well.

_Verifier.toml_ contains public in/output values computed when executing the Noir program.

_Nargo.toml_ contains the environmental options of your project.

_proofs_ and _contract_ directories will not be immediately visible until you create a proof or
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

When the command `nargo prove my_proof` is executed, two processes happen:

1. Noir creates a proof that `x` which holds the value of `1` and `y` which holds the value of `2`
   is not equal. This not equal constraint is due to the line `assert(x != y)`.

2. Noir creates and stores the proof of this statement in the _proofs_ directory and names the proof
   file _my_proof_. Opening this file will display the proof in hex format.

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

This command looks for proof inputs in the default **Prover.toml** and generates proof `p`:

```bash
nargo prove p
```

This command looks for proof inputs in the custom **OtherProver.toml** and generates proof `pp`:

```bash
nargo prove -p OtherProver pp
```

## Verifying a Proof

When the command `nargo verify my_proof` is executed, two processes happen:

1. Noir checks in the _proofs_ directory for a file called _my_proof_

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

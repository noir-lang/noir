---
title: Creating a Project
description:
  Learn how to create and verify your first Noir program using Nargo, a programming language for
  zero-knowledge proofs.
keywords:
  [
    Nargo,
    Noir,
    zero-knowledge proofs,
    programming language,
    create Noir program,
    verify Noir program,
    step-by-step guide,
  ]
sidebar_position: 1

---

Now that we have installed Nargo, it is time to make our first hello world program!

## Create a Project Directory

Noir code can live anywhere on your computer. Let us create a _projects_ folder in the home
directory to house our Noir programs.

For Linux, macOS, and Windows PowerShell, create the directory and change directory into it by
running:

```sh
mkdir ~/projects
cd ~/projects
```

## Create Our First Nargo Project

Now that we are in the projects directory, create a new Nargo project by running:

```sh
nargo new hello_world
```

> **Note:** `hello_world` can be any arbitrary project name, we are simply using `hello_world` for
> demonstration.
>
> In production, the common practice is to name the project folder as `circuits` for better
> identifiability when sitting alongside other folders in the codebase (e.g. `contracts`, `scripts`,
> `test`).

A `hello_world` folder would be created. Similar to Rust, the folder houses _src/main.nr_ and
_Nargo.toml_ which contain the source code and environmental options of your Noir program
respectively.

### Intro to Noir Syntax

Let us take a closer look at _main.nr_. The default _main.nr_ generated should look like this:

```rust
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

The first line of the program specifies the program's inputs:

```rust
x : Field, y : pub Field
```

Program inputs in Noir are private by default (e.g. `x`), but can be labeled public using the
keyword `pub` (e.g. `y`). To learn more about private and public values, check the
[Data Types](../../noir/concepts/data_types/index.md) section.

The next line of the program specifies its body:

```rust
assert(x != y);
```

The Noir syntax `assert` can be interpreted as something similar to constraints in other zk-contract languages.

For more Noir syntax, check the [Language Concepts](../../noir/concepts/comments.md) chapter.

## Build In/Output Files

Change directory into _hello_world_ and build in/output files for your Noir program by running:

```sh
cd hello_world
nargo check
```

A _Prover.toml_ file will be generated in your project directory, to allow specifying input values to the program.

## Execute Our Noir Program

Now that the project is set up, we can execute our Noir program.

Fill in input values for execution in the _Prover.toml_ file. For example:

```toml
x = "1"
y = "2"
```

Execute your Noir program:

```sh
nargo execute witness-name
```

The witness corresponding to this execution will then be written to the file `./target/witness-name.gz`.

## Prove Our Noir Program

:::info

Nargo no longer handles communicating with backends in order to generate proofs. In order to prove/verify your Noir programs, you'll need an installation of [bb](../barretenberg/index.md).

:::

Prove the valid execution of your Noir program using `bb`:

```sh
bb prove -b ./target/hello_world.json -w ./target/witness-name.gz -o ./proof
```

A new file called `proof` will be generated in your project directory, containing the generated proof for your program.

## Verify Our Noir Program

Once a proof is generated, we can verify correct execution of our Noir program by verifying the proof file.

Verify your proof by running:

```sh
bb write_vk -b ./target/hello_world.json -o ./target/vk
bb verify -k ./target/vk -p ./proof
```

The verification will complete in silence if it is successful. If it fails, it will log the corresponding error instead.

Congratulations, you have now created and verified a proof for your very first Noir program!

In the [next section](./project_breakdown.md), we will go into more detail on each step performed.

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

Now that we have installed Nargo and a proving backend, it is time to make our first hello world program!

### 1. Create a new project directory

Noir code can live anywhere on your computer. Let us create a _projects_ folder in the home
directory to house our first Noir program.

Create the directory and change directory into it by running:

```sh
mkdir ~/projects
cd ~/projects
```

## Nargo

Nargo provides the ability to initiate and execute Noir projects. Read the [Nargo installation](../installation/index.md) section to learn more about Nargo and how to install it.

### 2. Create a new Noir project

Now that we are in the projects directory, create a new Nargo project by running:

```sh
nargo new hello_world
```

`hello_world` can be any arbitrary project name, we are simply using `hello_world` for demonstration.

In production, it is common practice to name the project folder, `circuits`, for clarity amongst other folders in the codebase (like: `contracts`, `scripts`, and `test`).

A `hello_world` folder would be created. Similar to Rust, the folder houses _src/main.nr_ and
_Nargo.toml_ which contain the source code and environmental options of your Noir program
respectively.

#### Intro to Noir Syntax

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

### 3. Build in/output files

Change directory into _hello_world_ and build in/output files for your Noir program by running:

```sh
cd hello_world
nargo check
```

A _Prover.toml_ file will be generated in your project directory, to allow specifying input values to the program.

### 4. Execute the Noir program

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

The command also automatically compiles your Noir program if it was not already / was edited, which you may notice the compiled artifacts being written to the file `./target/hello_world.json`.

## Proving Backend

Proving backends provide the ability to generate and verify proofs of executing Noir programs, following Noir's tooling that compiles and executes the programs. Read the [proving backend installation](../backend/index.md) section to learn more about proving backends and how to install them.

Barretenberg is used as an example here to demonstrate how proving and verifying could be implemented and used. Read the [`bb` installation](../backend/index.md#example-installing-bb) section for how to install Barretenberg's CLI tool; refer to [`bb`'s documentation](https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/bb/readme.md) for full details about the tool.

### 5. Prove an execution of the Noir program

Using Barretenberg as an example, prove the valid execution of your Noir program running:

```sh
bb prove -b ./target/hello_world.json -w ./target/witness-name.gz -o ./target/proof
```

The proof generated will then be written to the file `./target/proof`.

:::tip
Since the params for `nargo` and `bb` often specify multiple filenames to read from or write to, remember to check each command is referring to the desired filenames.
Or for greater certainty, delete the target folder and go through each step again (compile, witness, prove, ...) to ensure files generated in past commands are being referenced in future ones.
:::

### 6. Verify the execution proof

Once a proof is generated, we can verify correct execution of our Noir program by verifying the proof file.

Using Barretenberg as an example, compute the verification key for the Noir program by running:

```sh
bb write_vk -b ./target/hello_world.json -o ./target/vk
```

And verify your proof by running:

```sh
bb verify -k ./target/vk -p ./target/proof
```

If successful, the verification will complete in silence; if unsuccessful, the command will trigger logging of the corresponding error.

Congratulations, you have now created and verified a proof for your very first Noir program!

In the [next section](./project_breakdown.md), we will go into more detail on each step performed.

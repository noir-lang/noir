---
title: Create A Project
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

For Windows CMD, run:

```sh
> mkdir "%USERPROFILE%\projects"
> cd /d "%USERPROFILE%\projects"
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
_Nargo.toml_ that contains the source code and environmental options of your Noir program
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
[Data Types](../language_concepts/data_types) section.

The next line of the program specifies its body:

```rust
assert(x != y);
```

The Noir syntax `assert` can be interpreted as something similar to constraints in other zk-contract languages.

For more Noir syntax, check the [Language Concepts](../language_concepts/comments) chapter.

## Build In/Output Files

Change directory into _hello_world_ and build in/output files for your Noir program by running:

```sh
cd hello_world
nargo check
```

Two additional files would be generated in your project directory:

_Prover.toml_ houses input values, and _Verifier.toml_ houses public values.

## Prove Our Noir Program

Now that the project is set up, we can create a proof of correct execution on our Noir program.

Fill in input values for execution in the _Prover.toml_ file. For example:

```toml
x = "1"
y = "2"
```

Prove the valid execution of your Noir program:

```sh
nargo prove
```

A new folder _proofs_ would then be generated in your project directory, containing the proof file
`<project-name>.proof`, where the project name is defined in Nargo.toml.

The _Verifier.toml_ file would also be updated with the public values computed from program
execution (in this case the value of `y`):

```toml
y = "0x0000000000000000000000000000000000000000000000000000000000000002"
```

> **Note:** Values in _Verifier.toml_ are computed as 32-byte hex values.

## Verify Our Noir Program

Once a proof is generated, we can verify correct execution of our Noir program by verifying the
proof file.

Verify your proof by running:

```sh
nargo verify
```

The verification will complete in silence if it is successful. If it fails, it will log the
corresponding error instead.

Congratulations, you have now created and verified a proof for your very first Noir program!

In the [next section](breakdown), we will go into more detail on each step performed.

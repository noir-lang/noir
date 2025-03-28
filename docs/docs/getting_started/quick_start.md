---
title: Quick Start
tags: []
sidebar_position: 0
---

## Installation

The easiest way to develop with Noir is using Nargo the CLI tool. It provides you the ability to start new projects, compile, execute and test Noir programs from the terminal.

You can use `noirup` the installation script to quickly install and update Nargo:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
noirup
```

Once installed, you can [set up shell completions for the `nargo` command](setting_up_shell_completions).

Nargo provides the ability to initiate and execute Noir projects. Let's initialize the traditional `hello_world`:

```sh
nargo new hello_world
```

Two files will be created.

- `src/main.nr` contains a simple boilerplate circuit
- `Nargo.toml` contains environmental options, such as name, author, dependencies, and others.

Glancing at _main.nr_ , we can see that inputs in Noir are private by default, but can be labeled public using the keyword `pub`. This means that we will _assert_ that we know a value `x` which is different from `y` without revealing `x`:

```rust
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

To learn more about private and public values, check the [Data Types](../noir/concepts/data_types/index.md) section.

## Compiling and executing

We can now use `nargo` to generate a _Prover.toml_ file, where our input values will be specified:

```sh
cd hello_world
nargo check
```

Let's feed some valid values into this file:

```toml
x = "1"
y = "2"
```

We're now ready to compile and execute our Noir program. By default the `nargo execute` command will do both, and generate the `witness` that we need to feed to our proving backend:

```sh
nargo execute
```

The witness corresponding to this execution will then be written to the file _./target/witness-name.gz_.

The command also automatically compiles your Noir program if it was not already / was edited, which you may notice the compiled artifacts being written to the file _./target/hello_world.json_.

With circuit compiled and witness generated, we're ready to prove.

## Next steps

Noir is often called a "proving frontend", because it doesn't do any proving by itself. The compilation and the witness (often called _partial witness_) needs to be fed into a proving backend. Proving backends provide you the abilities to generate proofs, verify proofs, generate smart contracts and more for your Noir programs.

They also provide [blackbox functions](https://github.com/noir-lang/noir/blob/master/acvm-repo/acir/src/circuit/black_box_functions.rs): implementations that match specific Noir opcodes. Currently, only Aztec's Barretenberg supports all the blackbox functions in the Noir language. Follow up with its [quickstart guide]()

For the full list of proving backends compatible with Noir, visit [Awesome Noir](https://github.com/noir-lang/awesome-noir/?tab=readme-ov-file#proving-backends).

---
title: Quick Start
tags: []
sidebar_position: 0
---

## Installation

### Noir

The easiest way to develop with Noir is using Nargo the CLI tool. It provides you the ability to start new projects, compile, execute and test Noir programs from the terminal.

You can use `noirup` the installation script to quickly install and update Nargo:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
noirup
```

Once installed, you can [set up shell completions for the `nargo` command](setting_up_shell_completions).

### Proving backend

After installing Noir, we install a proving backend to work with our Noir programs.

Proving backends provide you the abilities to generate proofs, verify proofs, generate smart contracts and more for your Noir programs.

Different proving backends provide different tools for working with Noir programs, here we will use the [Barretenberg proving backend](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg) developed by Aztec Labs as an example.

You can use the `bbup` installation script to quickly install and update BB, Barretenberg's CLI tool:

You can find the full list of proving backends compatible with Noir in Awesome Noir.

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup
```

For the full list of proving backends compatible with Noir, visit [Awesome Noir](https://github.com/noir-lang/awesome-noir/?tab=readme-ov-file#proving-backends).

## Nargo

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

### Compiling and executing

We can now use `nargo` to generate a _Prover.toml_ file, where our input values will be specified:

```sh
cd hello_world
nargo check

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

## Proving backend

Different proving backends may provide different tools and commands to work with Noir programs. Here Barretenberg's `bb` CLI tool is used as an example:

```sh
bb prove -b ./target/hello_world.json -w ./target/hello_world.gz -o ./target/proof
```

:::tip

Naming can be confusing, specially as you pass them to the `bb` commands. If unsure, it won't hurt to delete the target folder and start fresh to make sure you're using the most recent versions of the compiled circuit and witness.

:::

The proof is now generated in the `target` folder. To verify it we first need to compute the verification key from the compiled circuit, and use it to verify:

```sh
bb write_vk -b ./target/hello_world.json -o ./target/vk
bb verify -k ./target/vk -p ./target/proof
```

:::info

Notice that in order to verify a proof, the verifier knows nothing but the circuit, which is compiled and used to generate the verification key. This is obviously quite important: private inputs remain private.

As for the public inputs, you may have noticed they haven't been specified. This behavior varies with each particular backend, but barretenberg typically attaches them to the proof. You can see them by parsing and splitting it. For example for if your public inputs are 32 bytes:

```bash
head -c 32 ./target/proof | od -An -v -t x1 | tr -d $' \n'
```

:::

Congratulations, you have now created and verified a proof for your very first Noir program!

In the [next section](./project_breakdown.md), we will go into more detail on each step performed.

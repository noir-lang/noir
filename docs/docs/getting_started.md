---
title: Getting Started
tags: []
---

## A Quick Start with Noirenberg

Noir is a backend-agnostic programming language for writing zero-knowledge proofs. This means you need to pair it with a proving backend. You can visit the [nargo installation](./getting_started/installation/index.md) page for details on using Noir without any particular backend.

As an example, you can use the convenient binary [noirenberg](https://raw.githubusercontent.com/noir-lang/noirenberg/main/install.sh) script, which installs Noir along with Aztec Lab's [Barretenberg backend](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg):

```bash
curl -fsSL noiren.be | bash
```

:::info

It's good practice to inspect what you run. This script is hosted [here](https://raw.githubusercontent.com/AztecProtocol/noirenberg/main/install.sh) and installs [`nvm`](https://github.com/nvm-sh/nvm/tree/master), [`node`](https://nodejs.org/en), and the [`noirenberg`](https://raw.githubusercontent.com/noir-lang/noirenberg/main/npx.ts) binaries.

:::

We now have `noirenberg` ready.  Open a new terminal and run:

```bash
noirenberg
```

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

To learn more about private and public values, check the [Data Types](../../noir/concepts/data_types/index.md) section.

### Compiling and executing

We can now use `nargo` to generate a _Prover.toml_ file, where our input values will be specified:

```sh
nargo check
```

Let's feed some valid values into this file:

```toml
x = "1"
y = "2"
```

We're now ready to compile and execute our Noir program. By default the `nargo execute` command will do both, and generate the `witness` that we need to feed to our proving backend:

```sh
nargo execute hello-witness
```

You can now see the witness and the compiled circuit in the `target` folder. We're ready to prove

## Barretenberg

Barretenberg is a proving backend maintained by [Aztec Labs](https://www.aztec-labs.com/).

Proving backends provide the ability to generate and verify proofs. In this example, `noirenberg` already installed `bb`, Barretenberg's CLI tool. You can read more about it in its [documentation](https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/bb/readme.md). Let's use it to prove the valid execution of your Noir program:

```sh
bb prove -b ./target/hello_world.json -w ./target/hello-witness.gz -o ./target/proof
```

:::tip

Naming can be confusing, specially as you pass them to the `bb` commands. If unsure, it won't hurt to delete the target folder and start anew to make sure you're using the most recent versions of the compiled circuit and witness.

:::

The proof is now generated in the `target` folder. To verify it we first need to compute the verification key from the compiled circuit, and use it to verify:

```sh
bb write_vk -b ./target/hello_world.json -o ./target/vk
bb verify -k ./target/vk -p ./target/proof
```

:::info

Notice that in order to verify a proof, the verifier knows nothing but the circuit, which is compiled and used to generate the verification key. This is obviously quite important: private inputs remain private.

As for the public inputs, you may have noticed they haven't been specified. This behaviour varies with each particular backend, but barretenberg tipically attaches them to the proof. You can see them by parsing and splitting it. For example for if your public inputs are 32 bytes:

```bash
head -c 32 ./target/proof | od -An -v -t x1 | tr -d $' \n'
```

:::

Congratulations, you have now created and verified a proof for your very first Noir program!

In the [next section](./getting_started/project_breakdown.md), we will go into more detail on each step performed.

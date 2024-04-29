---
title: Writing your first smart contract
---

In this guide, we will create our first Aztec.nr smart contract. We will build a simple private counter. This contract will get you started with the basic setup and syntax of Aztec.nr, but doesn't showcase the awesome stuff Aztec is capable of.

If you already have some experience with Noir and want to build a cooler contract that utilizes both private and public state, you might want to check out the [token contract tutorial instead](../tutorials/writing_token_contract.md).

## Prerequisites

- You have followed the [quickstart](./quickstart.md)
- Running Aztec Sandbox

## Set up a project

Create a new directory called `aztec-private-counter`

```bash
mkdir aztec-private-counter
```

then create a `contracts` folder inside where our Aztec.nr contract will live:

```bash
cd aztec-private-counter
mkdir contracts
```

Inside contracts create the following file structure:

```tree
.
|-aztec-private-counter
| |-contracts
| | |--counter
| | |  |--src
| | |  |  |--main.nr
| | |  |--Nargo.toml
```

The file `main.nr` will soon turn into our smart contract!

Add the following content to `Nargo.toml`:

```toml
[package]
name = "counter"
type = "contract"
authors = [""]
compiler_version = ">=0.18.0"

[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/aztec" }
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/value-note"}
easy_private_state = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/easy-private-state"}
```

## Define the functions

Go to `main.nr` and start with this contract initialization:

```rust
contract Counter {
}
```

This defines a contract called `Counter`.

## Imports

We need to define some imports.

Write this within your contract at the top

#include_code imports /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

`context::{PrivateContext, Context}`

Context gives us access to the environment information such as `msg.sender`. We are also importing `PrivateContext` to access necessary information for our private functions. We’ll be using it in the next step.

`map::Map`

Map is a private state variable that functions like a dictionary, relating Fields to other state variables.

`value_note`

Notes are fundamental to how Aztec manages privacy. A note is a privacy-preserving representation of an amount of tokens associated with an address, while encrypting the amount and owner. In this contract, we are using the `value_note` library. This is a type of note interface for storing a single Field, eg a balance - or, in our case, a counter.

We are also using `balance_utils` from this import, a useful library that allows us to utilize value notes as if they are simple balances.

`EasyPrivateUint`

This allows us to store our counter in a way that acts as an integer, abstracting the note logic.

## Declare storage

Add this below the imports. It declares the storage variables for our contract. We are going to store a mapping of values for each `AztecAddress`.

#include_code storage_struct /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

## Keep the counter private

Now we’ve got a mechanism for storing our private state, we can start using it to ensure the privacy of balances.

Let’s create a constructor method to run on deployment that assigns an initial supply of tokens to a specified owner. This function is called `initialize`, but behaves like a constructor. It is the `#[aztec(initializer)]` decorator that specifies that this function behaves like a constructor. Write this:

#include_code constructor /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

This function accesses the counts from storage. Then it assigns the passed initial counter to the `owner`'s counter privately using `at().add()`.

We have annotated this and other functions with `#[aztec(private)]` which are ABI macros so the compiler understands it will handle private inputs. Learn more about functions and annotations [here](../contracts/writing_contracts/functions/main.md).

## Incrementing our counter

Now let’s implement the `increment` function we defined in the first step.

#include_code increment /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

The `increment` function works very similarly to the `constructor`, but instead directly adds 1 to the counter rather than passing in an initial count parameter.

## Prevent double spending

Because our counters are private, the network can't directly verify if a note was spent or not, which could lead to double-spending. To solve this, we use a nullifier - a unique identifier generated from each spent note and its owner. Although this isn't really an issue in this simple smart contract, Aztec injects a special function called `compute_note_hash_and_nullifier` to determine these values for any given note produced by this contract.

## Getting a counter

The last thing we need to implement is the function in order to retrieve a counter. In the `getCounter` we defined in the first step, write this:

#include_code get_counter /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

This function is `unconstrained` which allows us to fetch data from storage without a transaction. We retrieve a reference to the `owner`'s `counter` from the `counters` Map. The `get_balance` function then operates on the owner's counter. This yields a private counter that only the private key owner can decrypt.

## Compile

Now we've written a simple Aztec.nr smart contract, we can compile it with `aztec-nargo`.

### Compile the smart contract

In `./contracts/counter/` directory, run this:

```bash
aztec-nargo compile
```

This will compile the smart contract and create a `target` folder with a `.json` artifact inside.

After compiling, you can generate a typescript class. In the same directory, run this:

```bash
aztec-builder target -o src/artifacts
```

You can now use the artifact and/or the TS class in your Aztec.js! If you skipped the Aztec.js getting-started guide, you can follow it [here](aztecjs-getting-started.md). This will teach you about deploying and calling contracts in Aztec.js.

## Install Noir LSP (recommended)

Install the [Noir Language Support extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir) to get syntax highlighting, syntax error detection and go-to definitions for your Aztec contracts.

Once the extension is installed, check your nargo binary by hovering over `Nargo` in the status bar on the bottom right of the application window. Click to choose the path to `aztec-nargo` (or regular `nargo`, if you have that installed).

You can print the path of your `aztec-nargo` executable by running:

```bash
which aztec-nargo
```

To specify a custom nargo executable, go to the VSCode settings and search for "noir", or click extension settings on the `noir-lang` LSP plugin.
Update the `Noir: Nargo Path` field to point to your desired `aztec-nargo` executable.

## What's next?

The next recommmended steps are follow the tutorials in order. They will teach you more about contracts, Aztec.js, and how Aztec works in general.

To follow the series of tutorials, start with the private voting contract [here](../tutorials/writing_private_voting_contract.md).

Alternatively, you can read about the high level architecture on the [Core Components page](../../learn/about_aztec/technical_overview.md). You can also explore Aztec's [hybrid state model](../../learn/concepts/hybrid_state/main.md) and [the lifecycle of a transaction](../../learn/concepts/transactions.md).


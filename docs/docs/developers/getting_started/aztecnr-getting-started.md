---
title: Getting Started with Aztec.nr
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
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/value-note"}
easy_private_state = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/easy-private-state"}
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

#include_code imports /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

`context::{PrivateContext, Context}`

Context gives us access to the environment information such as `msg.sender`. We are also importing `PrivateContext` to access necessary information for our private functions. We’ll be using it in the next step.

`map::Map`

Map is a private state variable that functions like a dictionary, relating Fields to other state variables. You can learn more about it [here](../contracts/syntax/main.md).

`value_note`

Notes are fundamental to how Aztec manages privacy. A note is a privacy-preserving representation of an amount of tokens associated with an address, while encrypting the amount and owner. In this contract, we are using the `value_note` library. This is a type of note interface for storing a single Field, eg a balance - or, in our case, a counter.

We are also using `balance_utils` from this import, a useful library that allows us to utilize value notes as if they are simple balances.

`EasyPrivateUint`

This allows us to store our counter in a way that acts as an integer, abstracting the note logic.

## Implement a Storage struct

In this step, we will initiate a `Storage` struct to store balances in a private way. The vast majority Aztec.nr smart contracts will need this.

#include_code storage_struct /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

We are only storing one variable - `counts` as a `Map` of `EasyPrivateUint`. This means our `count` will act as a private integer, and we can map it to an address.

#include_code storage_init /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

This `init` method is creating and initializing a `Storage` instance. This instance includes a `Map` named `counters`. Each entry in this `Map` represents an account's counter.

## Keep the counter private

Now we’ve got a mechanism for storing our private state, we can start using it to ensure the privacy of balances.

Let’s create a `constructor` method to run on deployment that assigns an initial supply of tokens to a specified owner. In the constructor we created in the first step, write this:

#include_code constructor /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

This function accesses the counts from storage. Then it assigns the passed initial counter to the `owner`'s counter privately using `at().add()`.

We have annotated this and other functions with `#[aztec(private)]` which are ABI macros so the compiler understands it will handle private inputs. Learn more about functions and annotations [here](../contracts/syntax/functions.md).

## Incrementing our counter

Now let’s implement the `increment` function we defined in the first step.

#include_code increment /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

The `increment` function works very similarly to the `constructor`, but instead directly adds 1 to the counter rather than passing in an initial count parameter.

## Prevent double spending

Because our counters are private, the network can't directly verify if a note was spent or not, which could lead to double-spending. To solve this, we use a nullifier - a unique identifier generated from each spent note and its owner. Although this isn't really an issue in this simple smart contract, Aztec requires a contract that has any private functions to include this function.

Add a new function into your contract as shown below:

#include_code nullifier /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

Here, we're computing both the note hash and the nullifier. The nullifier computation uses Aztec’s `compute_note_hash_and_nullifier` function, which takes details about the note's attributes eg contract address, nonce, storage slot, and preimage.

## Getting a counter

The last thing we need to implement is the function in order to retrieve a counter. In the `getCounter` we defined in the first step, write this:

#include_code get_counter /yarn-project/noir-contracts/contracts/counter_contract/src/main.nr rust

This function is `unconstrained` which allows us to fetch data from storage without a transaction. We retrieve a reference to the `owner`'s `counter` from the `counters` Map. The `get_balance` function then operates on the owner's counter. This yields a private counter that only the private key owner can decrypt.

## Test with the CLI

Now we've written a simple Aztec.nr smart contract, it's time to ensure everything works by testing with the CLI.

### Compile the smart contract

In `./contracts/counter/` directory, run this:

```bash
aztec-nargo compile
```

This will compile the smart contract and create a `target` folder with a `.json` artifact inside.

After compiling, you can generate a typescript class. In the same directory, run this:

```bash
aztec-cli codegen target -o src/artifacts --ts
```

### Deploy

You can use the previously generated artifact to deploy the smart contract. Our constructor takes two arguments - `initial_counter` and `owner` so let's make sure to pass those in.

`initial_counter` can be any uint. In this guide we'll pick 100, but you can pick anything.

For the `owner` you can get the account addresses in your sandbox by running:

```bash
aztec-cli get-accounts
```

This will return something like this:

```bash
➜ counter aztec-cli get-accounts
Accounts found:

Address: 0x25048e8c1b7dea68053d597ac2d920637c99523651edfb123d0632da785970d0
Public Key: 0x27c20118733174347b8082f578a7d8fb84b3ad38be293715eee8119ee5cd8a6d0d6b7d8124b37359663e75bcd2756f544a93b821a06f8e33fba68cc8029794d9
Partial Address: 0x077fed6015ea2e4aabfd566b16d9528e79dc0f1d8573716a3f4de1f02962e8c9

Address: 0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483
Public Key: 0x08145e8e8d46f51cda8d4c9cad81920236366abeafb8d387002bad879a3e87a81570b04ac829e4c007141d856d5a36d3b9c464e0f3c1c99cdbadaa6bb93f3257
Partial Address: 0x092908a7140034c7add7f2fac103abc41bedd5474cf09b1c9c16e5331282de77

Address: 0x0402655a1134f3f248e9f2032c27b26d2c3ab57eaab3189541895c13f3622eba
Public Key: 0x13e6151ea8e7386a5e7c4c5221047bf73d0b1b7a2ad14d22b7f73e57c1fa00c614bc6da69da1b581b09ee6cdc195e5d58ae4dce01b63bbb744e58f03855a94dd
Partial Address: 0x211edeb823ef3e042e91f338d0d83d0c90606dba16f678c701d8bb64e64e2be5
```

Use one of these `address`es as the `owner`. You can either copy it or export.

To deploy the counter contract, [ensure the sandbox is running](../cli/sandbox-reference.md) and run this in the root of your Noir project:

```bash
aztec-cli deploy contracts/counter/src/artifacts/Counter.json --args 100 0x2a0f32c34c5b948a7f9766f0c1aad70a86c0ee649f56208e936be4324d49b0b9
```

You can also test the functions by applying what you learned in the [quickstart](./quickstart.md).

Congratulations, you have now written, compiled, and deployed your first Aztec.nr smart contract!

## Install Noir LSP (recommended)

Install the [Noir Language Support extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir) to get syntax highlighting, syntax error detection and go-to definitions for your Aztec contracts.

Once the extension is installed, go to your VSCode settings, search for "noir" and update the `Noir: Nargo Path` field to point to your `aztec-nargo` executable.

You can print the path of your `aztec-nargo` executable by running:

```bash
which aztec-nargo
```

## What's next?

Now you can explore.

**Interested in learning more about how Aztec works under the hood?**

Understand the high level architecture [here](../../learn/about_aztec/technical_overview.md).

**Want to write more advanced smart contracts?**

Follow the token contract tutorial [here](../tutorials/writing_token_contract.md).

**Ready to dive into Aztec and Ethereum cross-chain communication?**

Read the [Portals page](../../learn/concepts/communication/cross_chain_calls.md) and learn how to practically implement portals in the [token bridge tutorial](../tutorials/token_portal/main.md).

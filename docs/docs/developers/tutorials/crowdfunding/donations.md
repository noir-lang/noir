---
title: Build a donations contract
tags: [developers, tutorial, example]
---

# Build a donations contract

In this tutorial we'll create two contracts related to crowdfunding:

- A crowdfunding contract with two core components
  - Fully private donations
  - Verifiable withdrawals to the operator
- A reward contract for anyone else to anonymously reward donors

Along the way you will:

- Install Aztec developer tools
- Setup a new Noir contract project
- Add base Aztec dependencies
- Call between private and public contexts
- Wrap an address with its interface (token)
- Create custom private value notes

## Setup

### Install tools

Please ensure that the you already have [Installed the Sandbox](https://docs.aztec.network/developers/getting_started/quickstart#install-the-sandbox).

And if using VSCode, see [here](https://docs.aztec.network/developers/contracts/main#install-noir-lsp-recommended) to install Noir LSP, where you'll benefit from syntax highlighting, profiling, and more.

### Create an Aztec project

Use `aztec-nargo` in a terminal to create a new Aztec contract project named "crowdfunding":

```sh
aztec-nargo new --contract crowdfunding
```

Inside the new `crowdfunding` directory you will have a base to implement the Aztec smart contract.

Use `aztec-nargo --help` to see other commands.

## Private donations

1. An "Operator" begins a Crowdfunding campaign (contract), specifying:

- an existing token address
- their account address
- a deadline timestamp

2. Any address can donate (in private context)

- private transfer token from sender to contract
- transaction receipts allow private claims via another contract

3. Only the operator can withdraw from the fund

### 1. Create a campaign

#### Initialize

Open the project in your preferred editor. If using VSCode and the LSP, you'll be able to select the `aztec-nargo` binary to use (instead of `nargo`).

In `main.nr`, rename the contract from `Main`, to `Crowdfunding`.

#include_code empty-contract /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

Replace the example functions with an initializer that takes the required campaign info as parameters. Notice use of `#[aztec(...)]` macros inform the compiler that the function is a public initializer.

```rust
#include_code init-header /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr raw
  //...
}
```

More about initializers [here](../../contracts/writing_contracts/functions/initializers.md).

#### Dependencies

When you compile the contracts by running `aztec-nargo compile` in your project directory, you'll notice it cannot resolve `AztecAddress`. (Or hovering over in VSCode)

```rust
#include_code init-header-error /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr raw
  //...
}
```

Add the required dependency by going to your project's `Nargo.toml` file, and adding `aztec` from the `aztec-nr` framework. It resides in the `aztec-packages` mono-repo:

```rust
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/aztec" }
```

A word about versions:
- Choose the aztec packages version to match your aztec tools as seen here - `aztec-cli -V`
- Check that your `compiler_version` in Nargo.toml is satisified by your aztec compiler - `aztec-nargo -V`

More about versions [here](https://docs.aztec.network/developers/versions-updating).

Inside the Crowdfunding contract definition, use the dependency that defines the address type `AztecAddress` (same syntax as Rust)

```rust
use dep::aztec::protocol_types::address::AztecAddress;
```

The `aztec::protocol_types` can be browsed [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/noir-protocol-circuits/crates/types/src). And like rust dependencies, the relative path inside the dependency corresponds to `address::AztecAddress`.


#### Storage

To retain the initializer parameters in the contract's Storage, we'll need to declare them in a preceding `Storage` struct:

#include_code storage /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

The `ValueNote` type is in the top-level of the Aztec.nr framework, namely [noir-projects/aztec-nr](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/value-note/src/value_note.nr). Like before, you'll need to add the crate to Nargo.toml

(See [here](https://docs.aztec.network/developers/contracts/resources/dependencies) for common dependencies).

---

Back in main.nr, reference `use` of the type

```rust
use dep::value_note::value_note::ValueNote;
```

Now complete the initializer by setting the storage variables with the parameters:

#include_code init /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

You can compile the code so far with `aztec-nargo compile`.

### 2. Taking private donations

#### Checking campaign duration against the timestamp

To check that the donation occurs before the campaign deadline, we must access the public `timestamp`. It is one of several [Public Global Variables](https://docs.aztec.network/developers/contracts/references/globals#public-global-variables).

Declare an Aztec function that is public and internal

```rust
#include_code deadline-header /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr raw
    //...
}
```

Read the deadline from storage and assert that the `timestamp` from this context is before the deadline

#include_code deadline /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

---

Since donations are to be private, the donate function will have the user's private context which has these [Private Global Variables](https://docs.aztec.network/developers/contracts/references/globals#private-global-variables). So from the private context there is a little extra to call the (public internal) `_check_deadline` function.

```rust
#include_code call-check-deadline /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr raw
  //...
}
```

Namely calling `enqueue` and passing the (mutable) context.

Now conclude adding all dependencies to the `Crowdfunding` contract:

#include_code all-deps /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

Like before, you can find these and other `aztec::protocol_types` [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/noir-protocol-circuits/crates/types/src).


#### Interfacing with another contract

The token being used for donations is stored simply as an `AztecAddress` (named `donation_token`). so to easily use it as a token, we let the compiler know that we want the address to have a Token interface. Here we will use a maintained example Token contract.

Add this `Token` contract to Nargo.toml:

```
token = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/noir-contracts/contracts/token_contract" }
```

With the dependency already `use`d at the start of the contract, the token contract can be called to make the transfer from msg sender to this contract. 

:::note
The user must have authorised this action (concept [here](../../../learn/concepts/accounts/main#authorizing-actions)), example use of `createAuthWit` in 'full donor flow' test [here](../../../../../yarn-project/end-to-end/src/e2e_crowdfunding_and_claim.test.ts).
:::

#### Creating and storing a private receipt note

The last thing to do is create a new value note and add it to the `donation_receipts`. So the full donation function is now

#include_code donate /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

### 3. Operator withdrawals

The remaining function to implement, `withdraw`, is reasonably straight-forward:
1. make sure the address calling is the operator address
2. transfer tokens from the contract to the operator
3. reveal that an amount has been withdrawn to the operator

The last point is achieved by emitting an unencrypted event log, more [here](https://docs.aztec.network/developers/contracts/writing_contracts/events/emit_event#unencrypted-events).

Copy the last function into your Crowdfunding contract:

#include_code operator-withdrawals /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

You should be able to compile successfully with `aztec-nargo compile`.

## Conclusion

For comparison, the full Crowdfunding contract can be found [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/noir-contracts/contracts/crowdfunding_contract).

### Next steps?

If a new token wishes to honour donors with free tokens based on donation amounts, this is possible via the donation_receipts (a `PrivateSet`).
See [claim_contract](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/noir-contracts/contracts/claim_contract).
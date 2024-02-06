---
title: Public State
---

On this page we will look at how to manage public state in Aztec contracts. We will look at how to declare public state, how to read and write to it, and how to use it in your contracts.

For a higher level overview of the state model in Aztec, see the [state model](../../../../learn/concepts/hybrid_state/main.md) page, or jump back to the previous page on [Storage](./main.md).

## Overview

The `PublicState` struct is generic over the variable type `T`. The type _must_ implement Serialize and Deserialize traits, as specified here:

#include_code serialize /yarn-project/noir-protocol-circuits/src/crates/types/src/traits.nr rust
#include_code deserialize /yarn-project/noir-protocol-circuits/src/crates/types/src/traits.nr rust

The struct contains a `storage_slot` which, similar to Ethereum, is used to figure out _where_ in storage the variable is located. Notice that while we don't have the exact same [state model](../../../../learn/concepts/hybrid_state/main.md) as EVM chains it will look similar from the contract developers point of view.

You can find the details of `PublicState` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr).

:::info
An example using a larger struct can be found in the [lending example](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/contracts/lending_contract)'s use of an [`Asset`](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/yarn-project/noir-contracts/contracts/lending_contract/src/asset.nr).
:::

### `new`

When declaring the storage for `T` as a persistent public storage variable, we use the `PublicState::new()` constructor. As seen below, this takes the `storage_slot` and the [`Context`](../context.md), which in this case is used to share interface with other structures. You can view the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr).

#### Single value example

Say that we wish to add `admin` public state variable into our storage struct. In the struct we can define it as:

#include_code storage-leader-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do:

#include_code storage-leader-init /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

We have specified that we are storing a `Field` that should be placed in storage slot `1`. This is just a single value, and is similar to the following in solidity:

```solidity
address internal admin;
```

#### Mapping example

Say we want to have a group of `minters` that are able to mint assets in our contract, and we want them in public storage, because [access control in private is quite cumbersome](../../../../learn/concepts/communication/cross_chain_calls.md#a-note-on-l2-access-control). In the `Storage` struct we can add it as follows:

#include_code storage-minters-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do it as follows:

#include_code storage-minters-init /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

In this case, specifying that we are dealing with a map of Fields, and that it should be put at slot 2.

This would be similar to the following in solidity:

```solidity
mapping(address => bool) internal minters;
```

### `read`

On the `PublicState` structs we have a `read` method to read the value at the location in storage.

#### Reading from our `admin` example

For our `admin` example from earlier, this could be used as follows to check that the stored value matches the `msg_sender()`.

#include_code read_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

#### Reading from our `minters` example

As we saw in the Map earlier, a very similar operation can be done to perform a lookup in a map.

#include_code read_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `write`

We have a `write` method on the `PublicState` struct that takes the value to write as an input and saves this in storage. It uses the serialization method to serialize the value which inserts (possibly multiple) values into storage.

#### Writing to our `admin` example

#include_code write_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

#### Writing to our `minters` example

#include_code write_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

---

## Stable Public State

`StablePublicState` is a special type of `PublicState` that can be read from both public and private!

Since private execution is based on historical data, the user can pick ANY of its prior values to read from. This is why it `MUST` not be updated after the contract is deployed. The variable should be initialized at the constructor and then never changed.

This makes the stable public variables useful for stuff that you would usually have in `immutable` values in solidity. For example this can be the name of a token or its number of decimals.

Just like the `PublicState` it is generic over the variable type `T`. The type `MUST` implement Serialize and Deserialize traits.

You can find the details of `StablePublicState` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/state_vars/stable_public_state.nr).

### `new`

Is done exactly like the `PublicState` struct, but with the `StablePublicState` struct.

#include_code storage-stable-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#include_code storage-stable-init /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `initialize`

#include_code initialize_decimals /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

:::warning Should only be called as part of the deployment.
If this is called outside the deployment transaction multiple values could be used down the line, potentially breaking the contract.

Currently this is not constrained as we are in the middle of changing deployments.
:::

### `read_public`

Reading the value is like `PublicState`, simply with `read_public` instead of `read`.
#include_code read_decimals_public /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `read_private`

Reading the value is like `PublicState`, simply with `read_private` instead of `read`. This part can only be executed in private.

#include_code read_decimals_private /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

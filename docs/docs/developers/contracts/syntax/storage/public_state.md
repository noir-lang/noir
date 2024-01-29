---
title: Public State
---

On this page we will look at how to manage public state in Aztec contracts. We will look at how to declare public state, how to read and write to it, and how to use it in your contracts.

For a higher level overview of the state model in Aztec, see the [state model](../../../../learn/concepts/hybrid_state/main.md) page, or jump back to the previous page on [Storage](./main.md).

## Overview

The `PublicState` struct is generic over the variable type `T` and its serialized size `T_SERIALIZED_LEN`.

:::info
Currently, the length of the types must be specified when declaring the storage struct but the intention is that this will be inferred in the future.
:::

The struct contains a `storage_slot` which, similar to Ethereum, is used to figure out _where_ in storage the variable is located. Notice that while we don't have the exact same [state model](../../../../learn/concepts/hybrid_state/main.md) as EVM chains it will look similar from the contract developers point of view.

Beyond the struct, the `PublicState` also contains `serialization_methods`, which is a struct with methods that instruct the `PublicState` how to serialize and deserialize the variable. You can find the details of `PublicState` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr).

The Aztec.nr library provides serialization methods for various common types.

:::info
An example using a larger struct can be found in the [lending example](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/contracts/lending_contract)'s use of an [`Asset`](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/yarn-project/noir-contracts/contracts/lending_contract/src/asset.nr).
:::

### `new`

When declaring the storage for `T` as a persistent public storage variable, we use the `PublicState::new()` constructor. As seen below, this takes the `storage_slot` and the `serialization_methods` as arguments along with the [`Context`](../context.mdx), which in this case is used to share interface with other structures. You can view the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr).

#### Single value example

Say that we wish to add `admin` public state variable into our storage struct. In the struct we can define it as:

#include_code storage_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do:

#include_code storage_admin_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

We have specified that we are storing a `Field` that should be placed in storage slot `1`. This is just a single value, and is similar to the following in solidity:

```solidity
address internal admin;
```

:::info
We know its verbose, and are working on making it less so.
:::

#### Mapping example

Say we want to have a group of `minters` that are able to mint assets in our contract, and we want them in public storage, because [access control in private is quite cumbersome](../../../../learn/concepts/communication/cross_chain_calls.md#a-note-on-l2-access-control). In the `Storage` struct we can add it as follows:

#include_code storage_minters /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do it as follows:

#include_code storage_minters_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

In this case, specifying that we are dealing with a map of Fields, and that it should be put at slot 2.

This would be similar to the following in solidity:

```solidity
mapping(address => bool) internal minters;
```

### `read`

On the `PublicState` structs we have a `read` method to read the value at the location in storage and using the specified deserialization method to deserialize it.

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

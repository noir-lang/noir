---
title: Public State
---

On this page we will look at how to manage public state in Aztec contracts. We will look at how to declare public state, how to read and write to it, and how to use it in your contracts.

For a higher level overview of the state model in Aztec, see the [state model](../../../../learn/concepts/hybrid_state/main.md) page, or jump back to the previous page on [Storage](./main.md).

## Overview

The `PublicMutable` (formerly known as `PublicState`) struct is generic over the variable type `T`. The type _must_ implement Serialize and Deserialize traits, as specified here:

#include_code serialize /noir-projects/noir-protocol-circuits/src/crates/types/src/traits.nr rust
#include_code deserialize /noir-projects/noir-protocol-circuits/src/crates/types/src/traits.nr rust

The struct contains a `storage_slot` which, similar to Ethereum, is used to figure out _where_ in storage the variable is located. Notice that while we don't have the exact same [state model](../../../../learn/concepts/hybrid_state/main.md) as EVM chains it will look similar from the contract developers point of view.

You can find the details of `PublicMutable` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/aztec/src/state_vars/public_mutable.nr).

:::info
An example using a larger struct can be found in the [lending example](https://github.com/AztecProtocol/aztec-packages/tree/master/noir-projects/noir-contracts/contracts/lending_contract)'s use of an [`Asset`](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/noir-projects/noir-contracts/contracts/lending_contract/src/asset.nr).
:::

### `new`

When declaring the storage for `T` as a persistent public storage variable, we use the `PublicMutable::new()` constructor. As seen below, this takes the `storage_slot` and the `serialization_methods` as arguments along with the [`Context`](../../writing_contracts/functions/context.md), which in this case is used to share interface with other structures. You can view the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/aztec/src/state_vars/public_mutable.nr).

#### Single value example

Say that we wish to add `admin` public state variable into our storage struct. In the struct we can define it as:

#include_code storage-leader-declaration /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do:

#include_code storage-leader-init /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

We have specified that we are storing a `Field` that should be placed in storage slot `1`. This is just a single value, and is similar to the following in solidity:

```solidity
address internal admin;
```

#### Mapping example

Say we want to have a group of `minters` that are able to mint assets in our contract, and we want them in public storage, because [access control in private is quite cumbersome](../../../../learn/concepts/communication/cross_chain_calls.md#a-note-on-l2-access-control). In the `Storage` struct we can add it as follows:

#include_code storage-minters-declaration /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do it as follows:

#include_code storage-minters-init /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

In this case, specifying that we are dealing with a map of Fields, and that it should be put at slot 2.

This would be similar to the following in solidity:

```solidity
mapping(address => bool) internal minters;
```

### `read`

On the `PublicMutable` structs we have a `read` method to read the value at the location in storage.

#### Reading from our `admin` example

For our `admin` example from earlier, this could be used as follows to check that the stored value matches the `msg_sender()`.

#include_code read_admin /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

#### Reading from our `minters` example

As we saw in the Map earlier, a very similar operation can be done to perform a lookup in a map.

#include_code read_minter /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

### `write`

We have a `write` method on the `PublicMutable` struct that takes the value to write as an input and saves this in storage. It uses the serialization method to serialize the value which inserts (possibly multiple) values into storage.

#### Writing to our `admin` example

#include_code write_admin /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

#### Writing to our `minters` example

#include_code write_minter /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

---

## Public Immutable

`PublicImmutable` is a type that can be written once during a contract deployment and read later on from public only.

Just like the `PublicMutable` it is generic over the variable type `T`. The type `MUST` implement Serialize and Deserialize traits.

You can find the details of `PublicImmutable` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/aztec/src/state_vars/public_immutable.nr).

### `new`

Is done exactly like the `PublicMutable` struct, but with the `PublicImmutable` struct.

#include_code storage-public-immutable-declaration /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#include_code storage-public-immutable /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `initialize`

#include_code initialize_public_immutable /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `read`

Reading the value is just like `PublicMutable`.
#include_code read_public_immutable /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

## Shared Immutable

`SharedImmutable` (formerly known as `StablePublicState`) is a type which is very similar to `PublicImmutable` but with an addition of a private getter (can be read from private).

Since private execution is based on historical data, the user can pick ANY of its prior values to read from. This is why it `MUST` not be updated after the contract is deployed. The variable should be initialized at the constructor and then never changed.

This makes the immutable public variables useful for stuff that you would usually have in `immutable` values in solidity. For example this can be the name of a token or its number of decimals.

Just like the `PublicMutable` it is generic over the variable type `T`. The type `MUST` implement Serialize and Deserialize traits.

You can find the details of `SharedImmutable` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/aztec/src/state_vars/shared_immutable.nr).

:::info
The word `Shared` in Aztec protocol means read/write from public, read only from private.
:::

### `new`

Is done exactly like the `PublicMutable` struct, but with the `SharedImmutable` struct.

#include_code storage-shared-immutable-declaration /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#include_code storage-shared-immutable /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `initialize`

#include_code initialize_decimals /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

:::warning Should only be called as part of the deployment.
If this is called outside the deployment transaction multiple values could be used down the line, potentially breaking the contract.

Currently this is not constrained as we are in the middle of changing deployments.
:::

### `read_public`

Reading the value is like `PublicMutable`, simply with `read_public` instead of `read`.
#include_code read_decimals_public /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

### `read_private`

Reading the value is like `PublicMutable`, simply with `read_private` instead of `read`. This part can only be executed in private.

#include_code read_decimals_private /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

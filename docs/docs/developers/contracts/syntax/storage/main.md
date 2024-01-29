---
title: Storage
---

Smart contracts rely on storage, acting as the persistent memory on the blockchain. In Aztec, because of its hybrid, privacy-first architecture, the management of this storage is more complex than other blockchains like Ethereum.

You control this storage in Aztec using the `Storage` struct. This struct serves as the housing unit for all your smart contract's state variables - the data it needs to keep track of and maintain.

These state variables come in two forms: public and private. Public variables are visible to anyone, and private variables remain hidden within the contract.

Aztec.nr has a few abstractions to help define the type of data your contract holds. These include Singletons, ImmutableSingletons, Set, and Map.

On this and the following pages in this section, you’ll learn:

- How to manage a smart contract's storage structure
- The distinctions and applications of public and private state variables
- How to use Singleton, ImmutableSingleton, Set, and Map
- An overview of 'notes' and the UTXO model
- Practical implications of Storage in real smart contracts
  In an Aztec.nr contract, storage is to be defined as a single struct, that contains both public and private state variables.

## Public and private state variables

Public state variables can be read by anyone, while private state variables can only be read by their owner (or people whom the owner has shared the decrypted data or note viewing key with).

Public state follows the Ethereum style account model, where each contract has its own key-value datastore. Private state follows a UTXO model, where note contents (pre-images) are only known by the sender and those able to decrypt them - see ([state model](../../../../learn/concepts/hybrid_state/main.md) and [private/public execution](../../../../learn/concepts/communication/public_private_calls/main.md)) for more background.

## Storage struct

:::info
The struct **must** be called `Storage` for the Aztec.nr library to properly handle it (this will be relaxed in the future).
:::

```rust
struct Storage {
  // public state variables
  // private state variables
}
```

:::danger
If your contract uses storage (has Storage struct defined), you **MUST** include a `compute_note_hash_and_nullifier` function to allow PXE to process encrypted events. See [encrypted events](../events.md#processing-encrypted-events) for more.

If you don't yet have any private state variables defined you can use this placeholder function:

#include_code compute_note_hash_and_nullifier_placeholder /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust
:::

Since Aztec.nr is written in Noir, which is state-less, we need to specify how the storage struct should be initialized to read and write data correctly. This is done by specifying an `init` function that is run in functions that rely on reading or altering the state variables. This `init` function must declare the Storage struct with an instantiation defining how variables are accessed and manipulated. The function MUST be called `init` for the Aztec.nr library to properly handle it (this will be relaxed in the future).

```rust
impl Storage {
  fn init(context: Context) -> Self {
    Storage {
      // (public state variables)::new()
      // (private state variables)::new()
    }
  }
}
```

If you have defined a `Storage` struct following this naming scheme, then it will be made available to you through the reserved `storage` keyword within your contract functions.

:::warning Using slot `0` is not supported!
No storage values should be initialized at slot `0` - storage slots begin at `1`. This is a known issue that will be fixed in the future.
:::

## Map

A `map` is a state variable that "maps" a key to a value. It can be used with private or public storage variables.

:::info
In Aztec.nr, keys are always `Field`s, or types that can be serialized as Fields, and values can be any type - even other maps. `Field`s are finite field elements, but you can think of them as integers.
:::

It includes a [`Context`](../context.mdx) to specify the private or public domain, a `storage_slot` to specify where in storage the map is stored, and a `start_var_constructor` which tells the map how it should operate on the underlying type. This includes how to serialize and deserialize the type, as well as how commitments and nullifiers are computed for the type if it's private.

You can view the implementation in the Aztec.nr library [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/state_vars/map.nr).

### `new`

When declaring the storage for a map, we use the `Map::new()` constructor. As seen below, this takes the `storage_slot` and the `start_var_constructor` along with the [`Context`](../context.mdx).

We will see examples of map constructors for public and private variables in later sections.

#### As private storage

When declaring a mapping in private storage, we have to specify which type of Note to use. In the example below, we are specifying that we want to use the `Singleton` note type.

In the Storage struct:

#include_code storage-map-singleton-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

In the `Storage::init` function:

#include_code state_vars-MapSingleton /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### Public Example

When declaring a public mapping in Storage, we have to specify that the type is public by declaring it as `PublicState` instead of specifying a note type like with private storage above.

In the Storage struct:

#include_code storage_minters /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

In the `Storage::init` function:

#include_code storage_minters_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `at`

When dealing with a Map, we can access the value at a given key using the `::at` method. This takes the key as an argument and returns the value at that key.

This function behaves similarly for both private and public maps. An example could be if we have a map with `minters`, which is mapping addresses to a flag for whether they are allowed to mint tokens or not.

#include_code read_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

Above, we are specifying that we want to get the storage in the Map `at` the `msg_sender()`, read the value stored and check that `msg_sender()` is indeed a minter. Doing a similar operation in Solidity code would look like:

```solidity
require(minters[msg.sender], "caller is not minter");
```

## Further Reading

- Managing [Public State](./public_state.md)
- Jump to the page on [Private State](./private_state.md)

## Concepts mentioned

- [Hybrid State Model](../../../../learn/concepts/hybrid_state/main.md)
- [Public-private execution](../../../../learn/concepts/communication/public_private_calls/main.md)
- [Function Contexts](../context.mdx)

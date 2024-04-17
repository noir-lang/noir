---
title: How to implement a Slow Updates Tree
---

To learn more about the Slow Updates Tree, go [here](./main.md)

On this page you will learn how to implement a slow updates tree into your contract, and an example of a token blacklisting contract that uses the slow updates tree.

# How to implement a slow updates tree

1. Store the SlowTree address in private storage as a FieldNote

#include_code constructor noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

2. Store the SlowTree address in public storage and initialize an instance of SlowMap using this address

#include_code write_slow_update_public noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

3. Now you can read and update from private functions:

#include_code get_and_update_private noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

4. Or from public functions:

#include_code get_public noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

View the [reference](../../../references/slow_updates_tree.md) for more information.

## Exploring an example integration through a **`TokenBlacklist`** Smart Contract

The `TokenBlacklist` contract is a token contract that does not allow blacklisted accounts to perform mints or transfers. In this section we will go through how this is achieved through the slow updates tree.

You can find the full code for the TokenBlacklist smart contract [here](https://github.com/AztecProtocol/aztec-packages/tree/master/noir-projects/noir-contracts/contracts/token_blacklist_contract).

### Importing SlowMap

The contract first imports the **`SlowMap`** interface:

#include_code interface noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

This interface allows the contract to interact with its attached SlowTree. It abstracts these functions so they do not have to be implemented in the TokenBlacklist contract.

### Constructor and initialization

The contract's constructor takes the address of the slow updates contract:

#include_code constructor noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

This initialization sets up the connection between the **`TokenBlacklist`** contract and a previously deployed SlowTree, allowing it to use the interface to directly interact with the SlowTree.

### Private transfer function utilizing the slow updates tree

In the private transfer function, the contract uses the interface to check if a user is blacklisted:

#include_code transfer_private noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

Here, the contract reads the roles of the sender and recipient from the SlowTree using the **`read_at`** function in the interface. It checks if either party is blacklisted, and if so, the transaction does not go ahead.

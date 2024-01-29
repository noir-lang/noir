---
title: Oracles
---

On this page you will learn:

1. [A list of inbuilt oracles](#inbuilt-oracles)
3. [How to use the debug_log oracle](#how-to-use-the-debug-oracle)
3. [How to use the auth_witness oracle](#how-to-use-the-auth_witness-oracle)
4. [How to use the pop_capsule oracle for arbitrary data](#how-to-use-the-popCapsule-oracle)
4. [Reference](#oracles-reference)

## Inbuilt oracles

- [`debug_log`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/debug_log.nr) - Provides a couple of debug functions that can be used to log information to the console. Read more about debugging [here](../../debugging/main.md).
- [`auth_witness`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/authwit/src/auth_witness.nr) - Provides a way to fetch the authentication witness for a given address. This is useful when building account contracts to support approve-like functionality.
- [`get_l1_to_l2_message`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/get_l1_to_l2_message.nr) - Useful for application that receive messages from L1 to be consumed on L2, such as token bridges or other cross-chain applications.
- [`notes`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/notes.nr) - Provides a lot of functions related to notes, such as fetches notes from storage etc, used behind the scenes for value notes and other pre-build note implementations.
- [`logs`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/logs.nr) - Provides the to log encrypted and unencrypted data.

Find a full list [on GitHub](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec-nr/aztec/src/oracle).

:::note
Please note that it is **not** possible to write a custom oracle for your dapp. Oracles are implemented in the PXE, so all users of your dapp would have to use a PXE service with your custom oracle included. If you want to inject some arbitrary data that does not have a dedicated oracle, you can use [popCapsule](#how-to-use-the-pop_capsule-oracle).
:::

## How to use the popCapsule oracle

`popCapsule` is used for passing artbitrary data. We have not yet included this in Aztec.nr, so it is a bit more complex than the other oracles. You can follow this how-to:

### 1. Define the pop_capsule function

In a new file on the same level as your `main.nr`, implement an unconstrained function that calls the pop_capsule oracle:

#include_code pop_capsule yarn-project/noir-contracts/contracts/slow_tree_contract/src/capsule.nr rust

### 2. Import this into your smart contract

If it lies in the same directory as your smart contract, you can import it like this:

#include_code import_pop_capsule yarn-project/noir-contracts/contracts/slow_tree_contract/src/main.nr rust

### 3. Use it as any other oracle

Now it becomes a regular oracle you can call like this:

#include_code pop_capsule yarn-project/noir-contracts/contracts/slow_tree_contract/src/main.nr rust



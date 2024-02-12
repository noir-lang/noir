---
title: How to use the popCapsule oracle
---

On this page you will learn how to use the `popCapsule` oracle. To see what other oracles are available in Aztec.nr, go [here](./inbuilt_oracles.md).

## How to use the popCapsule oracle

`popCapsule` is used for passing artbitrary data. We have not yet included this in Aztec.nr, so it is a bit more complex than the other oracles. You can follow this how-to:

### 1. Define the pop_capsule function

In a new file on the same level as your `main.nr`, implement an unconstrained function that calls the pop_capsule oracle:

#include_code pop_capsule noir-projects/noir-contracts/contracts/slow_tree_contract/src/capsule.nr rust

### 2. Import this into your smart contract

If it lies in the same directory as your smart contract, you can import it like this:

#include_code import_pop_capsule noir-projects/noir-contracts/contracts/slow_tree_contract/src/main.nr rust

### 3. Use it as any other oracle

Now it becomes a regular oracle you can call like this:

#include_code pop_capsule noir-projects/noir-contracts/contracts/slow_tree_contract/src/main.nr rust

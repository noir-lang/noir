---
title: Calling Other Functions
sidebar_position: 2
---


<!-- TODO finish this guide. i think we accidentally deleted it because this page makes no sense -->

A contract is a collection of persistent [state variables](../../../aztec/concepts/storage/index.md), and [functions](../../../aztec/concepts/smart_contracts/functions/index.md) which may manipulate these variables. 

Functions and state variables within a contract's scope are said to belong to that contract. A contract can only access and modify its own state.

If a contract wishes to access or modify another contract's state, it must make a call to an external function of the other contract. For anything to happen on the Aztec network, an external function of a contract needs to be called.

### Contract

A contract may be declared and given a name using the `contract` keyword (see snippet below). By convention, contracts are named in `PascalCase`.

```rust title="contract keyword"
// highlight-next-line
contract MyContract {

    // Imports 

    // Storage 

    // Functions
}
```
:::info A note for vanilla Noir devs
There is no [`main()`](https://noir-lang.org/docs/getting_started/project_breakdown/#mainnr) function within a Noir `contract` scope. More than one function can be an entrypoint.
:::

To understand how to call a function from another contract, follow the  [crowdfunding tutorial](../../../tutorials/contract_tutorials/crowdfunding_contract.md).
---
title: Functions
---

Functions serve as the building blocks of smart contracts. Functions can be either **public**, ie they are publicly available for anyone to see and can directly interact with public state, or **private**, meaning they are executed completely client-side in the [PXE](../../../../learn/concepts/pxe/main.md). Read more about how private functions work [here](./inner_workings.md#private-functions).

For a more practical guide of using multiple types of functions, follow the [token tutorial](../../../tutorials/writing_token_contract.md).

Currently, any function is "mutable" in the sense that it might alter state. However, we also support support static calls, similarly to EVM. A static call is essentially a call that does not alter state (it keeps state static). 

## Constructors

Every smart contract has a private `constructor` function which is called when the contract is deployed. 

A special constructor function must be declared within a contract's scope.

A constructor doesn't have a name, because its purpose is clear: to initialize contract state.
In Aztec terminology, a constructor is always a 'private function' (i.e. it cannot be a public function).
A constructor behaves almost identically to any other function. It is just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

## Oracles

There are also special oracle functions, which can get data from outside of the smart contract. In the context of Aztec, oracles are often used to get user-provided inputs.

Explore this section to learn:

- [How function visibility works in Aztec](./visibility.md)
- [Public, private, and unconstrained functions](./public_private_unconstrained.md), and how to write them
- How to write a [constructor](./write_constructor.md)
- [Calling functions from within the same smart contract and from different contracts](./call_functions.md), including calling private functions from private functions, public from public, and even private from public
- [Oracles](../oracles/main.md) and how Aztec smart contracts might use them
- [How functions work under the hood](./inner_workings.md)
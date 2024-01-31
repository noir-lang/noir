---
title: Functions
---

Functions serve as the building blocks of smart contracts. Functions can be either **public**, ie they are publicly available for anyone to see and can directly interact with public state, or **private**, meaning they are executed completely client-side in the [PXE](../../../../learn/concepts/pxe/main.md). Read more about how private functions work [here](./inner_workings.md#private-functions).

For a more practical guide of using multiple types of functions, follow the [token tutorial](../../../tutorials/writing_token_contract.md).

Every smart contract has a private `constructor` function which is called when the contract is deployed. There are also special oracle functions, which can get data from outside of the smart contract. In the context of Aztec, oracles are often used to get user-provided inputs.

Currently, any function is "mutable" in the sense that it might alter state. In the future, we will support static calls, similarly to EVM. A static call is essentially a call that does not alter state (it keeps state static). 

Explore this section to learn:

- [How function visibility works in Aztec](./visibility.md)
- [Public, private, and unconstrained functions](./public_private_unconstrained.md), and how to write them
- How [constructors](./constructor.md) work and remain private
- [Calling functions from within the same smart contract and from different contracts](./calling_functions.md), including calling private functions from private functions, public from public, and even private from public
- [Oracles](./oracles) and how Aztec smart contracts might use them
- [How functions work under the hood](./inner_workings.md)
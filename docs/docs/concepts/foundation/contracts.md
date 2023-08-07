---
title: Smart Contracts
---

import Disclaimer from '../../misc/common/\_disclaimer.mdx';

<Disclaimer/>

A "smart contract" is defined as a set of public and private functions written as Noir circuits. These functions operate on public and private state stored by a contract. Each function is represented as a ZK SNARK verification key, where the contract is uniquely described by the set of its verification keys, and stored in the Aztec Contracts tree.

[Noir](https://noir-lang.org) is a programming language designed for converting high-level programs into ZK circuits. Based on Rust, the goal is to present an idiomatic way of writing private smart contracts that is familiar to Ethereum developers. Noir is under active development adding features such as contracts, functions and storage variables.

The end goal is a language that is intuitive to use for developers with no cryptographic knowledge and empowers developers to easily write programable private smart contracts.

There are no plans for EVM compatibility or to support Solidity in Aztec. The privacy-first nature of Aztec is fundamentally incompatible with the EVM architecture and Solidity's semantics. In addition, the heavy use of client-side proof construction makes this impractical.

## Enabling Transaction Semantics: The Aztec Kernel

There are two kernel circuits in Aztec, the private kernel and the public kernel. Each circuit validates the correct execution of a particular function call.

A transaction is built up by generating proofs for multiple recursive iterations of kernel circuits. Each call in the call stack is modelled as new iteration of the kernel circuit and are managed by a [FIFO](<https://en.wikipedia.org/wiki/FIFO_(computing_and_electronics)>) queue containing pending function calls. There are two call stacks, one for private calls and one for public calls.

One iteration of a kernel circuit will pop a call off of the stack and execute the call. If the call triggers subsequent contract calls, these are pushed onto the stack.

Private kernel proofs are generated first. The transaction is ready to move to the next phase when the private call stack is empty.

The public kernel circuit takes in proof of a public/private kernel circuit with an empty private call stack, and operates recursively until the public call stack is also empty.

A transaction is considered complete when both call stacks are empty.

The only information leaked about the transaction is:

1. The number of private state updates triggered
2. The set of public calls generated

The addresses of all private calls are hidden from observers.

---
title: Transactions
---

Transactions on Aztec start with a call from Aztec.js or the Aztec CLI, which creates a request containing transaction details. This request moves to the Private Execution Environment (PXE) which simulates and processes it. Then the PXE interacts with the Aztec Node which uses the sequencer to ensure that all the transaction details are enqueued properly. The sequencer then submits the block to the rollup contract, and the transaction is successfully mined.

On this page you'll learn:

- The step-by-step process of sending a transaction on Aztec
- The role of components like PXE, Aztec Node, ACIR simulator, and the sequencer
- The Aztec Kernel and its two circuits: private and public, and how they execute function calls
- The call stacks for private & public functions and how they determine a transaction's completion

<a href="https://raw.githubusercontent.com/AztecProtocol/aztec-packages/2fa143e4d88b3089ebbe2a9e53645edf66157dc8/docs/static/img/sandbox_sending_a_tx.svg"><img src="/img/sandbox_sending_a_tx.svg" alt="Sending a transaction" /></a>

See [this diagram](https://raw.githubusercontent.com/AztecProtocol/aztec-packages/2fa143e4d88b3089ebbe2a9e53645edf66157dc8/docs/static/img/sandbox_sending_a_tx.svg) for an in-depth overview of the transaction execution process. It highlights 3 different types of transaction execution: contract deployments, private transactions and public transactions.

See the page on [contract communication](./communication/main.md) for more context on transaction execution.

## Enabling Transaction Semantics: The Aztec Kernel

There are two kernel circuits in Aztec, the private kernel and the public kernel. Each circuit validates the correct execution of a particular function call.

A transaction is built up by generating proofs for multiple recursive iterations of kernel circuits. Each call in the call stack is modeled as new iteration of the kernel circuit and are managed by a [FIFO](<https://en.wikipedia.org/wiki/FIFO_(computing_and_electronics)>) queue containing pending function calls. There are two call stacks, one for private calls and one for public calls.

One iteration of a kernel circuit will pop a call off of the stack and execute the call. If the call triggers subsequent contract calls, these are pushed onto the stack.

Private kernel proofs are generated first. The transaction is ready to move to the next phase when the private call stack is empty.

The public kernel circuit takes in proof of a public/private kernel circuit with an empty private call stack, and operates recursively until the public call stack is also empty.

A transaction is considered complete when both call stacks are empty.

The only information leaked about the transaction is:

1. The number of private state updates triggered
2. The set of public calls generated

The addresses of all private calls are hidden from observers.

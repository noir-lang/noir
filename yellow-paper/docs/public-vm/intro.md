# Introduction

:::note reference
Many terms and definitions are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

An Aztec transaction may include one or more **public execution requests**. A public execution request is a request to execute a specified contract's public bytecode given some arguments. Execution of a contract's public bytecode is performed by the **Aztec Virtual Machine (AVM)**.

> A public execution request may originate from a public call enqueued by a transaction's private segment ([`enqueuedPublicFunctionCalls`](../calls/enqueued-calls.md)), or from a public [fee preparation](../gas-and-fees#fee-preparation) or [fee distribution](../gas-and-fees#fee-distribution) call.

In order to execute public contract bytecode, the AVM requires some context. An [**execution context**](./context) contains all information necessary to initiate AVM execution, including the relevant contract's bytecode and all state maintained by the AVM. A **contract call** initializes an execution context and triggers AVM execution within that context.

Instruction-by-instruction, the AVM [executes](./execution) the bytecode specified in its context. An **instruction** is a decoded bytecode entry that, when executed, modifies the AVM's execution context (in particular its [state](./state)) according to the instruction's definition in the ["AVM Instruction Set"](./instruction-set). Execution within a context ends when the AVM encounters a [**halt**](./execution#halting).

During execution, additional contract calls may be made. While an [**initial contract call**](./context#initial-contract-calls) initializes a new execution context directly from a public execution request, a [**nested contract call**](./nested-calls) occurs _during_ AVM execution and is triggered by a **contract call instruction** ([`CALL`](./instruction-set#isa-section-call), [`STATICCALL`](./instruction-set#isa-section-staticcall), or [`DELEGATECALL`](./instruction-set#isa-section-delegatecall)). It initializes a new execution context (**nested context**) from the current one (**calling context**) and triggers execution within it. When nested call's execution completes, execution proceeds in the calling context.

A **caller** is a contract call's initiator. The caller of an initial contract call is an Aztec sequencer. The caller of a nested contract call is the AVM itself executing in the calling context.

## Outline

- [**State**](./state): the state maintained by the AVM
- [**Memory model**](./memory-model): the AVM's type-tagged memory model
- [**Execution context**](./context): the AVM's execution context and its initialization for initial contract calls
- [**Execution**](#execution): control flow, gas tracking, normal halting, and exceptional halting
- [**Nested contract calls**](./nested-calls): the initiation of a contract call from an instruction as well as the processing of nested execution results, gas refunds, and state reverts
- [**Instruction set**](./instruction-set): the list of all instructions supported by the AVM
- [**AVM Circuit**](./avm-circuit): the AVM as a SNARK circuit for proving execution

> The sections prior to the "AVM Circuit" are meant to provide a high-level definition of the Aztec Virtual Machine as opposed to a specification of its SNARK implementation. They therefore mostly omit SNARK or circuit-centric verbiage except when particularly relevant to the high-level architecture.

> For an explanation of the AVM's bytecode, refer to ["AVM Bytecode"](../bytecode#avm-bytecode).

## Public contract bytecode

<!-- TODO: Merge this section into ../bytecode -->

A contract's public bytecode is a series of execution instructions for the AVM. Refer to the ["AVM Instruction Set"](./instruction-set) for the details of all supported instructions along with how they modify AVM state.

The entirety of a contract's public code is represented as a single block of bytecode with a maximum of `MAX_PUBLIC_INSTRUCTIONS_PER_CONTRACT` ($2^{15} = 32768$) instructions. The mechanism used to distinguish between different "functions" in an AVM bytecode program is left as a higher-level abstraction (_e.g._ similar to Solidity's concept of a function selector).

> See the [Bytecode Validation Circuit](./bytecode-validation-circuit) to see how a contract's bytecode can be validated and committed to.

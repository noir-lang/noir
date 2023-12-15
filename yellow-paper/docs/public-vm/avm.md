---
sidebar_position: 0
---

# Aztec Virtual Machine

:::important disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::
:::note reference
Many terms and definitions here are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

## Introduction
An Aztec transaction may include one or more public execution requests. A public execution request represents an initial "message call" to a contract, providing input data and triggering the execution of that contract's public code in the Aztec Virtual Machine. Given a message call to a contract, the AVM executes the corresponding code one instruction at a time, treating each instruction as a transition function on its state.

> Public execution requests may originate as [`enqueuedPublicFunctionCalls`](../calls/enqueued-calls.md) triggered during the transaction's private execution.

This document contains the following sections:
- **Public contract bytecode** (aka AVM bytecode)
- **Execution Context**, outlining the AVM's environment and state
- **Execution**, outlining control flow, gas tracking, halting, and reverting
- **Nested calls**, outlining the initiation of message calls, processing of sub-context results, gas refunds, and world state reverts

The **["AVM Instruction Set"](./InstructionSet)** document supplements this one with the list of all supported instructions and their associated state transition functions.

> Note: The Aztec Virtual Machine, while designed with a SNARK implementation in mind, is not strictly tied to any particular implementation and therefore is defined without SNARK or circuit-centric verbiage. That being said, considerations for a SNARK implementation are raised or linked when particularly relevant or helpful.

## Public contract bytecode
A contract's public bytecode is a series of execution instructions for the AVM. When a message call is made to a contract, the AVM retrieves the corresponding bytecode from the world state (`worldState.contracts[address].bytecode`) and triggers execution of the first instruction (`bytecode[0]`). The world state is described in more detail later.

> Note: While a Noir contract may have multiple public functions, they are inlined so that the **entirety of a contract's public code exists in a single bytecode**. Internal calls to Noir functions within the same contract are compiled to simple program-counter changes, as are internal returns. In a manner similar to the Ethereum Virtual Machine, the AVM is not itself aware of function selectors and internal function calls. The Noir compiler may implement these constructs by treating the first word in a message call's calldata as a function selector, and beginning a contract's bytecode with a series of conditional jumps.

> Note: See the [Bytecode Validation Circuit](./bytecode-validation-circuit.md) to see how a contract's bytecode can be validated and committed to.

## Execution Context
:::note REMINDER
Many terms and definitions here are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

An "Execution Context" includes the information necessary to trigger AVM execution along with the state maintained by the AVM throughout execution:
```
AVMContext {
    environment: ExecutionEnvironment,
    machineState: MachineState,
    worldState: WorldState,
    accruedSubstate: AccruedSubstate,
    results: MessageCallResults,
}
```

The first two entries, "Execution Environment" and "Machine State", share the same lifecycle. They contain information pertaining to a single message call and are initialized prior to the start of a call's execution.

> When a nested message call is made, a new environment and machine state are initialized by the caller. In other words, a nested message call has its own environment and machine state, although their initialization will be partially derived from the caller's context.

The "Execution Environment" is fully specified by a message call's execution agent and remains constant throughout a call's execution.
```
ExecutionEnvironment {
    address,
    origin,
    l1GasPrice,
    l2GasPrice,
    calldata: [],
    sender,
    portal,
    bytecode: [],
    blockHeader: BlockHeader,
    globalVariables: PublicGlobalVariables,
    messageCallDepth,
    isStaticCall,
}
```

"Machine State" is partially specified by the execution agent, and otherwise begins as empty or uninitialized for each message call. This state is transformed on an instruction-per-instruction basis.
```
MachineState {
    l1GasLeft,
    l2GasLeft,
    pc,
    memory: uninitialized,
}
```

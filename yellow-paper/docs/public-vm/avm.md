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

An "Execution Context" includes the information necessary to initiate AVM execution along with the state maintained by the AVM throughout execution:
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

> When a nested message call is made, a new environment and machine state are initialized by the caller. In other words, a nested message call has its own environment and machine state which are _partially_ derived from the caller's context.

The "Execution Environment" is fully specified by a message call's execution agent and remains constant throughout a call's execution.
```
ExecutionEnvironment {
    address,
    storageAddress,
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
    isDelegateCall,
}
```

"Machine State" is partially specified by the execution agent, and otherwise begins as empty or uninitialized for each message call. This state is transformed on an instruction-per-instruction basis.
```
MachineState {
    l1GasLeft,
    l2GasLeft,
    pc,
    memory: offset => value,  // uninitialized at start
}
```

"World State" contains persistable VM state. If a message call succeeds, its world state updates are applied to the calling context (whether that be a parent call's context or the transaction context). If a message call fails, its world state updates are rejected by its caller. When a _transaction_ succeeds, its world state updates persist into future transactions.
```
WorldState {
    publicStorage: (address, slot) => value,          // read/write
    noteHashes: (address, index) => noteHash,         // read & append only
    nullifiers: (address, index) => nullifier,        // read & append only
    l1l2messageHashes: (address, key) => messageHash, // read only
    contracts: (address) => bytecode,                 // read only
}
```

> Note: the notation `key => value` describes a mapping from `key` to `value`.

> Note: each member of the world state is implemented as an independent merkle tree with different properties.

The "Accrued Substate", as coined in the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper), contains information that is accrued throughout transaction execution to be "acted upon immediately following the transaction." These are append-only arrays containing state that is not relevant to other calls or transactions. Similar to world state, if a message call succeeds, its substate is appended to its calling context, but if it fails its substate is dropped by its caller.
```
AccruedSubstate {
    logs: [],           // append-only
    l2toL1Messages: [], // append-only
}
```

Finally, when a message call halts, it sets the context's "Message Call Results" to communicate results to the caller.
```
MessageCallResults {
    reverted: boolean,
    output: [] | undefined,
}
```

### Context initialization for initial call
This section outlines AVM context initialization specifically for a **public execution request's initial message call** (_i.e._ not a nested message call). Context initialization for nested message calls will be explained in a later section.

When AVM execution is initiated for a public execution request, the AVM context is initialized as follows:
```
context = AVMContext {
    environment: INITIAL_EXECUTION_ENVIRONMENT,
    machineState: INITIAL_MACHINE_STATE,
    accruedSubstate: empty,
    worldState: <latest world state>,
    results: INITIAL_MESSAGE_CALL_RESULTS,
}
```
> Note: Since world state persists between transactions, the latest state is injected into a new AVM context.

Given a `PublicCallRequest` and its parent `TxRequest`, these above-listed "`INITIAL_*`" entries are defined as follows:
```
INITIAL_EXECUTION_ENVIRONMENT = ExecutionEnvironment {
    address: PublicCallRequest.contractAddress,
    storageAddress: PublicCallRequest.CallContext.storageContractAddress,
    origin: TxRequest.origin,
    l1GasPrice: TxRequest.l1GasPrice,
    l2GasPrice: TxRequest.l2GasPrice,
    calldata: PublicCallRequest.args,
    sender: PublicCallRequest.CallContext.msgSender,
    portal: PublicCallRequest.CallContext.portalContractAddress,
    bytecode: worldState.contracts[PublicCallRequest.contractAddress],
    blockHeader: <latest block header>,
    globalVariables: <latest global variable values>
    messageCallDepth: 0,
    isStaticCall: PublicCallRequest.CallContext.isStaticCall,
    isDelegateCall: PublicCallRequest.CallContext.isDelegateCall,
}

INITIAL_MACHINE_STATE = MachineState {
    l1GasLeft: TxRequest.l1GasLimit,
    l2GasLeft: TxRequest.l2GasLimit,
    pc: 0,
    memory: uninitialized,
}

INITIAL_MESSAGE_CALL_RESULTS = MessageCallResults {
    reverted: false,
    output: undefined,
}
```

> Note: unlike memory in the Ethereum Virtual Machine, uninitialized memory in the AVM is not readable! A memory cell must be written (and therefore [type-tagged](./state-model#types-and-tagged-memory)) before it can be read.

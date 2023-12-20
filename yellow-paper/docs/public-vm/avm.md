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
An Aztec transaction may include one or more **public execution requests**. A public execution request represents an initial **message call** to a contract, providing input data and triggering the execution of that contract's public code in the Aztec Virtual Machine. Given a message call to a contract, the AVM executes the corresponding code one instruction at a time, treating each instruction as a transition function on its state.

> Public execution requests may originate as [`enqueuedPublicFunctionCalls`](../calls/enqueued-calls.md) triggered during the transaction's private execution.

This document contains the following sections:
- [**Public contract bytecode**](#public-contract-bytecode) (aka AVM bytecode)
- [**Execution context**](#execution-context), outlining the AVM's environment and state
- [**Execution**](#execution), outlining control flow, gas tracking, halting, and reverting
- [**Nested calls**](#nested-calls), outlining the initiation of message calls, processing of sub-context results, gas refunds, and world state reverts

Refer to the **["AVM Instruction Set"](./InstructionSet)** for the list of all supported instructions and their associated state transition functions.

For details on the AVM's "tagged" memory model, refer to the **["AVM Memory Model"](./state-model.md)**.

> Note: The Aztec Virtual Machine, while designed with a SNARK implementation in mind, is not strictly tied to any particular implementation and therefore is defined without SNARK or circuit-centric verbiage. That being said, considerations for a SNARK implementation are raised or linked when particularly relevant or helpful.

## Public contract bytecode
A contract's public bytecode is a series of execution instructions for the AVM. When a message call is made to a contract, the AVM retrieves the corresponding bytecode from the world state (`worldState.contracts[address].bytecode`) and triggers execution of the first instruction (`bytecode[0]`). The world state is described in more detail later.

> Note: While a Noir contract may have multiple public functions, they are inlined so that the **entirety of a contract's public code exists in a single bytecode**. Internal calls to Noir functions within the same contract are compiled to simple program-counter changes, as are internal returns. In a manner similar to the Ethereum Virtual Machine, the AVM is not itself aware of function selectors and internal function calls. The Noir compiler may implement these constructs by treating the first word in a message call's calldata as a function selector, and beginning a contract's bytecode with a series of conditional jumps.

> Note: See the [Bytecode Validation Circuit](./bytecode-validation-circuit.md) to see how a contract's bytecode can be validated and committed to.

Refer to ["Bytecode"](/docs/bytecode) for more information.

## Execution Context
:::note REMINDER
Many terms and definitions here are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

An **execution context** includes the information necessary to initiate AVM execution along with the state maintained by the AVM throughout execution:
```
AVMContext {
    environment: ExecutionEnvironment,
    machineState: MachineState,
    worldState: WorldState,
    accruedSubstate: AccruedSubstate,
    results: MessageCallResults,
}
```

The first two entries, **execution environment** and **machine state**, share the same lifecycle. They contain information pertaining to a single message call and are initialized prior to the start of a call's execution.

> When a nested message call is made, a new environment and machine state are initialized by the caller. In other words, a nested message call has its own environment and machine state which are _partially_ derived from the caller's context.

The **execution environment** is fully specified by a message call's execution agent and remains constant throughout a call's execution.
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

**Machine state** is partially specified by the execution agent, and otherwise begins as empty or uninitialized for each message call. This state is transformed on an instruction-per-instruction basis.
```
MachineState {
    l1GasLeft,
    l2GasLeft,
    pc,
    memory: offset => value,  // uninitialized at start
}
```

**World state** contains persistable VM state. If a message call succeeds, its world state updates are applied to the calling context (whether that be a parent call's context or the transaction context). If a message call fails, its world state updates are rejected by its caller. When a _transaction_ succeeds, its world state updates persist into future transactions.
```
WorldState {
    publicStorage: (address, slot) => value,           // read/write
    noteHashes: (address, index) => noteHash,          // read & append only
    nullifiers: (address, index) => nullifier,         // read & append only
    l1l2messageHashes: (address, key) => messageHash,  // read only
    contracts: (address) => {bytecode, portalAddress}, // read only
}
```

> Note: the notation `key => value` describes a mapping from `key` to `value`.

> Note: each member of the world state is implemented as an independent merkle tree with different properties.

The **accrued substate**, as coined in the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper), contains information that is accrued throughout transaction execution to be "acted upon immediately following the transaction." These are append-only arrays containing state that is not relevant to other calls or transactions. Similar to world state, if a message call succeeds, its substate is appended to its calling context, but if it fails its substate is dropped by its caller.
```
AccruedSubstate {
    logs: [],           // append-only
    l2toL1Messages: [], // append-only
}
```

Finally, when a message call halts, it sets the context's **message call results** to communicate results to the caller.
```
MessageCallResults {
    reverted: boolean,
    output: [] | undefined,
}
```

### Context initialization for initial call
This section outlines AVM context initialization specifically for a **public execution request's initial message call** (_i.e._ not a nested message call). Context initialization for nested message calls will be explained [in a later section](#context-initialization-for-a-nested-call).
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
    internalCallStack: empty,
}

INITIAL_MESSAGE_CALL_RESULTS = MessageCallResults {
    reverted: false,
    output: undefined,
}
```

> Note: unlike memory in the Ethereum Virtual Machine, uninitialized memory in the AVM is not readable! A memory cell must be written (and therefore [type-tagged](./state-model#types-and-tagged-memory)) before it can be read.

## Execution
With an initialized context (and therefore an initial program counter of 0), the AVM can execute a message call starting with the very first instruction in its bytecode.

### Program Counter and Control Flow
The program counter (machine state's `pc`) determines which instruction to execute (`instr = environment.bytecode[pc]`). Each instruction's state transition function updates the program counter in some way, which allows the VM to progress to the next instruction at each step.

Most instructions simply increment the program counter by 1. This allows VM execution to flow naturally from instruction to instruction. Some instructions ([`JUMP`](./InstructionSet#isa-section-jump), [`JUMPI`](./InstructionSet#isa-section-jumpi), `INTERNALCALL`) modify the program counter based on inputs.

The `INTERNALCALL` instruction jumps to the destination specified by its input (sets `pc` to that destination), but first it pushes the current `pc+1` to `machineState.internalCallStack`. The `INTERNALRETURN` instruction pops a destination from `machineState.internalCallStack` and jumps there.

> Jump destinations can only be constants from the contract bytecode, or destinations popped from `machineState.internalCallStack`. A jump destination will never originate from main memory.

### Gas limits and tracking
Each instruction has an associated `l1GasCost` and `l2GasCost`. Before an instruction is executed, the VM enforces that there is sufficient gas remaining via the following assertions:
```
assert machineState.l1GasLeft - instr.l1GasCost > 0
assert machineState.l2GasLeft - instr.l2GasCost > 0
```
> Note: many instructions (like arithmetic operations) have 0 `l1GasCost`. Instructions only incur an L1 cost if they modify world state or accrued substate.

If these assertions pass, the machine state's gas left is decreased prior to the instruction's core execution:
```
machineState.l1GasLeft -= instr.l1GasCost
machineState.l2GasLeft -= instr.l2GasCost
```

If either of these assertions _fail_ for an instruction, this triggers an exceptional halt. The gas left is set to 0 and execution reverts.
```
machineState.l1GasLeft = 0
machineState.l2GasLeft = 0
```
> Reverting and exceptional halts will be covered in more detail [in a later section](#halting).

### Gas cost notes and examples
A instruction's gas cost is loosely derived from its complexity. Execution complexity of some instructions changes based on inputs. Here are some examples and important notes:
- [`JUMP`](./InstructionSet/#isa-section-jump) is an example of an instruction with constant gas cost. Regardless of its inputs, the instruction always incurs the same `l1GasCost` and `l2GasCost`.
- The [`SET`](./InstructionSet/#isa-section-set) instruction operates on a different sized constant (based on its `dst-type`). Therefore, this instruction's gas cost increases with the size of its input.
- Instructions that operate on a data range of a specified "size" scale in cost with that size. An example of this is the [`CALLDATACOPY`](./InstructionSet/#isa-section-calldatacopy) argument which copies `copySize` words from `environment.calldata` to memory.
- The [`CALL`](./InstructionSet/#isa-section-call)/[`STATICCALL`](./InstructionSet/#isa-section-call)/`DELEGATECALL` instruction's gas cost is determined by its `l*Gas` arguments, but any gas unused by the triggered message call is refunded after its completion ([more on this later](#updating-the-calling-context-after-nested-call-halts)).
- An instruction with "offset" arguments (like [`ADD`](./InstructionSet/#isa-section-add) and many others), has increased cost for each offset argument that is flagged as "indirect".

> Implementation detail: an instruction's gas cost will roughly align with the number of rows it corresponds to in the SNARK execution trace including rows in the sub-operation table, memory table, chiplet tables, etc.

> Implementation detail: an instruction's gas cost takes into account the costs of associated downstream computations. So, an instruction that triggers accesses to the public data tree (`SLOAD`/`SSTORE`) incurs a cost that accounts for state access validation in later circuits (public kernel or rollup). An instruction that triggers a nested message call (`CALL`/`STATICCALL`/`DELEGATECALL`) incurs a cost accounting for the nested call's execution and an added execution of the public kernel circuit.

## Halting
A message call's execution can end with a **normal halt** or **exceptional halt**. A halt ends execution within the current context and returns control flow to the calling context.

### Normal halting
A normal halt occurs when the VM encounters an explicit halting instruction ([`RETURN`](./InstructionSet/#isa-section-return) or [`REVERT`](./InstructionSet/#isa-section-revert)). Such instructions consume gas normally and optionally initialize some output data before finally halting execution within the current context.
```
machineState.l1GasLeft -= instr.l1GasCost
machineState.l2GasLeft -= instr.l2GasCost
// results.reverted remains false
results.output = machineState.memory[instr.args.retOffset:instr.args.retOffset+instr.args.retSize]
```
> Definitions: `retOffset` and `retSize` here are arguments to the [`RETURN`](./InstructionSet/#isa-section-return) and [`REVERT`](./InstructionSet/#isa-section-revert) instructions. If `retSize` is 0, the context will have no output. Otherwise, these arguments point to a region of memory to output.

> Note: `results.output` is only relevant when the caller is a message call itself. When a public execution request's initial message call halts normally, its `results.output` is ignored.

### Exceptional halting
An exceptional halt is not explicitly triggered by an instruction but instead occurs when one of the following halting conditions is met:
1. **Insufficient gas**
    ```
    assert machineState.l1GasLeft - instr.l1GasCost > 0
    assert machineState.l2GasLeft - instr.l2GasCost > 0
    ```
1. **Invalid instruction encountered**
    ```
    assert environment.bytecode[machineState.pc].opcode <= MAX_AVM_OPCODE
    ```
1. **Failed memory tag check**
    - Defined per-instruction in the [Instruction Set](./InstructionSet)
1. **Jump destination past end of bytecode**
    ```
    assert machineState.pc >= environment.bytecode.length
    ```
1. **World state modification attempt during a static call**
    ```
    assert !environment.isStaticCall
        OR environment.bytecode[machineState.pc].opcode not in WS_MODIFYING_OPS
    ```
    > Definition: `WS_MODIFYING_OPS` represents the list of all opcodes corresponding to instructions that modify world state.

When an exceptional halt occurs, the context is flagged as consuming all off its allocated gas and marked as `reverted` with no output data, and then execution within the current context ends.
```
machineState.l1GasLeft = 0
machineState.l2GasLeft = 0
results.reverted = true
// results.output remains undefined
```

## Nested calls
During a message call's execution, an instruction may be encountered that triggers another message call. A message call triggered in this way may be referred to as a **nested call**. The purpose of the [`CALL`](./InstructionSet/#isa-section-call), [`STATICCALL`](./InstructionSet/#isa-section-staticcall), and `DELEGATECALL` instructions is to initiate nested calls.


### Context initialization for a nested call
Initiation of a nested call requires the creation of a new context (or **sub-context**).
```
subContext = AVMContext {
    environment: nestedExecutionEnvironment, // defined below
    machineState: nestedMachineState,        // defined below
    worldState: callingContext.worldState,
    accruedSubstate: empty,
    results: INITIAL_MESSAGE_CALL_RESULTS,
}
```
While some context members are initialized as empty (as they are for an initial message call), other entries are derived from the calling context or from the message call instruction's arguments (`instr.args`).

The world state is forwarded as-is to the sub-context. Any updates made to the world state before this message call instruction was encountered are carried forward into the sub-context.

The environment and machine state for the new sub-context are initialized as shown below. Here, the `callingContext` refers to the context in which the nested message call instruction was encountered.
```
// some assignments reused below
isStaticCall = instr.opcode == STATICCALL_OP
isDelegateCall = instr.opcode == DELEGATECALL_OP
contract = callingContext.worldState.contracts[instr.args.addr]

nestedExecutionEnvironment = ExecutionEnvironment {
    address: instr.args.addr,
    storageAddress: isDelegateCall ? callingContext.environment.storageAddress : instr.args.addr,
    origin: callingContext.origin,
    l1GasPrice: callingContext.l1GasPrice,
    l2GasPrice: callingContext.l2GasPrice,
    calldata: instr.args.calldata,
    sender: callingContext.address,
    portal: contract.portal,
    bytecode: contract.bytecode,
    blockHeader: callingContext.blockHeader,
    globalVariables: callingContext.globalVariables,
    messageCallDepth: callingContext.messageCallDepth + 1,
    isStaticCall: isStaticCall,
    isDelegateCall: isDelegateCall,
}

nestedMachineState = MachineState {
    l1GasLeft: callingContext.machineState.memory[instr.args.gasOffset],
    l2GasLeft: callingContext.machineState.memory[instr.args.gasOffset+1],
    pc: 0,
    memory: uninitialized,
    internalCallStack: empty,
}
```
> Note: the sub-context machine state's `l*GasLeft` is initialized based on the call instruction's `gasOffset` argument. The caller allocates some amount of L1 and L2 gas to the nested call. It does so using the instruction's `gasOffset` argument. In particular, prior to the message call instruction, the caller populates `M[gasOffset]` with the sub-context's initial `l1GasLeft`. Likewise it populates `M[gasOffset+1]` with `l2GasLeft`.

> Note: recall that `INITIAL_MESSAGE_CALL_RESULTS` is the same initial value used during [context initialization for a public execution request's initial message call](#context-initialization-for-initial-call).
> `STATICCALL_OP` and `DELEGATECALL_OP` refer to the 8-bit opcode values for the `STATICCALL` and `DELEGATECALL` instructions respectively.

### Updating the calling context after nested call halts
When a message call's execution encounters an instruction that itself triggers a message call, the nested call executes until it reaches a halt. At that point, control returns to the caller, and the calling context is updated based on the sub-context and the message call instruction's transition function. The components of that transition function are defined below.

The success or failure of the nested call is captured into memory at the offset specified by the call instruction's `successOffset` input:
```
context.machineState.memory[instr.args.successOffset] = !subContext.results.reverted
```

Recall that a nested call is allocated some gas. In particular, the call instruction's `gasOffset` input points to an L1 and L2 gas allocation for the nested call. As shown in the [section above](#context-initialization-for-a-nested-call), a nested call's `subContext.machineState.l1GasLeft` is initialized to `context.machineState.memory[instr.args.gasOffset]`. Likewise, `l2GasLeft` is initialized from `gasOfffset+1`.

As detailed in [the gas section above](#gas-cost-notes-and-examples), every instruction has an associated `instr.l1GasCost` and `instr.l2GasCost`. A nested call instruction's cost is the same as its initial `l*GasLeft` and `l2GasLeft`. Prior to the nested call's execution, this cost is subtracted from the calling context's remaining gas.

When a nested call completes, any of its allocated gas that remains unused is refunded to the caller.
```
context.l1GasLeft += subContext.machineState.l1GasLeft
context.l2GasLeft += subContext.machineState.l2GasLeft
```

If a nested call halts normally with a [`RETURN`](./InstructionSet/#isa-section-return) or [`REVERT`](./InstructionSet/#isa-section-revert), it may have some output data (`subContext.results.output`). The caller's `retOffset` and `retSize` arguments to the nested call instruction specify a region in memory to place output data when the nested call returns.
```
if instr.args.retSize > 0:
    context.memory[instr.args.retOffset:instr.args.retOffset+instr.args.retSize] = subContext.results.output
```

As long as a nested call has not reverted, its updates to the world state and accrued substate will be absorbed into the calling context.
```
if !subContext.results.reverted AND instr.opcode != STATICCALL_OP:
    context.worldState = subContext.worldState
    context.accruedSubstate.append(subContext.accruedSubstate)
```
> Reminder: a nested call cannot make updates to the world state or accrued substate if it is a [`STATICCALL`](./InstructionSet/#isa-section-staticcall).
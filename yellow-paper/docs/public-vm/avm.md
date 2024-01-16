# Aztec Virtual Machine

:::note reference
Many terms and definitions here are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

## Introduction

An Aztec transaction may include one or more **public execution requests**. A public execution request is a request to execute a specified contract's public bytecode given some arguments. Execution of a contract's public bytecode is performed by the **Aztec Virtual Machine (AVM)**.

> A public execution request may originate from a public call enqueued by a transaction's private segment ([`enqueuedPublicFunctionCalls`](../calls/enqueued-calls.md)), or from a public [fee preparation](../gas-and-fees#fee-preparation) or [fee distribution](../gas-and-fees#fee-distribution) call.

In order to execute public contract bytecode, the AVM requires some context. An [**execution context**](#execution-context) contains all information necessary to initiate AVM execution, including the relevant contract's bytecode and all state maintained by the AVM. A **contract call** initializes an execution context and triggers AVM execution within that context.

Instruction-by-instruction, the AVM [executes](#execution) the bytecode specified in its context. An **instruction** is a bytecode entry that, when executed, modifies the AVM's execution context according to the instruction's definition in the ["AVM Instruction Set"](./instruction-set). Execution within a context ends when the AVM encounters a [**halt**](#halting).

During execution, additional contract calls may be made. While an [**initial contract call**](#initial-contract-calls) initializes a new execution context directly from a public execution request, a [**nested contract call**](#nested-contract-calls) occurs _during_ AVM execution and is triggered by a **contract call instruction** ([`CALL`](./instruction-set#isa-section-call), [`STATICCALL`](./instruction-set#isa-section-call), or `DELEGATECALL`). It initializes a new execution context (**nested context**) from the current one (**calling context**) and triggers execution within it. When nested call's execution completes, execution proceeds in the calling context.

A **caller** is a contract call's initiator. The caller of an initial contract call is an Aztec sequencer. The caller of a nested contract call is the AVM itself executing in the calling context.

## Outline

- [**Public contract bytecode**](#public-contract-bytecode) (aka AVM bytecode)
- [**Execution context**](#execution-context), outlining the AVM's environment and state
- [**Execution**](#execution), outlining control flow, gas tracking, halting, and reverting
- [**Initial contract calls**](#initial-contract-calls), outlining the initiation of a contract call from a public execution request
- [**Nested contract calls**](#nested-contract-calls), outlining the initiation of a contract call from an instruction as well as the processing of nested execution results, gas refunds, and state reverts

> This document is meant to provide a high-level definition of the Aztec Virtual Machine as opposed to a specification of its SNARK implementation. The document therefore mostly omits SNARK or circuit-centric verbiage except when particularly relevant to high-level design decisions.

This document is supplemented by the following resources:
- **[AVM Instruction Set](./instruction-set)**
- **[AVM Memory Model](./state-model.md)**
- **[AVM Circuit](./avm-circuit.md)**

## Public contract bytecode

A contract's public bytecode is a series of execution instructions for the AVM. Refer to the ["AVM Instruction Set"](./instruction-set) for the details of all supported instructions along with how they modify AVM state.

The entirety of a contract's public code is represented as a single block of bytecode with a maximum of `MAX_PUBLIC_INSTRUCTIONS_PER_CONTRACT` ($2^{15} = 32768$) instructions. The mechanism used to distinguish between different "functions" in an AVM bytecode program is left as a higher-level abstraction (_e.g._ similar to Solidity's concept of a function selector).

> See the [Bytecode Validation Circuit](./bytecode-validation-circuit.md) to see how a contract's bytecode can be validated and committed to.

## Execution Context

:::note REMINDER
Many terms and definitions here are borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).
:::

Initialized by a contract call, an **execution context** includes the information necessary to initiate AVM execution along with all state maintained by the AVM throughout execution:

```
AvmContext {
    environment: ExecutionEnvironment,
    machineState: MachineState,
    worldState: WorldState,
    journal: Journal,
    accruedSubstate: AccruedSubstate,
    results: ContractCallResults,
}
```

### Execution Environment

A context's **execution environment** remains constant throughout the context's execution. When a contract call initializes its execution context, it fully specifies the execution environment.

```
ExecutionEnvironment {
    address: AztecAddress,
    storageAddress: AztecAddress,
    origin: AztecAddress,
    l1GasPrice: field,
    l2GasPrice: field,
    daGasPrice: field,
    sender: AztecAddress,
    portal: AztecAddress,
    blockHeader: BlockHeader,
    globalVariables: PublicGlobalVariables,
    contractCallDepth: field,
    isStaticCall: boolean,
    isDelegateCall: boolean,
    calldata: [field; <calldata-length>],
    bytecode: [field; <bytecode-length>],
}
```

### Machine State

A context's **machine state** is transformed on an instruction-per-instruction basis. When a contract call initializes its execution context, it specifies the first few fields (`*GasLeft`) of the machine state and initializes the remaining members as follows:

```
MachineState {
    l1GasLeft: field,
    l2GasLeft: field,
    daGasLeft: field,
    pc: field = 0,
    internalCallStack: Vector<field> = [], // initialized as empty
    memory: [field; 32768] = [0, ..., 0],  // all 32768 (2^32) entries are initialized to zero
}
```

The machine state's entries are defined as follows:
- `l1GasLeft`: how much L1 gas remains
- `l2GasLeft`: how much L2 gas remains
- `daGasLeft`: how much DA (data availability) gas remains
- `pc` (program counter): index to the current instruction being executed in the context's bytecode
- `internalCallStack`: a stack of program counters pushed to and popped from by `INTERNALCALL` and `INTERNALRETURN` instructions
- `memory`: a $2^{32}$ entry memory space accessible by user code (bytecode instructions) and initialized to all zeros upon context initialization

### World State

The AVM has access to a subset of [Aztec's persistent global state](../state), and an AVM execution context exposes a limited interface to that state. In particular, while much of Aztec's state is implemented as readable/writeable merkle trees, these structures are exposed in the AVM as simple mappings or vectors with access-limitations.

An execution context's **world state** is its interface to Aztec's global state. When an [_initial_ contract call](#initial-contract-calls) is made, its context is initialized with a snapshot of Aztec's latest global state. When a [_nested_ contract call](#nested-contract-calls) is made, its context is initialized with a snapshot of the calling context's current world state.

When a context's execution [halts](#halting), the caller accepts or rejects its world state modifications. If execution [returned](./instruction-set#isa-section-return) without reverting, the caller accepts its world state modifications and updates its world state accordingly. If execution reverted ([normally](./instruction-set#isa-section-revert) or [exceptionally](#exceptional-halting)), the caller rejects its world state modifications.

A context's world state interface is defined as follows:
```
WorldState {
    contracts: AztecAddress => {bytecode, portalAddress}, // read-only from within AVM
    blockHeaders: Vector<BlockHeader>,                    // read-only from within AVM
    publicStorage: (AztecAddress, field) => value,        // read/write
    l1ToL2Messages: (AztecAddress, field) => message,     // read-only from within AVM
    l2ToL1Messages: Vector<[field; <msg-length>]>,        // append-only (no reads) from within AVM
    noteHashes: Vector<field>,                            // append-only (no reads) from within AVM
    nullifiers: Vector<field>,                            // append-only (no reads) from within AVM
}
```
> The notation `key => value` describes a mapping from `key` to `value`.

> Read-only or append-only structures in the world state may have more open access-limitations outside of the AVM, but the AVM's interface imposes the above-listed access-limitations. As an example, private execution can deploy new contracts, but contract deployments are not supported by the AVM. Thus `contracts` is read-only here.


### Journal

**Journal** tracks all world state accesses (reads and writes) that have taken place thus far during a contract call's execution. Unlike world state, a context's journal is accepted by its caller regardless of whether execution reverts.
```
Journal {
    nestedCalls: Vector<(AztecAddress, boolean)>,
    blockHeaderReads: Vector<(field, BlockHeader)>,
    publicStorageAccesses: Vector<StorageReadContext | StorageWriteContext>,
    l1ToL2MessageReads: Vector<(L1toL2MessageContext, [field; <msg-length>])>,
    newL2ToL1Messages: Vector<(L2toL1MessageContext, [field; <msg-length>])>,
    newNoteHashes: Vector<NoteHashContext>,
    newNullifiers: Vector<NullifierContext>,
}
```
> The types tracked in the journal vectors are listed in Aztec's [private function types specification](../circuits/private-function#types) and Aztec's [public kernel types specification](../circuits/public-kernel-tail#types).

> This journal structure is important for imposing limitations on the maximum number of allowable world state accesses, and for communicating the list of state accesses to the public kernel circuit.

### Accrued Substate
The **accrued substate**, as coined in the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper), contains information that is accrued throughout transaction execution to be "acted upon immediately following the transaction." These are append-only arrays containing state that is not relevant to other calls. Similar to world state, if a contract call returns normally, its substate is appended to its calling context, but if it reverts its substate is rejected by its caller.

```
AccruedSubstate {
    unencryptedLogs: Vector<[field; <log-length>]>,
}
```

### Contract Call Results
Finally, when a contract call halts, it sets the context's **contract call results** to communicate results to the caller.

```
ContractCallResults {
    reverted: boolean,
    output: [field; <output-length>],
}
```

## Execution

Once an execution context has been initialized for a contract call, the machine state's program counter determines which instruction the AVM executes. For any contract call, the program counter starts at zero, and so instruction execution begins with the very first entry in a contract's bytecode.

### Program Counter and Control Flow

The program counter (`machineState.pc`) determines which instruction the AVM executes next (`instr = environment.bytecode[pc]`). Each instruction's execution updates the program counter in some way, which allows the AVM to progress to the next instruction at each step.

Most instructions simply increment the program counter by 1. This allows VM execution to flow naturally from instruction to instruction. Some instructions ([`JUMP`](./instruction-set#isa-section-jump), [`JUMPI`](./instruction-set#isa-section-jumpi), `INTERNALCALL`) modify the program counter based on arguments.

The `INTERNALCALL` instruction pushes `machineState.pc+1` to `machineState.internalCallStack` and then updates `pc` to the instruction's destination argument (`instr.args.loc`). The `INTERNALRETURN` instruction pops a destination from `machineState.internalCallStack` and assigns the result to `pc`.

> An instruction will never assign program counter a value from memory (`machineState.memory`). A `JUMP`, `JUMPI`, or `INTERNALCALL` instruction's destination is a constant from the program bytecode. This property allows for easier static program analysis.

### Gas limits and tracking
> See ["Gas and Fees"](../gas-and-fees) for a deeper dive into Aztec's gas model and for definitions of each type of gas.

Each instruction has an associated `l1GasCost`, `l2GasCost`, and `daGasCost`. Before an instruction is executed, the VM enforces that there is sufficient gas remaining via the following assertions:
```
assert machineState.l1GasLeft - instr.l1GasCost > 0
assert machineState.l2GasLeft - instr.l2GasCost > 0
assert machineState.daGasLeft - instr.daGasCost > 0
```

> Many instructions (like arithmetic operations) have 0 `l1GasCost` and `daGasCost`. Instructions only incur an L1 or DA cost if they modify the world state, journal, or accrued substate.

If these assertions pass, the machine state's gas left is decreased prior to the instruction's core execution:

```
machineState.l1GasLeft -= instr.l1GasCost
machineState.l2GasLeft -= instr.l2GasCost
machineState.daGasLeft -= instr.daGasCost
```

If either of these assertions _fail_ for an instruction, this triggers an exceptional halt. The gas left is set to 0 and execution reverts.

```
machineState.l1GasLeft = 0
machineState.l2GasLeft = 0
machineState.daGasLeft = 0
```

> Reverting and exceptional halts are covered in more detail in the ["Halting" section](#halting).

### Gas cost notes and examples

An instruction's gas cost is meant to reflect the computational cost of generating a proof of its correct execution. For some instructions, this computational cost changes based on inputs. Here are some examples and important notes:
- [`JUMP`](./instruction-set/#isa-section-jump) is an example of an instruction with constant gas cost. Regardless of its inputs, the instruction always incurs the same `l1GasCost`, `l2GasCost`, and `daGasCost`.
- The [`SET`](./instruction-set/#isa-section-set) instruction operates on a different sized constant (based on its `dst-type`). Therefore, this instruction's gas cost increases with the size of its input.
- Instructions that operate on a data range of a specified "size" scale in cost with that size. An example of this is the [`CALLDATACOPY`](./instruction-set/#isa-section-calldatacopy) argument which copies `copySize` words from `environment.calldata` to `machineState.memory`.
- The [`CALL`](./instruction-set/#isa-section-call)/[`STATICCALL`](./instruction-set/#isa-section-call)/`DELEGATECALL` instruction's gas cost is determined by its `*Gas` arguments, but any gas unused by the nested contract call's execution is refunded after its completion ([more on this later](#updating-the-calling-context-after-nested-call-halts)).
- An instruction with "offset" arguments (like [`ADD`](./instruction-set/#isa-section-add) and many others), has increased cost for each offset argument that is flagged as "indirect".

> An instruction's gas cost will roughly align with the number of rows it corresponds to in the SNARK execution trace including rows in the sub-operation table, memory table, chiplet tables, etc.

> An instruction's gas cost takes into account the costs of associated downstream computations. An instruction that triggers accesses to the public data tree (`SLOAD`/`SSTORE`) incurs a cost that accounts for state access validation in later circuits (public kernel or rollup). A contract call instruction (`CALL`/`STATICCALL`/`DELEGATECALL`) incurs a cost accounting for the nested call's complete execution as well as any work required by the public kernel circuit for this additional call.

## Halting

A context's execution can end with a **normal halt** or **exceptional halt**. A halt ends execution within the current context and returns control flow to the calling context.

### Normal halting

A normal halt occurs when the VM encounters an explicit halting instruction ([`RETURN`](./instruction-set#isa-section-return) or [`REVERT`](./instruction-set#isa-section-revert)). Such instructions consume gas normally and optionally initialize some output data before finally halting the current context's execution.

```
machineState.l1GasLeft -= instr.l1GasCost
machineState.l2GasLeft -= instr.l2GasCost
machineState.daGasLeft -= instr.daGasCost
results.reverted = instr.opcode == REVERT
results.output = machineState.memory[instr.args.retOffset:instr.args.retOffset+instr.args.retSize]
```

> Definitions: `retOffset` and `retSize` here are arguments to the [`RETURN`](./instruction-set/#isa-section-return) and [`REVERT`](./instruction-set#isa-section-revert) instructions. If `retSize` is 0, the context will have no output. Otherwise, these arguments point to a region of memory to output.

> `results.output` is only relevant when the caller is a contract call itself. In other words, it is only relevant for [nested contract calls](#nested-contract-calls). When an [initial contract call](#initial-contract-calls) (initiated by a public execution request) halts normally, its `results.output` is ignored.

### Exceptional halting

An exceptional halt is not explicitly triggered by an instruction but instead occurs when an exceptional condition is met.

When an exceptional halt occurs, the context is flagged as consuming all off its allocated gas and is marked as `reverted` with no output data, and then execution within the current context ends.

```
machineState.l1GasLeft = 0
machineState.l2GasLeft = 0
machineState.daGasLeft = 0
results.reverted = true
// results.output remains empty
```

The AVM's exceptional halting conditions area listed below:

1. **Insufficient gas**
    ```
    assert machineState.l1GasLeft - instr.l1GasCost > 0
    assert machineState.l2GasLeft - instr.l2GasCost > 0
    assert machineState.daGasLeft - instr.l2GasCost > 0
    ```
1. **Invalid instruction encountered**
    ```
    assert environment.bytecode[machineState.pc].opcode <= MAX_AVM_OPCODE
    ```
1. **Jump destination past end of bytecode**
    ```
    assert environment.bytecode[machineState.pc].opcode not in {JUMP, JUMPI, INTERNALCALL}
        OR instr.args.loc < environment.bytecode.length
    ```
1. **Failed memory tag check**
    - Defined per-instruction in the [Instruction Set](./instruction-set)
1. **Maximum memory index ($2^{32}$) exceeded**
    ```
    for offset in instr.args.*Offset:
        assert offset < 2^32
    ```
1. **World state modification attempt during a static call**
    ```
    assert !environment.isStaticCall
        OR environment.bytecode[machineState.pc].opcode not in WS_MODIFYING_OPS
    ```
    > Definition: `WS_MODIFYING_OPS` represents the list of all opcodes corresponding to instructions that modify world state.
1. **Maximum contract call depth (1024) exceeded**
    ```
    assert environment.contractCallDepth <= 1024
    assert environment.bytecode[machineState.pc].opcode not in {CALL, STATICCALL, DELEGATECALL}
        OR environment.contractCallDepth < 1024
    ```
1. **Maximum contract call calls per execution request (1024) exceeded**
    ```
    assert journal.nestedCalls.length <= 1024
    assert environment.bytecode[machineState.pc].opcode not in {CALL, STATICCALL, DELEGATECALL}
        OR journal.nestedCalls.length < 1024
    ```
1. **Maximum internal call depth (1024) exceeded**
    ```
    assert machineState.internalCallStack.length <= 1024
    assert environment.bytecode[machineState.pc].opcode != INTERNALCALL
        OR environment.contractCallDepth < 1024
    ```
1. **Maximum world state accesses (1024-per-type) exceeded**
    ```
    assert publicStorageAccesses.length <= 1024
        AND l1ToL2MessagesReads.length <= 1024
        AND newL2ToL1Messages.length <= 1024
        AND newNoteHashes.length <= 1024
        AND newNullifiers.length <= 1024

    // Storage
    assert environment.bytecode[machineState.pc].opcode not in {SLOAD, SSTORE}
        OR publicStorageAccesses.length < 1024

    // L1 to L2 messages
    assert environment.bytecode[machineState.pc].opcode != GETL1TOL2MSG
        OR l1ToL2MessagesReads.length < 1024

    // L2 to L1 messages
    assert environment.bytecode[machineState.pc].opcode != SENDL2TOL1MSG
        OR newL2ToL1Messages.length < 1024

    // Note hashes
    assert environment.bytecode[machineState.pc].opcode != EMITNOTEHASH
        OR newNoteHashes.length < 1024

    // Nullifiers
    assert environment.bytecode[machineState.pc].opcode != EMITNULLIFIER
        OR newNullifiers.length < 1024
    ```
    > Definition: `WS_MODIFYING_OPS` represents the list of all opcodes corresponding to instructions that modify world state.

## Initial contract calls

An **initial contract call** initializes a new execution context from a public execution request.

### Context initialization for initial contract calls

An initial contract call initializes its execution context as follows:
```
context = AvmContext {
    environment = INITIAL_EXECUTION_ENVIRONMENT,
    machineState = INITIAL_MACHINE_STATE,
    worldState = <latest world state>,
    journal = INITIAL_JOURNAL,
    accruedSubstate = INITIAL_ACCRUED_SUBSTATE,
    results = INITIAL_MESSAGE_CALL_RESULTS,
}
```

> Since world state persists between transactions, the latest state is injected into a new AVM context.

Given a [`PublicCallRequest`](../transactions/tx-object#public-call-request) and its parent [`TxRequest`](../transactions/local-execution#execution-request), these above-listed "`INITIAL_*`" entries are defined as follows:

```
INITIAL_EXECUTION_ENVIRONMENT = ExecutionEnvironment {
    address = PublicCallRequest.contractAddress,
    storageAddress = PublicCallRequest.CallContext.storageContractAddress,
    origin = TxRequest.origin,
    l1GasPrice = TxRequest.l1GasPrice,
    l2GasPrice = TxRequest.l2GasPrice,
    daGasPrice = TxRequest.daGasPrice,
    sender = PublicCallRequest.CallContext.msgSender,
    portal = PublicCallRequest.CallContext.portalContractAddress,
    blockHeader = <latest block header>,
    globalVariables = <latest global variable values>
    contractCallDepth = 0,
    isStaticCall = PublicCallRequest.CallContext.isStaticCall,
    isDelegateCall = PublicCallRequest.CallContext.isDelegateCall,
    calldata = PublicCallRequest.args,
    bytecode = worldState.contracts[PublicCallRequest.contractAddress],
}

INITIAL_MACHINE_STATE = MachineState {
    l1GasLeft = TxRequest.l1GasLimit,
    l2GasLeft = TxRequest.l2GasLimit,
    daGasLeft = TxRequest.daGasLimit,
    pc = 0,
    internalCallStack = [], // initialized as empty
    memory = [0, ..., 0],   // all 32768 (2^32) entries are initialized to zero
}

INITIAL_JOURNAL = Journal {
    nestedCalls = [],           // initialized as empty
    blockHeaderReads = [],      // initialized as empty
    publicStorageAccesses = [], // initialized as empty
    l1ToL2MessageReads = [],    // initialized as empty
    newL2ToL1Messages = [],     // initialized as empty
    newNoteHashes = [],         // initialized as empty
    newNullifiers = [],         // initialized as empty
}

INITIAL_ACCRUED_SUBSTATE = AccruedSubstate {
    unencryptedLogs = [], // initialized as empty
}

INITIAL_MESSAGE_CALL_RESULTS = ContractCallResults {
    reverted = false,
    output = [], // initialized as empty
}
```

## Nested contract calls

To review, a **nested contract call** occurs _during_ AVM execution and is triggered by a contract call instruction ([`CALL`](./instruction-set/#isa-section-call), [`STATICCALL`](./instruction-set/#isa-section-call), or `DELEGATECALL`). It initializes a new execution context (**nested context**) from the current one (the **calling context**) along with the call instruction's arguments. A nested contract call triggers AVM execution in that new context, and returns execution to the calling context upon completion.

### Context initialization for nested calls

A nested contract call initializes its execution context as follows:

```
nestedContext = AvmContext {
    environment: nestedExecutionEnvironment, // defined below
    machineState: nestedMachineState,        // defined below
    worldState: callingContext.worldState,
    journal: callingContext.journal,
    accruedSubstate: INITIAL_ACCRUED_SUBSTATE,
    results: INITIAL_MESSAGE_CALL_RESULTS,
}
```

While some context members are initialized as empty (as they are for an initial contract call), other entries are derived from the calling context or from the contract call instruction's arguments (`instr.args`).

The world state is forwarded as-is to the nested context. Any updates made to the world state before this contract call instruction was encountered are carried forward into the nested context.

The environment and machine state for the new context are initialized as shown below:

```
// some assignments reused below
isStaticCall = instr.opcode == STATICCALL_OP
isDelegateCall = instr.opcode == DELEGATECALL_OP
contract = callingContext.worldState.contracts[instr.args.addr]
calldataStart = instr.args.argsOffset
calldataEnd = calldataStart + instr.args.argsSize

nestedExecutionEnvironment = ExecutionEnvironment {
    address: instr.args.addr,
    storageAddress: isDelegateCall ? callingContext.environment.storageAddress : instr.args.addr,
    origin: callingContext.origin,
    l1GasPrice: callingContext.l1GasPrice,
    l2GasPrice: callingContext.l2GasPrice,
    daGasPrice: callingContext.daGasPrice,
    sender: callingContext.address,
    portal: contract.portal,
    blockHeader: callingContext.blockHeader,
    globalVariables: callingContext.globalVariables,
    contractCallDepth: callingContext.contractCallDepth + 1,
    isStaticCall: isStaticCall,
    isDelegateCall: isDelegateCall,
    calldata: callingContext.memory[calldataStart:calldataEnd],
    bytecode: contract.bytecode,
}

nestedMachineState = MachineState {
    l1GasLeft: callingContext.machineState.memory[instr.args.gasOffset],
    l2GasLeft: callingContext.machineState.memory[instr.args.gasOffset+1],
    daGasLeft: callingContext.machineState.memory[instr.args.gasOffset+2],
    pc = 0,
    internalCallStack = [], // initialized as empty
    memory = [0, ..., 0],   // all 32768 (2^32) entries are initialized to zero
}
```
> The nested context's machine state's `*GasLeft` is initialized based on the call instruction's `gasOffset` argument. The caller allocates some amount of L1, L2, and DA gas to the nested call. It does so using the instruction's `gasOffset` argument. In particular, prior to the contract call instruction, the caller populates `M[gasOffset]` with the nested context's initial `l1GasLeft`. Likewise it populates `M[gasOffset+1]` with `l2GasLeft` and `M[gasOffset+2]` with `daGasLeft`.

> Recall that initial values named as `INITIAL_*` are the same ones used during [context initialization for an initial contract call](#context-initialization-for-initial-contract-calls).

> `STATICCALL_OP` and `DELEGATECALL_OP` refer to the 8-bit opcode values for the `STATICCALL` and `DELEGATECALL` instructions respectively.

### Updating the calling context after nested call halts

A nested context's execution proceeds until it reaches a [halt](#halting). At that point, control returns to the caller, and the calling context is updated based on the nested context and the contract call instruction's transition function. The components of that transition function are defined below.

The success or failure of the nested call is captured into memory at the offset specified by the call instruction's `successOffset` input:

```
context.machineState.memory[instr.args.successOffset] = !nestedContext.results.reverted
```

Recall that a contract call is allocated some gas. In particular, the nested call instruction's `gasOffset` input points to an L1, L2, and DA gas allocation for the nested call. As shown in the [section above](#context-initialization-for-nested-calls), a nested call's `machineState.l1GasLeft` is initialized to `callingContext.machineState.memory[instr.args.gasOffset]`. Likewise, `l2GasLeft` is initialized from `gasOffset+1` and `daGasLeft` from `gasOffset+2`.

As detailed in [the gas section above](#gas-cost-notes-and-examples), every instruction has an associated `instr.l1GasCost`, `instr.l2GasCost`, and `instr.daGasCost`. A nested call instruction's cost is the same as its initial `*GasLeft`. Prior to the nested context's execution, this cost is subtracted from the calling context's remaining gas.

When a nested context halts, any of its allocated gas that remains unused is refunded to the caller.

```
context.l1GasLeft += nestedContext.machineState.l1GasLeft
context.l2GasLeft += nestedContext.machineState.l2GasLeft
context.daGasLeft += nestedContext.machineState.daGasLeft
```

If a nested context halts normally with a [`RETURN`](./instruction-set#isa-section-return) or [`REVERT`](./instruction-set#isa-section-revert), it may have some output data (`nestedContext.results.output`). The nested call instruction's `retOffset` and `retSize` arguments specify a region in the calling context's memory to place output data when the nested context halts.

```
if instr.args.retSize > 0:
    context.memory[instr.args.retOffset:instr.args.retOffset+instr.args.retSize] = nestedContext.results.output
```

As long as a nested context has not reverted, its updates to the world state and accrued substate will be absorbed into the calling context.

```
if !nestedContext.results.reverted AND instr.opcode != STATICCALL_OP:
    context.worldState = nestedContext.worldState
    context.accruedSubstate.append(nestedContext.accruedSubstate)
```

Regardless of whether a nested context has reverted, its journal updates are absorbed into the calling context along with a new `nestedCalls` entry.
```
context.journal = nestedContext.journal
context.journal.append(nestedContext.address, nestedContext.machineState.reverted)
```

> Reminder: a nested call cannot make updates to the world state, journal, or accrued substate if it is a [`STATICCALL`](./instruction-set/#isa-section-staticcall).

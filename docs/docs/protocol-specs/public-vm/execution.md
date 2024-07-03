# Execution, Gas, Halting


Execution of an AVM program, within a provided [execution context](./context), includes the following steps:

1. Fetch contract bytecode and decode into a vector of [AVM instructions](./instruction-set)
1. Repeat the next step until a [halt](#halting) is reached
1. Execute the instruction at the index specified by the context's [program counter](#program-counter-and-control-flow)
    - Instruction execution will update the program counter

The following shorthand syntax is used to refer to this execution routine in the ["Instruction Set"](./instruction-set), ["Nested execution"](./nested-calls#nested-execution), and other sections:

```jsx
execute(context)
```

## Bytecode fetch and decode

Before execution begins, a contract's bytecode is retrieved.
```jsx
bytecode = context.worldState.contracts[context.environment.address].bytecode
```

> As described in ["Contract Deployment"](../contract-deployment), contracts are not stored in a dedicated tree. A [contract class](../contract-deployment/classes) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractClass` structure (which contains the bytecode) and a nullifier representing the class identifier. A [contract instance](../contract-deployment/instances) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractInstance` structure and a nullifier representing the contract address.

> Thus, the syntax used above for bytecode retrieval is shorthand for:
>1. Perform a membership check of the contract instance address nullifier
>1. Retrieve the `ContractInstance` from a database that tracks all such unencrypted logs
>    ```jsx
>    contractInstance = contractInstances[context.environment.address]
>    ```
>1. Perform a membership check of the contract class identifier nullifier
>1. Retrieve the `ContractClass` and its bytecode from a database that tracks all such unencrypted logs
>    ```jsx
>    contractClass = contractClasses[contractInstance.contract_class_id]
>    bytecode = contractClass.packed_public_bytecode
>    ```

The bytecode is then decoded into a vector of `instructions`. An instruction is referenced throughout this document according to the following interface:

| Member     | Description |
| ---        | ---         |
| `opcode`   | The 8-bit opcode value that identifies the operation an instruction is meant to perform. |
| `indirect` | Toggles whether each memory-offset argument is an indirect offset. Rightmost bit corresponds to 0th offset arg, etc. Indirect offsets result in memory accesses like `M[M[offset]]` instead of the more standard `M[offset]`. |
| `inTag`    | The [tag/size](./memory-model.md#tags-and-tagged-memory) to check inputs against and/or tag the destination with. |
| `args`     | Named arguments as specified for an instruction in the ["Instruction Set"](./instruction-set). As an example, `instr.args.aOffset` refers to an instructions argument named `aOffset`. |
| `execute`  | Apply this instruction's transition function to an execution context (_e.g._ `instr.execute(context)`). |


## Instruction execution

Once bytecode has been fetched and decoded into the `instructions` vector, instruction execution begins.

The AVM executes the instruction at the index specified by the context's program counter.
```jsx
while (!halted)
    instr = instructions[machineState.pc]
    instr.execute(context)
```

An instruction's execution mutates the context's state as specified in the ["Instruction Set"](./instruction-set).

## Program Counter and Control Flow

A context is initialized with a program counter of zero, and so instruction execution always begins with a contract's the very first instruction.

The program counter specifies which instruction the AVM will execute next, and each instruction's execution updates the program counter in some way. This allows the AVM to progress to the next instruction at each step.

Most instructions simply increment the program counter by 1. This allows VM execution to flow naturally from instruction to instruction. Some instructions ([`JUMP`](./instruction-set#isa-section-jump), [`JUMPI`](./instruction-set#isa-section-jumpi), [`INTERNALCALL`](./instruction-set#isa-section-internalcall)) modify the program counter based on arguments.

The `INTERNALCALL` instruction pushes `machineState.pc+1` to `machineState.internalCallStack` and then updates `pc` to the instruction's destination argument (`instr.args.loc`). The `INTERNALRETURN` instruction pops a destination from `machineState.internalCallStack` and assigns the result to `pc`.

> An instruction will never assign program counter a value from memory (`machineState.memory`). A `JUMP`, `JUMPI`, or `INTERNALCALL` instruction's destination is a constant from the program bytecode. This property allows for easier static program analysis.

## Gas checks and tracking
> See ["Gas and Fees"](../gas-and-fees) for a deeper dive into Aztec's gas model and for definitions of each type of gas.

Each instruction has an associated `l2GasCost` and `daGasCost`. The AVM uses these values to enforce that sufficient gas is available before executing an instruction, and to deduct the cost from the context's remaining gas. The process of checking and charging gas is referred to in other sections using the following shorthand:

```jsx
chargeGas(context, l2GasCost, daGasCost)
```

### Checking gas

Before an instruction is executed, the VM enforces that there is sufficient gas remaining via the following assertions:
```
assert machineState.l2GasLeft - instr.l2GasCost >= 0
assert machineState.daGasLeft - instr.daGasCost >= 0
```

> Many instructions (like arithmetic operations) have 0 `daGasCost`. Instructions only incur a DA cost if they modify the [world state](./state#avm-world-state) or [accrued substate](./state#accrued-substate).

### Charging gas

If these assertions pass, the machine state's gas left is decreased prior to the instruction's core execution:

```
machineState.l2GasLeft -= instr.l2GasCost
machineState.daGasLeft -= instr.daGasCost
```

If either of these assertions _fail_ for an instruction, this triggers an exceptional halt. The gas left is set to 0 and execution reverts.

```
machineState.l2GasLeft = 0
machineState.daGasLeft = 0
```

> Reverting and exceptional halts are covered in more detail in the ["Halting" section](#halting).

### Gas cost notes and examples

An instruction's gas cost is meant to reflect the computational cost of generating a proof of its correct execution. For some instructions, this computational cost changes based on inputs. Here are some examples and important notes:

- All instructions have a base cost. [`JUMP`](./instruction-set/#isa-section-jump) is an example of an instruction with constant gas cost. Regardless of its inputs, the instruction always incurs the same `l2GasCost` and `daGasCost`.
- The [`SET`](./instruction-set/#isa-section-set) instruction operates on a different sized constant (based on its `dstTag`). Therefore, this instruction's gas cost increases with the size of its input.
- In addition to the base cost, the cost of an instruction increases with the number of reads and writes to memory. This is affected by the total number of input and outputs: the gas cost for [`AND`](./instruction-set/#isa-section-and) should be greater than that of [`NOT`](./instruction-set/#isa-section-not) since it takes one more input.
- Input parameters flagged as "indirect" require an extra memory access, so these should further increase the gas cost of the instruction.
- The base cost for instructions that operate on a data range of a specified "size" scale in cost with that size, but only if they perform an operation on the data other than copying. For example, [`CALLDATACOPY`](./instruction-set/#isa-section-calldatacopy) copies `copySize` words from `environment.calldata` to `machineState.memory`, so its increased cost is captured by the extra memory accesses. On the other hand, [`SSTORE`](./instruction-set#isa-section-sstore) requires accesses to persistent storage proportional to `srcSize`, so its base cost should also increase.
- The [`CALL`](./instruction-set#isa-section-call)/[`STATICCALL`](./instruction-set#isa-section-staticcall)/[`DELEGATECALL`](./instruction-set#isa-section-delegatecall) instruction's gas cost is determined by its `*Gas` arguments, but any gas unused by the nested contract call's execution is refunded after its completion ([more on this later](./nested-calls#updating-the-calling-context-after-nested-call-halts)).

> An instruction's gas cost will roughly align with the number of rows it corresponds to in the SNARK execution trace including rows in the sub-operation table, memory table, chiplet tables, etc.

> An instruction's gas cost takes into account the costs of associated downstream computations. An instruction that triggers accesses to the public data tree (`SLOAD`/`SSTORE`) incurs a cost that accounts for state access validation in later circuits (public kernel or rollup). A contract call instruction (`CALL`/`STATICCALL`/`DELEGATECALL`) incurs a cost accounting for the nested call's complete execution as well as any work required by the public kernel circuit for this additional call.

## Halting

A context's execution can end with a **normal halt** or **exceptional halt**. A halt ends execution within the current context and returns control flow to the calling context.

### Normal halting

A normal halt occurs when the VM encounters an explicit halting instruction ([`RETURN`](./instruction-set#isa-section-return) or [`REVERT`](./instruction-set#isa-section-revert)). Such instructions consume gas normally and optionally initialize some output data before finally halting the current context's execution.

```
machineState.l2GasLeft -= instr.l2GasCost
machineState.daGasLeft -= instr.daGasCost
results.reverted = instr.opcode == REVERT
results.output = machineState.memory[instr.args.retOffset:instr.args.retOffset+instr.args.retSize]
```

> Definitions: `retOffset` and `retSize` here are arguments to the [`RETURN`](./instruction-set/#isa-section-return) and [`REVERT`](./instruction-set#isa-section-revert) instructions. If `retSize` is 0, the context will have no output. Otherwise, these arguments point to a region of memory to output.

> `results.output` is only relevant when the caller is a contract call itself. In other words, it is only relevant for [nested contract calls](./nested-calls). When an [initial contract call](./context#initial-contract-calls) (initiated by a public execution request) halts normally, its `results.output` is ignored.

### Exceptional halting

An exceptional halt is not explicitly triggered by an instruction but instead occurs when an exceptional condition is met.

When an exceptional halt occurs, the context is flagged as consuming all of its allocated gas and is marked as `reverted` with _no output data_, and then execution within the current context ends.

```
machineState.l2GasLeft = 0
machineState.daGasLeft = 0
results.reverted = true
// results.output remains empty
```

The AVM's exceptional halting conditions area listed below:

1. **Insufficient gas**
    ```
    assert machineState.l2GasLeft - instr.l2GasCost >= 0
    assert machineState.daGasLeft - instr.l2GasCost >= 0
    ```
1. **Invalid instruction encountered**
    ```
    assert instructions[machineState.pc].opcode <= MAX_AVM_OPCODE
    ```
1. **Jump destination past end of program**
    ```
    assert instructions[machineState.pc].opcode not in {JUMP, JUMPI, INTERNALCALL}
        OR instr.args.loc < instructions.length
    ```
1. **Failed memory tag check**
    - Defined per-instruction in the [Instruction Set](./instruction-set)
1. **Out of bounds memory access (max memory offset is $2^{32}-1$)**
    ```
    for offset in instr.args.*Offset:
        assert offset < 2^32
    ```
1. **World state modification attempt during a static call**
    ```
    assert !environment.isStaticCall
        OR instructions[machineState.pc].opcode not in WS_AS_MODIFYING_OPS
    ```
    > Definition: `WS_AS_MODIFYING_OPS` represents the list of all opcodes corresponding to instructions that modify world state or accrued substate.
1. **Maximum contract call depth (1024) exceeded**
    ```
    assert environment.contractCallDepth <= 1024
    assert instructions[machineState.pc].opcode not in {CALL, STATICCALL, DELEGATECALL}
        OR environment.contractCallDepth < 1024
    ```
1. **Maximum contract call calls per execution request (1024) exceeded**
    ```
    assert worldStateAccessTrace.contractCalls.length <= 1024
    assert instructions[machineState.pc].opcode not in {CALL, STATICCALL, DELEGATECALL}
        OR worldStateAccessTrace.contractCalls.length < 1024
    ```
1. **Maximum internal call depth (1024) exceeded**
    ```
    assert machineState.internalCallStack.length <= 1024
    assert instructions[machineState.pc].opcode != INTERNALCALL
        OR environment.contractCallDepth < 1024
    ```
1. **Maximum world state accesses (1024-per-category) exceeded**
    ```
    assert worldStateAccessTrace.publicStorageReads.length <= 1024
        AND worldStateAccessTrace.publicStorageWrites.length <= 1024
        AND worldStateAccessTrace.noteHashChecks.length <= 1024
        AND worldStateAccessTrace.noteHashes.length <= 1024
        AND worldStateAccessTrace.nullifierChecks.length <= 1024
        AND worldStateAccessTrace.nullifiers.length <= 1024
        AND worldStateAccessTrace.l1ToL2MessageChecks.length <= 1024
        AND worldStateAccessTrace.archiveChecks.length <= 1024

    // Storage
    assert instructions[machineState.pc].opcode != SLOAD
        OR worldStateAccessTrace.publicStorageReads.length < 1024
    assert instructions[machineState.pc].opcode != SSTORE
        OR worldStateAccessTrace.publicStorageWrites.length < 1024

    // Note hashes
    assert instructions[machineState.pc].opcode != NOTEHASHEXISTS
        OR noteHashChecks.length < 1024
    assert instructions[machineState.pc].opcode != EMITNOTEHASH
        OR noteHashes.length < 1024

    // Nullifiers
    assert instructions[machineState.pc].opcode != NULLIFIEREXISTS
        OR nullifierChecks.length < 1024
    assert instructions[machineState.pc].opcode != EMITNULLIFIER
        OR nullifiers.length < 1024

    // Read L1 to L2 messages
    assert instructions[machineState.pc].opcode != L1TOL2MSGEXISTS
        OR worldStateAccessTrace.l1ToL2MessagesChecks.length < 1024

    // Archive tree & Headers
    assert instructions[machineState.pc].opcode != HEADERMEMBER
        OR archiveChecks.length < 1024
    ```
1. **Maximum accrued substate entries (per-category) exceeded**
    ```
    assert accruedSubstate.unencryptedLogs.length <= MAX_UNENCRYPTED_LOGS
        AND accruedSubstate.sentL2ToL1Messages.length <= MAX_SENT_L2_TO_L1_MESSAGES

    // Unencrypted logs
    assert instructions[machineState.pc].opcode != EMITUNENCRYPTEDLOG
        OR unencryptedLogs.length < MAX_UNENCRYPTED_LOGS

    // Sent L2 to L1 messages
    assert instructions[machineState.pc].opcode != SENDL2TOL1MSG
        OR sentL2ToL1Messages.length < MAX_SENT_L2_TO_L1_MESSAGES
    ```
    > Note that ideally the AVM should limit the _total_ accrued substate entries per-category instead of the entries per-call.

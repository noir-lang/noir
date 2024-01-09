---
sidebar_position: 1
---

# AVM Circuit

## Memory
To process a public execution request, the AVM executes the request's initial message call along with any nested calls it encounters. Execution of a message call requires some context including an `ExecutionEnvironment` and `MachineState`. Separate instances of these constructs must exist for each message call.

AVM instructions may read from or write to these constructs (explicitly or indirectly), and therefore it is natural to represent them in the AVM circuit via a memory table. Since each call must have its own `ExecutionEnvironment` and `MachineState`, each entry in the memory table must specify which call it corresponds to. This is accomplished via a `callPointer` column. The memory table is sorted first by `callPointer` and thus all memory accesses for a given message call are grouped.

User code has explicit access to a construct known as **user memory**, also known as `MachineState.memory`. When an AVM instruction performs an access like `M[offset]`, it is accessing user memory.

The remainder of a call's `ExecutionEnvironment` and `MachineState` is not explicitly addressable by user code. This remaining context lives in a construct known as **protected memory** and is accessible only via dedicated instructions (like `ADDRESS`, `JUMP`, `CALL`, etc...).

> Note: the fact that this context is implemented as protected circuit memory is not relevant to user code or even to the high-level AVM specification.

Therefore, for a given call the VM circuit's memory table is subdivided into user and protected memories. This is accomplished via a `userMemory` column which flags each of a call's memory table entries as either a user or protected memory access.

The VM circuit's memory is sorted first by `callPointer` and next by the `userMemory` flag (before standard sorting by memory address, timestamp, etc...). Thus, the memory table is organized as follows:
- VM circuit memory
    - call `0` memory
        - protected memory
        - user memory
    - call `1` memory
        - protected memory
        - user memory
    - ...
    - call `n-1` memory
        - protected memory
        - user memory

### Protected memory offsets
As mentioned above, a call's `ExecutionEnvironment` and `MachineState` (except for `MachineState.memory`) reside in protected memory, and so each of their members has a dedicated offset. These offsets are referred to according to the following pattern:
- `ENVIRONMENT_ADDRESS_OFFSET`: offset to `ExecutionEnvironment.address` within a call's protected memory subregion
- `ENVIRONMENT_L1GASPRICE`: offset to `ExecutionEnvironment.l1GasPrice` within a call's protected memory subregion
- `MACHINESTATE_L1GASLEFT`: offset to `MachineState.l1GasLeft` within a call's protected memory subregion
- `MACHINESTATE_PC`: offset to `MachineState.pc` within a call's protected memory subregion
- `MACHINESTATE_INTERNALCALLSTACK`: offset to `MachineState.internalCallStack` within a call's protected memory subregion

> Note: A call's `ExecutionEnvironment.bytecode` and `ExecutionEnvironment.calldata` are not included in the protected memory region because they are handled in a special manner. This will be expanded on in a later section.
> For complete definitions of `ExecutionEnvironment` and `MachineState` see the [AVM's high level specification](./avm.md).

### Protected memory and user memory examples
An instruction like `ADDRESS` serves as great example because it performs a read from protected memory and a write to user memory: `M[dstOffset] = ExecutionEnvironment.address` (see [Instruction Set](./InstructionSet) for more details). Below, this operation is deconstructed into its two memory accesses:
1. `ExecutionEnvironment.address`
    - memory read
    - flags: `callPointer`, `userMemory = 0` (protected memory access)
    - offset: `ENVIRONMENT_ADDRESS_OFFSET`
1. `M[dstOffset] =`
    - memory write
    - flags: `callPointer`, `userMemory = 1` (user memory access)
    - offset: `dstOffset`

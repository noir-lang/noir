---
sidebar_position: 1
---

# AVM Circuit

## Call pointer
Each message call processed within a single VM circuit execution is assigned a unique **call pointer**. There is certain information that must be tracked by the VM circuit on a per-call basis. For example, each call will correspond to the execution of a different contract's bytecode, and each call will access call-specific memory. As a per-call unique identifier, call pointer enables bytecode and memory lookups, among other things, on a per-call basis.

Call pointers are assigned based on execution order. A request's initial message call is assigned call pointer of `1`. The first nested message call encountered during execution is assigned call pointer of `2`. The VM circuit tracks the highest call pointer assigned thus far, and whenever a nested call instruction is encountered, it increments that value and assigns the result to that call.

### "Input" and "output" call pointers
It is important to note that the initial call's pointer is `1`, not `0`. The zero call pointer is a special case known as the "input" call pointer.

As expanded on later, the VM circuit memory table has a separate section for each call pointer. The memory table section for the **input call pointer** is reserved for the initial call's `ExecutionEnvironment` and initial `MachineState` as they appear in the circuit's inputs. This will be expanded on later.

## Bytecode
The VM circuit's primary purpose is to prove execution of the correct sequence of instructions given a message call's bytecode and inputs. The circuit will prove correct execution of any nested message calls as well. Each nested call will have its own bytecode and inputs, but will be processed within the same circuit.

Thus, a circuit column is assembled to contain the bytecode for all of a request's message calls (initial and nested). If a request's execution contains message call's to contracts A, B, C, and D (in that order), the VM circuit's bytecode column will contain A's bytecode, followed by B's, C's, and finally D's. Each one will be zero-padded to some constant length `CONTRACT_BYTECODE_MAX_LENGTH`.

The bytecode column will be paired with a call pointer column and program counter column. These three columns make up the **bytecode table**, where an instruction is paired with the call pointer and program counter it corresponds to.

Each row in the execution trace will also contain a call pointer and program counter, enabling a lookup into the bytecode table to retrieve the proper instruction (opcode and arguments). Through this mechanism, the VM circuit enforces that every executed instruction corresponds to the correct entry in the bytecode column.

Each contract's public bytecode is committed to during contract deployment. As part of the AVM circuit verification algorithm, the bytecode column (as a concatenation of all relevant contract bytecodes) is verified against the corresponding bytecode commitments. This is expanded on in ["Bytecode Validation Circuit"](./bytecode-validation-circuit.md).

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

## Circuit I/O

### How do "Public Inputs" work in the AVM circuit?
ZK circuit proof systems generally define some mechanism for "public inputs" for which witness values must be communicated in full to a verifier. The AVM proof system defines its own mechanism for public inputs in which it flags certain trace columns as "public input columns". Any public input columns must be communicated in full to a verifier.

### AVM public inputs structure
The VM circuit's I/O is defined as the `AvmPublicInputs` structure detailed below:
```
AvmSideEffects {
    newNoteHashes,
    newNullifiers,
    newL2ToL1Messages,
    unencryptedLogs,
}
AvmPublicInputs {
    initialEnvironment: ExecutionEnvironment & {l1GasLeft, l2GasLeft, daGasLeft},
    calldata: [],
    sideEffects: AvmSideEffects,
    storageAccesses,
    gasResults: {l1GasLeft, l2GasLeft, daGasLeft},
}
```

### AVM public input columns
The `AvmPublicInputs` structure is represented in the VM trace via the following public input columns:
1. `initialEnvironment` has a dedicated column and is used to initialize the initial call's `ExecutionEnvironment` and `MachineState`
1. `calldata` has its own dedicated public input column
1. `sideEffects: AvmSideEffects`
    - This represents the final `AccruedSubstate` of the initial message call
    - There is a separate sub-table (columns) for each side-effect vector
        - Each row in the `newNoteHashes` sub-table contains `{contractAddress, noteHash}`
        - Each row in the `newNullifiers` sub-table contains `{contractAddress, nullifier}`
        - Each row in the `newL2ToL1Messages` sub-table contains `{contractAddress, wordIndex, messageWord}`
            - where a message containing N words takes up N entries with increasing `wordIndex`
        - Each row in the `unencryptedLogs` sub-table contains `{contractAddress, wordIndex, logWord}`
            - where a log containing N words takes up N entries with increasing `wordIndex`
    - Side effects are present in the trace in execution-order
1. `storageAccesses`
    - This contains the first and last public storage access for each slot that is accessed during execution
    - Each row in the `storageAccesses` sub-table contains `{contractAddress, slot, value}`
    - Storage accesses are present in the trace in execution-order
1. `gasResults: AvmGasResults`
    - This is derived from the _final_ `MachineState` of the initial message call

### Initial call's protected memory
Any lookup into protected memory from a request's initial message call must retrieve a value matching the `initialEnvironment` public inputs column\*. To enforce this, an equivalence check is applied between the `initialEnvironment` column and the memory trace for protected memory accesses that use call pointer `1`.

> \* `MachineState` has entries (`pc`, `internalCallStack`) that are not initialized from inputs. Accesses to these entries from the initial message call do _not_ trigger lookups into a public inputs column.

> Note: protected memory is irrelevant for the "input call pointer" itself (`0`). The initial call's protected memory (call pointer `1`) is constructed to match the public inputs column. The "input call pointer" is only relevant for `calldata` as explained next.

### Initial call's calldata
Similarly, any lookup into calldata from a request's initial message call must retrieve a value matching the `calldata` public inputs column. To enforce this, an equivalence check is applied between the `calldata` column and the memory trace for user memory accesses that use "input call pointer".

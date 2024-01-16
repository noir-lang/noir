# AVM Circuit
The AVM circuit's purpose is to prove execution of a sequence of instructions for a public execution request. Regardless of whether execution succeeds or reverts, the circuit always generates a valid proof of execution.

## Circuit Architecture
The circuit is comprised of the following components:
- **Bytecode Table**: includes bytecode for all calls, indexed by call pointer and program counter.
- **Instruction Controller**: fetches an instruction from the Bytecode Table. Decodes the instructions into sub-operations to be forwarded to other modules.
- **Intermediate Registers**: for staging sub-operation inputs and outputs.
- **Control Flow Unit**: maintains program counter and call pointer. Processes control-flow sub-operations (program counter increments, internal call stack operations, contract call operations).
- **Gas Controller**: tracks remaining gas for current call. Processes gas tracking sub-operations.
- **Memory Controller**: processes memory sub-operations to load and store data between memory and intermediate registers.
- **Storage Controller**: processes storage sub-operations to load and store data between storage and intermediate registers.
- **Side-effect Accumulator**: processes side-effect sub-operations by pushing to a side-effect vector.
- **Chiplets**: perform compute operations on intermediate registers. Some chiplets include the ALU, Conditional Unit, Type Converter, and Crypto/Hash Gadgets.
- **Circuit I/O**: data structures used to ingest circuit inputs and emit outputs.

## Bytecode Table
To review, the AVM circuit's primary purpose is to prove execution of the proper sequence of instructions given a contract call's bytecode and inputs. The circuit will prove correct execution of any nested contract calls as well. Each nested call will have its own bytecode and inputs, but will be processed within the same circuit.

Prior to the VM circuit's execution, a vector is assembled to contain the bytecode for all of a request's contract calls (initial and nested). If a request's execution contains contract calls to contracts A, B, C, and D (in that order), the VM circuit's bytecode vector will contain A's bytecode, followed by B's, C's, and finally D's. Each one will be zero-padded to some constant length `MAX_PUBLIC_INSTRUCTIONS_PER_CONTRACT`.

Each entry in the bytecode vector will be paired with a call pointer and program counter. This **Bytecode Table** maps a call pointer and program counter to an instruction, and is used by the Instruction Controller to fetch instructions.
> Note: "call pointer" is expanded on in a later section.

Each contract's public bytecode is committed to during contract deployment. As part of the AVM circuit verification algorithm, the bytecode vector (as a concatenation of all relevant contract bytecodes) is verified against the corresponding bytecode commitments. This is expanded on in ["Bytecode Validation Circuit"](./bytecode-validation-circuit.md). While the AVM circuit enforces that the correct instructions are executed according to its bytecode table, the verifier checks that bytecode table against the previously validated bytecode commitments.

## Instruction Controller
The Instruction Controller's responsibilities include instruction fetching and decoding.

### Instruction fetching
The Instruction Controller's **instruction fetch** mechanism makes use of the bytecode table to determine which instruction to execute based on the call pointer and program counter. Each instruction fetch corresponds to a circuit lookup to enforce that the correct instruction is processed for a given contract and program counter.

The combination of the instruction fetch circuitry, the bytecode table, and the ["Bytecode Validation Circuit"](./bytecode-validation-circuit.md) ensure that VM circuit processes the proper sequence of instructions.

### Instruction decoding and sub-operations
An instruction (its opcode, flags, and arguments) represents some high-level VM operation. For example, an `ADD` instruction says "add two items from memory and store the result in memory". The Instruction Controller **instruction decode** mechanism decodes instructions into sub-operations. While an instruction likely requires many circuit components, a **sub-operation** is a smaller task that can be fed to just one VM circuit component for processing. By decoding an instruction into sub-operations, the VM circuit translates high-level instructions into smaller achievable tasks. To continue with the `ADD` example, it would translate "add two items from memory and store the result in memory" to "load an item from memory, load another item from memory, add them, and store the result to memory."

A **pre-computed/hardcoded sub-operations table** maps instruction opcodes to sub-operations. This provides the Instruction Controller with everything it needs to decode an instruction.

The Instruction Controller forwards sub-operations according to the following categorizations:
- Control flow sub-operations are forwarded to the Control Flow Unit
- Gas tracking sub-operations are forwarded to the Gas Controller
- Memory sub-operations are forwarded to the Memory Controller
- Storage sub-operations are forwarded to the Storage Controller
- Side-effect sub-operations are forwarded to the Side-effect Controller
- A chiplet sub-operation is forwarded to the proper chiplet

**TODO: table of all sub-operations by category (copy from hackmd with updates)**

> Note: for simple instructions (like `ADD`), the instruction can be fetched and all sub-operations can be processed in a single clock cycle. Since the VM circuit has limited resources, some complex instructions (like `CALLDATACOPY`) involve too many sub-operations to be processed in one clock cycle. A "clock cycle" in the AVM circuit represents the smallest subdivision of time during which some parallel operations can be performed. A clock cycle corresponds to a row in the circuit's **operations trace**. Simple instructions correspond to only a single row in this trace, but complex instructions span multiple rows. A `CLK` column tracks the clock cycle for each row and its set of sub-operations.

#### Decoding example
The `ADD` instruction is decoded into two `LOAD` memory sub-operations, an `ADD` ALU (chiplet) sub-operation, and a `STORE` memory sub-operation.

Take the following `ADD` instruction as an example: `ADD<u32> aOffset bOffset dstOffset`. Assuming this instruction is executed as part of contract call with pointer `C`, it is decoded into the following sub-operations:
```
// Load word from call's memory into register Ia (index 0)
LOAD 0 aOffset // Ia = M<C>[aOffset]
// Load word from call's memory into register Ib (index 1)
LOAD 1 bOffset // Ib = M<C>[aOffset]
// Use the ALU chiplet in ADD<32> mode to add registers Ia and Ib
// Place the results in Ic
ADD<u32> // Ic = ALU_ADD<u32>(Ia, Ib)
// Store results of addition from register Ic (index 2) to memory
STORE 2 dstOffset
```

> Note: the `ADD` instruction is an example of a "simple" instruction that can be fully processed in a single clock cycle. All four of the above-listed sub-operations happen in one clock cycle and therefore take up only a single row in the circuit's operations trace.

## Intermediate Registers
User code (AVM bytecode) has no concept of "registers", and so instructions often operate directly on user memory. Sub-operations on the other hand operate on intermediate registers. The only circuit component that has direct access to memory is the Memory Controller (further explained later), and therefore only memory sub-operations access memory. All other sub-operations operate on **intermediate registers** which serve as a staging ground between memory and the various processing components of the VM circuit.

Three intermediate registers exist: $I_a$, $I_b$, and $I_c$.

> Refer to ["AVM State Model"](./state-model) for more details on the absence of "external registers" in the AVM.

## Control Flow Unit
Processes updates to the program counter and call pointer to ensure that execution proceeds properly from one instruction to the next.

### Program Counter
The Control Flow Unit's **program counter** is an index into the bytecode of the current call's contract. For most instructions, the Control Flow Unit will simply increment the program counter. Certain instructions (like `JUMP`) decode into control flow sub-operations (like `PCSTORE`). The Control Flow Unit processes such instructions to update the program counter.

### Call Pointer
A **contract call pointer** uniquely identifies a contract call among all contract calls processed by the current circuit. The Control Flow Unit tracks the currently active call pointer and the next available one. When a nested contract call is encountered, it assigns it the next available call pointer (`callPointer = nextCallPointer++`) and increments that next pointer value. It then sets the program counter to 0.

A request's initial contract call is assigned call pointer of `1`. The first nested contract call encountered during execution is assigned call pointer of `2`. The Control Flow Unit assigns call pointers based on execution order.

There is certain information that must be tracked by the VM circuit on a per-call basis. For example, each call will correspond to the execution of a different contract's bytecode, and each call will access call-specific memory. As a per-call unique identifier, the contract call pointer enables bytecode and memory lookups, among other things, on a per-call basis.

#### "Input" and "output" call pointers
It is important to note that the initial contract call's pointer is `1`, not `0`. The zero call pointer is a special case known as the "input" call pointer.

As expanded on later, the VM circuit memory table has a separate section for each call pointer. The memory table section for the **input call pointer** is reserved for the initial call's `calldata`. This will be expanded on later.

### Internal Call Stack
**TODO**
### Nested contract calls
**TODO**
#### Initializing nested call context
**TODO**
#### Snapshotting and restoring context
**TODO**

## Memory Controller
The VM circuit's **Memory Controller** processes loads and stores between intermediate registers and memory.

### Memory Sub-operations
When decoded, instructions that operate on memory map to some Memory Controller sub-operations. A memory read maps to a `LOAD` sub-operation which loads a word from memory into an intermediate register. The memory offset for this sub-operation is generally specified by an instruction argument. Similarly, a memory write maps to a `STORE` sub-operation which stores a word from an intermediate register to memory.

### User Memory
**TODO**

### Calldata
**TODO**
#### Initial call's calldata
Any lookup into calldata from a request's initial contract call must retrieve a value matching the `calldata` public inputs column. To enforce this, an equivalence check is applied between the `calldata` column and the memory trace for user memory accesses that use "input call pointer".

## Storage Controller
**TODO**

## Side-effect Accumulator
**TODO**

## Chiplets
**TODO**

## Circuit I/O
### How do "Public Inputs" work in the AVM circuit?
ZK circuit proof systems generally define some mechanism for "public inputs" for which witness values must be communicated in full to a verifier. The AVM proof system defines its own mechanism for public inputs in which it flags certain trace columns as "public input columns". Any public input columns must be communicated in full to a verifier.

### AVM public inputs structure
The VM circuit's I/O (`AvmPublicInputs`) is defined below:
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
> The `ExecutionEnvironment` structure is defined in [the AVM's high level specification](./avm.md).

### AVM public input columns
The `AvmPublicInputs` structure is represented in the VM trace via the following public input columns:
1. `initialEnvironment` has a dedicated column and is used to initialize the initial call's `AvmContext.ExecutionEnvironment` and `AvmContext.MachineState`
1. `calldata` has its own dedicated public input column
1. `sideEffects: AvmSideEffects`
    - This represents the final `AccruedSubstate` of the initial contract call
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
    - This is derived from the _final_ `AvmContext.MachineState` of the initial contract call

# AVM Circuit
The AVM circuit's purpose is to prove execution of a sequence of instructions for a public execution request. Regardless of whether execution succeeds or reverts, the circuit always generates a valid proof of execution.

## Introduction

### Circuit Architecture Outline
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

Each contract's public bytecode is committed to during contract deployment. As part of the AVM circuit verification algorithm, the bytecode vector (as a concatenation of all relevant contract bytecodes) is verified against the corresponding bytecode commitments. This is expanded on in ["Bytecode Validation Circuit"](./bytecode-validation-circuit). While the AVM circuit enforces that the correct instructions are executed according to its bytecode table, the verifier checks that bytecode table against the previously validated bytecode commitments.

## Instruction Controller
The Instruction Controller's responsibilities include instruction fetching and decoding.

### Instruction fetching
The Instruction Controller's **instruction fetch** mechanism makes use of the bytecode table to determine which instruction to execute based on the call pointer and program counter. Each instruction fetch corresponds to a circuit lookup to enforce that the correct instruction is processed for a given contract and program counter.

The combination of the instruction fetch circuitry, the bytecode table, and the ["Bytecode Validation Circuit"](./bytecode-validation-circuit) ensure that VM circuit processes the proper sequence of instructions.

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

> Refer to ["AVM State Model"](./memory-model) for more details on the absence of "external registers" in the AVM.

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

This table tracks all memory `Read` or `Write` operations. As introduced in the ["Memory State Model"](./memory-model.md), a memory cell is indexed by a 32-bit unsigned integer (`u32`), i.e., the memory capacity is of $2^{32}$ words. Each word is associated with a tag defining its type (`uninitialized`, `u8`, `u16`, `u32`, `u64`, `u128`, `field`). At the beginning of a new call, each memory cell is of type `uninitialized` and has value 0.

The main property enforcement of this table concerns read/write consistency of every memory cell. This must ensure:

- Each initial read on a memory cell must have value 0 (`uninitialized` is compatible with any other type).
- Each read on a memory cell must have the same value and the same tag as those set by the last write on this memory cell.

In addition, this table ensures that the instruction tag corresponding to a memory operation is the same as the memory cell tag. The instruction tag is passed to the memory controller and added to the pertaining row(s) of this table. Note that this is common for an instruction to generate several memory operations and thus several rows in this table.

The user memory table essentially consists of the following colums:

- `CALL_PTR`: call pointer uniquely identifying the contract call
- `CLK`: clock value of the memory operation
- `ADDR`: address (type `u32`) pertaining to the memory operation
- `VAL`: value which is read (resp. written) from (resp. to) the memory address
- `TAG`: tag associated to this memory address
- `IN_TAG`: tag of the pertaining instruction
- `RW`: boolean indicating whether memory operation is read or write
- `TAG_ERR`: boolean set to true if there is a mismatch between `TAG` and `IN_TAG`

To facilitate consistency check, the rows are sorted by `CALL_PTR` then by `ADDR` and then by `CLK` in ascending (arrow of time) order. Any (non-initial) read operation row is constrained to have the same `VAL` and `TAG` than the previous row. A write operation does not need to be constrained.

The tag consistency check can be performed within every row (order of rows does not matter).

Note that `CLK` also plays the role of a foreign key to point to the corresponding sub-operation. This is crucial to enforce consistency of copied values between the sub-operations and memory table.

### Calldata
**TODO**
#### Initial call's calldata
Any lookup into calldata from a request's initial contract call must retrieve a value matching the `calldata` public inputs column. To enforce this, an equivalence check is applied between the `calldata` column and the memory trace for user memory accesses that use "input call pointer".

## Storage Controller
**TODO**

## Side-effect Accumulator
**TODO**

## Chiplets

A chiplet is essentially a sub-circuit for performing specialized sub-operations. A chiplet is defined as a dedicated table (a set of columns and relations) in the AVM circuit that is activated when the relevant sub-operation is used. The main rationale behind the use of chiplets is to offload specialized computations to a region of the circuit _outside the main operations trace and instruction controller_ where the computations might require many rows and/or additional dedicated columns. In addition, this approach offers strong modularity for the operations implemented as chiplets.

The interaction between a chiplet and the instruction controller follows the following pattern:

1. The **inputs** of a chiplet sub-operation are loaded to the respective intermediate registers (usually $I_a$, $I_b$).
2. The dedicated chiplet fetches/copies the content of the intermediate registers from the **operations trace** in its own table and executes the operation.
3. The output of the operation is copied back to the **operations trace** in a specific register (usually $I_c$). This register is usually involved in a write memory sub-operation.

In addition to the mentioned inputs and output, some other relevant information such as the instruction tag might be copied as well to the chiplet.

In the circuit, the transmission of the input/output between the **chiplet trace** and the **operations trace** are performed through lookup or permutation constraints, i.e., they ensure that all relevant intermediate registers have the same values between both traces. The unique key of this mapping is `CLK` which is basically used as a "DB foreign key" from the **chiplet trace** pointing to corresponding entry in the **operations trace**.

The planned chiplets for the AVM are:

- **ALU**: Arithmetic and bitwise operations such as addition, multiplication, XOR, etc...
- **Type Converter**: Dedicated to casting words between different types and/or type constraints.
- **Gadgets:** Relevant cryptographic operations or other computationally intensive operations. There will likely be multiple chiplets of this category, including `Poseidon2Permutation`, `Keccakf1600`, and `ECADD`.

## Circuit I/O
### How do "Public Inputs" work in the AVM circuit?
ZK circuit proof systems generally define some mechanism for "public inputs" for which witness values must be communicated in full to a verifier. The AVM proof system defines its own mechanism for public inputs in which it flags certain trace columns as "public input columns". Any public input columns must be communicated in full to a verifier.

### AVM public inputs structure
The VM circuit's I/O (`AvmPublicInputs`) is defined below:

```
AvmSessionInputs {
    // Initializes Execution Environment
    feePerL1Gas: field,
    feePerL2Gas: field,
    feePerDaGas: field,
    globals: PublicGlobalVariables,
    address: AztecAddress,
    storageAddress: AztecAddress,
    sender: AztecAddress,
    contractCallDepth: field,
    isStaticCall: boolean,
    isDelegateCall: boolean,
    // Initializes Machine State
    l1GasLeft: field,
    l2GasLeft: field,
    daGasLeft: field,
}
AvmSessionResults {
    l1GasLeft: field,
    l2GasLeft: field,
    daGasLeft: field,
    reverted: boolean,
}
AvmSessionPublicInputs {
    sessionInputs: AvmSessionInputs,
    calldata: [field; MAX_CALLDATA_LENGTH],
    worldStateAccessTrace: WorldStateAccessTrace,
    accruedSubstate: AccruedSubstate,
    sessionResults: AvmSessionResults,
}
```

> The `WorldStateAccessTrace` and `AccruedSubstate` types are defined in ["State"](./state). Their vectors are assigned constant/maximum lengths when used as circuit inputs.

### AVM public input columns
The `AvmPublicInputs` structure is represented in the VM trace via the following public input columns:
1. `sessionInputs` has a dedicated column and is used to initialize the initial call's `AvmContext.ExecutionEnvironment` and `AvmContext.MachineState`.
1. `calldata` occupies its own public input column as it is handled differently from the rest of the `ExecutionEnvironment`. It is used to initialize the initial call's `AvmContext.ExecutionEnvironment.calldata`.
    - Equivalence is enforced between this `calldata` column and the "input call pointer"'s memory. Through this mechanism, the initial call's `calldata` is placed in a region memory that can be referenced via the `CALLDATACOPY` instruction from within the initial call.
1. `worldStateAccessTrace` is a trace of all world state accesses. Each of its component vectors has a dedicated set of public input columns (a sub-table). An instruction that reads or writes world state must match a trace entry. The [trace type definition in the "State" section] lists, for each trace vector, the instruction that populate its entries.
1. `accruedSubstate` contains the final `AccruedSubstate`.
    - This includes the accrued substate of all _unreverted_ sub-contexts.
    - Reverted substate is not present in the Circuit I/O as it does not require further validation/processing by downstream circuits.
1. `sessionResults` has a dedicated column and represents the core "results" of the AVM session processed by this circuit (remaining gas, reverted).

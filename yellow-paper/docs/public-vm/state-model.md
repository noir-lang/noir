# The Aztec VM State Model

The goal of this note is to describe the VM state model and to specify "internal" VM abstractions that can be mapped to circuit designs.

# A memory-only state model

The AVM possesses three distinct data regions, accessed via distinct VM instructions: memory, calldata and returndata

![](./gen/images/state-model/memory.png)

All data regions are linear blocks of memory where each memory cell stores a finite field element.

#### Main Memory

Main memory stores the internal state of the current program being executed.
Can be written to as well as read.

The main memory region stores _type tags_ alongside data values. [Type tags are explained further on in this document](#type tags).

#### Calldata

Read-only data structure that stores the input data when executing a public function.

#### Returndata

When a function is called from within the public VM, the return parameters of the called function are present in returndata.

### Registers (and their absence in the AVM)

The AVM does not have external registers. i.e. a register that holds a persistent value that can be operated on from one opcode to the next.

For example, in the x86 architecture, there exist 8 registers (%rax, %rbx etc). Instructions can operate either directly on register values (e.g. `add %rax %rbx`) or on values in memory that the register values point to (e.g. `add (%rax) (%rbx)`).

> The AVM does not support registers as this would require each register to exist as a column in the VM execution trace. "registers" can be implemented as a higher-level abstraction by a compiler producing AVM bytecode, by reserving fixed regions of memory to represent registers.

### Memory addressing mode

In the AVM, an instruction operand `X` can refer to one of three quantities:

1. A literal value `X`
2. A memory address `M[X]`
3. An indirect memory address `M[M[X]]`

Indirect memory addressing is required in order to support read/writes into dynamically-sized data structures (the address parameter `X` is part of the program bytecode, which is insufficient to describe the location in memory of a dynamically-sized data structure).

Memory addresses must be tagged to be a `u32` type.

# Tagged memory

We define a `tag` to refer to the potential maximum value of a cell of main memory. The following tags are supported:

| tag value | maximum memory cell value |
| --------- | ------------------------- |
| 0         | 0                         |
| 1         | $2^8 - 1$                 |
| 2         | $2^{16} - 1$              |
| 3         | $2^{32} - 1$              |
| 4         | $2^{64} - 1$              |
| 5         | $2^{128} - 1$             |
| 6         | $p - 1$                   |

Note: $p$ describes the modulus of the finite field that the AVM circuit is defined over (i.e. number of points on the BN254 curve).

The purpose of a tag is to inform the VM of the maximum possible length of an operand value that has been loaded from memory.

Multiple AVM instructions explicitly operate over range-constrained input parameters (e.g. ADD32). The maximum allowable value for an instruction's input parameters is defined via an _instruction tag_. Two potential scenarios result:

1. A VM instruction's tag value matches the input parameter tag values
2. A VM instruction's tag value does not match the input parameter tag values

If case 2 is triggered, an error flag is raised.

---

### Writing into memory

It is required that all VM instructions that write into main memory explicitly define the tag of the output value and ensure the value is appropriately constrained to be consistent with the assigned tag.

---

### MOV and tag conversions

The MOV instruction copies data from between memory cell, perserving tags.

The only VM instruction that can be used to cast between tags is CAST. There are 2 modes to MOV:

1. The destination tag describes a maximum value that is _less than_ the source tag
2. The destination tag describes a maximum value that is _greater than or equal to_ the source tag

For Case 1, range constraints must be applied to ensure the destination value is consistent with the source value after tag truncations have been applied.

Case 2 is trivial as no additional consistency checks must be performed between soruce and destination values.

---

### Calldata/returndata and tag conversions

All elements in calldata/returndata are implicitly tagged as field elements (i.e. maximum value is $p - 1$). To perform a tag conversion, calldata/returndata must be copied into main memory, followed by an appropriate MOV instruction.

## VM threat model, security requirements

TODO: move this somewhere else, doesn't quite fit.

An honest Prover must always be able to construct a satsisfiable proof for an AVM program, even if the program throws an error.
This implies constraints produced by the AVM **must** be satisfiable.

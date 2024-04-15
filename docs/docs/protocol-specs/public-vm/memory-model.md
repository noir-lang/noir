# Memory Model

This section describes the AVM memory model, and in particular specifies "internal" VM abstractions that can be mapped to the VM's circuit architecture.

## A memory-only state model

The AVM possesses three distinct data regions, accessed via distinct VM instructions: memory, calldata and returndata

![](/img/protocol-specs/public-vm/memory.png)

All data regions are linear blocks of memory where each memory cell stores a finite field element.

#### Main Memory

Main memory stores the internal state of the current program being executed.
Can be written to as well as read.

The main memory region stores [_type tags_](#types-and-tagged-memory) alongside data values.

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

## Types and Tagged Memory

### Terminology/legend

- `M[X]`: main memory cell at offset `X`
- `tag`: a value referring to a memory cell's type (its maximum potential value)
- `T[X]`: the tag associated with memory cell at offset `X`
- `inTag`: an instruction's tag to check input operands against. Present for many but not all instructions.
- `dstTag`: the target type of a `CAST` instruction, also used to tag the destination memory cell
- `ADD<X>`: shorthand for an `ADD` instruction with `inTag = X`
- `ADD<X> aOffset bOffset dstOffset`: an full `ADD` instruction with `inTag = X`. See [here](./instruction-set#isa-section-add) for more details.
- `CAST<X>`: a `CAST` instruction with `dstTag`: `X`. `CAST` is the only instruction with a `dstTag`. See [here](./instruction-set#isa-section-cast) for more details.

### Tags and tagged memory

A `tag` refers to the maximum potential value of a cell of main memory. The following tags are supported:

| tag value | maximum memory cell value | shorthand     |
| --------- | ------------------------- | ------------- |
| 0         | 0                         | uninitialized |
| 1         | $2^8 - 1$                 | `u8`          |
| 2         | $2^{16} - 1$              | `u16`         |
| 3         | $2^{32} - 1$              | `u32`         |
| 4         | $2^{64} - 1$              | `u64`         |
| 5         | $2^{128} - 1$             | `u128`        |
| 6         | $p - 1$                   | `field`       |
| 7         | reserved                  | reserved      |

> Note: $p$ describes the modulus of the finite field that the AVM circuit is defined over (i.e. number of points on the BN254 curve).
> Note: `u32` is used for offsets into the VM's 32-bit addressable main memory

The purpose of a tag is to inform the VM of the maximum possible length of an operand value that has been loaded from memory.

#### Checking input operand tags

Many AVM instructions explicitly operate over range-constrained input parameters (e.g. `ADD<inTag>`). The maximum allowable value for an instruction's input parameters is defined via an `inTag` (instruction/input tag). Two potential scenarios result:

1. A VM instruction's tag value matches the input parameter tag values
2. A VM instruction's tag value does _not_ match the input parameter tag values

If case 2 is triggered, an error flag is raised and the current call's execution reverts.

#### Writing into memory

It is required that all VM instructions that write into main memory explicitly define the tag of the destination value and ensure the value is appropriately constrained to be consistent with the assigned tag. You can see an instruction's "**Tag updates**" in its section of the instruction set document (see [here for `ADD`](./instruction-set#isa-section-add) and [here for `CAST`](./instruction-set#isa-section-cast)).

#### Standard tagging example: `ADD`

```
# ADD<u32> aOffset bOffset dstOffset
assert T[aOffset] == T[bOffset] == u32 // check inputs against inTag, revert on mismatch
T[dstOffset] = u32                     // tag destination with inTag
M[dstOffset] = M[aOffset] + M[bOffset] // perform the addition
```

#### `MOV` and tag preservation

The `MOV` instruction copies data from one memory cell to another, preserving tags. In other words, the destination cell's tag will adopt the value of the source:

```
# MOV srcOffset dstOffset
T[dstOffset] = T[srcOffset] // preserve tag
M[dstOffset] = M[srcOffset] // perform the move
```

Note that `MOV` does not have an `inTag` and therefore does not need to make any assertions regarding the source memory cell's type.

#### `CAST` and tag conversions

The only VM instruction that can be used to cast between tags is `CAST`. Two potential scenarios result:

1. The destination tag describes a maximum value that is _less than_ the source tag
2. The destination tag describes a maximum value that is _greater than or equal to_ the source tag

For Case 1, range constraints must be applied to ensure the destination value is consistent with the source value after tag truncations have been applied.

Case 2 is trivial as no additional consistency checks must be performed between source and destination values.

```
# CAST<u64> srcOffset dstOffset
T[dstOffset] = u64                         // tag destination with dstTag
M[dstOffset] = cast<to: u64>(M[srcOffset]) // perform cast
```

#### Indirect `MOV` and extra tag checks

A `MOV` instruction may flag its source and/or destination offsets as "indirect". An indirect memory access performs `M[M[offset]]` instead of the standard `M[offset]`. Memory offsets must be `u32`s since main memory is a 32-bit addressable space, and so indirect memory accesses include additional checks.

Additional checks for a `MOV` with an indirect source offset:

```
# MOV srcOffset dstOffset      // with indirect source
assert T[srcOffset] == u32     // enforce that `M[srcOffset]` is itself a valid memory offset
T[dstOffset] = T[T[srcOffset]] // tag destination to match indirect source tag
M[dstOffset] = M[M[srcOffset]] // perform move from indirect source
```

Additional checks for a `MOV` with an indirect destination offset:

```
# MOV srcOffset dstOffset      // with indirect destination
assert T[dstOffset] == u32     // enforce that `M[dstOffset]` is itself a valid memory offset
T[T[dstOffset]] = T[srcOffset] // tag indirect destination to match source tag
M[M[dstOffset]] = M[srcOffset] // perform move to indirect destination
```

Additional checks for a `MOV` with both indirect source and destination offsets:

```
# MOV srcOffset dstOffset                  // with indirect source and destination
assert T[srcOffset] == T[dstOffset] == u32 // enforce that `M[*Offset]` are valid memory offsets
T[T[dstOffset]] = T[T[srcOffset]]          // tag indirect destination to match indirect source tag
M[M[dstOffset]] = M[M[srcOffset]]          // perform move to indirect destination
```

#### Calldata/returndata and tag conversions

All elements in calldata/returndata are implicitly tagged as field elements (i.e. maximum value is $p - 1$). To perform a tag conversion, calldata/returndata must be copied into main memory (via [`CALLDATACOPY`](./instruction-set#isa-section-calldatacopy) or [`RETURN`'s `offset` and `size`](./instruction-set#isa-section-return)), followed by an appropriate `CAST` instruction.

```
# Copy calldata to memory and cast a word to u64
CALLDATACOPY cdOffset size offsetA // copy calldata to memory at offsetA
CAST<u64> offsetA dstOffset        // cast first copied word to a u64
```

This would perform the following:

```
# CALLDATACOPY cdOffset size offsetA
T[offsetA:offsetA+size] = field                            // CALLDATACOPY assigns the field tag
M[offsetA:offsetA+size] = calldata[cdOffset:cdOffset+size] // copy calldata to memory
# CAST<u64> offsetA dstOffset
T[offsetA] = u64                                           // CAST assigns a new tag
M[dstOffset] = cast<u64>(offsetA)                          // perform the cast operation
```

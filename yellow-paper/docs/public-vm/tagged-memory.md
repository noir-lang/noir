# Tagged Memory - An instruction-set centric explanation

## Explanation of Tagged Memory
Every word in memory will have an associated `type-tag` (unset, u8, u16, u32, u64, u128, field). For memory address `a`, we refer to the corresponding memory word's `type-tag` as `T[a]`.

Every instruction will be flagged with an `op-type` in bytecode (u8, u16, u32, u64, u128, field).

If an instruction uses a "source operand" as a memory location (e.g. `z = M[s0] + y`), the VM first retrieves the `type-tag` referenced by the operand (`T[s0]`) and enforces that it matches `op-type`. The VM enforces this for all source operands used for direct memory reads.

If an instruction uses a "dest operand" as a memory location (e.g. `M[d0] = x + y`), when the VM assigns a word to that memory location, it also assigns the corresponding `type-tag` (`T[d0] = op-type`). The VM performs this tag assignment for all dest operands used for direct memory writes.

**If an instruction fails any of its operand type-tag-checks, the current call's execution reverts!**

### `ADD<32>` example
`ADD<32>` is an `ADD` instruction with `op-type` u32. As elaborated on later, an `ADD` performs `M[d0] = M[s0] + M[s1]`. In this case, both `s0` and `s1` are "source operands" used for direct memory reads to retrieve inputs to an addition. So, the VM enforces the `op-type(u32) == T[s0] == T[s1]`. `d0` here is a "dest operand" used for a direct memory write to store the output of the addition. So, the VM tags memory location `d0` with `type-tag` of u32: `T[d0] = op-type(u32)`.

Here is a summary of what is happening for `ADD<32>`:
```
assert T[s0] == u32  // enforce that source memory locations' type-tags == op-type
assert T[s1] == u32
T[d0] = u32  // tag destination memory location as op-type
M[d0] = M[s0] + M[s1]
```


### Type tags and `CASTs`

`CAST` is different from other instructions in that it will be flagged with an additional `dest-type`. So, a `CAST` will assign `dest-type` (instead of `op-type`) to the memory location specified by its "dest operand" `d0`. `CAST<32, 64>` enforces that `T[s0]` matches u32 (the `op-type`) and assigns `T[d0] = u64` (the `dest-type`).

Here is a summary of what is happening for a `CAST<32, 64>`:
```
assert T[s0] == u32  // enforce that source memory location's type-tags == op-type
T[d0] = u64  // tag destination memory location as dest-type
M[d0] = M[s0]
```

### Type tags and indirect `MOVs`
A basic `MOV` instruction performs direct memory accesses and operates in the same as a simple `ADD` instruction as outlined above. A simple `MOV<64>` would do:
```
assert T[s0] == u64  // enforce that source memory location's type-tag == op-type
T[d0] = u64  // tag destination memory location with op-type
M[d0] = M[s0]
```

Consider a `MOV<64, s0-indirect>`, which treats s0 as an indirect memory pointer to perform `M[d0] = M[M[s0]]`. Here, the VM first needs to enforce that `M[s0]` is a valid memory address (has type u32), and it then needs to perform the standard check that resulting word has type matching `op-type`:
```
assert T[s0] == u32  // enforce that the direct source memory location contains a valid address (type-tag == u32)
assert T[M[s0]] == u64  // enforce that the indirect source memory location's type-tag == op-type
T[d0] = u64  // tag destination memory location with op-type
M[d0] = M[M[s0]]
```

Similarly, a `MOV<64, d0-indirect>` treats d0 as an indirect memory pointer to perform `M[M[d0]] = M[s0]`, and here the VM first needs to enforce that `M[d0]` is a valid memory address (has type u32) before assigning the destination location its type tag:
```
assert T[s0] == u64  // enforce that source memory location's type-tag == op-type
assert T[d0] == u32  // enforce that the direct destination memory location contains a valid address (type-tag == u32)
T[M[d0]] = u64  // tag indirect destination memory location with op-type
M[M[d0]] = M[s0]
```


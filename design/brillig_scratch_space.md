# Brillig scratch space

This records why Brillig's *scratch space* exists, what is allowed to use it, and the invariants
that keep those uses from clobbering each other. The implementation lives in
`compiler/noirc_evaluator/src/brillig/brillig_ir/registers.rs` (the `ScratchSpace` allocator) and
`brillig_ir.rs` (the `ReservedRegisters` helpers).

# Memory layout

A Brillig program's memory is partitioned into fixed and dynamic regions, in this order:

```
{reserved} {scratch} {globals} {entry point (call data + return data)} {stack} {heap}
```

Scratch space sits immediately after the reserved registers. It is **direct-addressed**: a scratch
slot is a fixed absolute memory address (`MemoryAddress::direct`), the same in every call frame.
It begins at `ScratchSpace::start()` (`= ReservedRegisters::len()`, currently address `@3`) and
spans `max_scratch_space` slots.

Its size is a compile-time layout parameter, `MAX_SCRATCH_SPACE = 64` by default, overridable via
the hidden `--max-scratch-space` flag down to `MIN_SCRATCH_SPACE = 2`.

# Why it exists

Calling a Brillig *procedure* (a shared, pre-compiled routine such as `array_copy` or
`prepare_vector_insert`) must not require the caller to save and restore its stack frame. The stack
is relative-addressed, so a naive call would have to dump the caller's live registers to the heap
and reload them afterwards.

Scratch space avoids this. Because it is direct-addressed and lives outside every stack frame, the
caller and callee share the same absolute scratch addresses without any save/restore: the caller
writes arguments into the low scratch slots, calls the procedure, and reads the returns back out of
those same slots.

# Who uses scratch space

There are three distinct consumers. The first two are the same underlying mechanism — a procedure's
register allocator *is* the scratch space; the third is unrelated code borrowing a few fixed slots.

1. **Procedure argument/return passing.** The caller (running in a `Stack` context) lays down `N`
   consecutive slots with `make_scratch_registers::<N>()`, writes arguments, calls the procedure,
   and reads returns. The callee reserves the matching slots with `allocate_scratch_registers::<N>()`.

2. **Procedure-local temporaries.** A procedure body is compiled with `BrilligContext<F, ScratchSpace>`
   — i.e. its register allocator is the scratch space itself. Therefore *every* `allocate_register`
   / `allocate_single_addr_*` inside a procedure allocates a scratch slot, not a stack slot. A
   procedure never touches the stack, so scratch is its only working memory. This — not the argument
   count — is what drives peak scratch demand: the arguments occupy the low slots, and the procedure's
   internal working set is allocated above them. (`MIN_SCRATCH_SPACE = 2` exists because even a
   minimal program's `CheckMaxStackDepth` procedure allocates one scratch temporary.)

3. **The register-spilling machinery** (in ordinary `Stack`-context block codegen) borrows the first
   three scratch slots as fixed transient registers, without going through the allocator:
   - `ReservedRegisters::spill_scratch()` → `@3`, `@4`: materialize a spill-slot address
     (`spill_base + offset`) before a load/store.
   - `ReservedRegisters::spill_conditional_value()` → `@5`: hold a value across the
     `load → cmov → store` sequence of a conditional spill store.

# Safety guarantees

These invariants are what make the three uses coexist safely. Breaking any of them silently
corrupts memory rather than failing loudly, so they are worth stating explicitly.

- **Block codegen never allocates from scratch space for user values.** Ordinary (non-procedure)
  codegen allocates everything on the stack. This is exactly what lets the spilling machinery treat
  `@3`/`@4`/`@5` as always-available fixed scratch registers — nothing else in a `Stack` context can
  be holding a value there.

- **`@3`/`@4` and `@5` are disjoint.** The conditional-store value in `@5` is held across an inner
  load/store whose address computation reuses `@3`/`@4`. Overlapping them would clobber the value
  mid-sequence.

- **Procedures cannot call procedures.** A procedure context is constructed with
  `can_call_procedures = false`. Call sites that would emit a procedure call (e.g. array/vector copy
  in `codegen_memory.rs`) fall back to inline codegen when this flag is false. This guarantees scratch
  "arenas" never nest: at most one procedure's argument/temporary layout is live at a time, so
  argument slots and a callee's temporaries can never collide across a nested call.

- **Direct addressing makes scratch call-invariant.** Because scratch slots are absolute addresses,
  their contents are unaffected by stack-pointer changes across calls — the property the whole
  argument-passing scheme relies on.

- **Bounds are enforced lazily, not globally.** The `ScratchSpace` allocator asserts every allocation
  stays within `[start(), start() + max_scratch_space)` ("Scratch space too deep"). There is no
  static computation of the true maximum scratch demand across all procedures; in practice the peak
  is small (≈7 slots for `prepare_vector_push` / `prepare_vector_insert` with copy-counting enabled)
  and always fits well under the default 64. If a future procedure's working set grows past the
  configured `max_scratch_space`, this assertion is what will catch it.

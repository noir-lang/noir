# Brillig scratch space

This records why Brillig's *scratch space* exists, what is allowed to use it, and the invariants
that keep those uses from clobbering each other. The implementation lives in
[`registers.rs`][registers] (the `ScratchSpace` allocator) and [`brillig_ir.rs`][brillig_ir] (the
`ReservedRegisters` helpers).

# Memory layout

A Brillig program's memory is partitioned into fixed and dynamic regions, in this order (see the
module docs at the top of [`registers.rs`][registers]):

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

Calling a Brillig *procedure* (a shared, pre-compiled routine such as [`array_copy`][array_copy] or
[`prepare_vector_insert`][prepare_vector_insert]) must not require the caller to save and restore its
stack frame. The stack is relative-addressed, so a naive call would have to dump the caller's live
registers to the heap and reload them afterwards.

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
   Both helpers are defined in [`registers.rs`][registers]; each routine under
   [`procedures/`][procedures] uses them (see [`array_copy`][array_copy] for a minimal example).

2. **Procedure-local temporaries.** A procedure body is compiled with `BrilligContext<F, ScratchSpace>`
   — i.e. its register allocator is the scratch space itself. Therefore *every* `allocate_register`
   / `allocate_single_addr_*` inside a procedure allocates a scratch slot, not a stack slot. A
   procedure never touches the stack, so scratch is its only working memory. This — not the argument
   count — is what drives peak scratch demand: the arguments occupy the low slots, and the procedure's
   internal working set is allocated above them. (`MIN_SCRATCH_SPACE = 2` exists because even a
   minimal program's [`CheckMaxStackDepth`][check_max_stack_depth] procedure allocates one scratch
   temporary.)

3. **The register-spilling machinery** (in ordinary `Stack`-context block codegen, see
   [`brillig_block.rs`][brillig_block]) borrows the first three scratch slots as fixed transient
   registers, without going through the allocator. The helpers are defined in
   [`brillig_ir.rs`][brillig_ir]:
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
  be holding a value there. *Enforced by* `stack_allocations_are_relative_never_scratch` in
  [`registers.rs`][registers], which checks the `Stack` allocator only ever yields `Relative`
  addresses.

- **`@3`/`@4` and `@5` are disjoint.** The conditional-store value in `@5` is held across an inner
  load/store whose address computation reuses `@3`/`@4`. Overlapping them would clobber the value
  mid-sequence. *Enforced by* `spill_scratch_slots_are_distinct_and_in_scratch_space` in
  [`brillig_ir.rs`][brillig_ir].

- **Procedures cannot call procedures.** A procedure context is constructed with
  `can_call_procedures = false` (see `new_for_procedure` in [`brillig_ir.rs`][brillig_ir]). Call
  sites that would emit a procedure call (e.g. array/vector copy in
  [`codegen_memory.rs`][codegen_memory]) fall back to inline codegen when this flag is false. This
  guarantees scratch "arenas" never nest: at most one procedure's argument/temporary layout is live
  at a time, so argument slots and a callee's temporaries can never collide across a nested call.
  *Enforced by* `procedures_do_not_call_procedures` in [`procedures/mod.rs`][procedures], which
  asserts no procedure's bytecode contains a `Call` opcode.

- **Direct addressing makes scratch call-invariant.** Because scratch slots are absolute addresses,
  their contents are unaffected by stack-pointer changes across calls — the property the whole
  argument-passing scheme relies on. This is a property of the Brillig VM rather than a compiler
  invariant, so it is covered only indirectly, by execution tests that call procedures and check
  results.

- **Scratch is sized by `max`, not `sum` — spilling adds nothing on top of the procedure peak.** The
  spill slots `@3`/`@4`/`@5` and the up-to-18 slots a procedure uses share the same physical region,
  but never overlap in time, so the region only has to fit the larger of the two. Spilling is a
  `Stack`-context mechanism (its `SpillManager` lives on `FunctionContext`); procedures compile in a
  `ScratchSpace` context and never spill. A regular function keeps its values on the stack, and its
  only scratch use is the transient `@3`/`@4`/`@5`, which are consumed within a single spill
  load/store sequence and are never held across a procedure call — the spilled *values* themselves go
  to a heap region (`spill_base = free_memory_pointer`, bumped in the prologue), not to scratch. So
  when a caller later writes procedure arguments into those same low slots, nothing live is there to
  clobber. The whole-program scratch requirement is therefore `max(peak procedure demand,
  spill_slots) = max(18, 3) = 18`, not their sum. (The `MIN_SCRATCH_SPACE = 2` floor is sized by the
  smallest procedure — `CheckMaxStackDepth`, which needs two scratch slots — not by spilling. The
  spilling helpers actually occupy three fixed slots, `@3`–`@5`.)

- **Bounds are enforced lazily, not globally.** The `ScratchSpace` allocator asserts every allocation
  stays within `[start(), start() + max_scratch_space)` ("Scratch space too deep"). There is no
  static computation of the true maximum scratch demand across all procedures; the compiler does not
  prove a tight bound, it just traps any allocation that would overflow the configured region.

  The peak is driven by procedure-local temporaries (point 2 above), *not* by argument counts. The
  argument/return handshake for the widest procedures is only 6–7 slots, but a procedure also holds
  its live locals in scratch, so the real high-water mark is larger. Measured across the current
  procedures it is **18 scratch slots** (`@3`–`@20`, driven by [`vector_remove`][vector_remove]) —
  still comfortably under the default 64. The test `peak_scratch_demand_across_procedures` in
  [`procedures/mod.rs`][procedures] pins this figure by reading it straight from the generated
  bytecode; if a procedure change moves the peak, that test fails and both it and this number must be
  updated. If a working set ever grows past the configured `max_scratch_space`, the allocator
  assertion above is the backstop that catches it — exercised by
  `scratch_allocation_beyond_capacity_panics` in [`registers.rs`][registers].

# Relevant files

- [`brillig_ir/registers.rs`][registers] — memory-layout module docs, `LayoutConfig`, the
  `ScratchSpace` allocator, `MIN_SCRATCH_SPACE` / `MAX_SCRATCH_SPACE`, and the
  `make_scratch_registers` / `allocate_scratch_registers` helpers.
- [`brillig_ir.rs`][brillig_ir] — `ReservedRegisters` (`spill_scratch`, `spill_conditional_value`)
  and `new_for_procedure` (`can_call_procedures = false`).
- [`brillig_gen/brillig_block.rs`][brillig_block] — the register-spilling machinery that borrows
  `@3`/`@4`/`@5`.
- [`brillig_ir/codegen_memory.rs`][codegen_memory] — a procedure-call site with an inline fallback
  gated on `can_call_procedures`.
- [`brillig_ir/entry_point.rs`][entry_point] — entry-point codegen that lays out the regions after
  scratch (globals, calldata, stack).
- [`brillig_ir/procedures/`][procedures] — the procedures themselves, e.g.
  [`array_copy`][array_copy], [`prepare_vector_insert`][prepare_vector_insert],
  [`prepare_vector_push`][prepare_vector_push], [`vector_remove`][vector_remove], and
  [`check_max_stack_depth`][check_max_stack_depth]. The peak-scratch guard test
  (`peak_scratch_demand_across_procedures`) also lives here.

[registers]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/registers.rs
[brillig_ir]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir.rs
[brillig_block]: ../../compiler/noirc_evaluator/src/brillig/brillig_gen/brillig_block.rs
[codegen_memory]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/codegen_memory.rs
[entry_point]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/entry_point.rs
[procedures]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures
[array_copy]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/array_copy.rs
[prepare_vector_insert]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/prepare_vector_insert.rs
[prepare_vector_push]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/prepare_vector_push.rs
[vector_remove]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/vector_remove.rs
[check_max_stack_depth]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/check_max_stack_depth.rs

# Brillig scratch space

Records why Brillig has a *scratch space* region, what is allowed to use it, and why its sizing and
safety rules are the way they are. For the mechanics, see [`registers.rs`][registers] (the
`ScratchSpace` allocator) and [`brillig_ir.rs`][brillig_ir] (the `ReservedRegisters` helpers).

# Why it exists

Brillig memory is laid out as `{reserved} {scratch} {globals} {entry point} {stack} {heap}`. Scratch
space is a small, fixed, **direct-addressed** region immediately after the reserved registers.

It exists so that calling a shared *procedure* (a pre-compiled routine such as
[`array_copy`][array_copy]) does not force the caller to save and restore its stack frame. The stack
is relative-addressed, so passing arguments through it would mean dumping the caller's live registers
to the heap and reloading them afterwards. Because scratch slots are absolute addresses that live
outside every stack frame, caller and callee share them directly — the caller writes arguments there,
calls the procedure, and reads results back — with no save/restore.

# Who uses it, and why they don't collide

There are three consumers but only two mechanisms:

1. **Procedures** use scratch as their *entire* working memory: a procedure body is compiled against
   the `ScratchSpace` allocator rather than the stack, so its arguments, returns, and local
   temporaries all live in scratch. A procedure's local working set — not its argument count — is
   what dominates its scratch demand.

2. **Register spilling** (in ordinary `Stack`-context codegen) borrows a few fixed slots
   (`@3`/`@4`/`@5`) as transient registers while it computes spill-slot addresses and conditionally
   stores values. The spilled *values* live on the heap; scratch only ever holds transient address
   arithmetic.

The two never collide because they never run at the same time. Ordinary codegen keeps its values on
the stack and touches the fixed spill slots only transiently; procedures run to completion — and
cannot call other procedures, so their scratch usage never nests — before control returns. The
whole-program scratch requirement is therefore the *maximum* of the two demands, never their sum.

# Sizing and the `MIN_SCRATCH_SPACE` floor

`max_scratch_space` defaults to `MAX_SCRATCH_SPACE = 64`, comfortably above the real peak (currently
18 slots, driven by the widest procedure, [`vector_remove`][vector_remove]). The hidden
`--max-scratch-space` flag exists mainly so tests can vary the layout, and may lower it to
`MIN_SCRATCH_SPACE = 2` — the smallest value the minimal-program procedure
([`CheckMaxStackDepth`][check_max_stack_depth]) needs.

We deliberately keep that floor low rather than raising it to cover every feature. Procedures and
spilling can each need more than two slots, but how much is program-dependent, so instead of
pessimistically sizing the floor for the worst case we let each feature assert its own requirement
when it is actually used — and fail loudly at compile time instead of silently writing outside the
region:

- Procedures allocate *through* the `ScratchSpace` allocator, which asserts (`"Scratch space too
  deep"`) if an allocation would overflow the configured region.
- Spilling uses fixed direct addresses that bypass the allocator, so codegen asserts
  `max_scratch_space >= NUM_SPILL_SCRATCH_SLOTS` whenever a function spills.

# Safety guarantees

The design rests on a few invariants. Most would silently corrupt memory if violated, which is why
several are backed by an assertion or regression test, linked inline.

- **Block codegen never puts user values in scratch.** Ordinary codegen allocates on the stack; this
  is exactly what lets the spill machinery treat `@3`/`@4`/`@5` as always-available. *Enforced by*
  `stack_allocations_are_relative_never_scratch` in [`registers.rs`][registers].

- **The spill slots `@3`/`@4` and `@5` are disjoint.** `@5` holds a value across a sequence that
  reuses `@3`/`@4` for addressing, so overlap would corrupt it. *Enforced by*
  `spill_scratch_slots_are_distinct_and_in_scratch_space` in [`brillig_ir.rs`][brillig_ir].

- **Procedures cannot call procedures** (`can_call_procedures = false`, with call sites falling back
  to inline codegen). This keeps scratch "arenas" from nesting, so one procedure's slots can never
  collide with a callee's. *Enforced by* `procedures_do_not_call_procedures` in
  [`procedures/mod.rs`][procedures].

- **Scratch is call-invariant.** Direct addressing means scratch contents are unaffected by
  stack-pointer changes across calls — the property the whole argument-passing scheme relies on. This
  is a property of the Brillig VM, so it is covered only indirectly, by execution tests.

- **Scratch demand is bounded by assertions, not a global proof** (see the sizing section). The peak
  procedure demand is pinned by `peak_scratch_demand_across_procedures` in
  [`procedures/mod.rs`][procedures]; the overflow assertions are exercised by
  `scratch_allocation_beyond_capacity_panics` in [`registers.rs`][registers] (procedures) and
  `spilling_with_too_small_scratch_space_panics` / `spilling_with_minimum_scratch_space_compiles` in
  [`spill.rs`][spill_tests] (spilling).

# Relevant files

- [`brillig_ir/registers.rs`][registers] — the memory-layout module docs, `LayoutConfig`, the
  `ScratchSpace` allocator, and `MIN_SCRATCH_SPACE` / `MAX_SCRATCH_SPACE`.
- [`brillig_ir.rs`][brillig_ir] — `ReservedRegisters` (the spill scratch slots and
  `NUM_SPILL_SCRATCH_SLOTS`) and `new_for_procedure` (`can_call_procedures = false`).
- [`brillig_gen/brillig_block.rs`][brillig_block] — the register-spilling machinery.
- [`brillig_ir/codegen_memory.rs`][codegen_memory] — a procedure-call site with the inline fallback.
- [`brillig_ir/entry_point.rs`][entry_point] — entry-point codegen, which lays out the regions after
  scratch.
- [`brillig_ir/procedures/`][procedures] — the procedures, e.g. [`array_copy`][array_copy],
  [`prepare_vector_insert`][prepare_vector_insert], [`prepare_vector_push`][prepare_vector_push],
  [`vector_remove`][vector_remove], and [`check_max_stack_depth`][check_max_stack_depth]; the
  `peak_scratch_demand_across_procedures` guard test also lives here.

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
[spill_tests]: ../../compiler/noirc_evaluator/src/brillig/brillig_gen/tests/spill.rs
[check_max_stack_depth]: ../../compiler/noirc_evaluator/src/brillig/brillig_ir/procedures/check_max_stack_depth.rs

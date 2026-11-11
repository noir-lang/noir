use acvm::FieldElement;

use crate::{
    assert_artifact_snapshot,
    brillig::{
        BrilligOptions,
        brillig_gen::tests::{
            execute_brillig_from_ssa_with_options, ssa_to_brillig_artifacts_with_options,
        },
        brillig_ir::{LayoutConfig, ReservedRegisters, registers::MAX_SCRATCH_SPACE},
    },
    ssa::ir::map::Id,
};

/// The prologue reserves the spill region with `<dst> = const u32 N` at bytecode
/// index 1, where `N == SpillManager::max_spill_offset` — the number of slots the
/// frame allocates on every call. Extract `N`.
fn spill_region_size(bytecode: &str) -> u32 {
    let line = bytecode
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("1:") && line.contains("const u32"))
        .expect("prologue spill-region constant at bytecode index 1");
    line.rsplit(' ').next().unwrap().parse().expect("spill-region constant is a u32")
}

/// Regression: a transiently spilled value that is reloaded for one last use and
/// then dies in the same block must return its spill slot to the free list.
///
/// `max_stack_frame_size = 6` leaves 4 usable registers (sp[2..5]), but `main`
/// takes 5 parameters, so codegen spills from the first instruction. As the chain
/// is evaluated, several values are transiently spilled, reloaded into a register
/// for a single use, and then die — all within `b0`. At the moment such a value
/// dies it has a transient slot but is back in a register, so it is not currently
/// spilled.
///
/// `convert_ssa_instruction`'s dead-variable cleanup must still call `remove_spill`
/// for it; otherwise the `SpillRecord` lingers in `records` and its offset is never
/// pushed back onto `free_spill_slots`. The next `allocate_spill_offset` then bumps
/// `next_spill_offset` instead of reusing the dead slot, so `max_spill_offset` — and
/// the spill region the prologue reserves on every call — grows by one for each such
/// pattern.
///
/// The leak has no soundness impact — the skipped offset is never reused
/// incorrectly — so it is purely a memory-footprint regression. Here the true
/// peak of simultaneously live spills is 5 slots, but the bug inflates the
/// reserved spill region to 6.
#[test]
fn brillig_spill_reloaded_transient_value_reclaims_slot_when_it_dies() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32, v3: u32, v4: u32):
        v5 = unchecked_add v0, v1
        v6 = unchecked_add v2, v3
        v7 = unchecked_add v4, v5
        v8 = unchecked_add v6, v7
        v9 = unchecked_add v0, v8
        return v9
    }
    ";

    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };

    // A leak-free allocator reserves 5 slots (the peak number of values spilled
    // at once). The bug reserves 6 because a reloaded transient value's slot is
    // never reclaimed when it dies.
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    let spill_region = spill_region_size(&main.to_string());
    assert_eq!(
        spill_region, 5,
        "spill region grew to {spill_region}: a reloaded transient value leaked its \
         spill slot instead of returning it to the free list when it died"
    );
}

/// Verify that spill/reload instructions are emitted when register
/// pressure exceeds the stack frame limit.
///
/// Uses `max_stack_frame_size = 6` with `start_offset = 2` (spill enabled), leaving
/// 4 usable register slots (sp[2..5]). The program has 2 params (v0, v1) occupying
/// 2 slots, then computes v2, v3 filling the remaining 2 — so computing v4 forces
/// a spill. Uses unchecked arithmetic to avoid overflow-check temporaries.
#[test]
fn brillig_spill_and_reload() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = unchecked_add v0, v1
        v3 = unchecked_add v0, u32 2
        v4 = unchecked_add v1, u32 3
        v5 = unchecked_add v2, v3
        v6 = unchecked_add v5, v4
        return v6
    }
    ";

    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);

    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    // Bytecode layout:
    //   0-2: Prologue — save spill base (sp[1]), reserve 1 spill slot
    //   3:   v2 = v0 + v1
    //   4:   Init constant 2
    //   5:   Spill v2 → spill[0]  (single store, offset 0 uses sp[1] directly)
    //   6:   v3 = v0 + 2  (reuses sp[4] freed by spill)
    //   7:   Init constant 3
    //   8:   v4 = v1 + 3
    //   9:   Reload v2 from spill[0]  (single load, offset 0 uses sp[1] directly)
    //   10:  v5 = v2 + v3
    //   11:  v6 = v5 + v4
    //   12:  Move result to return slot
    //   13:  Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[1] = @1
     1: @3 = const u32 1
     2: @1 = u32 add @1, @3
     3: sp[4] = u32 add sp[2], sp[3]
     4: sp[5] = const u32 2
     5: store sp[4] at sp[1]
     6: sp[4] = u32 add sp[2], sp[5]
     7: sp[2] = const u32 3
     8: sp[5] = u32 add sp[3], sp[2]
     9: sp[3] = load sp[1]
    10: sp[2] = u32 add sp[3], sp[4]
    11: sp[3] = u32 add sp[2], sp[5]
    12: sp[2] = sp[3]
    13: return
    ");
}

/// Verify that successor block params are eagerly spilled to ensure
/// consistent access across predecessor blocks.
///
/// Uses `max_stack_frame_size = 5` with `start_offset = 2` (spill enabled), leaving
/// 3 usable register slots. Block b0 jumps to b1 passing 3 params (all copies of v0),
/// which forces all successor params to be eagerly spilled to memory slots so that the
/// Jmp site writes consistently to spill slots and b1 reloads them consistently.
///
/// Without eager spilling, different predecessors could make different spill decisions for the
/// same param, leading to incorrect results.
#[test]
fn brillig_spill_successor_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        jmp b1(v0, v0, v0)
      b1(v1: u32, v2: u32, v3: u32):
        v4 = unchecked_add v1, v2
        v5 = unchecked_add v4, v3
        return v5
    }
    ";

    let layout = LayoutConfig::new(5, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);

    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    // Bytecode layout:
    //   0-2:   Prologue — save spill base (sp[1]), reserve 3 spill slots
    //   3:     Spill v0 → spill[0]  (single store, offset 0 uses sp[1] directly)
    //   4-6:   Spill v0 → spill[1]  (const 1, add addr, store)
    //   7-9:   Spill v0 → spill[2]  (const 2, add addr, store)
    //   10:    b1: Reload v1 from spill[0]  (single load, offset 0 uses sp[1] directly)
    //   11-13: Reload v2 from spill[1]  (const 1, add addr, load)
    //   14:    v4 = v1 + v2
    //   15-17: Reload v3 from spill[2]  (const 2, add addr, load)
    //   18:    v5 = v4 + v3
    //   19:    Move result to return slot
    //   20:    Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[1] = @1
     1: @3 = const u32 3
     2: @1 = u32 add @1, @3
     3: store sp[2] at sp[1]
     4: @4 = const u32 1
     5: @3 = u32 add sp[1], @4
     6: store sp[2] at @3
     7: @4 = const u32 2
     8: @3 = u32 add sp[1], @4
     9: store sp[2] at @3
    10: sp[3] = load sp[1]
    11: @4 = const u32 1
    12: @3 = u32 add sp[1], @4
    13: sp[4] = load @3
    14: sp[2] = u32 add sp[3], sp[4]
    15: @4 = const u32 2
    16: @3 = u32 add sp[1], @4
    17: sp[4] = load @3
    18: sp[3] = u32 add sp[2], sp[4]
    19: sp[2] = sp[3]
    20: return
    ");
}

/// Verify that permanently spilled non-param live-ins that die without being
/// reloaded by instruction codegen don't cause an ICE.
///
/// The `IfElse` instruction's `else_condition` is included in `for_each_value`
/// (and therefore in liveness / `last_uses`) but is NOT accessed by
/// `codegen_if_else`. When this value is a non-param live-in that was
/// permanently spilled at block entry, it remains in the `spilled` map
/// without ever being added to `available_variables`. The dead variable
/// cleanup must skip `mark_unavailable` for such values.
///
/// Uses `max_stack_frame_size = 6` (4 usable slots after `start_offset = 2`).
/// Block b0 has 4 params filling all slots, so computing v4 and v5 forces
/// spills. At the `JmpIf`, `spill_non_param_live_ins(b1)` permanently spills
/// v1–v5. In b1, the `IfElse` codegen reloads v4, v1, v2 but NOT v5
/// (`else_condition`). When v5 appears in `last_uses`, the cleanup sees it
/// as spilled but not available — this previously caused an ICE.
#[test]
fn brillig_spill_jmpif_diamond_dead_else_condition() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32, v3: u32):
        v4 = eq v0, u32 0
        v5 = not v4
        jmpif v4 then: b1(), else: b2()
      b1():
        v6 = if v4 then v1 else (if v5) v2
        v7 = unchecked_add v6, v3
        jmp b3(v7)
      b2():
        jmp b3(v1)
      b3(v8: u32):
        return v8
    }
    ";

    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    // This should compile without ICE. Previously, the dead variable cleanup
    // for v5 (else_condition) would assert because v5 was permanently spilled
    // at b1 entry but never reloaded into available_variables.
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert!(!main.to_string().is_empty());
}

/// Regression test for `ensure_permanent_spill` when the record is not permanent and not currently spilled.
#[test]
fn brillig_spill_case4_diamond_wrong_output() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = unchecked_add v0, v1
        v3 = unchecked_add v0, u32 2
        v4 = unchecked_add v1, u32 3
        v5 = unchecked_add v2, v3
        v6 = eq v0, u32 0
        jmpif v6 then: b1(), else: b2()
      b1():
        v11 = unchecked_add v2, v4
        jmp b3(v11)
      b2():
        v7 = unchecked_add v4, u32 10
        v8 = unchecked_add v5, u32 20
        v9 = unchecked_add v7, v8
        v10 = unchecked_add v2, v9
        jmp b3(v10)
      b3(v12: u32):
        return v12
    }
    ";

    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };

    // v0=0, v1=42 → v6=true → takes b1.
    // Correct: v2=42, v4=45, v11=42+45=87
    // Wrong:  v11=45 (if reads v4's register instead of v2's)
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(0u32), FieldElement::from(42u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(87u32)]);
}

/// Fixed-behavior snapshot for issue #12266.
///
/// `v3 = v0 + v1` transient-spills `v0`. `v4 = v3 + v0` reloads `v0` into a
/// fresh register, leaving its record in "transient + reloaded" state. The
/// terminator `jmp b1(v0, v1, v2)` permanent-spills `v0` via the
/// `spill_non_param_live_ins` short-circuit, then reloads every arg to write
/// it into b1's eagerly-spilled param slot. The fix frees the reloaded
/// register immediately when the permanent-spill short-circuit fires, so the
/// final reload fits without any extra eviction.
///
/// We know there is no leak because the jump-argument setup goes straight from
/// storing the second argument to reloading `v2`. The buggy compiler inserted
/// an extra `const u32 0; add; store sp[2] at @3` spill in between, which is
/// absent here.
#[test]
fn brillig_spill_does_not_leak_reloaded_permanent_values_bytecode() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v3 = unchecked_add v0, v1
        v4 = unchecked_add v3, v0
        jmp b1(v0, v1, v2)
      b1(v5: u32, v6: u32, v7: u32):
        v8 = unchecked_add v0, v1
        v9 = unchecked_add v8, v2
        return v9
    }
    ";

    let layout = LayoutConfig::new(5, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    // Fixed snapshot: after storing the second jump argument at 44-46, the code
    // immediately reloads `v2` at 47-49. There is no extra spill of `v0`
    // between those steps, which is the leak fingerprint from the buggy path.
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[1] = @1
     1: @3 = const u32 7
     2: @1 = u32 add @1, @3
     3: store sp[2] at sp[1]
     4: @4 = const u32 4
     5: @3 = u32 add sp[1], @4
     6: store sp[3] at @3
     7: sp[3] = load sp[1]
     8: @4 = const u32 5
     9: @3 = u32 add sp[1], @4
    10: store sp[4] at @3
    11: @4 = const u32 4
    12: @3 = u32 add sp[1], @4
    13: sp[4] = load @3
    14: sp[2] = u32 add sp[3], sp[4]
    15: @4 = const u32 6
    16: @3 = u32 add sp[1], @4
    17: store sp[2] at @3
    18: @4 = const u32 6
    19: @3 = u32 add sp[1], @4
    20: sp[3] = load @3
    21: sp[4] = load sp[1]
    22: sp[2] = u32 add sp[3], sp[4]
    23: sp[2] = load sp[1]
    24: @4 = const u32 1
    25: @3 = u32 add sp[1], @4
    26: store sp[2] at @3
    27: @4 = const u32 4
    28: @3 = u32 add sp[1], @4
    29: sp[3] = load @3
    30: @4 = const u32 2
    31: @3 = u32 add sp[1], @4
    32: store sp[3] at @3
    33: @4 = const u32 5
    34: @3 = u32 add sp[1], @4
    35: sp[4] = load @3
    36: @4 = const u32 3
    37: @3 = u32 add sp[1], @4
    38: store sp[4] at @3
    39: sp[3] = load sp[1]
    40: @4 = const u32 4
    41: @3 = u32 add sp[1], @4
    42: sp[4] = load @3
    43: sp[2] = u32 add sp[3], sp[4]
    44: @4 = const u32 5
    45: @3 = u32 add sp[1], @4
    46: sp[4] = load @3
    47: sp[3] = u32 add sp[2], sp[4]
    48: sp[2] = sp[3]
    49: return
    ");
}

/// Quantify the opcode saving from batching consecutive spill-slot stores.
///
/// `codegen_make_array` calls `ensure_register_capacity(4)`. With all three
/// parameters live across the array construction and only 4 usable registers,
/// that one call must spill four values at once into freshly allocated,
/// consecutive spill slots (0–3).
/// Each consecutive slot after the first in a run is emitted as the increment
/// `@3 = u32 add @3, @2` followed by a `store` — 2 opcodes — in place of the
/// `const` + `add` + `store` (3 opcodes) the per-slot path would emit.
#[test]
fn brillig_spill_batch_reduces_consecutive_spill_opcodes() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = make_array [u32 0] : [u32; 1]
        v5 = array_get v4, index u32 0 -> u32
        v6 = unchecked_add v5, v0
        v7 = unchecked_add v6, v1
        v8 = unchecked_add v7, v2
        return v8
    }
    ";

    let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);
    let bytecode = brillig.ssa_function_to_brillig[&Id::test_new(0)].to_string();

    // An increment-based spill advances the held scratch address to the next
    // consecutive slot with `@3 = u32 add @3, @2` instead of recomputing the
    // address with `const` + `add`.
    let incremental_spills =
        bytecode.lines().filter(|line| line.contains("@3 = u32 add @3, @2")).count();
    assert!(
        incremental_spills >= 2,
        "expected the make_array batch to emit at least two increment-based spills, \
         got {incremental_spills}"
    );
}

/// Regression for issue #12266.
///
/// On the previously buggy path, `spill_non_param_live_ins` marked reloaded values as
/// spilled without releasing their registers. With this 4-parameter shape and a
/// 2-register Brillig layout, that stale state reaches the next block as an
/// active transient spill and ICEs with "Transient spill leaked across block boundary"
/// at [`begin_block`][crate::brillig::brillig_gen::spill_manager::SpillManager::begin_block].
#[test]
fn brillig_spill_does_not_cause_transient_spill_leak() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32, v3: u32):
        v4 = unchecked_add v0, v1
        v5 = unchecked_add v4, v0
        v6 = unchecked_add v5, v2
        v7 = unchecked_add v6, v1
        jmp b1(v0, v1, v2, v3)
      b1(v8: u32, v9: u32, v10: u32, v11: u32):
        v12 = unchecked_add v0, v1
        v13 = unchecked_add v12, v3
        return v13
    }";

    let layout = LayoutConfig::new(4, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);
    let _ = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
}

/// Regression: the condition register of a `jmpif` was freed by the second
/// `spill_non_param_live_ins` call inside `jmp_setup`, then reused for a u32 arg.
///
/// Both the condition `v3` and the then-arg `v2` are non-param live-ins to `b1`
/// (`v2` appears as the `IfElse` else-value). The first `spill_non_param_live_ins`
/// permanently spills both. `convert_ssa_single_addr_value` reloads `v3` into
/// `R_cond`. Inside `jmp_setup`, `spill_non_param_live_ins` fires a second time.
/// The buggy code detected that `v3` had a spill record and was not currently
/// marked spilled (`was_reloaded`), and freed `R_cond`. `convert_ssa_value(v2)` then
/// reloaded `v2` (u32) into the freed `R_cond` slot. `JumpIf R_cond` failed at
/// runtime with "condition value is not a boolean: Bit size for value 32".
///
/// The fix checks `was_transient_reloaded` instead, which excludes already-permanent
/// records, so `R_cond` is kept alive through the `JumpIf`.
#[test]
fn brillig_spill_jmpif_condition_register_reuse() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v3 = eq v0, v1
        v4 = not v3
        jmpif v3 then: b1(v2), else: b2()
      b1(v5: u32):
        v6 = if v3 then v5 else (if v4) v2
        jmp b3(v6)
      b2():
        jmp b3(v1)
      b3(v7: u32):
        return v7
    }
    ";

    // 5 usable registers (sp[2..6]). The 3 params plus the two successor-block params
    // (v5 for b1, v7 for b3) fill all 5 slots; the successor params are immediately
    // spilled and freed, leaving exactly enough room for v3 and v4. By the time
    // jmpif fires, v2, v3, and v4 are all in registers and all get permanently
    // spilled by spill_non_param_live_ins(b1). Both v3 and v2 end up as non-param
    // live-ins to b1, triggering the double-call pattern that exposed the bug.
    let layout = LayoutConfig::new(7, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };

    // v0=1, v1=1 → v3=true → then-branch taken; v5=v2=42, v6=42.
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(1u32), FieldElement::from(1u32), FieldElement::from(42u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(42u32)]);
}

/// Regression: a `JmpIf` whose then-arguments must be written into eagerly
/// spilled successor-block param slots used to emit those spill-slot writes
/// unconditionally. When the else branch was taken at runtime, the spill slot
/// for the then-destination param was already overwritten with the then-arg,
/// and any later reload from that slot returned the wrong value.
///
/// Here `b1` loops back to itself via a `JmpIf` whose then-arg is `v4 = v2 + 1`
/// and whose else-edge falls through to `b2`. After the loop exits with v2 = 5,
/// `b2` reloads `v2` from its spill slot. The buggy code wrote v4 = 6 into
/// v2's spill slot before testing the condition, so the final reload produced
/// 6 instead of 5 and the program returned 16 instead of 15.
#[test]
fn brillig_spill_jmpif_then_arg_does_not_overwrite_param_slot() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        jmp b1(v0)
      b1(v2: u32):
        v3 = lt v2, u32 5
        v4 = unchecked_add v2, u32 1
        jmpif v3 then: b1(v4), else: b2()
      b2():
        v5 = unchecked_add v2, v1
        return v5
    }
    ";

    let layout = LayoutConfig::new(7, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(0u32), FieldElement::from(10u32)],
        &options,
    );

    assert_eq!(result, vec![FieldElement::from(15u32)]);
}

/// A long live chain that forces register spilling with a small stack frame, and only two
/// parameters so the entry point stays small enough to compile.
const SPILLING_SRC: &str = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = unchecked_add v0, v1
        v3 = unchecked_add v2, v0
        v4 = unchecked_add v3, v1
        v5 = unchecked_add v4, v0
        v6 = unchecked_add v5, v1
        v7 = unchecked_add v6, v0
        v8 = unchecked_add v7, v1
        v9 = unchecked_add v2, v8
        v10 = unchecked_add v3, v9
        v11 = unchecked_add v4, v10
        v12 = unchecked_add v5, v11
        v13 = unchecked_add v6, v12
        return v13
    }
    ";

/// Spilling uses the fixed scratch slots `@3`/`@4`/`@5`, which bypass the `ScratchSpace` allocator's
/// bounds check. Compiling a spilling function with fewer than `NUM_SPILL_SCRATCH_SLOTS` scratch
/// slots must fail loudly rather than silently emitting writes outside the scratch region — the
/// analogue of the allocator's "Scratch space too deep" assertion for procedures.
#[test]
#[should_panic(expected = "too small for spilling")]
fn spilling_with_too_small_scratch_space_panics() {
    let too_small = ReservedRegisters::NUM_SPILL_SCRATCH_SLOTS - 1;
    let layout = LayoutConfig::new(8, 16, too_small);
    let options = BrilligOptions { layout, ..Default::default() };
    // Panics in codegen while building the spilling function. If the program failed to spill the
    // assertion would not fire and this `#[should_panic]` test would fail, so it also guards that
    // the program still spills.
    let _ = ssa_to_brillig_artifacts_with_options(SPILLING_SRC, &options);
}

/// The boundary case: exactly `NUM_SPILL_SCRATCH_SLOTS` scratch slots is enough to spill.
#[test]
fn spilling_with_minimum_scratch_space_compiles() {
    let layout = LayoutConfig::new(8, 16, ReservedRegisters::NUM_SPILL_SCRATCH_SLOTS);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(SPILLING_SRC, &options);
    assert!(!brillig.ssa_function_to_brillig[&Id::test_new(0)].byte_code.is_empty());
}

/// With an 8-slot frame every then-argument of the `JmpIf` must be reloaded, and the
/// condition `v8` is the least recently used value when the last reload (`v2`) needs
/// a register. Evicting it must not change the predicate used by the `JumpIf`.
#[test]
fn brillig_spill_jmpif_condition_survives_then_argument_reload() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = lt v0, v1
        v3 = unchecked_add v0, v1
        v4 = unchecked_add v3, v1
        v5 = unchecked_add v4, v1
        v6 = unchecked_add v5, v1
        v7 = unchecked_add v6, v1
        v8 = lt v1, v0
        jmpif v8 then: b1(v3, v4, v5, v6, v7, v2), else: b2()
      b1(v9: u32, v10: u32, v11: u32, v12: u32, v13: u32, v14: u1):
        v15 = unchecked_add v9, v10
        v16 = unchecked_add v15, v11
        v17 = unchecked_add v16, v12
        v18 = unchecked_add v17, v13
        v19 = cast v14 as u32
        v20 = unchecked_add v18, v19
        jmp b3(v20)
      b2():
        jmp b3(u32 7777)
      b3(v21: u32):
        return v21
    }
    ";

    let layout = LayoutConfig::new(8, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };

    // v8 = 1 < 2 is true: then-edge, 3 + 4 + 5 + 6 + 7 + cast(2 < 1) = 25.
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(2u32), FieldElement::from(1u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(25u32)]);

    // v8 = 2 < 1 is false: else-edge.
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(1u32), FieldElement::from(2u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(7777u32)]);
}

/// At the `jmpif` the frame is full: `v3`..`v6` were reloaded from their permanent slots for
/// the constraints and are live into both successors, and `v10` is the condition. Promoting
/// those reloaded live-ins must leave them evictable, so the then-edge can still get a register
/// for its private copy of the condition.
#[test]
fn brillig_spill_jmpif_with_then_args_fits_full_frame_of_reloaded_live_ins() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u1, v1: u32):
        v3 = unchecked_add v1, u32 1
        v4 = unchecked_add v1, u32 1
        v5 = unchecked_add v1, u32 2
        v6 = unchecked_add v1, u32 2
        jmp b1()
      b1():
        constrain v3 == v4
        constrain v5 == v6
        v10 = not v0
        jmpif v10 then: b2(v0), else: b3()
      b2(v11: u1):
        v12 = cast v11 as u32
        v13 = unchecked_add v3, v4
        v14 = unchecked_add v13, v5
        v15 = unchecked_add v14, v6
        v16 = unchecked_add v15, v12
        jmp b4(v16)
      b3():
        v17 = cast v0 as u32
        v18 = unchecked_add v3, v4
        v19 = unchecked_add v18, v5
        v20 = unchecked_add v19, v6
        v21 = unchecked_add v20, v17
        v22 = unchecked_add v21, u32 1000
        jmp b4(v22)
      b4(v23: u32):
        return v23
    }
    ";

    let layout = LayoutConfig::new(8, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };

    // v0 = false: then-edge, 1 + 1 + 2 + 2 + cast(false) = 6.
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(false), FieldElement::from(0u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(6u32)]);

    // v0 = true: else-edge, 1 + 1 + 2 + 2 + cast(true) + 1000 = 1007.
    let result = execute_brillig_from_ssa_with_options(
        src,
        vec![FieldElement::from(true), FieldElement::from(0u32)],
        &options,
    );
    assert_eq!(result, vec![FieldElement::from(1007u32)]);
}

fn execute_at_eight_slots(src: &str, inputs: &[u32]) -> Vec<FieldElement> {
    let layout = LayoutConfig::new(8, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    execute_brillig_from_ssa_with_options(
        src,
        inputs.iter().map(|&input| FieldElement::from(input)).collect(),
        &options,
    )
}

fn field_elements(values: &[u32]) -> Vec<FieldElement> {
    values.iter().map(|&value| FieldElement::from(value)).collect()
}

/// A self-loop `jmpif` swaps its own spilled parameters `v3` and `v4`. Making room for the
/// predicate copy evicts both, so each must be read before its slot is overwritten.
#[test]
fn brillig_spill_jmpif_then_edge_swaps_own_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        jmp b1(v0, v1, u1 1, u32 0)
      b1(v3: u32, v4: u32, v5: u1, v6: u32):
        v8 = unchecked_add v3, v4
        v9 = not v5
        v10 = unchecked_add v6, v6
        jmpif v5 then: b1(v4, v3, v9, v10), else: b2(v3, v4)
      b2(v11: u32, v12: u32):
        return v11, v12
    }
    ";
    assert_eq!(execute_at_eight_slots(src, &[1, 2]), field_elements(&[2, 1]));
}

/// A loop latch `jmp b1(v3, v2, v6)` swaps the header's spilled parameters `v2` and `v3`:
/// storing `v3` into `v2`'s slot must not happen before `v2` has been read from it.
#[test]
fn brillig_spill_jmp_swaps_spilled_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        jmp b1(v0, v1, u32 0)
      b1(v2: u32, v3: u32, v4: u32):
        v20 = unchecked_add v2, u32 1
        v21 = unchecked_add v2, u32 2
        v22 = unchecked_add v2, u32 3
        v23 = unchecked_add v2, u32 4
        v24 = unchecked_add v2, u32 5
        v25 = unchecked_add v20, v21
        v26 = unchecked_add v25, v22
        v27 = unchecked_add v26, v23
        v28 = unchecked_add v27, v24
        v5 = lt v4, u32 1
        jmpif v5 then: b2(), else: b3()
      b2():
        v6 = unchecked_add v4, u32 1
        jmp b1(v3, v2, v6)
      b3():
        return v2, v3, v28
    }
    ";
    // One swap, then exit: v28 = (2+1) + (2+2) + (2+3) + (2+4) + (2+5) = 25.
    assert_eq!(execute_at_eight_slots(src, &[1, 2]), field_elements(&[2, 1, 25]));
}

/// A three-way rotation of spilled parameters forms a cycle of slot stores that no
/// ordering can satisfy without a temporary.
#[test]
fn brillig_spill_jmp_rotates_spilled_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        jmp b1(v0, v1, v2, u32 0)
      b1(v3: u32, v4: u32, v5: u32, v6: u32):
        v20 = unchecked_add v3, u32 1
        v21 = unchecked_add v3, u32 2
        v22 = unchecked_add v3, u32 3
        v23 = unchecked_add v3, u32 4
        v24 = unchecked_add v20, v21
        v25 = unchecked_add v24, v22
        v26 = unchecked_add v25, v23
        v7 = lt v6, u32 1
        jmpif v7 then: b2(), else: b3()
      b2():
        v8 = unchecked_add v6, u32 1
        jmp b1(v4, v5, v3, v8)
      b3():
        return v3, v4, v5, v26
    }
    ";
    // One rotation, then exit: v26 = (2+1) + (2+2) + (2+3) + (2+4) = 18.
    assert_eq!(execute_at_eight_slots(src, &[1, 2, 3]), field_elements(&[2, 3, 1, 18]));
}

/// A `jmpif` self-loop whose then-edge swaps five disjoint pairs of its spilled parameters:
/// five cycles of slot stores on one edge. Each cycle's temporary must be released before
/// the next one is taken, or the copies alone exhaust the 6 usable registers.
/// Pack the 4-bit parameter values into a single return value to fit the tiny frame.
#[test]
fn brillig_spill_jmpif_then_edge_swaps_five_pairs_of_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u1):
        jmp b1(u32 1, u32 2, u32 3, u32 4, u32 5, u32 6, u32 7, u32 8, u32 9, u32 10, v0)
      b1(v1: u32, v2: u32, v3: u32, v4: u32, v5: u32, v6: u32, v7: u32, v8: u32, v9: u32, v10: u32, v11: u1):
        v12 = not v11
        jmpif v11 then: b1(v2, v1, v4, v3, v6, v5, v8, v7, v10, v9, v12), else: b2()
      b2():
        v30 = cast v1 as u64
        v31 = unchecked_mul v30, u64 16
        v32 = cast v2 as u64
        v33 = unchecked_add v31, v32
        v34 = unchecked_mul v33, u64 16
        v35 = cast v3 as u64
        v36 = unchecked_add v34, v35
        v37 = unchecked_mul v36, u64 16
        v38 = cast v4 as u64
        v39 = unchecked_add v37, v38
        v40 = unchecked_mul v39, u64 16
        v41 = cast v5 as u64
        v42 = unchecked_add v40, v41
        v43 = unchecked_mul v42, u64 16
        v44 = cast v6 as u64
        v45 = unchecked_add v43, v44
        v46 = unchecked_mul v45, u64 16
        v47 = cast v7 as u64
        v48 = unchecked_add v46, v47
        v49 = unchecked_mul v48, u64 16
        v50 = cast v8 as u64
        v51 = unchecked_add v49, v50
        v52 = unchecked_mul v51, u64 16
        v53 = cast v9 as u64
        v54 = unchecked_add v52, v53
        v55 = unchecked_mul v54, u64 16
        v56 = cast v10 as u64
        v57 = unchecked_add v55, v56
        return v57
    }
    ";
    assert_eq!(execute_at_eight_slots(src, &[1]), vec![FieldElement::from(0x21436587a9u64)]);
    assert_eq!(execute_at_eight_slots(src, &[0]), vec![FieldElement::from(0x123456789au64)]);
}

/// A loop latch `jmp` that swaps six disjoint pairs of the header's spilled parameters.
/// The result packs each 4-bit value in parameter order.
#[test]
fn brillig_spill_jmp_swaps_six_pairs_of_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0():
        jmp b1(u32 1, u32 2, u32 3, u32 4, u32 5, u32 6, u32 7, u32 8, u32 9, u32 10, u32 11, u32 12, u1 1)
      b1(v1: u32, v2: u32, v3: u32, v4: u32, v5: u32, v6: u32, v7: u32, v8: u32, v9: u32, v10: u32, v11: u32, v12: u32, v13: u1):
        jmpif v13 then: b2(), else: b3()
      b2():
        v14 = not v13
        jmp b1(v2, v1, v4, v3, v6, v5, v8, v7, v10, v9, v12, v11, v14)
      b3():
        v30 = cast v1 as u64
        v31 = unchecked_mul v30, u64 16
        v32 = cast v2 as u64
        v33 = unchecked_add v31, v32
        v34 = unchecked_mul v33, u64 16
        v35 = cast v3 as u64
        v36 = unchecked_add v34, v35
        v37 = unchecked_mul v36, u64 16
        v38 = cast v4 as u64
        v39 = unchecked_add v37, v38
        v40 = unchecked_mul v39, u64 16
        v41 = cast v5 as u64
        v42 = unchecked_add v40, v41
        v43 = unchecked_mul v42, u64 16
        v44 = cast v6 as u64
        v45 = unchecked_add v43, v44
        v46 = unchecked_mul v45, u64 16
        v47 = cast v7 as u64
        v48 = unchecked_add v46, v47
        v49 = unchecked_mul v48, u64 16
        v50 = cast v8 as u64
        v51 = unchecked_add v49, v50
        v52 = unchecked_mul v51, u64 16
        v53 = cast v9 as u64
        v54 = unchecked_add v52, v53
        v55 = unchecked_mul v54, u64 16
        v56 = cast v10 as u64
        v57 = unchecked_add v55, v56
        v58 = unchecked_mul v57, u64 16
        v59 = cast v11 as u64
        v60 = unchecked_add v58, v59
        v61 = unchecked_mul v60, u64 16
        v62 = cast v12 as u64
        v63 = unchecked_add v61, v62
        return v63
    }
    ";
    assert_eq!(execute_at_eight_slots(src, &[]), vec![FieldElement::from(0x21436587a9cbu64)]);
}

/// A latch `jmp b1(v3, v4, v6)` into `b1(v2, v3, v4, ..)` is a chain, not a cycle: `v2`'s
/// slot can be written first, then `v3`'s, then `v4`'s, with no temporary.
#[test]
fn brillig_spill_jmp_shifts_spilled_params() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        jmp b1(v0, v1, u32 3, u32 0)
      b1(v2: u32, v3: u32, v4: u32, v5: u32):
        v20 = unchecked_add v2, u32 1
        v21 = unchecked_add v2, u32 2
        v22 = unchecked_add v2, u32 3
        v23 = unchecked_add v2, u32 4
        v24 = unchecked_add v20, v21
        v25 = unchecked_add v24, v22
        v26 = unchecked_add v25, v23
        v7 = lt v5, u32 1
        jmpif v7 then: b2(), else: b3()
      b2():
        v6 = unchecked_add v5, u32 1
        jmp b1(v3, v4, u32 9, v6)
      b3():
        return v2, v3, v4, v26
    }
    ";
    // One shift, then exit: v26 = (2+1) + (2+2) + (2+3) + (2+4) = 18.
    assert_eq!(execute_at_eight_slots(src, &[1, 2]), field_elements(&[2, 3, 9, 18]));
}

/// A latch `jmp b1(v3, v2, v2, ..)` reads `v2` from two stores: one in the `v2`/`v3` swap
/// and one into `v4`. `v2`'s slot may only be written after both have read it.
#[test]
fn brillig_spill_jmp_param_read_by_two_stores() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        jmp b1(v0, v1, u32 3, u32 0)
      b1(v2: u32, v3: u32, v4: u32, v5: u32):
        v20 = unchecked_add v2, u32 1
        v21 = unchecked_add v2, u32 2
        v22 = unchecked_add v2, u32 3
        v23 = unchecked_add v2, u32 4
        v24 = unchecked_add v20, v21
        v25 = unchecked_add v24, v22
        v26 = unchecked_add v25, v23
        v7 = lt v5, u32 1
        jmpif v7 then: b2(), else: b3()
      b2():
        v6 = unchecked_add v5, u32 1
        jmp b1(v3, v2, v2, v6)
      b3():
        return v2, v3, v4, v26
    }
    ";
    assert_eq!(execute_at_eight_slots(src, &[1, 2]), field_elements(&[2, 1, 1, 18]));
}

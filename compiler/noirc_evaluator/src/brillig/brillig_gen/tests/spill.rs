use acvm::FieldElement;

use crate::{
    assert_artifact_snapshot,
    brillig::{
        BrilligOptions,
        brillig_gen::tests::{
            execute_brillig_from_ssa_with_options, ssa_to_brillig_artifacts_with_options,
        },
        brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
    },
    ssa::ir::map::Id,
};

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
    //   10:    Jump to b1
    //   11:    Reload v1 from spill[0]  (single load, offset 0 uses sp[1] directly)
    //   12-14: Reload v2 from spill[1]  (const 1, add addr, load)
    //   15:    v4 = v1 + v2
    //   16-18: Reload v3 from spill[2]  (const 2, add addr, load)
    //   19:    v5 = v4 + v3
    //   20:    Move result to return slot
    //   21:    Return
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
    10: jump to 0 // -> 11: f0/b1
    11: sp[3] = load sp[1] // f0/b1
    12: @4 = const u32 1
    13: @3 = u32 add sp[1], @4
    14: sp[4] = load @3
    15: sp[2] = u32 add sp[3], sp[4]
    16: @4 = const u32 2
    17: @3 = u32 add sp[1], @4
    18: sp[4] = load @3
    19: sp[3] = u32 add sp[2], sp[4]
    20: sp[2] = sp[3]
    21: return
    ");
}

/// Verify that permanently spilled non-param live-ins that die without being
/// reloaded by instruction codegen don't cause an ICE.
///
/// The IfElse instruction's `else_condition` is included in `for_each_value`
/// (and therefore in liveness / `last_uses`) but is NOT accessed by
/// `codegen_if_else`. When this value is a non-param live-in that was
/// permanently spilled at block entry, it remains in the `spilled` map
/// without ever being added to `available_variables`. The dead variable
/// cleanup must skip `mark_unavailable` for such values.
///
/// Uses `max_stack_frame_size = 6` (4 usable slots after `start_offset = 2`).
/// Block b0 has 4 params filling all slots, so computing v4 and v5 forces
/// spills. At the JmpIf, `spill_non_param_live_ins(b1)` permanently spills
/// v1–v5. In b1, the IfElse codegen reloads v4, v1, v2 but NOT v5
/// (else_condition). When v5 appears in `last_uses`, the cleanup sees it
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
    39: jump to 0 // -> 40: f0/b1
    40: sp[3] = load sp[1] // f0/b1
    41: @4 = const u32 4
    42: @3 = u32 add sp[1], @4
    43: sp[4] = load @3
    44: sp[2] = u32 add sp[3], sp[4]
    45: @4 = const u32 5
    46: @3 = u32 add sp[1], @4
    47: sp[4] = load @3
    48: sp[3] = u32 add sp[2], sp[4]
    49: sp[2] = sp[3]
    50: return
    ");
}

/// Regression for issue #12266.
///
/// On the previously buggy path, `spill_non_param_live_ins` marked reloaded values as
/// spilled without releasing their registers. With this 4-parameter shape and a
/// 2-register Brillig layout, that stale state reaches the next block as an
/// active transient spill and ICEs with "Transient spill leaked across block boundary"
/// at [begin_block][crate::brillig::brillig_gen::spill_manager::SpillManager::begin_block].
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
/// (`v2` appears as the IfElse else-value). The first `spill_non_param_live_ins`
/// permanently spills both. `convert_ssa_single_addr_value` reloads `v3` into
/// R_cond. Inside `jmp_setup`, `spill_non_param_live_ins` fires a second time.
/// The buggy code detected that `v3` had a spill record and was not currently
/// marked spilled (`was_reloaded`), and freed R_cond. `convert_ssa_value(v2)` then
/// reloaded `v2` (u32) into the freed R_cond slot. `JumpIf R_cond` failed at
/// runtime with "condition value is not a boolean: Bit size for value 32".
///
/// The fix checks `was_transient_reloaded` instead, which excludes already-permanent
/// records, so R_cond is kept alive through the `JumpIf`.
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

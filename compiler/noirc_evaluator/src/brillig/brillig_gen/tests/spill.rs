use crate::{
    assert_artifact_snapshot,
    brillig::{
        BrilligOptions,
        brillig_gen::tests::ssa_to_brillig_artifacts_with_options,
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
    //   0-3:   Prologue — save spill base (sp[1]), reserve 1 spill slot
    //   4:     v2 = v0 + v1
    //   5:     Init constant 2
    //   6-8:   Spill v2 → spill[0]  (const offset, add addr, store)
    //   9:     v3 = v0 + 2  (reuses sp[4] freed by spill)
    //   10:    Init constant 3
    //   11:    v4 = v1 + 3
    //   12-14: Reload v2 from spill[0]  (const offset, add addr, load)
    //   15:    v5 = v2 + v3
    //   16:    v6 = v5 + v4
    //   17:    Move result to return slot
    //   18:    Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[1] = @1
     1: @3 = const u32 1
     2: @1 = u32 add @1, @3
     3: sp[4] = u32 add sp[2], sp[3]
     4: sp[5] = const u32 2
     5: @4 = const u32 0
     6: @3 = u32 add sp[1], @4
     7: store sp[4] at @3
     8: sp[4] = u32 add sp[2], sp[5]
     9: sp[2] = const u32 3
    10: sp[5] = u32 add sp[3], sp[2]
    11: @4 = const u32 0
    12: @3 = u32 add sp[1], @4
    13: sp[3] = load @3
    14: sp[2] = u32 add sp[3], sp[4]
    15: sp[3] = u32 add sp[2], sp[5]
    16: sp[2] = sp[3]
    17: return
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
    //   0-3:   Prologue — save spill base (sp[1]), reserve 3 spill slots
    //   4-6:   Spill v0 → spill[0]  (const 0, add addr, store)
    //   7-9:   Spill v0 → spill[1]  (const 1, add addr, store)
    //   10-12: Spill v0 → spill[2]  (const 2, add addr, store)
    //   13:    Jump to b1
    //   14-16: Reload v1 from spill[0]  (const 0, add addr, load)
    //   17-19: Reload v2 from spill[1]  (const 1, add addr, load)
    //   20:    v4 = v1 + v2
    //   21-23: Reload v3 from spill[2]  (const 2, add addr, load)
    //   24:    v5 = v4 + v3
    //   25:    Move result to return slot
    //   26:    Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: sp[1] = @1
     1: @3 = const u32 3
     2: @1 = u32 add @1, @3
     3: @4 = const u32 0
     4: @3 = u32 add sp[1], @4
     5: store sp[2] at @3
     6: @4 = const u32 1
     7: @3 = u32 add sp[1], @4
     8: store sp[2] at @3
     9: @4 = const u32 2
    10: @3 = u32 add sp[1], @4
    11: store sp[2] at @3
    12: jump to 0 // -> 13: f0/b1
    13: @4 = const u32 0 // f0/b1
    14: @3 = u32 add sp[1], @4
    15: sp[3] = load @3
    16: @4 = const u32 1
    17: @3 = u32 add sp[1], @4
    18: sp[4] = load @3
    19: sp[2] = u32 add sp[3], sp[4]
    20: @4 = const u32 2
    21: @3 = u32 add sp[1], @4
    22: sp[4] = load @3
    23: sp[3] = u32 add sp[2], sp[4]
    24: sp[2] = sp[3]
    25: return
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

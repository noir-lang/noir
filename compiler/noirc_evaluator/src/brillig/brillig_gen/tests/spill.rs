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
    //   5-6:   v3 = v0 + 2
    //   7-11:  Spill v2 → spill[0], then v4 = v0 + 3 (evicts v2 to free sp[4])
    //   12:    v4 = v1 + 3
    //   13-17: Reload v2 from spill[0], v5 = v2 + v3
    //   18:    v6 = v5 + v4
    //   19:    Move result to return slot
    //   20:    Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: call 0
     1: sp[1] = @1
     2: @3 = const u32 1
     3: @1 = u32 add @1, @3
     4: sp[4] = u32 add sp[2], sp[3]
     5: sp[5] = const u32 2
     6: @3 = sp[1]
     7: @4 = const u32 0
     8: @3 = u32 add @3, @4
     9: store sp[4] at @3
    10: sp[4] = u32 add sp[2], sp[5]
    11: sp[2] = const u32 3
    12: sp[5] = u32 add sp[3], sp[2]
    13: @3 = sp[1]
    14: @4 = const u32 0
    15: @3 = u32 add @3, @4
    16: sp[3] = load @3
    17: sp[2] = u32 add sp[3], sp[4]
    18: sp[3] = u32 add sp[2], sp[5]
    19: sp[2] = sp[3]
    20: return
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
    //   4-7:   Store v0 → spill[0]  (addr = spill_base + 0)
    //   8-11:  Store v0 → spill[1]  (addr = spill_base + 1)
    //   12-15: Store v0 → spill[2]  (addr = spill_base + 2)
    //   16:    Jump to b1
    //   17-20: Reload v1 from spill[0]
    //   21-24: Reload v2 from spill[1]
    //   25:    v4 = v1 + v2
    //   26-29: Reload v3 from spill[2]
    //   30:    v5 = v4 + v3
    //   31:    Move result to return slot
    //   32:    Return
    assert_artifact_snapshot!(main, @r"
    fn main
     0: call 0
     1: sp[1] = @1
     2: @3 = const u32 3
     3: @1 = u32 add @1, @3
     4: @3 = sp[1]
     5: @4 = const u32 0
     6: @3 = u32 add @3, @4
     7: store sp[2] at @3
     8: @3 = sp[1]
     9: @4 = const u32 1
    10: @3 = u32 add @3, @4
    11: store sp[2] at @3
    12: @3 = sp[1]
    13: @4 = const u32 2
    14: @3 = u32 add @3, @4
    15: store sp[2] at @3
    16: jump to 0
    17: @3 = sp[1]
    18: @4 = const u32 0
    19: @3 = u32 add @3, @4
    20: sp[3] = load @3
    21: @3 = sp[1]
    22: @4 = const u32 1
    23: @3 = u32 add @3, @4
    24: sp[4] = load @3
    25: sp[2] = u32 add sp[3], sp[4]
    26: @3 = sp[1]
    27: @4 = const u32 2
    28: @3 = u32 add @3, @4
    29: sp[4] = load @3
    30: sp[3] = u32 add sp[2], sp[4]
    31: sp[2] = sp[3]
    32: return
    ");
}

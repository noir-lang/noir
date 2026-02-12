use crate::{
    assert_artifact_snapshot,
    brillig::{
        BrilligOptions,
        brillig_gen::tests::ssa_to_brillig_artifacts_with_options,
        brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
    },
    ssa::ir::map::Id,
};

/// Snapshot test verifying that spill/reload instructions are emitted when register
/// pressure exceeds the stack frame limit.
///
/// Uses `max_stack_frame_size = 12` with `start_offset = 2` (spill enabled), leaving
/// 10 usable register slots. The program computes 10 independent values from v0 (keeping
/// all of them live until the final summation), which forces spilling when the 11th
/// register is needed. Uses unchecked arithmetic to avoid overflow-check temporaries.
#[test]
fn brillig_spill_and_reload() {
    // This program:
    // 1. Computes v2..v11 from v0, each needing their own register
    // 2. At the point v11 is computed, v2..v10 are all still live (used in the summation below)
    //    plus v0, requiring 11 registers total — exceeding the 10 available slots
    // 3. Then sums v2..v11 in a chain, consuming (and freeing) values one at a time
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = unchecked_add v0, v1
        v3 = unchecked_add v0, u32 2
        v4 = unchecked_add v0, u32 3
        v5 = unchecked_add v1, u32 4
        v6 = unchecked_add v1, u32 5
        v7 = unchecked_add v0, u32 6
        v8 = unchecked_add v1, u32 7
        v9 = unchecked_add v0, u32 8
        v10 = unchecked_add v1, u32 9
        v11 = unchecked_add v0, u32 10
        v12 = unchecked_add v2, v3
        v13 = unchecked_add v12, v4
        v14 = unchecked_add v13, v5
        v15 = unchecked_add v14, v6
        v16 = unchecked_add v15, v7
        v17 = unchecked_add v16, v8
        v18 = unchecked_add v17, v9
        v19 = unchecked_add v18, v10
        v20 = unchecked_add v19, v11
        return v20
    }
    ";

    let layout = LayoutConfig::new(12, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let brillig = ssa_to_brillig_artifacts_with_options(src, &options);

    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
     0: call 0
     1: sp[1] = @1
     2: @3 = const u32 2
     3: @1 = u32 add @1, @3
     4: sp[4] = u32 add sp[2], sp[3]
     5: sp[5] = const u32 2
     6: sp[6] = u32 add sp[2], sp[5]
     7: sp[5] = const u32 3
     8: sp[7] = u32 add sp[2], sp[5]
     9: sp[5] = const u32 4
    10: sp[8] = u32 add sp[3], sp[5]
    11: sp[5] = const u32 5
    12: sp[9] = u32 add sp[3], sp[5]
    13: sp[5] = const u32 6
    14: sp[10] = u32 add sp[2], sp[5]
    15: sp[5] = const u32 7
    16: sp[11] = u32 add sp[3], sp[5]
    17: sp[5] = const u32 8
    18: @3 = sp[1]
    19: @4 = const u32 0
    20: @3 = u32 add @3, @4
    21: store sp[4] at @3
    22: sp[4] = u32 add sp[2], sp[5]
    23: sp[5] = const u32 9
    24: @3 = sp[1]
    25: @4 = const u32 1
    26: @3 = u32 add @3, @4
    27: store sp[6] at @3
    28: sp[6] = u32 add sp[3], sp[5]
    29: sp[3] = const u32 10
    30: sp[5] = u32 add sp[2], sp[3]
    31: @3 = sp[1]
    32: @4 = const u32 0
    33: @3 = u32 add @3, @4
    34: sp[3] = load @3
    35: @3 = sp[1]
    36: @4 = const u32 0
    37: @3 = u32 add @3, @4
    38: store sp[7] at @3
    39: @3 = sp[1]
    40: @4 = const u32 1
    41: @3 = u32 add @3, @4
    42: sp[7] = load @3
    43: sp[2] = u32 add sp[3], sp[7]
    44: @3 = sp[1]
    45: @4 = const u32 0
    46: @3 = u32 add @3, @4
    47: sp[7] = load @3
    48: sp[3] = u32 add sp[2], sp[7]
    49: sp[2] = u32 add sp[3], sp[8]
    50: sp[3] = u32 add sp[2], sp[9]
    51: sp[2] = u32 add sp[3], sp[10]
    52: sp[3] = u32 add sp[2], sp[11]
    53: sp[2] = u32 add sp[3], sp[4]
    54: sp[3] = u32 add sp[2], sp[6]
    55: sp[2] = u32 add sp[3], sp[5]
    56: return
    ");
}

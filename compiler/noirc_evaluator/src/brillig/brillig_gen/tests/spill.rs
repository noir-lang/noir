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
    ssa::{interpreter::value::Value, ir::map::Id, ssa_gen::Ssa},
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
     3: @4 = const u32 0
     4: @3 = u32 add sp[1], @4
     5: store sp[2] at @3
     6: @4 = const u32 4
     7: @3 = u32 add sp[1], @4
     8: store sp[3] at @3
     9: @4 = const u32 0
    10: @3 = u32 add sp[1], @4
    11: sp[3] = load @3
    12: @4 = const u32 5
    13: @3 = u32 add sp[1], @4
    14: store sp[4] at @3
    15: @4 = const u32 4
    16: @3 = u32 add sp[1], @4
    17: sp[4] = load @3
    18: sp[2] = u32 add sp[3], sp[4]
    19: @4 = const u32 6
    20: @3 = u32 add sp[1], @4
    21: store sp[2] at @3
    22: @4 = const u32 0
    23: @3 = u32 add sp[1], @4
    24: store sp[3] at @3
    25: @4 = const u32 6
    26: @3 = u32 add sp[1], @4
    27: sp[3] = load @3
    28: @4 = const u32 4
    29: @3 = u32 add sp[1], @4
    30: store sp[4] at @3
    31: @4 = const u32 0
    32: @3 = u32 add sp[1], @4
    33: sp[4] = load @3
    34: sp[2] = u32 add sp[3], sp[4]
    35: @4 = const u32 0
    36: @3 = u32 add sp[1], @4
    37: sp[2] = load @3
    38: @4 = const u32 1
    39: @3 = u32 add sp[1], @4
    40: store sp[2] at @3
    41: @4 = const u32 4
    42: @3 = u32 add sp[1], @4
    43: sp[3] = load @3
    44: @4 = const u32 2
    45: @3 = u32 add sp[1], @4
    46: store sp[3] at @3
    47: @4 = const u32 5
    48: @3 = u32 add sp[1], @4
    49: sp[4] = load @3
    50: @4 = const u32 3
    51: @3 = u32 add sp[1], @4
    52: store sp[4] at @3
    53: jump to 0 // -> 54: f0/b1
    54: @4 = const u32 0 // f0/b1
    55: @3 = u32 add sp[1], @4
    56: sp[3] = load @3
    57: @4 = const u32 4
    58: @3 = u32 add sp[1], @4
    59: sp[4] = load @3
    60: sp[2] = u32 add sp[3], sp[4]
    61: @4 = const u32 5
    62: @3 = u32 add sp[1], @4
    63: sp[4] = load @3
    64: sp[3] = u32 add sp[2], sp[4]
    65: sp[2] = sp[3]
    66: return
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

#[test]
fn brillig_spill_vectors_reduced_full_prefix_to_tail_region() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field):
        v49 = make_array [Field 0, Field 0] : [Field]
        v50 = eq v0, Field 10
        v52 = eq v0, Field 20
        jmpif v50 then: b1(), else: b5()
      b1():
        jmp b3(u32 2, v49)
      b3(v2: u32, v3: [Field]):
        constrain v2 == u32 3
        v57 = array_get v3, index u32 5 minus 3 -> Field
        constrain v57 == Field 10
        jmp b7()
      b5():
        v53 = make_array [Field 0, Field 0, Field 10] : [Field]
        jmp b8(u32 3, v53)
      b7():
        v58 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b9(u32 4, v58)
      b8(v4: u32, v5: [Field]):
        jmp b3(v4, v5)
      b9(v6: u32, v7: [Field]):
        constrain v6 == u32 4
        v62 = array_get v7, index u32 6 minus 3 -> Field
        constrain v62 == Field 5
        jmpif v50 then: b10(), else: b11()
      b10():
        v70 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b12(u32 3, v70)
      b11():
        v68 = make_array [Field 0, Field 0, Field 0, Field 1, Field 2, Field 3, Field 4] : [Field]
        jmp b12(u32 7, v68)
      b12(v8: u32, v9: [Field]):
        constrain v8 == u32 7
        v72 = array_get v9, index u32 9 minus 3 -> Field
        constrain v72 == Field 4
        jmpif v50 then: b13(), else: b14()
      b13():
        v74 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b15(u32 3, v74)
      b14():
        v73 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b15(u32 4, v73)
      b15(v10: u32, v11: [Field]):
        jmpif v52 then: b16(), else: b17(v10, v11)
      b16():
        v76, v77 = call vector_push_back(v10, v11, Field 20) -> (u32, [Field])
        jmp b17(v76, v77)
      b17(v12: u32, v13: [Field]):
        v79, v80 = call vector_push_back(v12, v13, Field 15) -> (u32, [Field])
        v82, v83 = call vector_push_back(v79, v80, Field 30) -> (u32, [Field])
        constrain v82 == u32 6
        v84 = array_get v83, index u32 6 minus 3 -> Field
        constrain v84 == Field 5
        v85 = array_get v83, index u32 7 minus 3 -> Field
        constrain v85 == Field 15
        v87 = array_get v83, index u32 8 minus 3 -> Field
        constrain v87 == Field 30
        jmpif v50 then: b18(), else: b19()
      b18():
        v89 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b20(u32 3, v89)
      b19():
        v88 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b20(u32 4, v88)
      b20(v14: u32, v15: [Field]):
        v90, v91 = call vector_push_back(v14, v15, Field 30) -> (u32, [Field])
        jmpif v52 then: b21(), else: b22(v90, v91)
      b21():
        v92, v93 = call vector_push_back(v90, v91, Field 20) -> (u32, [Field])
        jmp b22(v92, v93)
      b22(v16: u32, v17: [Field]):
        v94, v95 = call vector_push_back(v16, v17, Field 15) -> (u32, [Field])
        jmpif v52 then: b23(v94, v95), else: b24()
      b23(v18: u32, v19: [Field]):
        v100, v101 = call vector_push_back(v18, v19, Field 60) -> (u32, [Field])
        constrain v100 == u32 8
        v102 = array_get v101, index u32 6 minus 3 -> Field
        constrain v102 == Field 5
        v103 = array_get v101, index u32 7 minus 3 -> Field
        constrain v103 == Field 30
        v104 = array_get v101, index u32 8 minus 3 -> Field
        constrain v104 == Field 15
        v105 = array_get v101, index u32 9 minus 3 -> Field
        constrain v105 == Field 50
        v107 = array_get v101, index u32 10 minus 3 -> Field
        constrain v107 == Field 60
        jmpif v50 then: b25(), else: b26()
      b24():
        v97, v98 = call vector_push_back(v94, v95, Field 50) -> (u32, [Field])
        jmp b23(v97, v98)
      b25():
        v109 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b27(u32 3, v109)
      b26():
        v108 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b27(u32 4, v108)
      b27(v20: u32, v21: [Field]):
        v110, v111 = call vector_push_back(v20, v21, Field 30) -> (u32, [Field])
        jmpif v52 then: b28(), else: b29(v110, v111)
      b28():
        v112, v113 = call vector_push_back(v110, v111, Field 20) -> (u32, [Field])
        jmp b29(v112, v113)
      b29(v22: u32, v23: [Field]):
        v115 = lt u32 0, v22
        constrain v115 == u1 1, \"Index out of bounds\"
        v118, v119, v120 = call vector_pop_back(v22, v23) -> (u32, [Field], Field)
        inc_rc v119
        constrain v118 == u32 4
        constrain v120 == Field 30
        v121, v122, v123 = call vector_pop_back(u32 4, v119) -> (u32, [Field], Field)
        constrain v121 == u32 3
        constrain v123 == v0
        jmpif v50 then: b30(), else: b31()
      b30():
        v125 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b32(u32 3, v125)
      b31():
        v124 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b32(u32 4, v124)
      b32(v24: u32, v25: [Field]):
        v126, v127 = call vector_push_back(v24, v25, Field 30) -> (u32, [Field])
        jmpif v52 then: b33(), else: b34(v126, v127)
      b33():
        v128, v129 = call vector_push_back(v126, v127, Field 20) -> (u32, [Field])
        v130, v131 = call vector_push_back(v128, v129, Field 15) -> (u32, [Field])
        jmp b34(v130, v131)
      b34(v26: u32, v27: [Field]):
        v133 = add v26, u32 1
        v134 = lt u32 1, v133
        constrain v134 == u1 1, \"Index out of bounds\"
        v136, v137 = call vector_insert(v26, v27, u32 1, Field 50) -> (u32, [Field])
        v138 = add v136, u32 1
        v139 = lt u32 6, v138
        constrain v139 == u1 1, \"Index out of bounds\"
        v141, v142 = call vector_insert(v136, v137, u32 6, Field 100) -> (u32, [Field])
        constrain v141 == u32 7
        v143 = array_get v142, index u32 4 minus 3 -> Field
        constrain v143 == Field 50
        v144 = array_get v142, index u32 5 minus 3 -> Field
        constrain v144 == Field 0
        v145 = array_get v142, index u32 8 minus 3 -> Field
        constrain v145 == Field 30
        v146 = array_get v142, index u32 9 minus 3 -> Field
        constrain v146 == Field 100
        jmpif v50 then: b35(), else: b36()
      b35():
        v148 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b37(u32 3, v148)
      b36():
        v147 = make_array [Field 0, Field 0, Field 10, v0] : [Field]
        jmp b37(u32 4, v147)
      b37(v28: u32, v29: [Field]):
        v149 = lt u32 2, v28
        constrain v149 == u1 1, \"Index out of bounds\"
        v151, v152, v153 = call vector_remove(v28, v29, u32 2) -> (u32, [Field], Field)
        inc_rc v152
        constrain v153 == Field 10
        jmpif v52 then: b38(), else: b39(v151, v152)
      b38():
        v154, v155 = call vector_push_back(v151, v152, Field 20) -> (u32, [Field])
        jmp b39(v154, v155)
      b39(v30: u32, v31: [Field]):
        v156, v157 = call vector_push_back(v30, v31, Field 15) -> (u32, [Field])
        jmpif v52 then: b40(v156), else: b41()
      b40(v32: u32):
        constrain v32 == u32 5
        v160 = make_array [Field 0, Field 0] : [Field]
        v161 = eq v0, Field 5
        jmpif v161 then: b42(), else: b43()
      b41():
        v158, v159 = call vector_push_back(v156, v157, Field 50) -> (u32, [Field])
        jmp b40(v158)
      b42():
        jmp b44(u32 2, v160)
      b43():
        jmpif v52 then: b45(), else: b46()
      b44(v33: u32, v34: [Field]):
        v163 = lt u32 0, v33
        constrain v163 == u1 1, \"Index out of bounds\"
        v164 = array_get v34, index u32 3 minus 3 -> Field
        constrain v164 == Field 0
        v165 = lt u32 1, v33
        constrain v165 == u1 1, \"Index out of bounds\"
        v166 = array_get v34, index u32 4 minus 3 -> Field
        constrain v166 == Field 0
        constrain v33 == u32 2
        jmpif v161 then: b47(), else: b48()
      b45():
        jmp b49(u32 2, v160)
      b46():
        v162 = make_array [Field 0, Field 0, Field 5] : [Field]
        jmp b49(u32 3, v162)
      b47():
        v168 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b50(u32 3, v168)
      b48():
        v167 = make_array [Field 0, Field 0, Field 5, v0] : [Field]
        jmp b50(u32 4, v167)
      b49(v35: u32, v36: [Field]):
        jmp b44(v35, v36)
      b50(v37: u32, v38: [Field]):
        v169 = lt u32 2, v37
        constrain v169 == u1 1, \"Index out of bounds\"
        v170 = array_get v38, index u32 5 minus 3 -> Field
        constrain v170 == Field 5
        constrain v37 == u32 3
        jmpif v161 then: b51(), else: b52()
      b51():
        v172 = make_array [Field 0, Field 0, v0] : [Field]
        jmp b53(u32 3, v172)
      b52():
        v171 = make_array [Field 0, Field 0, Field 0, Field 1, Field 2, Field 3, Field 4] : [Field]
        jmp b53(u32 7, v171)
      b53(v39: u32, v40: [Field]):
        v173 = lt u32 2, v39
        constrain v173 == u1 1, \"Index out of bounds\"
        v174 = array_get v40, index u32 5 minus 3 -> Field
        constrain v174 == Field 5
        constrain v39 == u32 3
        v177 = call to_be_radix(v0, u32 256) -> [u8; 32]
        v178 = eq v0, Field 0
        v179 = make_array [Field 0, Field 0] : [Field]
        jmpif v50 then: b54(), else: b55()
      b54():
        jmp b56(u32 2, v179)
      b55():
        jmpif v52 then: b57(), else: b58()
      b56(v41: u32, v42: [Field]):
        v181, v182 = call vector_push_back(v41, v42, Field 5) -> (u32, [Field])
        jmpif v178 then: b59(), else: b60()
      b57():
        jmp b61(u32 2, v179)
      b58():
        v180 = make_array [Field 0, Field 0, Field 10] : [Field]
        jmp b61(u32 3, v180)
      b59():
        inc_rc v182
        jmp b62(v181, v182)
      b60():
        inc_rc v182
        v183, v184 = call vector_push_back(v181, v182, Field 10) -> (u32, [Field])
        jmp b62(v183, v184)
      b61(v43: u32, v44: [Field]):
        jmp b56(v43, v44)
      b62(v45: u32, v46: [Field]):
        constrain v45 == u32 5
        v185 = array_get v46, index u32 3 minus 3 -> Field
        constrain v185 == Field 0
        v186 = array_get v46, index u32 4 minus 3 -> Field
        constrain v186 == Field 0
        v187 = array_get v46, index u32 5 minus 3 -> Field
        constrain v187 == Field 10
        v188 = array_get v46, index u32 6 minus 3 -> Field
        constrain v188 == Field 5
        v189 = array_get v46, index u32 7 minus 3 -> Field
        constrain v189 == Field 10
        v190 = truncate v0 to 32 bits, max_bit_size: 254
        v191 = cast v190 as u32
        v192 = mod v191, u32 3
        v193 = lt v192, u32 3
        constrain v193 == u1 1, \"Index out of bounds\"
        return
    }
    ";

    let ssa = Ssa::from_str(src).unwrap();
    let _ = ssa.interpret(vec![Value::field(FieldElement::from(5u32))]).unwrap();

    let layout = LayoutConfig::new(64, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    let result =
        execute_brillig_from_ssa_with_options(src, vec![FieldElement::from(5u32)], &options);
    assert_eq!(result, Vec::<FieldElement>::new());
}

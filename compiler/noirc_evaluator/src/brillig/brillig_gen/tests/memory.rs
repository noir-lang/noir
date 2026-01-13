use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests array element access by index code-gen for Brillig.
#[test]
fn brillig_array_get() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_get v1, index v0 -> u32
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[2] = const u32 10
     2: sp[3] = const u32 20
     3: sp[4] = const u32 30
     4: sp[5] = @1
     5: sp[6] = const u32 4
     6: @1 = u32 add @1, sp[6]
     7: sp[5] = indirect const u32 1
     8: sp[6] = u32 add sp[5], @2
     9: sp[7] = sp[6]
    10: store sp[2] at sp[7]
    11: sp[7] = u32 add sp[7], @2
    12: store sp[3] at sp[7]
    13: sp[7] = u32 add sp[7], @2
    14: store sp[4] at sp[7]
    15: sp[3] = u32 add sp[5], @2
    16: sp[4] = u32 add sp[3], sp[1]
    17: sp[2] = load sp[4]
    18: sp[1] = sp[2]
    19: return
    ");
}

// Tests setting an array element and retrieving it
#[test]
fn brillig_array_set() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v1 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v2 = array_set v1, index v0, value u32 99
        v3 = array_get v2, index v0 -> u32
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[2] = const u32 10
     2: sp[3] = const u32 20
     3: sp[4] = const u32 30
     4: sp[5] = @1
     5: sp[6] = const u32 4
     6: @1 = u32 add @1, sp[6]
     7: sp[5] = indirect const u32 1
     8: sp[6] = u32 add sp[5], @2
     9: sp[7] = sp[6]
    10: store sp[2] at sp[7]
    11: sp[7] = u32 add sp[7], @2
    12: store sp[3] at sp[7]
    13: sp[7] = u32 add sp[7], @2
    14: store sp[4] at sp[7]
    15: sp[2] = const u32 99
    16: @3 = sp[5]
    17: @4 = const u32 4
    18: call 0
    19: sp[3] = @5
    20: sp[4] = u32 add sp[3], @2
    21: sp[6] = u32 add sp[4], sp[1]
    22: store sp[2] at sp[6]
    23: sp[4] = u32 add sp[3], @2
    24: sp[5] = u32 add sp[4], sp[1]
    25: sp[2] = load sp[5]
    26: sp[1] = sp[2]
    27: return
    ");
}

// Tests array operations with reference counting inc_rc/dec_rc
#[test]
fn brillig_array_with_rc_ops() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        inc_rc v0
        v1 = array_set v0, index u32 0, value u32 99
        dec_rc v0
        v2 = array_get v1, index u32 0 -> u32
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[1] = const u32 10
     2: sp[2] = const u32 20
     3: sp[3] = const u32 30
     4: sp[4] = @1
     5: sp[5] = const u32 4
     6: @1 = u32 add @1, sp[5]
     7: sp[4] = indirect const u32 1
     8: sp[5] = u32 add sp[4], @2
     9: sp[6] = sp[5]
    10: store sp[1] at sp[6]
    11: sp[6] = u32 add sp[6], @2
    12: store sp[2] at sp[6]
    13: sp[6] = u32 add sp[6], @2
    14: store sp[3] at sp[6]
    15: sp[1] = load sp[4]
    16: sp[1] = u32 add sp[1], @2
    17: store sp[1] at sp[4]
    18: sp[1] = const u32 0
    19: sp[2] = const u32 99
    20: @3 = sp[4]
    21: @4 = const u32 4
    22: call 0
    23: sp[3] = @5
    24: sp[5] = u32 add sp[3], sp[1]
    25: store sp[2] at sp[5]
    26: sp[2] = load sp[4]
    27: sp[4] = u32 add sp[3], sp[1]
    28: sp[2] = load sp[4]
    29: sp[1] = sp[2]
    30: return
    ");
}

use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests AsSlice intrinsic code-gen for Brillig.
#[test]
fn brillig_as_slice() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_slice(v0) -> (u32, [u32])
        return v1
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
    15: sp[3] = const u32 3
    16: sp[1] = u32 div sp[3], @2
    17: sp[6] = const u32 3
    18: sp[5] = u32 add sp[3], sp[6]
    19: sp[2] = @1
    20: @1 = u32 add @1, sp[5]
    21: sp[2] = indirect const u32 1
    22: sp[5] = u32 add sp[2], @2
    23: store sp[3] at sp[5]
    24: sp[5] = u32 add sp[5], @2
    25: store sp[3] at sp[5]
    26: sp[6] = const u32 3
    27: sp[5] = u32 add sp[2], sp[6]
    28: sp[6] = u32 add sp[4], @2
    29: @3 = sp[6]
    30: @4 = sp[5]
    31: @5 = sp[3]
    32: call 0
    33: return
    ");
}

// Tests ToBits intrinsic code-gen for Brillig.
#[test]
fn brillig_to_bits() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field):
        v1 = call to_le_bits(v0) -> [u1; 8]
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[3] = const u32 2
     2: sp[4] = const bool 1
     3: sp[2] = @1
     4: sp[5] = const u32 9
     5: @1 = u32 add @1, sp[5]
     6: sp[2] = indirect const u32 1
     7: sp[5] = u32 add sp[2], @2
     8: sp[6] = const u32 8
     9: to_radix(input: sp[1], radix: sp[3], num_limbs: sp[6], output_pointer: sp[5], output_bits: sp[4])
    10: @3 = sp[5]
    11: @4 = sp[6]
    12: call 0
    13: sp[1] = sp[2]
    14: return
    ");
}

// Tests ToRadix intrinsic code-gen for Brillig.
#[test]
fn brillig_to_radix() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: u32):
        v2 = call to_le_radix(v0, v1) -> [u8; 8]
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[4] = const bool 0
     2: sp[3] = @1
     3: sp[5] = const u32 9
     4: @1 = u32 add @1, sp[5]
     5: sp[3] = indirect const u32 1
     6: sp[5] = u32 add sp[3], @2
     7: sp[6] = const u32 8
     8: to_radix(input: sp[1], radix: sp[2], num_limbs: sp[6], output_pointer: sp[5], output_bits: sp[4])
     9: @3 = sp[5]
    10: @4 = sp[6]
    11: call 0
    12: sp[1] = sp[3]
    13: return
    ");
}

// Tests FieldLessThan intrinsic code-gen for Brillig.
#[test]
fn brillig_field_less_than() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field):
        v2 = call field_less_than(v0, v1) -> u1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: call 0
    1: sp[3] = field lt sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
    ");
}

// Tests ArrayRefCount intrinsic code-gen for Brillig.
#[test]
fn brillig_array_ref_count() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1 = call array_refcount(v0) -> u32
        return v1
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
    16: return
    ");
}

// Tests SliceRefCount intrinsic code-gen for Brillig.
#[test]
fn brillig_slice_ref_count() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_slice(v0) -> (u32, [u32])
        v3 = call slice_refcount(v1, v2) -> u32
        return v3
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
    15: sp[3] = const u32 3
    16: sp[1] = u32 div sp[3], @2
    17: sp[6] = const u32 3
    18: sp[5] = u32 add sp[3], sp[6]
    19: sp[2] = @1
    20: @1 = u32 add @1, sp[5]
    21: sp[2] = indirect const u32 1
    22: sp[5] = u32 add sp[2], @2
    23: store sp[3] at sp[5]
    24: sp[5] = u32 add sp[5], @2
    25: store sp[3] at sp[5]
    26: sp[6] = const u32 3
    27: sp[5] = u32 add sp[2], sp[6]
    28: sp[6] = u32 add sp[4], @2
    29: @3 = sp[6]
    30: @4 = sp[5]
    31: @5 = sp[3]
    32: call 0
    33: sp[3] = load sp[2]
    34: sp[1] = sp[3]
    35: return
    ");
}

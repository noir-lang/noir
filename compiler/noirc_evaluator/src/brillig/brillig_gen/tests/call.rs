use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests AsVector intrinsic code-gen for Brillig.
#[test]
fn brillig_as_vector() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_vector(v0) -> (u32, [u32])
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[1] = @1
     2: @3 = const u32 2048
     3: @1 = u32 add @1, @3
     4: sp[2] = const u32 10
     5: sp[3] = const u32 20
     6: sp[4] = const u32 30
     7: sp[5] = @1
     8: sp[6] = const u32 4
     9: @1 = u32 add @1, sp[6]
    10: sp[5] = indirect const u32 1
    11: sp[6] = u32 add sp[5], @2
    12: sp[7] = sp[6]
    13: store sp[2] at sp[7]
    14: sp[7] = u32 add sp[7], @2
    15: store sp[3] at sp[7]
    16: sp[7] = u32 add sp[7], @2
    17: store sp[4] at sp[7]
    18: sp[4] = const u32 3
    19: sp[2] = u32 div sp[4], @2
    20: sp[7] = const u32 3
    21: sp[6] = u32 add sp[4], sp[7]
    22: sp[3] = @1
    23: @1 = u32 add @1, sp[6]
    24: sp[3] = indirect const u32 1
    25: sp[6] = u32 add sp[3], @2
    26: store sp[4] at sp[6]
    27: sp[6] = u32 add sp[6], @2
    28: store sp[4] at sp[6]
    29: sp[7] = const u32 3
    30: sp[6] = u32 add sp[3], sp[7]
    31: sp[7] = u32 add sp[5], @2
    32: @3 = sp[7]
    33: @4 = sp[6]
    34: @5 = sp[4]
    35: call 0
    36: return
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
     1: sp[1] = @1
     2: @3 = const u32 2048
     3: @1 = u32 add @1, @3
     4: sp[4] = const u32 2
     5: sp[5] = const bool 1
     6: sp[3] = @1
     7: sp[6] = const u32 9
     8: @1 = u32 add @1, sp[6]
     9: sp[3] = indirect const u32 1
    10: sp[6] = u32 add sp[3], @2
    11: sp[7] = const u32 8
    12: to_radix(input: sp[2], radix: sp[4], num_limbs: sp[7], output_pointer: sp[6], output_bits: sp[5])
    13: @3 = sp[6]
    14: @4 = sp[7]
    15: call 0
    16: sp[2] = sp[3]
    17: return
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
     1: sp[1] = @1
     2: @3 = const u32 2048
     3: @1 = u32 add @1, @3
     4: sp[5] = const bool 0
     5: sp[4] = @1
     6: sp[6] = const u32 9
     7: @1 = u32 add @1, sp[6]
     8: sp[4] = indirect const u32 1
     9: sp[6] = u32 add sp[4], @2
    10: sp[7] = const u32 8
    11: to_radix(input: sp[2], radix: sp[3], num_limbs: sp[7], output_pointer: sp[6], output_bits: sp[5])
    12: @3 = sp[6]
    13: @4 = sp[7]
    14: call 0
    15: sp[2] = sp[4]
    16: return
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
    1: sp[1] = @1
    2: @3 = const u32 2048
    3: @1 = u32 add @1, @3
    4: sp[4] = field lt sp[2], sp[3]
    5: sp[2] = sp[4]
    6: return
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
     1: sp[1] = @1
     2: @3 = const u32 2048
     3: @1 = u32 add @1, @3
     4: sp[2] = const u32 10
     5: sp[3] = const u32 20
     6: sp[4] = const u32 30
     7: sp[5] = @1
     8: sp[6] = const u32 4
     9: @1 = u32 add @1, sp[6]
    10: sp[5] = indirect const u32 1
    11: sp[6] = u32 add sp[5], @2
    12: sp[7] = sp[6]
    13: store sp[2] at sp[7]
    14: sp[7] = u32 add sp[7], @2
    15: store sp[3] at sp[7]
    16: sp[7] = u32 add sp[7], @2
    17: store sp[4] at sp[7]
    18: sp[2] = load sp[5]
    19: return
    ");
}

// Tests VectorRefCount intrinsic code-gen for Brillig.
#[test]
fn brillig_vector_ref_count() {
    let src = "
    brillig(inline) fn foo f0 {
      b0():
        v0 = make_array [u32 10, u32 20, u32 30] : [u32; 3]
        v1, v2 = call as_vector(v0) -> (u32, [u32])
        v3 = call vector_refcount(v1, v2) -> u32
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[1] = @1
     2: @3 = const u32 2048
     3: @1 = u32 add @1, @3
     4: sp[2] = const u32 10
     5: sp[3] = const u32 20
     6: sp[4] = const u32 30
     7: sp[5] = @1
     8: sp[6] = const u32 4
     9: @1 = u32 add @1, sp[6]
    10: sp[5] = indirect const u32 1
    11: sp[6] = u32 add sp[5], @2
    12: sp[7] = sp[6]
    13: store sp[2] at sp[7]
    14: sp[7] = u32 add sp[7], @2
    15: store sp[3] at sp[7]
    16: sp[7] = u32 add sp[7], @2
    17: store sp[4] at sp[7]
    18: sp[4] = const u32 3
    19: sp[2] = u32 div sp[4], @2
    20: sp[7] = const u32 3
    21: sp[6] = u32 add sp[4], sp[7]
    22: sp[3] = @1
    23: @1 = u32 add @1, sp[6]
    24: sp[3] = indirect const u32 1
    25: sp[6] = u32 add sp[3], @2
    26: store sp[4] at sp[6]
    27: sp[6] = u32 add sp[6], @2
    28: store sp[4] at sp[6]
    29: sp[7] = const u32 3
    30: sp[6] = u32 add sp[3], sp[7]
    31: sp[7] = u32 add sp[5], @2
    32: @3 = sp[7]
    33: @4 = sp[6]
    34: @5 = sp[4]
    35: call 0
    36: sp[4] = load sp[3]
    37: sp[2] = sp[4]
    38: return
    ");
}

use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Check that CheckMaxStackDepth is emitted for recursive functions and those reachable from them.
#[test]
fn brillig_check_max_stack_depth() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32):
        v2 = lt v0, u32 10
        jmpif v2 then: b1(), else: b2()
      b1():
        v5 = add v0, u32 1
        call f0(v5)
        jmp b3()
      b2():
        call f1()
        jmp b3()
      b3():
        return
    }
    brillig(inline) fn foo f1 {
      b0():
        call f2()
        return
    }
    brillig(inline) fn bar f2 {
      b0():
        return
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let main = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(main, @r"
    fn main
     0: call 0 // -> CheckMaxStackDepth
     1: sp[3] = u32 lt sp[2], @68
     2: jump if sp[3] to 0 // -> 10: f0/b1
     3: jump to 0 // -> 4: f0/b2
     4: sp[2] = const u32 3 // f0/b2
     5: sp[3] = @0
     6: @0 = u32 add @0, sp[2]
     7: call 0 // -> f1
     8: @0 = sp[0]
     9: jump to 0 // -> 21: f0/b3
    10: sp[3] = u32 add sp[2], @67 // f0/b1
    11: sp[4] = u32 lt_eq sp[2], sp[3]
    12: jump if sp[4] to 0 // -> 14: f0/b1/1
    13: call 0 // -> ErrorWithString
    14: sp[2] = const u32 4 // f0/b1/1
    15: sp[4] = @0
    16: sp[6] = sp[3]
    17: @0 = u32 add @0, sp[2]
    18: call 0 // -> 0: f0
    19: @0 = sp[0]
    20: jump to 0 // -> 21: f0/b3
    21: return // f0/b3
    ");

    let bar = &brillig.ssa_function_to_brillig[&Id::test_new(2)];
    assert_artifact_snapshot!(bar, @r"
    fn bar
    0: call 0 // -> CheckMaxStackDepth
    1: return
    ");
}

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
     0: sp[2] = const u32 10
     1: sp[3] = const u32 20
     2: sp[4] = const u32 30
     3: sp[5] = @1
     4: sp[6] = const u32 4
     5: @1 = u32 add @1, sp[6]
     6: sp[5] = indirect const u32 1
     7: sp[6] = u32 add sp[5], @2
     8: sp[7] = sp[6]
     9: store sp[2] at sp[7]
    10: sp[7] = u32 add sp[7], @2
    11: store sp[3] at sp[7]
    12: sp[7] = u32 add sp[7], @2
    13: store sp[4] at sp[7]
    14: sp[4] = const u32 3
    15: sp[2] = u32 div sp[4], @2
    16: sp[7] = const u32 3
    17: sp[6] = u32 add sp[4], sp[7]
    18: sp[3] = @1
    19: @1 = u32 add @1, sp[6]
    20: sp[3] = indirect const u32 1
    21: sp[6] = u32 add sp[3], @2
    22: store sp[4] at sp[6]
    23: sp[6] = u32 add sp[6], @2
    24: store sp[4] at sp[6]
    25: sp[7] = const u32 3
    26: sp[6] = u32 add sp[3], sp[7]
    27: sp[7] = u32 add sp[5], @2
    28: @3 = sp[7]
    29: @4 = sp[6]
    30: @5 = sp[4]
    31: call 0 // -> MemCopy
    32: return
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
     0: sp[4] = const u32 2
     1: sp[5] = const bool 1
     2: sp[3] = @1
     3: sp[6] = const u32 9
     4: @1 = u32 add @1, sp[6]
     5: sp[3] = indirect const u32 1
     6: sp[6] = u32 add sp[3], @2
     7: sp[7] = const u32 8
     8: to_radix(input: sp[2], radix: sp[4], num_limbs: sp[7], output_pointer: sp[6], output_bits: sp[5])
     9: @3 = sp[6]
    10: @4 = sp[7]
    11: call 0 // -> ArrayReverse
    12: sp[2] = sp[3]
    13: return
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
     0: sp[5] = const bool 0
     1: sp[4] = @1
     2: sp[6] = const u32 9
     3: @1 = u32 add @1, sp[6]
     4: sp[4] = indirect const u32 1
     5: sp[6] = u32 add sp[4], @2
     6: sp[7] = const u32 8
     7: to_radix(input: sp[2], radix: sp[3], num_limbs: sp[7], output_pointer: sp[6], output_bits: sp[5])
     8: @3 = sp[6]
     9: @4 = sp[7]
    10: call 0 // -> ArrayReverse
    11: sp[2] = sp[4]
    12: return
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
    0: sp[4] = field lt sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
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
     0: sp[2] = const u32 10
     1: sp[3] = const u32 20
     2: sp[4] = const u32 30
     3: sp[5] = @1
     4: sp[6] = const u32 4
     5: @1 = u32 add @1, sp[6]
     6: sp[5] = indirect const u32 1
     7: sp[6] = u32 add sp[5], @2
     8: sp[7] = sp[6]
     9: store sp[2] at sp[7]
    10: sp[7] = u32 add sp[7], @2
    11: store sp[3] at sp[7]
    12: sp[7] = u32 add sp[7], @2
    13: store sp[4] at sp[7]
    14: sp[2] = load sp[5]
    15: return
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
     0: sp[2] = const u32 10
     1: sp[3] = const u32 20
     2: sp[4] = const u32 30
     3: sp[5] = @1
     4: sp[6] = const u32 4
     5: @1 = u32 add @1, sp[6]
     6: sp[5] = indirect const u32 1
     7: sp[6] = u32 add sp[5], @2
     8: sp[7] = sp[6]
     9: store sp[2] at sp[7]
    10: sp[7] = u32 add sp[7], @2
    11: store sp[3] at sp[7]
    12: sp[7] = u32 add sp[7], @2
    13: store sp[4] at sp[7]
    14: sp[4] = const u32 3
    15: sp[2] = u32 div sp[4], @2
    16: sp[7] = const u32 3
    17: sp[6] = u32 add sp[4], sp[7]
    18: sp[3] = @1
    19: @1 = u32 add @1, sp[6]
    20: sp[3] = indirect const u32 1
    21: sp[6] = u32 add sp[3], @2
    22: store sp[4] at sp[6]
    23: sp[6] = u32 add sp[6], @2
    24: store sp[4] at sp[6]
    25: sp[7] = const u32 3
    26: sp[6] = u32 add sp[3], sp[7]
    27: sp[7] = u32 add sp[5], @2
    28: @3 = sp[7]
    29: @4 = sp[6]
    30: @5 = sp[4]
    31: call 0 // -> MemCopy
    32: sp[4] = load sp[3]
    33: sp[2] = sp[4]
    34: return
    ");
}

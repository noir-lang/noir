use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests Brillig u32 addition code-gen. It includes overflow check.
#[test]
fn brillig_add() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = add v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 add sp[2], sp[3]
    1: sp[5] = u32 lt_eq sp[2], sp[4]
    2: jump if sp[5] to 0 // -> 4: f0/b0/1
    3: call 0 // -> ErrorWithString
    4: sp[2] = sp[4] // f0/b0/1
    5: return
    ");
}

// Tests Brillig u32 subtraction code-gen. It includes underflow check
#[test]
fn brillig_sub() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = sub v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 sub sp[2], sp[3]
    1: sp[5] = u32 lt_eq sp[3], sp[2]
    2: jump if sp[5] to 0 // -> 4: f0/b0/1
    3: call 0 // -> ErrorWithString
    4: sp[2] = sp[4] // f0/b0/1
    5: return
    ");
}

// Tests Brillig u32 multiplication code-gen. It includes overflow check
#[test]
fn brillig_mul() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = mul v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];

    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[4] = u32 mul sp[2], sp[3]
     1: sp[6] = const u32 0
     2: sp[5] = u32 eq sp[6], sp[3]
     3: jump if sp[5] to 0 // -> 8: f0/b0/1
     4: sp[8] = u32 div sp[4], sp[3]
     5: sp[7] = u32 eq sp[8], sp[2]
     6: jump if sp[7] to 0 // -> 8: f0/b0/2
     7: call 0 // -> ErrorWithString
     8: sp[2] = sp[4] // f0/b0/1, f0/b0/2
     9: return
    ");
}

// Tests Brillig u32 division code-gen.
#[test]
fn brillig_div() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = div v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 div sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 modulo operation code-gen.
#[test]
fn brillig_mod() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = mod v0, v1
        return v2
    }
    ";
    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[6] = const u32 0
    1: sp[5] = u32 lt sp[6], sp[3]
    2: jump if sp[5] to 0 // -> 4: f0/b0/1
    3: call 0 // -> ErrorWithString
    4: sp[5] = u32 div sp[2], sp[3] // f0/b0/1
    5: sp[6] = u32 mul sp[5], sp[3]
    6: sp[4] = u32 sub sp[2], sp[6]
    7: sp[2] = sp[4]
    8: return
    ");
}

// A known non-zero constant divisor skips the divisor-zero pre-check.
#[test]
fn brillig_mod_constant_rhs() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v2 = mod v0, u32 3
        return v2
    }
    ";
    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = const u32 3
    1: sp[5] = u32 div sp[2], sp[3]
    2: sp[6] = u32 mul sp[5], sp[3]
    3: sp[4] = u32 sub sp[2], sp[6]
    4: sp[2] = sp[4]
    5: return
    ");
}

// Tests Brillig u32 equality comparison code-gen.
#[test]
fn brillig_eq() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 eq sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 less than comparison code-gen.
#[test]
fn brillig_lt() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = lt v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 lt sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 bitwise AND code-gen.
#[test]
fn brillig_and() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = and v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 and sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 bitwise OR code-gen.
#[test]
fn brillig_or() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = or v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 or sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 bitwise XOR code-gen.
#[test]
fn brillig_xor() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = xor v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = u32 xor sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig u32 left shift code-gen.
#[test]
fn brillig_shl() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = shl v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[6] = const u32 32
    1: sp[5] = u32 lt sp[3], sp[6]
    2: jump if sp[5] to 0 // -> 4: f0/b0/1
    3: call 0 // -> ErrorWithString
    4: sp[4] = u32 shl sp[2], sp[3] // f0/b0/1
    5: sp[2] = sp[4]
    6: return
    ");
}

// Tests Brillig u32 right shift code-gen.
#[test]
fn brillig_shr() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32, v1: u32):
        v2 = shr v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[6] = const u32 32
    1: sp[5] = u32 lt sp[3], sp[6]
    2: jump if sp[5] to 0 // -> 4: f0/b0/1
    3: call 0 // -> ErrorWithString
    4: sp[4] = u32 shr sp[2], sp[3] // f0/b0/1
    5: sp[2] = sp[4]
    6: return
    ");
}

// A constant shift amount below the bit size needs no runtime overflow check.
#[test]
fn brillig_shift_by_constant_in_range() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u128):
        v1 = shl v0, u128 64
        v2 = shr v0, u128 127
        v3 = add v1, v2
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = const u128 64
    1: sp[4] = u128 shl sp[2], sp[3]
    2: sp[3] = const u128 127
    3: sp[5] = u128 shr sp[2], sp[3]
    4: sp[2] = u128 add sp[4], sp[5]
    5: sp[3] = u128 lt_eq sp[4], sp[2]
    6: jump if sp[3] to 0 // -> 8: f0/b0/1
    7: call 0 // -> ErrorWithString
    8: return // f0/b0/1
    ");
}

// A constant shift amount at or above the bit size keeps the runtime overflow check.
#[test]
fn brillig_shift_by_constant_out_of_range() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: u32):
        v1 = shl v0, u32 32
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = const u32 32
    1: sp[6] = const u32 32
    2: sp[5] = u32 lt sp[3], sp[6]
    3: jump if sp[5] to 0 // -> 5: f0/b0/1
    4: call 0 // -> ErrorWithString
    5: sp[4] = u32 shl sp[2], sp[3] // f0/b0/1
    6: sp[2] = sp[4]
    7: return
    ");
}

// A negative signed constant shift amount keeps the runtime overflow check.
#[test]
fn brillig_shift_by_negative_signed_constant() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: i8):
        v1 = shl v0, i8 -1
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = const u8 255
    1: sp[6] = const u8 8
    2: sp[5] = u8 lt sp[3], sp[6]
    3: jump if sp[5] to 0 // -> 5: f0/b0/1
    4: call 0 // -> ErrorWithString
    5: sp[4] = u8 shl sp[2], sp[3] // f0/b0/1
    6: sp[2] = sp[4]
    7: return
    ");
}

// Tests Brillig Field addition.
#[test]
fn brillig_add_field() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = field add sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig Field subtraction.
#[test]
fn brillig_sub_field() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field):
        v2 = sub v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = field sub sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig Field multiplication
#[test]
fn brillig_mul_field() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field):
        v2 = mul v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = field mul sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

// Tests Brillig Field division
#[test]
fn brillig_div_field() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field):
        v2 = div v0, v1
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[4] = field field_div sp[2], sp[3]
    1: sp[2] = sp[4]
    2: return
    ");
}

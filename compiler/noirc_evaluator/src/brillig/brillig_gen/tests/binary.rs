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
    0: call 0
    1: sp[3] = u32 add sp[1], sp[2]
    2: sp[4] = u32 lt_eq sp[1], sp[3]
    3: jump if sp[4] to 0
    4: call 0
    5: sp[1] = sp[3]
    6: return
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
    0: call 0
    1: sp[3] = u32 sub sp[1], sp[2]
    2: sp[4] = u32 lt_eq sp[2], sp[1]
    3: jump if sp[4] to 0
    4: call 0
    5: sp[1] = sp[3]
    6: return
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
     0: call 0
     1: sp[3] = u32 mul sp[1], sp[2]
     2: sp[5] = const u32 0
     3: sp[4] = u32 eq sp[5], sp[2]
     4: jump if sp[4] to 0
     5: sp[7] = u32 div sp[3], sp[2]
     6: sp[6] = u32 eq sp[7], sp[1]
     7: jump if sp[6] to 0
     8: call 0
     9: sp[1] = sp[3]
    10: return
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
    0: call 0
    1: sp[3] = u32 div sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[4] = u32 div sp[1], sp[2]
    2: sp[5] = u32 mul sp[4], sp[2]
    3: sp[3] = u32 sub sp[1], sp[5]
    4: sp[1] = sp[3]
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
    0: call 0
    1: sp[3] = u32 eq sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 lt sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 and sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 or sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 xor sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 shl sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = u32 shr sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = field add sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = field sub sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = field mul sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
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
    0: call 0
    1: sp[3] = field field_div sp[1], sp[2]
    2: sp[1] = sp[3]
    3: return
    ");
}

use acvm::{acir::circuit::Opcode, assert_circuit_snapshot};

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn array_set_not_mutable() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = array_get v0, index v1 -> Field
        v4 = array_set v0, index v1, value v2
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // Note how the non-mutable array_set ends up using a different block (b1)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT b0 = [w0, w1, w2]
    READ w8 = b0[w3]
    INIT b1 = [w0, w1, w2]
    WRITE b1[w3] = w4
    ASSERT w9 = 0
    READ w10 = b1[w9]
    ASSERT w11 = 1
    READ w12 = b1[w11]
    ASSERT w13 = 2
    READ w14 = b1[w13]
    ASSERT w5 = w10
    ASSERT w6 = w12
    ASSERT w7 = w14
    ");
}

#[test]
fn array_set_mutable() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = array_get v0, index v1 -> Field
        v4 = array_set mut v0, index v1, value v2
        return v4
    }
    ";
    let program = ssa_to_acir_program(src);

    // Now how the mutable array_set ends up using the same block (b0)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT b0 = [w0, w1, w2]
    READ w8 = b0[w3]
    WRITE b0[w3] = w4
    ASSERT w9 = 0
    READ w10 = b0[w9]
    ASSERT w11 = 1
    READ w12 = b0[w11]
    ASSERT w13 = 2
    READ w14 = b0[w13]
    ASSERT w5 = w10
    ASSERT w6 = w12
    ASSERT w7 = w14
    ");
}

#[test]
fn does_not_generate_memory_blocks_without_dynamic_accesses() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2, v3 = call as_list(v0) -> (u32, [Field])
            call f1(u32 2, v3)
            v7 = array_get v0, index u32 0 -> Field
            constrain v7 == Field 0
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: [Field]):
              return
          }
        ";
    let program = ssa_to_acir_program(src);

    // Check that no memory opcodes were emitted.
    assert_eq!(program.functions.len(), 1);
    for opcode in &program.functions[0].opcodes {
        assert!(!matches!(opcode, Opcode::MemoryInit { .. }));
    }
}

#[test]
fn constant_array_access_out_of_bounds() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v2 = make_array [Field 0, Field 1] : [Field; 2]
        v4 = array_get v2, index u32 5 -> Field
        constrain v4 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // We expect a constant array access that is out of bounds (OOB) to be deferred to the runtime.
    // This means memory checks will be laid down and array access OOB checks will be handled there.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ASSERT w0 = 0
    ASSERT w1 = 1
    INIT b0 = [w0, w1]
    ASSERT w2 = 5
    READ w3 = b0[w2]
    ASSERT w3 = 0
    ");
}

#[test]
fn constant_array_access_in_bounds() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v2 = make_array [Field 0, Field 1] : [Field; 2]
        v4 = array_get v2, index u32 0 -> Field
        constrain v4 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // We know the circuit above to be trivially true
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].opcodes.len(), 0);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ");
}

#[test]
fn generates_memory_op_for_dynamic_read() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = array_get v0, index v1 -> Field
        constrain v2 == Field 10
        return
    }
    ";

    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: []
    INIT b0 = [w0, w1, w2]
    READ w4 = b0[w3]
    ASSERT w4 = 10
    ");
}

#[test]
fn generates_memory_op_for_dynamic_write() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = array_set v0, index v1, value Field 10
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // All logic after the write is expected as we generate new witnesses for return values
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: [w4, w5, w6]
    INIT b1 = [w0, w1, w2]
    ASSERT w7 = 10
    WRITE b1[w3] = w7
    ASSERT w8 = 0
    READ w9 = b1[w8]
    ASSERT w10 = 1
    READ w11 = b1[w10]
    ASSERT w12 = 2
    READ w13 = b1[w12]
    ASSERT w4 = w9
    ASSERT w5 = w11
    ASSERT w6 = w13
    ");
}

#[test]
fn generates_predicated_index_for_dynamic_read() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, predicate: bool):
        enable_side_effects predicate
        v3 = array_get v0, index v1 -> Field
        constrain v3 == Field 10
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // w0, w1, w2 represents the array
    // So w3 represents our index and w4 is our predicate
    // We can see that before the read we have `w3*w4 - w5 = 0`
    // As the index is zero this is a simplified version of `index*predicate + (1-predicate)*offset`
    // w5 is then used as the index which we use to read from the memory block
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    INIT b0 = [w0, w1, w2]
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 1
    ASSERT w5 = w3*w4
    READ w6 = b0[w5]
    ASSERT w6 = 10
    ");
}

#[test]
fn generates_predicated_index_and_dummy_value_for_dynamic_write() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, predicate: bool):
        enable_side_effects predicate
        v3 = array_set v0, index v1, value Field 10
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    // Similar to the `generates_predicated_index_for_dynamic_read` test we can
    // see how `w3*w4 - w8 = 0` forms our predicated index.
    // However, now we also have extra logic for generating a dummy value.
    // The original value we want to write is `Field 10` and our predicate is `w4`.
    // We read the value at the predicated index into `w9`. This is our dummy value.
    // We can then see how we form our new store value with:
    // `ASSERT -w4*w9 + 10*w4 + w9 - w10 = 0` -> (predicate*value + (1-predicate)*dummy)
    // `10*w4` -> predicate*value
    // `-w4*w9` -> (-predicate * dummy)
    // `w9` -> dummy
    // As expected, we then store `w10` at the predicated index `w8`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: [w5, w6, w7]
    INIT b0 = [w0, w1, w2]
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 1
    ASSERT w8 = w3*w4
    READ w9 = b0[w8]
    INIT b1 = [w0, w1, w2]
    ASSERT w10 = -w4*w9 + 10*w4 + w9
    WRITE b1[w8] = w10
    ASSERT w11 = 0
    READ w12 = b1[w11]
    ASSERT w13 = 1
    READ w14 = b1[w13]
    ASSERT w15 = 2
    READ w16 = b1[w15]
    ASSERT w5 = w12
    ASSERT w6 = w14
    ASSERT w7 = w16
    ");
}

#[test]
fn zero_length_array_constant() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v0 = make_array [] : [Field; 0]
        v2 = array_get v0, index u32 0 -> Field
        constrain v2 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // As we have a constant array the constraint we insert will be simplified down.
    // We expect ever expression to equal zero when executed. Thus, this circuit will always fail.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: []
    public parameters: []
    return values: []
    ASSERT 0 = 1
    ");
}

#[test]
fn zero_length_array_dynamic_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(predicate: bool):
        enable_side_effects predicate
        v0 = make_array [] : [Field; 0]
        v2 = array_get v0, index u32 0 -> Field
        constrain v2 == Field 0
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // Similar to the `zero_length_array_constant` test we inserted an always failing constraint
    // when an array access is attempted on a zero length array.
    // However, we must gate it by the predicate in case the branch is inactive.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    ASSERT w0 = 0
    ");
}

/// Tests this code:
/// ```noir
/// struct Bar {
///     inner: [Field; 3],
/// }
/// struct Foo {
///     a: Field,
///     b: [Field; 3],
///     bar: Bar,
/// }
/// fn main(x: [Foo; 4], index: u32) -> pub [Field; 3] {
///     x[index].bar.inner
/// }
/// ```
#[test]
fn non_homogenous_array_dynamic_access() {
    let src = r#"
    acir(inline) pure fn main f0 {
      b0(v0: [(Field, [Field; 3], [Field; 3]); 4], v1: u32):
        v2 = array_get v0, index v1 -> [Field; 3]
        return v2
    }
    "#;

    let program = ssa_to_acir_program(src);

    // b0 is our actual array input while b1 is our element type sizes array.
    // You can see that in `w44 = b1[w28]` we use the supplied witness index to read the flattened index from b1.
    // `w44` is then used to read from the b0 array.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28]
    public parameters: []
    return values: [w29, w30, w31]
    INIT b0 = [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27]
    ASSERT w32 = 0
    ASSERT w33 = 1
    ASSERT w34 = 4
    ASSERT w35 = 7
    ASSERT w36 = 8
    ASSERT w37 = 11
    ASSERT w38 = 14
    ASSERT w39 = 15
    ASSERT w40 = 18
    ASSERT w41 = 21
    ASSERT w42 = 22
    ASSERT w43 = 25
    ASSERT w44 = 28
    ASSERT w45 = 29
    ASSERT w46 = 32
    INIT b1 = [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46]
    READ w47 = b1[w28]
    READ w48 = b0[w47]
    ASSERT w49 = w47 + 1
    READ w50 = b0[w49]
    ASSERT w51 = w49 + 1
    READ w52 = b0[w51]
    ASSERT w29 = w48
    ASSERT w30 = w50
    ASSERT w31 = w52
    ");
}

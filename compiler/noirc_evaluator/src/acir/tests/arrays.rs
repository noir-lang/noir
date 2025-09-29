use acvm::{acir::circuit::Opcode, assert_circuit_snapshot};

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn does_not_generate_memory_blocks_without_dynamic_accesses() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
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
    EXPR w0 = 0
    EXPR w1 = 1
    INIT b0 = [w0, w1]
    EXPR w2 = 5
    READ w3 = b0[w2]
    EXPR w3 = 0
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
    EXPR w4 = 10
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
    EXPR w7 = 10
    WRITE b1[w3] = w7
    EXPR w8 = 0
    READ w9 = b1[w8]
    EXPR w10 = 1
    READ w11 = b1[w10]
    EXPR w12 = 2
    READ w13 = b1[w12]
    EXPR w9 = w4
    EXPR w11 = w5
    EXPR w13 = w6
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
    EXPR w5 = w3*w4
    READ w6 = b0[w5]
    EXPR w6 = 10
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
    // `EXPR -w4*w9 + 10*w4 + w9 - w10 = 0` -> (predicate*value + (1-predicate)*dummy)
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
    EXPR w8 = w3*w4
    READ w9 = b0[w8]
    INIT b1 = [w0, w1, w2]
    EXPR w10 = -w4*w9 + 10*w4 + w9
    WRITE b1[w8] = w10
    EXPR w11 = 0
    READ w12 = b1[w11]
    EXPR w13 = 1
    READ w14 = b1[w13]
    EXPR w15 = 2
    READ w16 = b1[w15]
    EXPR w12 = w5
    EXPR w14 = w6
    EXPR w16 = w7
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
    EXPR 0 = 1
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
    EXPR w0 = 0
    ");
}

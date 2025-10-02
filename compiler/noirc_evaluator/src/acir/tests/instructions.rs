use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

mod binary;

#[test]
fn constrain_equal() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        constrain v0 == v1
        return
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    ASSERT w1 = w0
    ");
}

#[test]
fn constrain_not_equal() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        constrain v0 != v1
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    // `0 = w0*w2 - w1*w2 - 1` is `0 = (w0 - w1)*w2 - 1`, then `1 = (w0 - w1)*w2` and finally
    // `w2 = 1/(w0 - w1)`. This means `w0 - w1` must not be zero, `w0 - w1 != 0` which is
    // the same as `w0 != w1`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [w0 - w1], outputs: [w2]
    ASSERT 0 = w0*w2 - w1*w2 - 1

    unconstrained func 0: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    ");
}

#[test]
fn cast() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8):
        v1 = cast v0 as Field
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // cast is a no-op
    // (casting in Noir might also involve a truncate operation if it's casting from
    // a larger integer type to a smaller one, but casting in SSA is just changing types)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 8
    ASSERT w1 = w0
    ");
}

#[test]
fn call_without_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = call f1(v0) -> Field
        return v1
    }

    acir(fold) fn one f1 {
      b0(v0: Field):
        v1 = add v0, Field 1
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    CALL func: 1, predicate: 1, inputs: [w0], outputs: [w2]
    ASSERT w2 = w1

    func 1
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w1 = w0 + 1
    ");
}

#[test]
fn call_with_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: u1):
        enable_side_effects v1
        v2 = call f1(v0) -> Field
        return v2
    }

    acir(fold) fn one f1 {
      b0(v0: Field):
        v1 = add v0, Field 1
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w1, bits: 1
    CALL func: 1, predicate: w1, inputs: [w0], outputs: [w3]
    ASSERT w3 = w2

    func 1
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w1 = w0 + 1
    ");
}

#[test]
fn not() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8):
        v1 = not v0
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 8
    ASSERT w1 = -w0 + 255
    ");
}

#[test]
fn truncate_u16_to_6_bits() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u16):
        v1 = truncate v0 to 6 bits, max_bit_size: 8
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // Truncating to 6 bits is the same as dividing by 2^6 = 64
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 16
    BRILLIG CALL func: 0, inputs: [w0, 64], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 2
    BLACKBOX::RANGE input: w3, bits: 6
    ASSERT w3 = w0 - 64*w2
    ASSERT w3 = w1

    unconstrained func 0: directive_integer_quotient
    0: @10 = const u32 2
    1: @11 = const u32 0
    2: @0 = calldata copy [@11; @10]
    3: @2 = field int_div @0, @1
    4: @1 = field mul @2, @1
    5: @1 = field sub @0, @1
    6: @0 = @2
    7: stop &[@11; @10]
    ");
}

#[test]
fn truncate_field_to_6_bits() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 6 bits, max_bit_size: 254
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // Truncating to 8 bits is the same as dividing by 2^6 = 64.
    // TODO: explain what overflow we try to avoid in the rest of the opcodes
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BRILLIG CALL func: 0, inputs: [w0, 64], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 248
    BLACKBOX::RANGE input: w3, bits: 6
    ASSERT w3 = w0 - 64*w2
    ASSERT w4 = -w2 + 342003794872488675347600089769644923258568193756500536620284440415247007744
    BLACKBOX::RANGE input: w4, bits: 248
    BRILLIG CALL func: 1, inputs: [-w2 + 342003794872488675347600089769644923258568193756500536620284440415247007744], outputs: [w5]
    ASSERT w6 = w2*w5 - 342003794872488675347600089769644923258568193756500536620284440415247007744*w5 + 1
    ASSERT 0 = -w2*w6 + 342003794872488675347600089769644923258568193756500536620284440415247007744*w6
    ASSERT w7 = w3*w6
    BLACKBOX::RANGE input: w7, bits: 0
    ASSERT w3 = w1

    unconstrained func 0: directive_integer_quotient
    0: @10 = const u32 2
    1: @11 = const u32 0
    2: @0 = calldata copy [@11; @10]
    3: @2 = field int_div @0, @1
    4: @1 = field mul @2, @1
    5: @1 = field sub @0, @1
    6: @0 = @2
    7: stop &[@11; @10]
    unconstrained func 1: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    ");
}

#[test]
fn range_check() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        range_check v0 to 8 bits
        return
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    ");
}

#[test]
fn make_array() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = make_array [v0, v1, Field 10] : [Field; 3]
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2, w3, w4]
    ASSERT w2 = w0
    ASSERT w3 = w1
    ASSERT w4 = 10
    ");
}

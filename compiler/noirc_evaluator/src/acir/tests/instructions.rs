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
        v1 = truncate v0 to 6 bits, max_bit_size: 16
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // Truncating to 6 bits is the same as taking the remainder of dividing by 2^6 = 64
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 16
    BRILLIG CALL func: 0, inputs: [w0, 64], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 10
    BLACKBOX::RANGE input: w3, bits: 6
    ASSERT w3 = w0 - 64*w2
    ASSERT w1 = w3

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
    //
    // There's more ACIR opcodes here compared to the u16 case as we need to ensure
    // `q*b+r < 2^max_q_bits*2^max_rhs_bits`.
    //
    // Refer to the final check done in `euclidean_division_var`.
    //
    // Starting from `BLACKBOX::RANGE input: w4, bits: 248` and up to `ASSERT 0 = -w2*w6 + ...`
    // is a set up to check that `q = q0`.
    //
    // `ASSERT w7 = w3*w6` and the following range check is the actual binding of the remainder
    // with `bound_constraint_with_offset`.
    //
    // The last opcode is the generation of the return witness.
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
    ASSERT 0 = w3*w6
    ASSERT w1 = w3

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
fn truncate_field_to_64_bits() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 64 bits, max_bit_size: 254
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BRILLIG CALL func: 0, inputs: [w0, 18446744073709551616], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 190
    BLACKBOX::RANGE input: w3, bits: 64
    ASSERT w3 = w0 - 18446744073709551616*w2
    ASSERT w4 = -w2 + 1186564023676924939888766319973246049704924238154051448977
    BLACKBOX::RANGE input: w4, bits: 190
    BRILLIG CALL func: 1, inputs: [-w2 + 1186564023676924939888766319973246049704924238154051448977], outputs: [w5]
    ASSERT w6 = w2*w5 - 1186564023676924939888766319973246049704924238154051448977*w5 + 1
    ASSERT 0 = -w2*w6 + 1186564023676924939888766319973246049704924238154051448977*w6
    ASSERT w7 = w3*w6 + 4331911350818177023*w6
    BLACKBOX::RANGE input: w7, bits: 63
    ASSERT w1 = w3

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
fn truncate_field_to_128_bits() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 128 bits, max_bit_size: 254
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BRILLIG CALL func: 0, inputs: [w0, 340282366920938463463374607431768211456], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 126
    BLACKBOX::RANGE input: w3, bits: 128
    ASSERT w4 = -w3 + 340282366920938463463374607431768211455
    BLACKBOX::RANGE input: w4, bits: 128
    ASSERT w3 = w0 - 340282366920938463463374607431768211456*w2
    ASSERT w5 = w2 + 20746827117051438823981594372716013474
    BLACKBOX::RANGE input: w5, bits: 126
    BRILLIG CALL func: 1, inputs: [-w2 + 64323764613183177041862057485226039389], outputs: [w6]
    ASSERT w7 = w2*w6 - 64323764613183177041862057485226039389*w6 + 1
    ASSERT 0 = -w2*w7 + 64323764613183177041862057485226039389*w7
    ASSERT w8 = w3*w7 + 31631953497925087476338759149270597631*w7
    BLACKBOX::RANGE input: w8, bits: 126
    ASSERT w1 = w3

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

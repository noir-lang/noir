use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn add_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w2 = w0 + w1
    ");
}

#[test]
fn sub_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = sub v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w2 = w0 - w1
    ");
}

#[test]
fn mul_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = mul v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w2 = w0*w1
    ");
}

#[test]
fn div_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = div v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // - w0 is v0
    // - w1 is v1
    // - w2 is v2
    // - w3 is 1/w1
    // - then w2 is w0 * w3 = w0 * 1 / w1 = w0 / w1 = v0 / v1
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func: 0, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    ASSERT w2 = w0*w3

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
fn eq_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // See noirc_evaluator::acir::acir_context::generated_acir::GeneratedAcir::is_zero
    // for an explanation of how equality is implemented.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w3 = w0 - w1
    BRILLIG CALL func: 0, inputs: [w3], outputs: [w4]
    ASSERT w5 = -w3*w4 + 1
    ASSERT 0 = w3*w5
    ASSERT w2 = w5

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
fn unchecked_add_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = unchecked_add v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    ASSERT w2 = w0 + w1
    ");
}

#[test]
fn checked_add_u8_no_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = add v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    ASSERT w3 = w0 + w1
    BLACKBOX::RANGE input: w3, bits: 8
    ASSERT w2 = w3
    ");
}

#[test]
fn checked_add_u8_with_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8, v2: u1):
        enable_side_effects v2
        v3 = add v0, v1
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    // Note how every operand in the addition (w0, w1) is multiplied by the predicate (w2)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 1
    ASSERT w4 = w0*w2 + w1*w2
    BLACKBOX::RANGE input: w4, bits: 8
    ASSERT w3 = w4
    ");
}

#[test]
fn checked_mul_u8_with_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8, v2: u1):
        enable_side_effects v2
        v3 = mul v0, v1
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    // w0 is multiplied by w1, then the result (w4) is multiplied by the predicate (w2)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 1
    ASSERT w4 = w0*w1
    ASSERT w5 = w2*w4
    BLACKBOX::RANGE input: w5, bits: 8
    ASSERT w3 = w5
    ");
}

#[test]
fn div_u8_no_predicate_by_witness() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = div v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // - w0 is v0
    // - w1 is v1
    // - w3 is 1/w1
    // - we check `0 = w1*w3 - 1` to ensure that an inverse must exist (so that w1 != 0)
    // - w4 is w0/w1
    // - w5 is w0 mod w1
    // - with `w6 = w1 - w5 - 1` and the next range check we ensure that w5 < w1
    // - `w5 = -w1*w4 + w0` can be read as `w0 = w1*w4 + w5`
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BRILLIG CALL func: 0, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    BRILLIG CALL func: 1, inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    ASSERT w6 = w1 - w5 - 1
    BLACKBOX::RANGE input: w6, bits: 8
    ASSERT w5 = -w1*w4 + w0
    ASSERT w2 = w4

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
    unconstrained func 1: directive_integer_quotient
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
fn div_u8_no_predicate_by_constant() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8):
        v1 = div v0, u8 7
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // - Note how w2, w3 and w4 are range-checked with less bits than 8
    // - with `w4 = w3 + 1` and the next range check we ensure that 0 <= w3 < 8
    // - `w3 = w0 - 7*w2` can be read as `w0 = 7*w2 + w3` where w2 is the division quotient
    //   and w3 is the remainder.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 8
    BRILLIG CALL func: 0, inputs: [w0, 7], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 6
    BLACKBOX::RANGE input: w3, bits: 3
    ASSERT w4 = w3 + 1
    BLACKBOX::RANGE input: w4, bits: 3
    ASSERT w3 = w0 - 7*w2
    ASSERT w1 = w2

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
fn div_u8_with_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8, v2: u1):
        enable_side_effects v2
        v3 = div v0, v1
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    // - w0, w1 and w2 are v0, v1 and v2 respectively
    // - w4 is 1/w1 (field inverse)
    // - in the first Brillig call and the follow three asserts we assert that w1 is not zero
    //   and the predicate (w2) is active
    // - `ASSERT w8 = w1 - w2 - w7` subtracts `rhs - predicate - remainder`. If the remainder is
    //   greater than the rhs the following range constraint will fail.
    //   See `bound_constraint_with_offset` for more info.
    // - Finally, the last three ASSERTs are the definition of integer division with predication
    //   (`a * predicate == (b * q + r) * predicate`) and the return witness creation.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 1
    BRILLIG CALL func: 0, inputs: [w1], outputs: [w4]
    ASSERT w5 = -w1*w4 + 1
    ASSERT 0 = w1*w5
    ASSERT 0 = w2*w5
    BRILLIG CALL func: 1, predicate: w2, inputs: [w0, w1], outputs: [w6, w7]
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    ASSERT w8 = w1 - w2 - w7
    BLACKBOX::RANGE input: w8, bits: 8
    ASSERT w9 = w1*w6 + w7
    ASSERT 0 = w0*w2 - w2*w9
    ASSERT w3 = w6

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
    unconstrained func 1: directive_integer_quotient
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
fn mod_u8_no_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = mod v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // This is similar to div_u8_no_predicate except that the return witness is constrained
    // against w5 instead of w4.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BRILLIG CALL func: 0, inputs: [w1], outputs: [w3]
    ASSERT 0 = w1*w3 - 1
    BRILLIG CALL func: 1, inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    ASSERT w6 = w1 - w5 - 1
    BLACKBOX::RANGE input: w6, bits: 8
    ASSERT w5 = -w1*w4 + w0
    ASSERT w2 = w5

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
    unconstrained func 1: directive_integer_quotient
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
fn eq_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = eq v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // This ends up being similar to eq_field with the addition of range checks on the inputs
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    ASSERT w3 = w0 - w1
    BRILLIG CALL func: 0, inputs: [w3], outputs: [w4]
    ASSERT w5 = -w3*w4 + 1
    ASSERT 0 = w3*w5
    ASSERT w2 = w5

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
fn lt_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = lt v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // `w0 - w1 + 256` will be:
    // - in [0, 255] if w0 < w1
    // - in [256, 511] if w0 >= w1
    // then dividing that by 256 will given:
    // - 0 if w0 < w1
    // - 1 if w0 >= w1
    //
    // `w4 = ...` checks the quotient/remainder relationship between w0, w1, w3 and w4
    //
    // Finally, `w3 = -w2 + 1` is just the opposite of `w2`, since above
    // we saw that we get 1 when `w0 >= w1`.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BRILLIG CALL func: 0, inputs: [w0 - w1 + 256, 256], outputs: [w3, w4]
    BLACKBOX::RANGE input: w3, bits: 1
    BLACKBOX::RANGE input: w4, bits: 8
    ASSERT w4 = w0 - w1 - 256*w3 + 256
    ASSERT w2 = -w3 + 1

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
fn and_u1() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = and v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 1
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w2 = w0*w1
    ");
}

#[test]
fn and_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = and v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::AND lhs: w0, rhs: w1, output: w3, bits: 8
    ASSERT w2 = w3
    ");
}

#[test]
fn or_u1() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = or v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // - If both w0 and w1 are 0, all terms are zero so w2 must be 0
    // - If either w0 or w1 is 1 but not both, `-w0*w1` is zero. Thus, w2 = w0 + w1 and w2 must be 1
    // - If both w0 and w1 are 1 w2 must be 1 too
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 1
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w2 = -w0*w1 + w0 + w1
    ");
}

#[test]
fn or_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = or v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // x | y = !!(x | y) = !(!x & !y)
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    ASSERT w3 = -w0 + 255
    ASSERT w4 = -w1 + 255
    BLACKBOX::AND lhs: w3, rhs: w4, output: w5, bits: 8
    ASSERT w2 = -w5 + 255
    ");
}

#[test]
fn xor_u1() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = xor v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // - If both w0 and w1 are 0, all terms are zero so w2 must be 0
    // - If either w0 or w1 is 1 but not both, `-2*w0*w1` is zero. Thus, w2 = w0 + w1 and w2 must be 1
    // - If both w0 and w1 are 1, `-2*w0*w1` is -2, then w2 must be 0
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 1
    BLACKBOX::RANGE input: w1, bits: 1
    ASSERT w2 = -2*w0*w1 + w0 + w1
    ");
}

#[test]
fn xor_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = xor v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::XOR lhs: w0, rhs: w1, output: w3, bits: 8
    ASSERT w2 = w3
    ");
}

#[test]
fn div_divisor_overflow_with_side_effects_enabled() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8):
        enable_side_effects u1 1
        v1 = div v0, u8 256
        return v0
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE input: w0, bits: 8
    ASSERT 0 = 1
    ASSERT w1 = w0
    ");
}

#[test]
fn div_divisor_overflow_without_side_effects_enabled() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8):
        enable_side_effects u1 0
        v1 = div v0, u8 256
        return v0
    }
    ";
    let program = ssa_to_acir_program(src);

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
fn div_divisor_overflow_with_dynamic_side_effects_enabled() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u1):
        enable_side_effects v1
        v2 = div v0, u8 256
        return v0
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 8
    ASSERT w1 = 0
    ASSERT w2 = w0
    ");
}

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
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    EXPR w2 = w0 + w1
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
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    EXPR w2 = w0 - w1
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
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    EXPR w2 = w0*w1
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
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func 0: inputs: [w1], outputs: [w3]
    EXPR 0 = w1*w3 - 1
    EXPR w2 = w0*w3

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
    current witness: w5
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    EXPR w3 = w0 - w1
    BRILLIG CALL func 0: inputs: [w3], outputs: [w4]
    EXPR w5 = -w3*w4 + 1
    EXPR 0 = w3*w5
    EXPR w5 = w2

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
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    EXPR w2 = w0 + w1
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
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    EXPR w3 = w0 + w1
    BLACKBOX::RANGE [w3]:8 bits []
    EXPR w3 = w2
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
    current witness: w4
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::RANGE [w2]:1 bits []
    EXPR w4 = w0*w2 + w1*w2
    BLACKBOX::RANGE [w4]:8 bits []
    EXPR w4 = w3
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

    // w0 is multiplied by w1, then the reslt (w4) is multiplied by the predicate (w2)
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w5
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::RANGE [w2]:1 bits []
    EXPR w4 = w0*w1
    EXPR w5 = w2*w4
    BLACKBOX::RANGE [w5]:8 bits []
    EXPR w5 = w3
    ");
}

#[test]
fn div_u8_no_predicate_by_var() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = div v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // - w3 is 1/w1
    // - we check `0 = w1*w3 - 1` to ensure that an inverse must exist (so that w1 != 0)
    // - w4 is w0/w1
    // - w5 is w0 mod w1
    // - with `w6 = w1 - w5 - 1` and the next range check we ensure that w5 < w1
    // - `w5 = -w1*w4 + w0` can be read as `w0 = w1*w4 + w5`
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w6
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BRILLIG CALL func 0: inputs: [w1], outputs: [w3]
    EXPR 0 = w1*w3 - 1
    BRILLIG CALL func 1: inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE [w4]:8 bits []
    BLACKBOX::RANGE [w5]:8 bits []
    EXPR w6 = w1 - w5 - 1
    BLACKBOX::RANGE [w6]:8 bits []
    EXPR w5 = -w1*w4 + w0
    EXPR w4 = w2

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
    // - `w3 = w0 - 7*w2` can be read as `w0 = 7*w2 + w3` which is the definition of divmod
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w4
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BLACKBOX::RANGE [w0]:8 bits []
    BRILLIG CALL func 0: inputs: [w0, 7], outputs: [w2, w3]
    BLACKBOX::RANGE [w2]:6 bits []
    BLACKBOX::RANGE [w3]:3 bits []
    EXPR w4 = w3 + 1
    BLACKBOX::RANGE [w4]:3 bits []
    EXPR w3 = w0 - 7*w2
    EXPR w2 = w1

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

    // TODO: explain this result
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w9
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: [w3]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::RANGE [w2]:1 bits []
    BRILLIG CALL func 0: inputs: [w1], outputs: [w4]
    EXPR w5 = -w1*w4 + 1
    EXPR 0 = w1*w5
    EXPR 0 = w2*w5
    BRILLIG CALL func 1: PREDICATE: w2
    inputs: [w0, w1], outputs: [w6, w7]
    BLACKBOX::RANGE [w6]:8 bits []
    BLACKBOX::RANGE [w7]:8 bits []
    EXPR w8 = w1 - w2 - w7
    BLACKBOX::RANGE [w8]:8 bits []
    EXPR w9 = w1*w6 + w7
    EXPR 0 = w0*w2 - w2*w9
    EXPR w6 = w3

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
    current witness: w6
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BRILLIG CALL func 0: inputs: [w1], outputs: [w3]
    EXPR 0 = w1*w3 - 1
    BRILLIG CALL func 1: inputs: [w0, w1], outputs: [w4, w5]
    BLACKBOX::RANGE [w4]:8 bits []
    BLACKBOX::RANGE [w5]:8 bits []
    EXPR w6 = w1 - w5 - 1
    BLACKBOX::RANGE [w6]:8 bits []
    EXPR w5 = -w1*w4 + w0
    EXPR w5 = w2

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

// No test for eq_u8 as it's similar to eq_field

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
    // `w4 = ...` checks the divmod relationship between w0, w1, w3 and w4
    //
    // Finally, `w3 = -w2 + 1` is just the opposite of `w2`, since above
    // we saw that we get 1 when `w0 >= w1`.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w4
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BRILLIG CALL func 0: inputs: [w0 - w1 + 256, 256], outputs: [w3, w4]
    BLACKBOX::RANGE [w3]:1 bits []
    BLACKBOX::RANGE [w4]:8 bits []
    EXPR w4 = w0 - w1 - 256*w3 + 256
    EXPR w3 = -w2 + 1

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
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:1 bits []
    BLACKBOX::RANGE [w1]:1 bits []
    EXPR w2 = w0*w1
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
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::AND [w0, w1]:8 bits [w3]
    EXPR w3 = w2
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
    // - If either w0 or w1 is 1 but not both, `-w0*w1` is zero so w2 = w0 + w1 then w2 must be 1
    // - If both w0 and w1 are 1 w2 must be 1 too
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:1 bits []
    BLACKBOX::RANGE [w1]:1 bits []
    EXPR w2 = -w0*w1 + w0 + w1
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
    current witness: w5
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    EXPR w3 = -w0 + 255
    EXPR w4 = -w1 + 255
    BLACKBOX::AND [w3, w4]:8 bits [w5]
    EXPR w5 = -w2 + 255
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
    // - If either w0 or w1 is 1 but not both, `-2*w0*w1` is zero so w2 = w0 + w1 then w2 must be 1
    // - If both w0 and w1 are 1, `-2*w0*w1` is -2, then w2 must be 0
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:1 bits []
    BLACKBOX::RANGE [w1]:1 bits []
    EXPR w2 = -2*w0*w1 + w0 + w1
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
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::XOR [w0, w1]:8 bits [w3]
    EXPR w3 = w2
    ");
}

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

    // Here:
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

    unconstrained func 0
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

    unconstrained func 0
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
fn checked_add_u8() {
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

// No tests for sub/mul since they'll be similar to the ones for add

#[test]
fn div_u8() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v2 = div v0, v1
        return v2
    }
    ";
    let program = ssa_to_acir_program(src);

    // Here:
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

    unconstrained func 0
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop &[@20; @21]
    unconstrained func 1
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

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
fn eq_field() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        return v2
    }
    ";

    // See noirc_evaluator::acir::acir_context::generated_acir::GeneratedAcir::is_zero
    // for an explanation of how equality is implemented.
    let program = ssa_to_acir_program(src);
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

//! A blackbox predicate can be a non-constant SSA value which only folds to a compile-time
//! zero during ACIR gen (e.g. `eq (x + 5), x`). Such calls are statically disabled, and
//! backends reject a constant zero predicate, so ACIR gen must not emit them.

use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

#[test]
fn multi_scalar_mul_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field):
        v1 = add v0, Field 5
        v2 = eq v1, v0
        v3 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860] : [(Field, Field); 1]
        v4 = make_array [v0, Field 0] : [(Field, Field); 1]
        v5 = call multi_scalar_mul(v3, v4, v2) -> [(Field, Field); 1]
        return v5
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1, w2]
    ASSERT w3 = 0
    BLACKBOX::MULTI_SCALAR_MUL points: [1, 17631683881184975370165255887551781615748388533673675138860], scalars: [w0, w3], predicate: 0, outputs: [w4, w5]
    ASSERT w1 = w4
    ASSERT w2 = w5
    ");
}

#[test]
fn embedded_curve_add_with_predicate_folded_to_zero_emits_no_opcode() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: Field):
        v1 = add v0, Field 5
        v2 = eq v1, v0
        v3 = call embedded_curve_add(Field 1, Field 17631683881184975370165255887551781615748388533673675138860, v0, v0, v2) -> [(Field, Field); 1]
        return v3
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1, w2]
    BLACKBOX::EMBEDDED_CURVE_ADD input1: [1, 17631683881184975370165255887551781615748388533673675138860], input2: [w0, w0], predicate: 0, outputs: [w3, w4]
    ASSERT w1 = w3
    ASSERT w2 = w4
    ");
}
